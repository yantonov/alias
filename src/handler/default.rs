use crate::config::Alias::{RegularAlias, ShellAlias};
use crate::config::Configuration;
use crate::environment::{Environment, expand_env};
use crate::handler::Handler;
use crate::process::CallContext;
use crate::{config, environment, process};
use std::env;

fn get_call_context(
    environment: &Environment,
    configuration: &Configuration,
) -> Result<CallContext, String> {
    let call_arguments = environment.call_arguments();
    let executable = get_executable(environment, configuration)?.ok_or(format!(
        "Cannot autodetect executable: {}",
        environment.executable_name()
    ))?;

    match configuration.resolve_alias(call_arguments)? {
        Some((alias, consumed)) => {
            let remaining = &call_arguments[consumed..];
            match alias {
                ShellAlias(cmd) => handle_shell_alias(remaining, environment.shell()?, cmd),
                RegularAlias(mut arguments) => {
                    arguments.extend_from_slice(remaining);
                    call_the_target(configuration, environment, &executable, arguments)
                }
            }
        }
        None => call_the_target(
            configuration,
            environment,
            &executable,
            call_arguments.to_vec(),
        ),
    }
}

fn get_executable(
    environment: &Environment,
    configuration: &Configuration,
) -> Result<Option<String>, String> {
    Ok(configuration
        .get_executable()?
        .map(|config| expand_env::expand_env_var(&config))
        .or_else(|| environment.try_detect_executable()))
}

// Whatever the arguments turned out to be, an expanded alias or the untouched
// command line, they reach the target the same way: it is started directly, or
// with run_as_shell it is handed to the shell, which then needs its path as the
// first argument.
//
// This is the only path a shell is looked up on outside of shell aliases, and
// it is looked up here rather than by the caller: run_as_shell is off by
// default, and a wrapper that never turns it on must not depend on a shell
// being nameable at all.
fn call_the_target(
    configuration: &Configuration,
    environment: &Environment,
    executable: &str,
    arguments: Vec<String>,
) -> Result<CallContext, String> {
    if !run_as_shell(configuration)? {
        return Ok(CallContext {
            executable: executable.to_string(),
            args: arguments,
        });
    }

    let mut args = vec![executable.to_string()];
    args.extend(arguments);
    Ok(CallContext {
        executable: environment.shell()?.to_string(),
        args,
    })
}

// Arguments left after the alias are appended the way git appends them: they
// travel to the shell as positional parameters, and the command gets a quoted
// "$@" so that they reach it whole, an argument containing spaces included.
// Nothing the user typed is ever concatenated into the command text.
//
// A command that gets no arguments is left exactly as written: a trailing
// "$@" would expand to nothing anyway, while showing up in shell diagnostics
// and disappearing into a command that happens to end with a comment.
fn handle_shell_alias(
    remaining: &[String],
    shell: &str,
    shell_command: String,
) -> Result<CallContext, String> {
    let command = if remaining.is_empty() {
        shell_command.clone()
    } else {
        format!("{} \"$@\"", shell_command)
    };

    // $0 is the command itself, so that shell diagnostics name what failed.
    let mut args = vec!["-c".to_string(), command, shell_command];
    for p in remaining {
        args.push(p.clone());
    }
    Ok(CallContext {
        executable: shell.to_string(),
        args,
    })
}

// A dry run prints the command that would have been executed and stops.
//
// The switch is an environment variable rather than a flag on purpose: the
// command line then reaches the resolution untouched, so what gets printed is
// what would have run, rather than a reconstruction of it. No name of ours can
// collide with a flag of the target program either.
const DRY_RUN: &str = "ALIAS_DRY_RUN";

// Set at all counts as on, whatever the value, the way NO_COLOR works. A value
// meaning 'off' only invites the question of which spellings of it count.
fn dry_run() -> bool {
    env::var_os(DRY_RUN).is_some()
}

// One argument per line: joining them with spaces would be ambiguous for the
// arguments that contain spaces, and those are exactly the ones people come
// here to look at.
fn print_call_context(call_context: &CallContext) {
    println!("dry run: {} is set, nothing is executed", DRY_RUN);
    println!("executable: {}", call_context.executable);
    if call_context.args.is_empty() {
        println!("argv: none");
    } else {
        println!("argv:");
        for (index, argument) in call_context.args.iter().enumerate() {
            println!("  [{}] {}", index + 1, argument);
        }
    }
}

fn run_as_shell(configuration: &Configuration) -> Result<bool, String> {
    match configuration.get_run_as_shell()? {
        None => Ok(false),
        Some(as_shell) => Ok(as_shell),
    }
}

fn execute(environment: &environment::Environment, configuration: &config::Configuration) {
    let call_context_result = get_call_context(environment, configuration);
    match call_context_result {
        Ok(call_context) => {
            // Deliberately here and not in process::execute, which also serves
            // the passthrough behind --help and --aliases: there is nothing to
            // explain about that one.
            if dry_run() {
                print_call_context(&call_context);
                return;
            }
            match process::execute(&call_context) {
                Ok(code) => process::exit(code),
                Err(error) => {
                    eprintln!("{}", error);
                    process::exit(None);
                }
            }
        }
        // Nothing was run: the wrapper could not work out what to run, because
        // of a broken alias, an undetectable target or a shell it needs and
        // cannot name. Its own failures all exit 1, the way a configuration
        // that does not parse already did, and the missing shell did before it
        // was moved out of startup.
        Err(error) => {
            eprintln!("{}", error);
            process::exit(Some(1));
        }
    }
}

pub struct DefaultHandler {}

impl Handler for DefaultHandler {
    fn handle(&self, environment: &Environment, configuration: &Configuration) {
        execute(environment, configuration)
    }
}

impl DefaultHandler {
    pub fn new() -> DefaultHandler {
        DefaultHandler {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell_alias(command: &str, remaining: &[&str]) -> CallContext {
        let remaining: Vec<String> = remaining.iter().map(|a| a.to_string()).collect();
        handle_shell_alias(&remaining, "/bin/sh", command.to_string())
            .expect("a shell alias resolves without a target executable")
    }

    #[test]
    fn a_command_without_arguments_is_handed_over_as_written() {
        let context = shell_alias("echo hi", &[]);
        assert_eq!("/bin/sh", context.executable);
        assert_eq!(vec!["-c", "echo hi", "echo hi"], context.args);
    }

    #[test]
    fn arguments_reach_the_command_through_a_quoted_parameter_expansion() {
        let context = shell_alias("echo hi", &["one", "two"]);
        assert_eq!(
            vec!["-c", "echo hi \"$@\"", "echo hi", "one", "two"],
            context.args
        );
    }

    // The quotes around $@ are what keeps this one argument rather than two.
    #[test]
    fn an_argument_containing_spaces_stays_a_single_argument() {
        let context = shell_alias("git commit -m", &["work in progress"]);
        assert_eq!(
            vec![
                "-c",
                "git commit -m \"$@\"",
                "git commit -m",
                "work in progress"
            ],
            context.args
        );
    }
}

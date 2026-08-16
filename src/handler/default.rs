use crate::config::Alias::{RegularAlias, ShellAlias};
use crate::config::Configuration;
use crate::environment::{Environment, expand_env};
use crate::handler::Handler;
use crate::process::CallContext;
use crate::{config, environment, process};

fn get_call_context(
    environment: &Environment,
    configuration: &Configuration,
) -> Result<CallContext, String> {
    let call_arguments = environment.call_arguments();
    let executable = get_executable(environment, configuration)?.ok_or(format!(
        "Cannot autodetect executable: {}",
        environment.executable_name()
    ))?;
    let shell = environment.shell();

    match configuration.resolve_alias(call_arguments)? {
        Some((alias, consumed)) => {
            let remaining = &call_arguments[consumed..];
            match alias {
                ShellAlias(cmd) => handle_shell_alias(remaining, shell, cmd),
                RegularAlias(alias_args) => {
                    handle_regular_alias(configuration, remaining, &executable, shell, alias_args)
                }
            }
        }
        None => {
            forward_call_to_target_application(configuration, call_arguments, executable, shell)
        }
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

fn forward_call_to_target_application(
    configuration: &Configuration,
    call_arguments: &[String],
    executable: String,
    shell: &str,
) -> Result<CallContext, String> {
    let mut args = Vec::new();
    let run_as_shell = run_as_shell(configuration)?;
    if run_as_shell {
        args.push(executable.clone());
    }
    for p in call_arguments {
        args.push(p.to_string());
    }

    Ok(CallContext {
        executable: if run_as_shell {
            shell.to_string()
        } else {
            executable
        },
        args,
    })
}

fn handle_regular_alias(
    configuration: &Configuration,
    remaining: &[String],
    executable: &str,
    shell: &str,
    alias_arguments: Vec<String>,
) -> Result<CallContext, String> {
    let mut args = Vec::new();
    let run_as_shell = run_as_shell(configuration)?;
    if run_as_shell {
        args.push(executable.to_string());
    }
    for a in alias_arguments {
        args.push(a);
    }
    for p in remaining {
        args.push(p.to_string());
    }
    Ok(CallContext {
        executable: if run_as_shell {
            shell.to_string()
        } else {
            executable.to_string()
        },
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

fn run_as_shell(configuration: &Configuration) -> Result<bool, String> {
    match configuration.get_run_as_shell()? {
        None => Ok(false),
        Some(as_shell) => Ok(as_shell),
    }
}

fn execute(environment: &environment::Environment, configuration: &config::Configuration) {
    let call_context_result = get_call_context(environment, configuration);
    match call_context_result {
        Ok(call_context) => match process::execute(&call_context) {
            Ok(code) => process::exit(code),
            Err(error) => {
                eprintln!("{}", error);
                process::exit(None);
            }
        },
        Err(error) => {
            eprintln!("{}", error);
            process::exit(None);
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

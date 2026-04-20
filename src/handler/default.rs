use crate::config::Alias::{RegularAlias, ShellAlias};
use crate::config::Configuration;
use crate::environment::{expand_env, Environment};
use crate::handler::Handler;
use crate::process::CallContext;
use crate::{config, environment, process};

fn get_call_context(environment: &Environment,
                    configuration: &Configuration) -> Result<CallContext, String> {
    let call_arguments = environment.call_arguments();
    let executable = get_executable(environment, configuration)?
        .ok_or(format!("Cannot autodetect executable: {}", environment.executable_name()))?;
    let shell = environment.shell();

    match configuration.resolve_alias(call_arguments)? {
        Some((alias, consumed)) => {
            let remaining = &call_arguments[consumed..];
            match alias {
                ShellAlias(cmd) => handle_shell_alias(remaining, &shell, cmd),
                RegularAlias(alias_args) => handle_regular_alias(configuration, remaining, &executable, &shell, alias_args),
            }
        }
        None => forward_call_to_target_application(configuration, call_arguments, executable, shell),
    }
}

fn get_executable(environment: &Environment, configuration: &Configuration) -> Result<Option<String>, String> {
    Ok(configuration.get_executable()?
        .map(|config| expand_env::expand_env_var(&config))
        .or_else(|| environment.try_detect_executable()))
}

fn forward_call_to_target_application(configuration: &Configuration, call_arguments: &[String], executable: String, shell: &str) -> Result<CallContext, String> {
    let mut args = Vec::new();
    let run_as_shell = run_as_shell(configuration)?;
    if run_as_shell {
        args.push(executable.clone());
    }
    for p in call_arguments {
        args.push(p.to_string());
    }

    Ok(CallContext {
        executable: if run_as_shell { shell.to_string() } else { executable },
        args,
    })
}

fn handle_regular_alias(configuration: &Configuration, remaining: &[String], executable: &str, shell: &str, alias_arguments: Vec<String>) -> Result<CallContext, String> {
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
        executable: if run_as_shell { shell.to_string() } else { executable.to_string() },
        args,
    })
}

fn handle_shell_alias(remaining: &[String], shell: &str, shell_command: String) -> Result<CallContext, String> {
    let mut args = vec!["-c".to_string(), shell_command, "script".to_string()];
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
        Some(as_shell) => {
            Ok(as_shell)
        }
    }
}

fn execute(environment: &environment::Environment,
           configuration: &config::Configuration) {
    let call_context_result = get_call_context(environment, configuration);
    match call_context_result {
        Ok(call_context) => {
            match process::execute(&call_context) {
                Ok(code) => process::exit(code),
                Err(error) => {
                    eprintln!("{}", error);
                    process::exit(None);
                }
            }
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(None);
        }
    }
}

pub struct DefaultHandler {}

impl Handler for DefaultHandler {
    fn handle(&self,
              environment: &Environment,
              configuration: &Configuration) {
        execute(environment, configuration)
    }
}

impl DefaultHandler {
    pub fn new() -> DefaultHandler {
        DefaultHandler {}
    }
}
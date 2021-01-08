use crate::{config, environment, process};
use crate::config::Alias::{RegularAlias, ShellAlias};
use crate::config::{Configuration, Alias};
use crate::environment::{Environment, expand_env, autodetect_executable::{OsCheckFile, autodetect_executable}};
use crate::handler::Handler;
use crate::process::CallContext;

fn get_call_context(environment: &environment::Environment,
                    configuration: &config::Configuration) -> Result<CallContext, String> {
    let call_arguments = environment.call_arguments();
    let executable = configuration.get_executable()?
        .map(|config| expand_env::expand_env_var(&config))
        .or_else(|| autodetect_executable(
            environment.executable_dir().as_path(),
            environment.executable_name().as_str(),
            &OsCheckFile {}))
        .ok_or(format!("Cannot autodetect executable: {}", environment.executable_name()))?;
    let shell = environment.shell();

    let aliased_command: Option<Alias> = if call_arguments.len() == 0 {
        None
    } else {
        configuration.get_alias(&call_arguments[0])?
    };

    match aliased_command {
        Some(alias) => {
            match alias {
                ShellAlias(shell_command) => {
                    let mut args = Vec::new();
                    args.push("-c".to_string());
                    args.push(shell_command.clone());
                    args.push("script".to_string());
                    for p in &call_arguments[1..call_arguments.len()] {
                        args.push(p.clone());
                    }
                    Ok(
                        CallContext {
                            executable: shell,
                            args: args.iter().map(|t| t.to_string()).collect(),
                        })
                }
                RegularAlias(alias_arguments) => {
                    let mut args = Vec::new();
                    let run_as_shell = run_as_shell(&configuration)?;
                    if run_as_shell {
                        args.push(executable.clone());
                    }
                    for a in alias_arguments {
                        args.push(a);
                    }
                    for p in &call_arguments[1..call_arguments.len()] {
                        args.push(p.to_string());
                    }
                    Ok(
                        CallContext {
                            executable: if run_as_shell { shell } else { executable },
                            args,
                        })
                }
            }
        }
        None => {
            let mut args = Vec::new();
            let run_as_shell = run_as_shell(&configuration)?;
            if run_as_shell {
                args.push(executable.clone());
            }
            for p in call_arguments {
                args.push(p.to_string());
            }
            Ok(
                CallContext {
                    executable: if run_as_shell { shell } else { executable },
                    args,
                })
        }
    }
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
    let call_context_result = get_call_context(&environment, &configuration);
    match call_context_result {
        Ok(call_context) => {
            let result = process::execute(&call_context);
            match result {
                Ok(_) => {
                    process::exit(result);
                }
                Err(error) => {
                    eprintln!("{}", error);
                    process::exit(Err(error));
                }
            }
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(Err(error));
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
        return DefaultHandler {};
    }
}
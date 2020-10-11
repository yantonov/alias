use crate::{config, environment, process};
use crate::config::Alias::{RegularAlias, ShellAlias};
use crate::config::Configuration;
use crate::environment::{Environment, expand_env_var};
use crate::handler::Handler;
use crate::process::CallContext;

fn get_call_context(environment: &environment::Environment,
                    configuration: &config::Configuration) -> Result<CallContext, String> {
    let call_arguments = environment.call_arguments();
    let executable = configuration.get_executable()?;

    if call_arguments.len() == 0 {
        return Ok(
            CallContext {
                executable: expand_env_var(&executable),
                args: Vec::new(),
            });
    }
    let aliased_command = configuration.get_alias(&call_arguments[0])?;

    match aliased_command {
        Some(alias) => {
            match alias {
                ShellAlias(shell_command) => {
                    let shell = environment.shell();
                    let mut args = Vec::new();
                    args.push("-c");
                    args.push(&shell_command);
                    args.push("script");
                    for p in &call_arguments[1..call_arguments.len()] {
                        args.push(&p);
                    }
                    Ok(
                        CallContext {
                            executable: shell,
                            args: args.iter().map(|t| t.to_string()).collect(),
                        })
                }
                RegularAlias(alias_arguments) => {
                    let mut args = Vec::new();
                    for a in alias_arguments {
                        args.push(a);
                    }
                    for p in &call_arguments[1..call_arguments.len()] {
                        args.push(p.to_string());
                    }
                    Ok(
                        CallContext {
                            executable: expand_env_var(&executable),
                            args,
                        })
                }
            }
        }
        None => {
            let mut args = Vec::new();
            for p in call_arguments {
                args.push(p.to_string());
            }
            Ok(
                CallContext {
                    executable: expand_env_var(&executable),
                    args,
                })
        }
    }
}

fn execute(environment: &environment::Environment,
           configuration: &config::Configuration) {
    let call_context = get_call_context(&environment, &configuration)
        .unwrap();
    let result = process::execute(&call_context);
    process::exit(result);
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
use crate::{config, environment, process};
use crate::config::Alias::{RegularAlias, ShellAlias};
use crate::config::{Configuration, Alias};
use crate::environment::{Environment, expand_env, autodetect_executable::{OsFileSystemWrapper, autodetect_executable}};
use crate::handler::Handler;
use crate::process::CallContext;

fn get_call_context(environment: &environment::Environment,
                    configuration: &config::Configuration) -> Result<CallContext, String> {
    let call_arguments = environment.call_arguments();
    let executable = get_executable(environment, configuration)?
        .ok_or(format!("Cannot autodetect executable: {}", environment.executable_name()))?;
    let shell = environment.shell();

    let aliased_command: Option<Alias> = if call_arguments.is_empty() {
        None
    } else {
        configuration.get_alias(&call_arguments[0])?
    };

    match aliased_command {
        Some(alias) => {
            match alias {
                ShellAlias(shell_command) => {
                    handle_shell_alias(&call_arguments, &shell, shell_command)
                }
                RegularAlias(alias_arguments) => {
                    handle_regular_alias(configuration, &call_arguments, &executable, &shell, alias_arguments)
                }
            }
        }
        None => {
            forward_call_to_target_application(configuration, call_arguments, executable, shell)
        }
    }
}

fn get_executable(environment: &Environment, configuration: &Configuration) -> Result<Option<String>, String> {
    Ok(configuration.get_executable()?
        .map(|config| expand_env::expand_env_var(&config))
        .or_else(|| autodetect_executable(
            environment.executable_dir().as_path(),
            environment.executable_name().as_str(),
            &OsFileSystemWrapper {})))
}

fn forward_call_to_target_application(configuration: &Configuration, call_arguments: &[String], executable: String, shell: String) -> Result<CallContext, String> {
    let mut args = Vec::new();
    let run_as_shell = run_as_shell(configuration)?;
    if run_as_shell {
        args.push(executable.clone());
    }
    for p in call_arguments {
        args.push(p.to_string());
    }

    Ok(CallContext {
        executable: if run_as_shell { shell } else { executable },
        args,
    })
}

fn handle_regular_alias(configuration: &Configuration, call_arguments: &&[String], executable: &String, shell: &String, alias_arguments: Vec<String>) -> Result<CallContext, String> {
    let mut args = Vec::new();
    let run_as_shell = run_as_shell(configuration)?;
    if run_as_shell {
        args.push(executable.clone());
    }
    for a in alias_arguments {
        args.push(a);
    }
    for p in &call_arguments[1..call_arguments.len()] {
        args.push(p.to_string());
    }
    Ok(CallContext {
        executable: if run_as_shell { shell.to_string() } else { executable.to_string() },
        args,
    })
}

fn handle_shell_alias(call_arguments: &&[String], shell: &String, shell_command: String) -> Result<CallContext, String> {
    let mut args = vec![
        "-c".to_string(),
        shell_command,
        "script".to_string()];
    for p in &call_arguments[1..call_arguments.len()] {
        args.push(p.clone());
    }
    Ok(CallContext {
        executable: shell.to_string(),
        args: args.iter().map(|t| t.to_string()).collect(),
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
        DefaultHandler {}
    }
}
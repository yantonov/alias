mod config;
mod environment;
mod process;

use config::Alias::{ShellAlias, RegularAlias};
use process::CallContext;
use environment::expand_env_var;

pub fn get_call_context(environment: &environment::Environment,
                        configuration: &config::Configuration) -> CallContext {
    let call_arguments = environment.get_call_arguments();
    if call_arguments.len() == 0 {
        return CallContext {
            executable: expand_env_var(&configuration.get_executable()),
            args: Vec::new()
        };
    }
    let aliased_command = configuration.get_alias(&call_arguments[0]);

    match aliased_command {
        Some(alias) => {
            match alias {
                ShellAlias(shell_command) => {
                    let shell = environment.get_shell();
                    let mut args = Vec::new();
                    args.push("-c");
                    args.push(&shell_command);
                    args.push("script");
                    for p in &call_arguments[1..call_arguments.len()] {
                        args.push(&p);
                    }
                    return CallContext {
                        executable: shell,
                        args: args.iter().map(|t| t.to_string()).collect()
                    };
                }
                RegularAlias(alias_arguments) => {
                    let mut args = Vec::new();
                    for a in alias_arguments {
                        args.push(a);
                    }
                    for p in &call_arguments[1..call_arguments.len()] {
                        args.push(p.to_string());
                    }
                    return CallContext {
                        executable: expand_env_var(&configuration.get_executable()),
                        args: args
                    };
                }
            }
        }
        None => {
            let mut args = Vec::new();
            for p in call_arguments {
                args.push(p.to_string());
            }
            return CallContext {
                executable: expand_env_var(&configuration.get_executable()),
                args: args
            };
        }
    }
}

fn main() {
    let environment = environment::get_environment();
    let configuration = config::read_configuration(environment.executable_dir());
    let call_context = get_call_context(&environment, &configuration);
    process::execute(&call_context);
}

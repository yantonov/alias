mod config;
mod environment;
mod process;

use config::Alias::{ShellAlias, RegularAlias};

fn main() {
    let environment = environment::get_environment();
    let configuration = config::read_configuration(environment.executable_dir());

    let call_arguments = environment.get_call_arguments();
    if call_arguments.len() == 0 {
        return;
    }

    let executable = configuration.get_executable();
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
                    process::execute(&shell, args);
                }
                RegularAlias(alias_arguments) => {
                    let mut args = Vec::new();
                    for a in alias_arguments {
                        args.push(a);
                    }
                    for p in &call_arguments[1..call_arguments.len()] {
                        args.push(p.to_string());
                    }
                    process::execute(executable, args);
                }
            }
        }
        None => {
            process::execute(executable, call_arguments);
        }
    }
}

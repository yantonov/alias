mod config;
mod environment;
mod process;

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
            if alias.starts_with("!") {
                let shell_command: String = alias
                    .chars()
                    .skip(1)
                    .collect();
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
            else {
                let mut args = Vec::new();
                let alias_arguments: Vec<&str> = alias.split(" ").collect();
                for a in alias_arguments {
                    args.push(a);
                }
                for p in &call_arguments[1..call_arguments.len()] {
                    args.push(&p);
                }
                process::execute(executable, args);
            }
        }
        None => {
            process::execute(executable, call_arguments);
        }
    }
}

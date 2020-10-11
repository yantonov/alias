use handler::alias_list::AliasListHandler;
use handler::default::DefaultHandler;
use handler::Handler;

mod config;
mod environment;
mod handler;
mod process;

fn get_handler(environment: &environment::Environment,
               _configuration: &config::Configuration) -> Box<dyn Handler> {
    let call_arguments = environment.call_arguments();

    let arg_count = call_arguments.len();

    if arg_count == 1 {
        let command = &call_arguments[0];
        if command == "--aliases" {
            return Box::new(AliasListHandler::new());
        }
    }

    Box::new(DefaultHandler::new())
}

fn main() {
    let environment = environment::system_environment();
    let executable_dir = environment.executable_dir();

    let config_file_path = &config::get_config_path(executable_dir);
    config::create_config_if_needed(config_file_path)
        .unwrap();
    let configuration = config::read_configuration(config_file_path)
        .unwrap();

    get_handler(&environment, &configuration)
        .handle(&environment, &configuration);
}

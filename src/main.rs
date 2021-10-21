use handler::alias_list::AliasListHandler;
use handler::default::DefaultHandler;
use handler::error::ErrorHandler;
use handler::Handler;
use config::empty_configuration;

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

    let configuration = config::get_configuration(executable_dir);

    match configuration {
        Ok(config) => {
            get_handler(&environment, &config)
                .handle(&environment, &config);
        }
        Err(e) => {
            ErrorHandler::new(e)
                .handle(&environment, &empty_configuration());
        }
    }
}

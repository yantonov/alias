use handler::alias_list::AliasListHandler;
use handler::default::DefaultHandler;
use handler::error::ErrorHandler;
use handler::help::HelpHandler;
use handler::version::VersionHandler;
use handler::Handler;
use config::empty_configuration;

mod config;
mod environment;
mod handler;
mod process;

fn get_handler(environment: &environment::Environment) -> Box<dyn Handler> {
    let call_arguments = environment.call_arguments();

    let arg_count = call_arguments.len();

    if arg_count == 1 {
        let command = &call_arguments[0];
        if command == "--aliases" {
            return Box::new(AliasListHandler::new());
        }
        if command == "--version" {
            return Box::new(VersionHandler::new());
        }
        if command == "--help" {
            return Box::new(HelpHandler::new());
        }
    }

    Box::new(DefaultHandler::new())
}

fn main() {
    let environment = match environment::system_environment() {
        Ok(env) => env,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let configuration = config::get_configuration(&environment);

    match configuration {
        Ok(config) => {
            get_handler(&environment)
                .handle(&environment, &config);
        }
        Err(e) => {
            ErrorHandler::new(e)
                .handle(&environment, &empty_configuration());
        }
    }
}

use crate::handler::default::DefaultHandler;
use crate::handler::Handler;

mod config;
mod environment;
mod handler;
mod process;

fn get_handler(_environment: &environment::Environment,
               _configuration: &config::Configuration) -> impl Handler {
    return DefaultHandler {};
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

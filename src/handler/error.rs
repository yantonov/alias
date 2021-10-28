use crate::config::Configuration;
use crate::environment::Environment;
use crate::handler::Handler;

pub struct ErrorHandler {
    error_message: String
}

impl Handler for ErrorHandler {
    fn handle(&self,
              _environment: &Environment,
              _configuration: &Configuration) {
        eprintln!("{}", self.error_message);
        std::process::exit(1);
    }
}

impl ErrorHandler {
    pub fn new(error_message: String) -> ErrorHandler {
        ErrorHandler { error_message }
    }
}
use crate::config::Configuration;
use crate::environment::Environment;
use crate::handler::Handler;

pub struct VersionHandler {}

impl Handler for VersionHandler {
    fn handle(&self,
              _environment: &Environment,
              _configuration: &Configuration) {
        println!("{}", env!("CARGO_PKG_VERSION"));
    }
}

impl VersionHandler {
    pub fn new() -> VersionHandler {
        VersionHandler {}
    }
}

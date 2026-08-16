use crate::config::Configuration;
use crate::environment::Environment;
use crate::handler::{passthrough, version_line, Handler};

pub struct VersionHandler {}

impl Handler for VersionHandler {
    fn handle(&self,
              environment: &Environment,
              configuration: &Configuration) {
        println!("{}", version_line());
        passthrough::try_passthrough(environment, configuration, &["--version"]);
    }
}

impl VersionHandler {
    pub fn new() -> VersionHandler {
        VersionHandler {}
    }
}

use crate::config::Configuration;
use crate::environment::Environment;
use crate::handler::{passthrough, Handler};

pub struct VersionHandler {}

impl Handler for VersionHandler {
    fn handle(&self,
              environment: &Environment,
              configuration: &Configuration) {
        println!("alias wrapper version {}", env!("CARGO_PKG_VERSION"));
        passthrough::try_passthrough(environment, configuration, &["--version"]);
    }
}

impl VersionHandler {
    pub fn new() -> VersionHandler {
        VersionHandler {}
    }
}

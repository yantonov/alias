use crate::config::Configuration;
use crate::environment::Environment;
use crate::handler::{passthrough, Handler};

pub struct AliasListHandler {}

impl Handler for AliasListHandler {
    fn handle(&self,
              environment: &Environment,
              configuration: &Configuration) {
        for (key, value) in configuration.list_aliases() {
            println!("{} = {}", key, value);
        }
        passthrough::try_passthrough(environment, configuration, &["--aliases"]);
    }
}

impl AliasListHandler {
    pub fn new() -> AliasListHandler {
        AliasListHandler {}
    }
}
use crate::config::Configuration;
use crate::environment::Environment;
use crate::handler::Handler;

pub struct AliasListHandler {}

impl Handler for AliasListHandler {
    fn handle(&self,
              _environment: &Environment,
              configuration: &Configuration) {
        for (key, value) in configuration.list_aliases() {
            println!("{} = {}", key, value);
        }
    }
}

impl AliasListHandler {
    pub fn new() -> AliasListHandler {
        AliasListHandler {}
    }
}
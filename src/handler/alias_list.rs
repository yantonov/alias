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
        for group in configuration.list_groups() {
            println!("\n{}:", group);
            for (key, value) in configuration.list_group_aliases(&group) {
                println!("  {} = {}", key, value);
            }
        }
        passthrough::try_passthrough(environment, configuration, &["--aliases"]);
    }
}

impl AliasListHandler {
    pub fn new() -> AliasListHandler {
        AliasListHandler {}
    }
}
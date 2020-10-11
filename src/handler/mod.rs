use crate::{config, environment};

mod default;

pub fn default_handler(environment: &environment::Environment,
                       configuration: &config::Configuration) {
    default::execute(environment, configuration)
}
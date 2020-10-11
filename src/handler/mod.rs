use crate::{config, environment};

pub mod default;

pub trait Handler {
    fn handle(&self,
              environment: &environment::Environment,
              configuration: &config::Configuration);
}

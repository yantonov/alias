use crate::{config, environment};

pub mod default;
pub mod alias_list;

pub trait Handler {
    fn handle(&self,
              environment: &environment::Environment,
              configuration: &config::Configuration);
}

use crate::{config, environment};

pub mod default;
pub mod alias_list;
pub mod error;
pub mod version;

pub trait Handler {
    fn handle(&self,
              environment: &environment::Environment,
              configuration: &config::Configuration);
}

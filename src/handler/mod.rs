use crate::{config, environment};

pub mod alias_list;
pub mod default;
pub mod error;
pub mod help;
pub mod passthrough;
pub mod version;

// Printed by --version and by --help, in both cases directly above the target
// program's own output, so it has to say plainly which of the two programs it
// describes. Kept in one place because two spellings of the same line is how
// they drifted apart before.
pub fn version_line() -> String {
    format!("alias wrapper version {}", env!("CARGO_PKG_VERSION"))
}

pub trait Handler {
    fn handle(&self, environment: &environment::Environment, configuration: &config::Configuration);
}

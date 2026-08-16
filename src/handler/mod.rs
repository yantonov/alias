use crate::config::Configuration;
use crate::environment::{Environment, expand_env};
use crate::{config, environment};
use std::fs;
use std::path::{Path, PathBuf};

pub mod alias_list;
pub mod default;
pub mod error;
pub mod help;
pub mod passthrough;
pub mod version;

// Where the target program is: named by the config, or looked up in PATH when
// it is not. Both the calls being wrapped and the flags answered by the wrapper
// itself go through here, and a wrapper that resolves to itself would loop in
// either of them.
pub fn get_executable(
    environment: &Environment,
    configuration: &Configuration,
) -> Result<Option<String>, String> {
    let executable = configuration
        .get_executable()?
        .map(|config| expand_env::expand_env_var(&config))
        .or_else(|| environment.try_detect_executable());

    match executable {
        Some(target) if is_the_wrapper_itself(&target, environment) => Err(format!(
            "the target executable is this wrapper itself ({}): it would call itself forever",
            target
        )),
        resolved => Ok(resolved),
    }
}

// Compared after resolution rather than as text: a symlink pointing at the
// wrapper, a path through a different mount point and the same path spelled
// another way are all the same file, and executing any of them starts the same
// endless chain. A path that cannot be resolved is compared as it stands,
// which is the best that can be said about it.
fn is_the_wrapper_itself(target: &str, environment: &Environment) -> bool {
    fn resolved(path: &Path) -> PathBuf {
        fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }

    resolved(Path::new(target)) == resolved(&environment.executable_path())
}

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

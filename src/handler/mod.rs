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

// Compared after resolution rather than as text: several paths can name one
// file — a mount point away, or a symlink — and executing any of them starts
// the same endless chain.
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

#[cfg(test)]
mod tests {
    use super::*;

    // A path resolves only if it is really there, so the wrapper the tests
    // compare against is put on disk under the name the environment reports.
    fn wrapper_in(directory: &Path) -> (Environment, PathBuf) {
        let environment = Environment::for_testing(directory.to_path_buf());
        let wrapper = environment.executable_path();
        fs::write(&wrapper, b"wrapper").expect("a wrapper on disk");
        (environment, wrapper)
    }

    fn is_the_wrapper(path: &Path, environment: &Environment) -> bool {
        is_the_wrapper_itself(
            path.to_str().expect("a path that is valid UTF-8"),
            environment,
        )
    }

    #[test]
    fn the_wrapper_reached_by_another_spelling_of_its_path_is_recognized() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let (environment, wrapper) = wrapper_in(directory.path());
        let detour = directory.path().join("sub");
        fs::create_dir(&detour).expect("a directory to detour through");

        let spelled_differently = detour.join("..").join(wrapper.file_name().unwrap());

        assert_ne!(
            wrapper, spelled_differently,
            "the two have to differ as text, or resolution is not what the test proves"
        );
        assert!(is_the_wrapper(&spelled_differently, &environment));
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_to_the_wrapper_is_the_wrapper() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let (environment, wrapper) = wrapper_in(directory.path());
        let link = directory.path().join("link");
        std::os::unix::fs::symlink(&wrapper, &link).expect("a symlink to the wrapper");

        assert!(is_the_wrapper(&link, &environment));
    }

    #[test]
    fn another_program_beside_the_wrapper_is_not_the_wrapper() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let (environment, _) = wrapper_in(directory.path());
        let other = directory.path().join("other");
        fs::write(&other, b"another program").expect("another program on disk");

        assert!(!is_the_wrapper(&other, &environment));
    }

    // Neither path is on disk, so neither of them resolves, and the text is all
    // that is left to go by.
    #[test]
    fn a_path_that_cannot_be_resolved_is_compared_as_it_stands() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let environment = Environment::for_testing(directory.path().join("missing"));

        assert!(is_the_wrapper(&environment.executable_path(), &environment));
        assert!(!is_the_wrapper(
            &directory.path().join("elsewhere"),
            &environment
        ));
    }
}

use crate::environment::autodetect_executable::{OsFileSystemWrapper, autodetect_executable};
use std::env;
use std::path::PathBuf;

pub mod autodetect_executable;
pub mod expand_env;

pub struct Environment {
    executable_name: String,
    executable_dir: PathBuf,
    args: Vec<String>,
    shell: String,
}

impl Environment {
    pub fn executable_name(&self) -> &String {
        &self.executable_name
    }

    pub fn executable_dir(&self) -> &PathBuf {
        &self.executable_dir
    }

    // Everything past the name the wrapper was called by. exec() is free to
    // hand a process an empty argv, so the tail is taken rather than sliced.
    pub fn call_arguments(&self) -> &[String] {
        self.args.get(1..).unwrap_or(&[])
    }

    pub fn shell(&self) -> &str {
        &self.shell
    }

    pub fn try_detect_executable(&self) -> Option<String> {
        let path_var = env::var("PATH").unwrap_or_default();
        autodetect_executable(
            self.executable_dir().as_path(),
            self.executable_name.as_str(),
            &path_var,
            &OsFileSystemWrapper {},
        )
    }
}

#[cfg(test)]
impl Environment {
    pub fn for_testing(executable_dir: PathBuf) -> Self {
        Environment {
            // Autodetection walks the real PATH, so the name has to be one no
            // machine can possibly have: whatever a test observes must not
            // depend on what happens to be installed next to it.
            executable_name: "alias-target-that-does-not-exist".to_string(),
            executable_dir,
            args: vec!["test".to_string()],
            shell: "/bin/sh".to_string(),
        }
    }
}

pub fn system_environment() -> Result<Environment, String> {
    let exe = env::current_exe().map_err(|_| "cannot get current executable".to_string())?;
    let executable_name = exe
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("cannot extract executable filename")?
        .to_string();
    let executable_dir = exe
        .parent()
        .ok_or("cannot get executable parent directory")?
        .to_path_buf();
    let shell = env::var("SHELL").map_err(|_| {
        "SHELL environment variable is not set: a POSIX shell is required".to_string()
    })?;
    Ok(Environment {
        executable_name,
        executable_dir,
        args: env::args().collect(),
        shell,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn environment_with(args: Vec<String>) -> Environment {
        Environment {
            executable_name: "git".to_string(),
            executable_dir: PathBuf::from("/bin"),
            args,
            shell: "/bin/sh".to_string(),
        }
    }

    #[test]
    fn call_arguments_drop_the_name_the_wrapper_was_called_by() {
        let environment = environment_with(vec!["git".to_string(), "co".to_string()]);
        assert_eq!(&["co".to_string()][..], environment.call_arguments());
    }

    #[test]
    fn call_arguments_are_empty_without_an_argument() {
        assert!(
            environment_with(vec!["git".to_string()])
                .call_arguments()
                .is_empty()
        );
    }

    #[test]
    fn call_arguments_are_empty_when_argv_is_empty() {
        assert!(environment_with(vec![]).call_arguments().is_empty());
    }
}

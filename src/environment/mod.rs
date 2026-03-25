use std::env;
use std::path::{PathBuf};
use crate::environment::autodetect_executable::{autodetect_executable, OsFileSystemWrapper};

pub mod expand_env;
pub mod autodetect_executable;

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

    pub fn call_arguments(&self) -> &[String] {
        &self.args[1..self.args.len()]
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
            &OsFileSystemWrapper {})
    }
}

#[cfg(test)]
impl Environment {
    pub fn for_testing(executable_dir: PathBuf) -> Self {
        Environment {
            executable_name: "test".to_string(),
            executable_dir,
            args: vec!["test".to_string()],
            shell: "/bin/sh".to_string(),
        }
    }
}

pub fn system_environment() -> Result<Environment, String> {
    let exe = env::current_exe()
        .map_err(|_| "cannot get current executable".to_string())?;
    let executable_name = exe
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("cannot extract executable filename")?
        .to_string();
    let executable_dir = exe
        .parent()
        .ok_or("cannot get executable parent directory")?
        .to_path_buf();
    let shell = env::var("SHELL")
        .map_err(|_| "SHELL environment variable is defined".to_string())?;
    Ok(Environment {
        executable_name,
        executable_dir,
        args: env::args().collect(),
        shell,
    })
}


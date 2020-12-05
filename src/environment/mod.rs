use std::env;
use std::path::{PathBuf};

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

    pub fn shell(&self) -> String {
        self.shell.to_string()
    }
}

struct SystemEnvironment {}

impl SystemEnvironment {
    pub fn executable_name(&self) -> Result<String, String> {
        env::current_exe()
            .map(|x| x
                .file_name()
                .expect("cannot detect filename")
                .to_str()
                .expect("cannot convert filename to string")
                .to_string())
            .map_err(|_| "cannot get current executable".to_string())
    }

    pub fn executable_dir(&self) -> Result<PathBuf, String> {
        let executable = env::current_exe()
            .map_err(|_| "cannot get current executable")?;
        match executable
            .parent()
            .map(|x| x.to_path_buf()) {
            None => Err("cannot get parent directory".to_string()),
            Some(v) => Ok(v),
        }
    }

    pub fn call_arguments(&self) -> Vec<String> {
        return env::args().collect();
    }

    pub fn shell(&self) -> Result<String, &str> {
        return env::var("SHELL")
            .map_err(|_| "SHELL environment variable is not defined");
    }
}

pub fn system_environment() -> Environment {
    let sys_env = SystemEnvironment {};
    return Environment {
        executable_name: sys_env.executable_name().unwrap(),
        executable_dir: sys_env.executable_dir().unwrap(),
        args: sys_env.call_arguments(),
        shell: sys_env.shell().unwrap(),
    };
}


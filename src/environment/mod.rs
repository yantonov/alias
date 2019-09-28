use std::env;
use std::path::PathBuf;

pub struct Environment {
    args: Vec<String>
}

impl Environment {
    pub fn executable_dir(&self) -> PathBuf {
        let executable = env::current_exe()
            .unwrap();
        return executable
            .parent()
            .unwrap()
            .to_path_buf();
    }

    pub fn get_call_arguments(&self) -> &[String] {
        return &self.args[1..self.args.len()];
    }

    pub fn get_shell(&self) -> String {
        return env::var("SHELL")
            .expect("SHELL environment variable is not defined");
    }
}

pub fn get_environment() -> Environment {
    return Environment {args: env::args().collect()};
}

use std::env;
use std::path::PathBuf;

use regex::{Captures, Regex};

pub struct Environment {
    args: Vec<String>
}

impl Environment {
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

    pub fn get_call_arguments(&self) -> &[String] {
        return &self.args[1..self.args.len()];
    }

    pub fn get_shell(&self) -> Result<String, &str> {
        return env::var("SHELL")
            .map_err(|_| "SHELL environment variable is not defined");
    }
}

pub fn get_environment() -> Environment {
    return Environment { args: env::args().collect() };
}

pub fn expand_env_var(path: &str) -> String {
    let re = Regex::new(r"(\$\{[^{}]+\})").unwrap();
    let expanded = re.replace_all(
        path,
        |captures: &Captures| -> String {
            let env_var = captures
                .get(1)
                .unwrap()
                .as_str()
                .to_string();
            return env::var(&env_var[2..(env_var.len() - 1)])
                .unwrap_or(env_var);
        });
    return expanded.into_owned();
}


#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn expand_existing_var() {
        env::set_var("ENV_VAR", "yes");
        assert_eq!("yes/replaced",
                   expand_env_var("${ENV_VAR}/replaced"));
    }

    #[test]
    fn not_existing_var_wait_unmodified_string() {
        assert_eq!("${NOT_EXISTING_VAR}/not_replaced",
                   expand_env_var("${NOT_EXISTING_VAR}/not_replaced"));
    }
}

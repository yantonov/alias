use std::env;
use std::path::PathBuf;
use regex::{Regex, Captures};

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
            return env::var(&env_var[2..(env_var.len()-1)])
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

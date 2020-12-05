use std::env;
use regex::{Captures, Regex};

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
    use std::path::Path;

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

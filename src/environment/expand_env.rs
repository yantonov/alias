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
            env::var(&env_var[2..(env_var.len() - 1)])
                .unwrap_or(env_var)
        });
    expanded.into_owned()
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn expand_existing_var() {
        unsafe {
                env::set_var("ENV_VAR", "yes");
        }
        assert_eq!("yes/replaced",
                   expand_env_var("${ENV_VAR}/replaced"));
    }

    #[test]
    fn not_existing_var_wait_unmodified_string() {
        assert_eq!("${NOT_EXISTING_VAR}/not_replaced",
                   expand_env_var("${NOT_EXISTING_VAR}/not_replaced"));
    }

    #[test]
    fn multiple_vars_in_one_string_are_all_expanded() {
        unsafe {
            env::set_var("EXPAND_MULTI_A", "foo");
            env::set_var("EXPAND_MULTI_B", "bar");
        }
        assert_eq!("foo/bar", expand_env_var("${EXPAND_MULTI_A}/${EXPAND_MULTI_B}"));
    }

    #[test]
    fn adjacent_vars_are_both_expanded() {
        unsafe {
            env::set_var("EXPAND_ADJ_A", "hello");
            env::set_var("EXPAND_ADJ_B", "world");
        }
        assert_eq!("helloworld", expand_env_var("${EXPAND_ADJ_A}${EXPAND_ADJ_B}"));
    }

    #[test]
    fn empty_string_returns_empty_string() {
        assert_eq!("", expand_env_var(""));
    }

    #[test]
    fn string_without_vars_is_returned_unchanged() {
        assert_eq!("no/vars/here", expand_env_var("no/vars/here"));
    }
}

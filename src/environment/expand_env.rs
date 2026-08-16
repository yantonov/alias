use std::env;

// Expands ${NAME} placeholders. A placeholder is '${', a non-empty name that
// contains no braces, and '}'. Anything that does not fit the shape, and any
// name that is not set in the environment, is left in place untouched.
pub fn expand_env_var(path: &str) -> String {
    let mut result = String::with_capacity(path.len());
    let mut rest = path;

    while let Some(marker) = rest.find("${") {
        let (before, placeholder) = rest.split_at(marker);
        result.push_str(before);

        let name = placeholder[2..]
            .split_once('}')
            .map(|(name, _)| name)
            .filter(|name| !name.is_empty() && !name.contains('{'));

        match name {
            Some(name) => {
                let placeholder_len = name.len() + "${}".len();
                match env::var(name) {
                    Ok(value) => result.push_str(&value),
                    Err(_) => result.push_str(&placeholder[..placeholder_len]),
                }
                rest = &placeholder[placeholder_len..];
            }
            None => {
                // Not a placeholder after all, resume scanning past the '$'.
                result.push('$');
                rest = &placeholder[1..];
            }
        }
    }

    result.push_str(rest);
    result
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
        assert_eq!("yes/replaced", expand_env_var("${ENV_VAR}/replaced"));
    }

    #[test]
    fn not_existing_var_wait_unmodified_string() {
        assert_eq!(
            "${NOT_EXISTING_VAR}/not_replaced",
            expand_env_var("${NOT_EXISTING_VAR}/not_replaced")
        );
    }

    #[test]
    fn multiple_vars_in_one_string_are_all_expanded() {
        unsafe {
            env::set_var("EXPAND_MULTI_A", "foo");
            env::set_var("EXPAND_MULTI_B", "bar");
        }
        assert_eq!(
            "foo/bar",
            expand_env_var("${EXPAND_MULTI_A}/${EXPAND_MULTI_B}")
        );
    }

    #[test]
    fn adjacent_vars_are_both_expanded() {
        unsafe {
            env::set_var("EXPAND_ADJ_A", "hello");
            env::set_var("EXPAND_ADJ_B", "world");
        }
        assert_eq!(
            "helloworld",
            expand_env_var("${EXPAND_ADJ_A}${EXPAND_ADJ_B}")
        );
    }

    #[test]
    fn empty_string_returns_empty_string() {
        assert_eq!("", expand_env_var(""));
    }

    #[test]
    fn string_without_vars_is_returned_unchanged() {
        assert_eq!("no/vars/here", expand_env_var("no/vars/here"));
    }

    #[test]
    fn empty_name_is_not_a_placeholder() {
        assert_eq!("${}", expand_env_var("${}"));
    }

    #[test]
    fn unterminated_placeholder_is_left_alone() {
        assert_eq!("${NOT_CLOSED", expand_env_var("${NOT_CLOSED"));
        assert_eq!("${", expand_env_var("${"));
    }

    #[test]
    fn name_containing_a_brace_is_not_a_placeholder() {
        assert_eq!("${A{B}", expand_env_var("${A{B}"));
    }

    #[test]
    fn scanning_resumes_after_a_rejected_placeholder() {
        unsafe {
            env::set_var("EXPAND_NESTED", "value");
        }
        // The outer '${' cannot be a placeholder because of the inner brace,
        // so only the inner one is expanded.
        assert_eq!("${value}", expand_env_var("${${EXPAND_NESTED}}"));
    }

    #[test]
    fn lone_dollar_signs_are_kept() {
        assert_eq!("$", expand_env_var("$"));
        assert_eq!("$$", expand_env_var("$$"));
    }

    #[test]
    fn text_around_a_placeholder_is_preserved() {
        unsafe {
            env::set_var("EXPAND_SURROUNDED", "mid");
        }
        assert_eq!(
            "pre/mid/post",
            expand_env_var("pre/${EXPAND_SURROUNDED}/post")
        );
    }
}

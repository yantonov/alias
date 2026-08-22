use std::fs;
use std::path::{Path, PathBuf};
use toml::map::Map;

use crate::environment::Environment;
use toml::Value;
use toml::value::Value::Table;

pub struct Configuration {
    config: Value,
}

pub enum Alias {
    ShellAlias(String),
    RegularAlias(Vec<String>),
}

pub enum AliasNode {
    Leaf(String),
    Group(Vec<(String, AliasNode)>),
}

fn resolve_in_table(
    table: &Map<String, Value>,
    args: &[String],
    consumed: usize,
) -> Result<Option<(Alias, usize)>, String> {
    if args.is_empty() {
        return Ok(None);
    }
    match table.get(&args[0]) {
        None => Ok(None),
        Some(v) => {
            if let Some(s) = v.as_str() {
                let alias =
                    parse_alias_str(s).map_err(|e| format!("bad alias '{}': {}", args[0], e))?;
                Ok(Some((alias, consumed + 1)))
            } else if let Some(t) = v.as_table() {
                resolve_in_table(t, &args[1..], consumed + 1)
            } else {
                Ok(None)
            }
        }
    }
}

fn build_alias_tree(table: &Map<String, Value>) -> Vec<(String, AliasNode)> {
    let mut entries: Vec<(String, AliasNode)> = table
        .iter()
        .filter_map(|(k, v)| {
            if let Some(s) = v.as_str() {
                Some((k.clone(), AliasNode::Leaf(s.to_string())))
            } else {
                v.as_table()
                    .map(|t| (k.clone(), AliasNode::Group(build_alias_tree(t))))
            }
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

// Splits an alias into arguments the same way git splits its own aliases. The
// rules are spelled out one per test below; an unterminated quote is an error
// rather than something quietly handed over to the target program.
fn split_arguments(value: &str) -> Result<Vec<String>, String> {
    let mut arguments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut started = false;
    let mut quote: Option<char> = None;
    let mut characters = value.chars();

    while let Some(c) = characters.next() {
        match quote {
            None if c.is_whitespace() => {
                if started {
                    arguments.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                started = true;
            }
            Some(opening) if c == opening => {
                quote = None;
            }
            _ => {
                started = true;
                if c == '\\' && quote != Some('\'') {
                    match characters.next() {
                        Some(escaped) => current.push(escaped),
                        None => return Err("ends with a backslash".to_string()),
                    }
                } else {
                    current.push(c);
                }
            }
        }
    }

    if quote.is_some() {
        return Err("unclosed quote".to_string());
    }
    if started {
        arguments.push(current);
    }
    Ok(arguments)
}

fn parse_alias_str(value: &str) -> Result<Alias, String> {
    if value.starts_with('!') {
        // Shell aliases are handed to the shell verbatim, it does its own
        // splitting.
        Ok(Alias::ShellAlias(value.chars().skip(1).collect()))
    } else {
        Ok(Alias::RegularAlias(split_arguments(value)?))
    }
}

impl Configuration {
    fn get_key(&self, key: &str) -> Result<&Value, String> {
        match self.config.get(key) {
            None => Err(format!("key '{}' was not found at configuration file", key)),
            Some(v) => Ok(v),
        }
    }

    fn value_as_str(&self, key: &str, value: &Value) -> Result<String, String> {
        match value.as_str() {
            None => Err(format!("'{}' key has no string type", key)),
            Some(v) => Ok(v.to_string()),
        }
    }

    fn value_as_boolean(&self, key: &str, value: &Value) -> Result<bool, String> {
        match value {
            Value::Boolean(bool_value) => Ok(*bool_value),
            _ => Err(format!("'{}' key has no boolean type", key)),
        }
    }

    pub fn get_executable(&self) -> Result<Option<String>, String> {
        let key = "executable";
        match self.get_key(key) {
            Ok(value) => {
                let as_str = self.value_as_str(key, value)?;
                Ok(Some(as_str))
            }
            Err(_) => Ok(None),
        }
    }

    pub fn get_run_as_shell(&self) -> Result<Option<bool>, String> {
        let key = "run_as_shell";
        match self.get_key(key) {
            Ok(value) => {
                let as_str = self.value_as_boolean(key, value)?;
                Ok(Some(as_str))
            }
            Err(_) => Ok(None),
        }
    }

    pub fn resolve_alias(&self, args: &[String]) -> Result<Option<(Alias, usize)>, String> {
        match self.config.get("alias").and_then(|v| v.as_table()) {
            Some(table) => resolve_in_table(table, args, 0),
            None => Ok(None),
        }
    }

    pub fn list_alias_tree(&self) -> Vec<(String, AliasNode)> {
        match self.config.get("alias").and_then(|v| v.as_table()) {
            Some(table) => build_alias_tree(table),
            None => vec![],
        }
    }
}

pub fn get_config_path(executable_dir: &Path) -> PathBuf {
    let config_file_name = "config.toml";

    executable_dir.join(config_file_name)
}

pub fn get_config_override_path(executable_dir: &Path) -> PathBuf {
    let config_file_name = "override.toml";

    executable_dir.join(config_file_name)
}

pub fn merge(config: &Configuration, override_config: &Configuration) -> Configuration {
    Configuration {
        config: merge_values(&config.config, &override_config.config),
    }
}

fn merge_values(v1: &Value, v2: &Value) -> Value {
    match v1 {
        Table(source_table) => match v2 {
            Table(other_table) => {
                let mut result = source_table.clone();
                for (key, value) in other_table.iter() {
                    let new_value = match result.get(key) {
                        None => value.clone(),
                        Some(old) => merge_values(&old.clone(), &value.clone()),
                    };
                    result.insert(key.clone(), new_value);
                }
                Table(result)
            }
            _ => v1.clone(),
        },
        _ => v2.clone(),
    }
}

// A detected path is serialized rather than pasted into a quoted string by
// hand. Windows paths are full of backslashes, and inside a toml basic string
// those are escapes: most of them are invalid and reject the file outright,
// while a few are valid and quietly turn part of the path into a control
// character. Either way the config the app writes for itself is unusable on
// the very next run.
fn executable_line(detected: Option<String>) -> String {
    match detected {
        Some(path) => format!("executable={}", Value::String(path)),
        None => "#executable=\"not-found\"".to_string(),
    }
}

// The sample config is a convenience, not a prerequisite. A wrapper is
// routinely installed into a directory nobody can write to: /usr/local/bin
// owned by root, an immutable image, the read only nix store. Failing to start
// there would take down plain forwarding as well, which needs nothing from the
// config file at all.
//
// So a sample that cannot be created is stepped over, and quietly: a proxy
// that does its job has no business printing a warning on every single call.
// The absence is reported by --aliases, which is the screen someone opens when
// the aliases they expected are not there.
fn create_config_if_needed(config_file_path: &Path, environment: &Environment) {
    if config_file_path.exists() {
        return;
    }

    let sample_config_content = format!(
        "{}\n\n[alias]\ntest_alias1=\"--help\"\n",
        executable_line(environment.try_detect_executable())
    );

    // Written in a single call: a half written file is worse than no file,
    // since the next launch would read it back as the configuration.
    let _ = fs::write(config_file_path, sample_config_content);
}

pub fn read_configuration(config_file_path: &Path) -> Result<Configuration, String> {
    let contents = fs::read_to_string(config_file_path).map_err(|_| {
        format!(
            "Something went wrong while reading the config file: {}",
            config_file_path.display()
        )
    })?;

    let config = contents.parse::<Value>().map_err(|e| {
        format!(
            "[ERROR] Cannot parse config file: {}. {}",
            config_file_path.display(),
            e
        )
    })?;

    Ok(Configuration { config })
}

pub fn empty_configuration() -> Configuration {
    Configuration {
        config: Table(Map::new()),
    }
}

// A file that is not there is not an error: without config.toml the wrapper
// forwards everything to the target, and override.toml is optional to begin
// with. A file that does exist and cannot be read or parsed is a different
// matter entirely, and is reported.
fn read_configuration_if_present(config_file_path: &Path) -> Result<Configuration, String> {
    if config_file_path.exists() {
        read_configuration(config_file_path)
    } else {
        Ok(empty_configuration())
    }
}

pub fn get_configuration(environment: &Environment) -> Result<Configuration, String> {
    let executable_dir = environment.executable_dir();
    let config_file_path = get_config_path(executable_dir);
    create_config_if_needed(&config_file_path, environment);
    let configuration = read_configuration_if_present(&config_file_path)?;

    let config_override_file_path = get_config_override_path(executable_dir);
    let override_configuration = read_configuration_if_present(&config_override_file_path)?;

    Ok(merge(&configuration, &override_configuration))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;

    fn get_table(section_name: &str, alias_name: &str, alias_value: &str) -> Value {
        let mut table: Map<String, Value> = Map::new();
        let mut section: Map<String, Value> = Map::new();
        section.insert(
            alias_name.to_string(),
            Value::String(alias_value.to_string()),
        );
        table.insert(section_name.to_string(), Table(section));
        Table(table)
    }

    #[test]
    fn values_from_both_tables_are_kept() {
        let origin = get_table("section", "first", "value1");
        let override_config = get_table("section", "second", "value2");
        let result = merge_values(&origin, &override_config);
        let section = result.get("section").expect("'section' not found");
        assert!(section.is_table());
        assert_eq!("value1", section.get("first").unwrap().as_str().unwrap());
        assert_eq!("value2", section.get("second").unwrap().as_str().unwrap());
    }

    #[test]
    fn a_section_only_in_the_override_is_added() {
        let origin = get_table("section1", "first", "value1");
        let override_config = get_table("section2", "second", "value2");
        let result = merge_values(&origin, &override_config);
        assert!(result.get("section1").is_some());
        assert!(result.get("section2").is_some());
    }

    #[test]
    fn the_override_wins_when_both_define_the_same_key() {
        let origin = get_table("section", "key", "value1");
        let override_config = get_table("section", "key", "value2");
        let result = merge_values(&origin, &override_config);
        let maybe_section = result.get("section");
        assert!(maybe_section.is_some());
        let section = maybe_section.unwrap();
        assert_eq!("value2", section.get("key").unwrap().as_str().unwrap());
    }

    fn parse_config(toml: &str) -> Configuration {
        Configuration {
            config: toml.parse::<Value>().expect("invalid test toml"),
        }
    }

    #[test]
    fn a_flat_alias_resolves_and_consumes_its_name() {
        let config = parse_config("[alias]\nco = \"checkout main\"");
        match config.resolve_alias(&["co".to_string()]).unwrap() {
            Some((Alias::RegularAlias(args), consumed)) => {
                assert_eq!(args, vec!["checkout", "main"]);
                assert_eq!(consumed, 1);
            }
            _ => panic!("expected RegularAlias with consumed=1"),
        }
    }

    #[test]
    fn an_alias_in_a_group_consumes_the_group_name_as_well() {
        let config = parse_config("[alias.docker]\nps = \"container ls\"");
        match config
            .resolve_alias(&["docker".to_string(), "ps".to_string()])
            .unwrap()
        {
            Some((Alias::RegularAlias(args), consumed)) => {
                assert_eq!(args, vec!["container", "ls"]);
                assert_eq!(consumed, 2);
            }
            _ => panic!("expected RegularAlias with consumed=2"),
        }
    }

    #[test]
    fn an_alias_in_a_nested_group_consumes_every_level_above_it() {
        let config = parse_config("[alias.docker.container]\nls = \"container ls\"");
        match config
            .resolve_alias(&[
                "docker".to_string(),
                "container".to_string(),
                "ls".to_string(),
            ])
            .unwrap()
        {
            Some((Alias::RegularAlias(args), consumed)) => {
                assert_eq!(args, vec!["container", "ls"]);
                assert_eq!(consumed, 3);
            }
            _ => panic!("expected RegularAlias with consumed=3"),
        }
    }

    #[test]
    fn a_name_that_matches_no_alias_resolves_to_nothing() {
        let config = parse_config("[alias]\nfoo = \"bar\"");
        assert!(
            config
                .resolve_alias(&["unknown".to_string()])
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_group_name_on_its_own_resolves_to_nothing() {
        let config = parse_config("[alias.docker]\nps = \"container ls\"");
        assert!(
            config
                .resolve_alias(&["docker".to_string()])
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_shell_alias_in_a_nested_group_keeps_its_command() {
        let config = parse_config("[alias.docker.container]\nclean = \"!docker system prune\"");
        match config
            .resolve_alias(&[
                "docker".to_string(),
                "container".to_string(),
                "clean".to_string(),
            ])
            .unwrap()
        {
            Some((Alias::ShellAlias(cmd), consumed)) => {
                assert_eq!(cmd, "docker system prune");
                assert_eq!(consumed, 3);
            }
            _ => panic!("expected ShellAlias with consumed=3"),
        }
    }

    #[test]
    fn the_alias_tree_carries_flat_aliases_and_every_level_of_a_group() {
        let config = parse_config(
            "[alias]\nfoo = \"bar\"\n\n[alias.docker.container]\nls = \"container ls\"",
        );
        let tree = config.list_alias_tree();
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].0, "docker");
        assert_eq!(tree[1].0, "foo");
        match &tree[1].1 {
            AliasNode::Leaf(v) => assert_eq!(v, "bar"),
            _ => panic!("expected Leaf for foo"),
        }
        match &tree[0].1 {
            AliasNode::Group(children) => {
                assert_eq!(children.len(), 1);
                assert_eq!(children[0].0, "container");
                match &children[0].1 {
                    AliasNode::Group(sub) => {
                        assert_eq!(sub.len(), 1);
                        assert_eq!(sub[0].0, "ls");
                        match &sub[0].1 {
                            AliasNode::Leaf(v) => assert_eq!(v, "container ls"),
                            _ => panic!("expected Leaf for ls"),
                        }
                    }
                    _ => panic!("expected Group for container"),
                }
            }
            _ => panic!("expected Group for docker"),
        }
    }

    #[test]
    fn executable_is_read_as_a_string() {
        assert_eq!(
            Some("/usr/bin/git".to_string()),
            parse_config("executable = \"/usr/bin/git\"")
                .get_executable()
                .unwrap()
        );
        assert_eq!(None, empty_configuration().get_executable().unwrap());
    }

    #[test]
    fn executable_that_is_not_a_string_is_rejected() {
        let error = parse_config("executable = 42")
            .get_executable()
            .expect_err("a number is not a string");
        assert!(
            error.contains("executable"),
            "the key has to be named: {}",
            error
        );
    }

    #[test]
    fn run_as_shell_is_read_as_a_boolean() {
        assert_eq!(
            Some(true),
            parse_config("run_as_shell = true")
                .get_run_as_shell()
                .unwrap()
        );
        assert_eq!(
            Some(false),
            parse_config("run_as_shell = false")
                .get_run_as_shell()
                .unwrap()
        );
        assert_eq!(None, empty_configuration().get_run_as_shell().unwrap());
    }

    #[test]
    fn run_as_shell_that_is_not_a_boolean_is_rejected() {
        let error = parse_config("run_as_shell = \"yes\"")
            .get_run_as_shell()
            .expect_err("a string is not a boolean");
        assert!(
            error.contains("run_as_shell"),
            "the key has to be named: {}",
            error
        );
    }

    fn split(value: &str) -> Vec<String> {
        split_arguments(value).expect("expected the value to split cleanly")
    }

    #[test]
    fn runs_of_whitespace_do_not_produce_empty_arguments() {
        assert_eq!(vec!["checkout", "main"], split("checkout  main"));
        // Deliberate deviation from git: git turns a trailing run of
        // whitespace into one empty argument, which is exactly the kind of
        // silent junk in argv this splitting is meant to remove.
        assert_eq!(vec!["checkout", "main"], split("  checkout\tmain  "));
    }

    #[test]
    fn double_quotes_group_an_argument() {
        assert_eq!(
            vec!["commit", "-m", "wip message"],
            split("commit -m \"wip message\"")
        );
    }

    #[test]
    fn single_quotes_group_an_argument() {
        assert_eq!(
            vec!["commit", "-m", "wip message"],
            split("commit -m 'wip message'")
        );
    }

    #[test]
    fn quotes_of_the_other_kind_are_literal_inside_a_quoted_argument() {
        assert_eq!(vec!["-m", "has\"dq"], split("-m 'has\"dq'"));
        assert_eq!(
            vec!["-m", "mixed 'inner' quotes"],
            split("-m \"mixed 'inner' quotes\"")
        );
    }

    #[test]
    fn backslash_escapes_the_next_character_outside_single_quotes() {
        assert_eq!(vec!["-m", "a b"], split("-m a\\ b"));
        assert_eq!(vec!["-m", "a\"b"], split("-m \"a\\\"b\""));
        // no C style escapes: \n is a literal n
        assert_eq!(vec!["-m", "anb"], split("-m \"a\\nb\""));
    }

    #[test]
    fn backslash_is_literal_inside_single_quotes() {
        assert_eq!(vec!["-m", "a\\ b"], split("-m 'a\\ b'"));
    }

    #[test]
    fn quotes_can_produce_an_empty_argument() {
        assert_eq!(vec!["run", ""], split("run \"\""));
    }

    #[test]
    fn an_empty_value_produces_no_arguments() {
        assert!(split("").is_empty());
        assert!(split("   ").is_empty());
    }

    #[test]
    fn a_trailing_backslash_is_rejected() {
        assert_eq!(
            Err("ends with a backslash".to_string()),
            split_arguments("-m a\\")
        );
        assert_eq!(
            Err("ends with a backslash".to_string()),
            split_arguments("\\")
        );
        // inside single quotes a backslash is an ordinary character
        assert_eq!(vec!["a\\"], split("'a\\'"));
    }

    #[test]
    fn an_unclosed_quote_is_rejected() {
        assert_eq!(
            Err("unclosed quote".to_string()),
            split_arguments("-m unbalanced\"quote")
        );
        assert_eq!(
            Err("unclosed quote".to_string()),
            split_arguments("-m 'still open")
        );
    }

    #[test]
    fn the_unclosed_quote_error_names_the_alias() {
        let config = parse_config("[alias]\npsn = \"ps --format=\\\"unclosed\"");
        let error = match config.resolve_alias(&["psn".to_string()]) {
            Err(error) => error,
            Ok(_) => panic!("expected an error for the unclosed quote"),
        };
        assert!(
            error.contains("psn"),
            "error should name the alias: {}",
            error
        );
        assert!(
            error.contains("unclosed quote"),
            "error should say why: {}",
            error
        );
    }

    #[test]
    fn a_shell_alias_is_not_split() {
        let config = parse_config("[alias]\nclean = \"!rm -rf  *.tmp\"");
        match config.resolve_alias(&["clean".to_string()]).unwrap() {
            Some((Alias::ShellAlias(cmd), _)) => assert_eq!("rm -rf  *.tmp", cmd),
            _ => panic!("expected ShellAlias"),
        }
    }

    #[test]
    fn a_quoted_argument_survives_alias_resolution() {
        let config = parse_config("[alias]\nci = 'commit -m \"wip\"'");
        match config.resolve_alias(&["ci".to_string()]).unwrap() {
            Some((Alias::RegularAlias(args), _)) => {
                assert_eq!(vec!["commit", "-m", "wip"], args);
            }
            _ => panic!("expected RegularAlias"),
        }
    }

    #[test]
    fn a_detected_windows_path_is_written_as_valid_toml() {
        // '\n' and '\t' are deliberate: those two are valid toml escapes, so a
        // hand quoted path carrying them parses and comes back corrupted.
        let detected = r"tools\new\target dir\app";
        let line = executable_line(Some(detected.to_string()));
        let parsed = line
            .parse::<Value>()
            .expect("the generated line has to be valid toml");
        assert_eq!(
            detected,
            parsed.get("executable").unwrap().as_str().unwrap()
        );
    }

    #[test]
    fn an_undetected_executable_is_written_as_a_comment() {
        let line = executable_line(None);
        assert!(
            line.starts_with('#'),
            "expected a commented out line, got: {}",
            line
        );
        assert!(line.parse::<Value>().unwrap().get("executable").is_none());
    }

    #[test]
    fn a_config_file_that_is_not_there_is_created() {
        let dir = tempfile::tempdir().unwrap();
        let env = Environment::for_testing(dir.path().to_path_buf());
        let result = get_configuration(&env);
        assert!(result.is_ok(), "expected Ok but got: {:?}", result.err());
        assert!(
            dir.path().join("config.toml").exists(),
            "config.toml should have been created"
        );
    }

    // A directory that does not exist stands in for one that cannot be written
    // to: creating a file inside it fails the same way on every operating
    // system, while a read only bit does not (windows lets a file be created
    // in a directory carrying that attribute).
    #[test]
    fn a_config_file_that_cannot_be_created_leaves_the_configuration_empty() {
        let dir = tempfile::tempdir().unwrap();
        let unwritable = dir.path().join("missing");
        let env = Environment::for_testing(unwritable.clone());

        let config = get_configuration(&env)
            .expect("a config file that cannot be created is not a fatal error");

        assert!(
            config.resolve_alias(&["co".to_string()]).unwrap().is_none(),
            "there are no aliases without a config file"
        );
        assert!(!unwritable.join("config.toml").exists());
    }

    #[test]
    fn a_config_file_that_is_there_is_read() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "[alias]\nco = \"checkout main\"\n",
        )
        .unwrap();
        let env = Environment::for_testing(dir.path().to_path_buf());
        let config = get_configuration(&env).unwrap();
        match config.resolve_alias(&["co".to_string()]).unwrap() {
            Some((Alias::RegularAlias(args), _)) => assert_eq!(args, vec!["checkout", "main"]),
            _ => panic!("expected RegularAlias"),
        }
    }

    #[test]
    fn aliases_from_the_override_file_are_added() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "[alias]\nco = \"checkout main\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("override.toml"),
            "[alias]\nst = \"status\"\n",
        )
        .unwrap();
        let env = Environment::for_testing(dir.path().to_path_buf());
        let config = get_configuration(&env).unwrap();
        assert!(
            config.resolve_alias(&["co".to_string()]).unwrap().is_some(),
            "co from config.toml should be present"
        );
        assert!(
            config.resolve_alias(&["st".to_string()]).unwrap().is_some(),
            "st from override.toml should be present"
        );
    }

    #[test]
    fn an_alias_the_override_file_redefines_is_replaced() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "[alias]\nco = \"checkout main\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("override.toml"),
            "[alias]\nco = \"checkout develop\"\n",
        )
        .unwrap();
        let env = Environment::for_testing(dir.path().to_path_buf());
        let config = get_configuration(&env).unwrap();
        match config.resolve_alias(&["co".to_string()]).unwrap() {
            Some((Alias::RegularAlias(args), _)) => assert_eq!(args, vec!["checkout", "develop"]),
            _ => panic!("expected RegularAlias"),
        }
    }
}

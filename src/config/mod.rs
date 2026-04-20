use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use toml::map::Map;

use crate::environment::Environment;
use toml::value::Value::Table;
use toml::Value;

pub struct Configuration {
    config: Value,
}

pub enum Alias {
    ShellAlias(String),
    RegularAlias(Vec<String>),
}

fn parse_alias_str(value: &str) -> Alias {
    if value.starts_with('!') {
        Alias::ShellAlias(value.chars().skip(1).collect())
    } else {
        Alias::RegularAlias(value.split(' ').map(|t| t.to_string()).collect())
    }
}

impl Configuration {
    fn get_key(&self, key: &str) -> Result<&Value, String> {
        match self.config.get(key) {
            None => Err(format!("key '{}' was not found at configuration file", key)),
            Some(v) => Ok(v),
        }
    }

    fn value_as_str<'a>(&self, key: &str, value: &'a Value) -> Result<String, String> {
        match value.as_str() {
            None => Err(format!("'{}' key has no string type", key)),
            Some(v) => Ok(v.to_string()),
        }
    }

    fn value_as_boolean<'a>(&self, key: &str, value: &'a Value) -> Result<bool, String> {
        match value {
            Value::Boolean(bool_value) => { Ok(*bool_value) }
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
            Err(_) => {
                Ok(None)
            }
        }
    }

    pub fn get_run_as_shell(&self) -> Result<Option<bool>, String> {
        let key = "run_as_shell";
        match self.get_key(key) {
            Ok(value) => {
                let as_str = self.value_as_boolean(key, value)?;
                Ok(Some(as_str))
            }
            Err(_) => {
                Ok(None)
            }
        }
    }

    pub fn get_alias(&self, command: &str) -> Result<Option<Alias>, String> {
        let alias = self.get_key("alias")?;

        Ok(match alias.get(command) {
            Some(a) if a.is_table() => None,
            Some(a) => Some(parse_alias_str(&self.value_as_str(command, a)?)),
            None => None,
        })
    }

    pub fn is_group(&self, name: &str) -> bool {
        self.config
            .get("alias")
            .and_then(|a| a.get(name))
            .map(|v| v.is_table())
            .unwrap_or(false)
    }

    pub fn get_group_alias(&self, group: &str, name: &str) -> Result<Option<Alias>, String> {
        let group_table = match self.config
            .get("alias")
            .and_then(|a| a.get(group))
            .and_then(|g| g.as_table())
        {
            Some(t) => t,
            None => return Ok(None),
        };
        Ok(match group_table.get(name) {
            Some(v) => {
                let s = v.as_str()
                    .ok_or_else(|| format!("'{}.{}' key has no string type", group, name))?;
                Some(parse_alias_str(s))
            }
            None => None,
        })
    }

    pub fn list_aliases(&self) -> Vec<(String, String)> {
        match self.config.get("alias").and_then(|x| x.as_table()) {
            Some(table) => {
                let mut aliases: Vec<(String, String)> = table.iter()
                    .filter_map(|(key, value)| {
                        value.as_str().map(|s| (key.clone(), s.to_string()))
                    })
                    .collect();
                aliases.sort_by(|a, b| a.0.cmp(&b.0));
                aliases
            }
            _ => vec![]
        }
    }

    pub fn list_groups(&self) -> Vec<String> {
        match self.config.get("alias").and_then(|x| x.as_table()) {
            Some(table) => {
                let mut groups: Vec<String> = table.iter()
                    .filter(|(_, v)| v.is_table())
                    .map(|(k, _)| k.clone())
                    .collect();
                groups.sort();
                groups
            }
            None => vec![],
        }
    }

    pub fn list_group_aliases(&self, group: &str) -> Vec<(String, String)> {
        match self.config
            .get("alias")
            .and_then(|a| a.get(group))
            .and_then(|g| g.as_table())
        {
            Some(table) => {
                let mut aliases: Vec<(String, String)> = table.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect();
                aliases.sort_by(|a, b| a.0.cmp(&b.0));
                aliases
            }
            None => vec![],
        }
    }
}

pub fn get_config_path(executable_dir: &Path) -> PathBuf {
    let config_file_name = "config.toml";

    executable_dir
        .join(config_file_name)
}

pub fn get_config_override_path(executable_dir: &Path) -> PathBuf {
    let config_file_name = "override.toml";

    executable_dir
        .join(config_file_name)
}

pub fn merge(config: &Configuration,
             override_config: &Configuration) -> Configuration {
    Configuration {
        config: merge_values(&config.config,
                             &override_config.config)
    }
}

fn merge_values(v1: &Value,
                v2: &Value) -> Value {
    match v1 {
        Table(source_table) => {
            match v2 {
                Table(other_table) => {
                    let mut result = source_table.clone();
                    for (key, value) in other_table
                        .iter() {
                        let new_value = match result.get(key) {
                            None => {
                                value.clone()
                            }
                            Some(old) => {
                                merge_values(&old.clone(),
                                             &value.clone())
                            }
                        };
                        result.insert(key.clone(), new_value);
                    }
                    Table(result)
                }
                _ => {
                    v1.clone()
                }
            }
        }
        _ => {
            v2.clone()
        }
    }
}

fn create_config_if_needed(config_file_path: &Path, environment: &Environment) -> Result<(), String> {
    if !config_file_path.exists() {
        let mut f = File::create(config_file_path)
            .map_err(|_| format!("Unable to create {} file", config_file_path.display()))?;

        let auto_detect_executable = environment.try_detect_executable();

        let sample_config_content = [
            &auto_detect_executable.map(|x| format!("executable=\"{}\"", x))
                .unwrap_or("#executable=\"not-found\"".to_string()),
            "",
            "[alias]",
            "test_alias1=\"--help\""
        ];
        for line in &sample_config_content {
            f.write_all(line.as_bytes()).map_err(|_| "Unable to write data")?;
            f.write_all("\n".as_bytes()).map_err(|_| "Unable to write data")?;
        }
    }
    Ok(())
}

pub fn read_configuration(config_file_path: &Path) -> Result<Configuration, String> {
    let contents = fs::read_to_string(config_file_path)
        .map_err(|_| format!("Something went wrong while reading the config file: {}",
                             config_file_path.display()))?;

    let config = contents
        .parse::<Value>()
        .map_err(|e|
            format!("[ERROR] Cannot parse config file: {}. {}",
                    config_file_path.display(),
                    e))?;

    Ok(Configuration { config })
}

pub fn empty_configuration() -> Configuration {
    Configuration { config: Table(Map::new()) }
}

pub fn get_configuration(environment: &Environment) -> Result<Configuration, String> {
    let executable_dir = environment.executable_dir();
    let config_file_path = get_config_path(executable_dir);
    create_config_if_needed(&config_file_path, environment)?;
    let configuration = read_configuration(&config_file_path)?;

    let config_override_file_path = get_config_override_path(executable_dir);
    let override_configuration = if config_override_file_path.exists() {
        read_configuration(&config_override_file_path)?
    } else {
        empty_configuration()
    };

    Ok(merge(&configuration, &override_configuration))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;

    fn get_table(section_name: &str, alias_name: &str, alias_value: &str) -> Value {
        let mut table: Map<String, Value> = Map::new();
        let mut section: Map<String, Value> = Map::new();
        section.insert(alias_name.to_string(), Value::String(alias_value.to_string()));
        table.insert(section_name.to_string(), Table(section));
        return Table(table);
    }

    #[test]
    fn merge_aliases() {
        let origin = get_table("section", "first", "value1");
        let override_config = get_table("section", "second", "value2");
        let result = merge_values(&origin, &override_config);
        let section = result.get("section").expect("'section' not found");
        assert!(section.is_table());
        assert_eq!("value1", section.get("first").unwrap().as_str().unwrap());
        assert_eq!("value2", section.get("second").unwrap().as_str().unwrap());
    }

    #[test]
    fn add_new_section() {
        let origin = get_table("section1", "first", "value1");
        let override_config = get_table("section2", "second", "value2");
        let result = merge_values(&origin, &override_config);
        assert!(result.get("section1").is_some());
        assert!(result.get("section2").is_some());
    }

    #[test]
    fn redefine_alias() {
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
    fn shell_alias_strips_bang_prefix() {
        let config = parse_config("[alias]\nfoo = \"!echo hello\"");
        match config.get_alias("foo").unwrap().unwrap() {
            Alias::ShellAlias(cmd) => assert_eq!(cmd, "echo hello"),
            Alias::RegularAlias(_) => panic!("expected ShellAlias"),
        }
    }

    #[test]
    fn regular_alias_is_split_into_args() {
        let config = parse_config("[alias]\nco = \"checkout main\"");
        match config.get_alias("co").unwrap().unwrap() {
            Alias::RegularAlias(args) => assert_eq!(args, vec!["checkout", "main"]),
            Alias::ShellAlias(_) => panic!("expected RegularAlias"),
        }
    }

    #[test]
    fn unknown_alias_returns_none() {
        let config = parse_config("[alias]\nfoo = \"bar\"");
        assert!(config.get_alias("baz").unwrap().is_none());
    }

    #[test]
    fn list_aliases_returns_unquoted_string_values() {
        let config = parse_config("[alias]\nco = \"checkout main\"\nst = \"status\"");
        let aliases = config.list_aliases();
        assert_eq!(aliases, vec![
            ("co".to_string(), "checkout main".to_string()),
            ("st".to_string(), "status".to_string()),
        ]);
    }

    #[test]
    fn list_aliases_excludes_group_entries() {
        let config = parse_config("[alias]\nfoo = \"bar\"\n\n[alias.docker]\nps = \"ps -a\"");
        let aliases = config.list_aliases();
        assert_eq!(aliases, vec![("foo".to_string(), "bar".to_string())]);
    }

    #[test]
    fn get_alias_returns_none_for_group_key() {
        let config = parse_config("[alias.docker]\nps = \"ps -a\"");
        assert!(config.get_alias("docker").unwrap().is_none());
    }

    #[test]
    fn is_group_true_for_nested_table() {
        let config = parse_config("[alias.docker]\nps = \"ps -a\"");
        assert!(config.is_group("docker"));
    }

    #[test]
    fn is_group_false_for_flat_alias() {
        let config = parse_config("[alias]\nfoo = \"bar\"\n\n[alias.docker]\nps = \"ps -a\"");
        assert!(!config.is_group("foo"));
        assert!(!config.is_group("unknown"));
    }

    #[test]
    fn get_group_alias_resolves_regular() {
        let config = parse_config("[alias.docker]\nps = \"container ls\"");
        match config.get_group_alias("docker", "ps").unwrap().unwrap() {
            Alias::RegularAlias(args) => assert_eq!(args, vec!["container", "ls"]),
            _ => panic!("expected RegularAlias"),
        }
    }

    #[test]
    fn get_group_alias_resolves_shell() {
        let config = parse_config("[alias.docker]\nclean = \"!docker system prune\"");
        match config.get_group_alias("docker", "clean").unwrap().unwrap() {
            Alias::ShellAlias(cmd) => assert_eq!(cmd, "docker system prune"),
            _ => panic!("expected ShellAlias"),
        }
    }

    #[test]
    fn get_group_alias_returns_none_for_unknown_subcommand() {
        let config = parse_config("[alias.docker]\nps = \"container ls\"");
        assert!(config.get_group_alias("docker", "unknown").unwrap().is_none());
    }

    #[test]
    fn list_groups_returns_sorted_names() {
        let config = parse_config("[alias]\nfoo = \"bar\"\n\n[alias.k8s]\nget = \"get pods\"\n\n[alias.docker]\nps = \"ps -a\"");
        assert_eq!(config.list_groups(), vec!["docker", "k8s"]);
    }

    #[test]
    fn list_group_aliases_returns_sorted_entries() {
        let config = parse_config("[alias.docker]\nrun = \"container run\"\nexec = \"exec -it\"");
        assert_eq!(config.list_group_aliases("docker"), vec![
            ("exec".to_string(), "exec -it".to_string()),
            ("run".to_string(), "container run".to_string()),
        ]);
    }

    #[test]
    fn get_configuration_creates_default_config_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let env = Environment::for_testing(dir.path().to_path_buf());
        let result = get_configuration(&env);
        assert!(result.is_ok(), "expected Ok but got: {:?}", result.err());
        assert!(dir.path().join("config.toml").exists(), "config.toml should have been created");
    }

    #[test]
    fn get_configuration_reads_existing_config() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "[alias]\nco = \"checkout main\"\n",
        ).unwrap();
        let env = Environment::for_testing(dir.path().to_path_buf());
        let config = get_configuration(&env).unwrap();
        match config.get_alias("co").unwrap().unwrap() {
            Alias::RegularAlias(args) => assert_eq!(args, vec!["checkout", "main"]),
            _ => panic!("expected RegularAlias"),
        }
    }

    #[test]
    fn get_configuration_merges_override_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "[alias]\nco = \"checkout main\"\n",
        ).unwrap();
        std::fs::write(
            dir.path().join("override.toml"),
            "[alias]\nst = \"status\"\n",
        ).unwrap();
        let env = Environment::for_testing(dir.path().to_path_buf());
        let config = get_configuration(&env).unwrap();
        assert!(config.get_alias("co").unwrap().is_some(), "co from config.toml should be present");
        assert!(config.get_alias("st").unwrap().is_some(), "st from override.toml should be present");
    }

    #[test]
    fn get_configuration_override_replaces_existing_alias() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "[alias]\nco = \"checkout main\"\n",
        ).unwrap();
        std::fs::write(
            dir.path().join("override.toml"),
            "[alias]\nco = \"checkout develop\"\n",
        ).unwrap();
        let env = Environment::for_testing(dir.path().to_path_buf());
        let config = get_configuration(&env).unwrap();
        match config.get_alias("co").unwrap().unwrap() {
            Alias::RegularAlias(args) => assert_eq!(args, vec!["checkout", "develop"]),
            _ => panic!("expected RegularAlias"),
        }
    }
}
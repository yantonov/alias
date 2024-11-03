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

        Ok(
            match alias.get(command) {
                Some(a) => {
                    let alias_value = self.value_as_str(command, a)?;

                    if alias_value.starts_with('!') {
                        let shell_command: String = alias_value
                            .chars()
                            .skip(1)
                            .collect();
                        Some(Alias::ShellAlias(shell_command))
                    } else {
                        let alias_arguments: Vec<String> = alias_value
                            .split(' ')
                            .into_iter()
                            .map(|t| t.to_string())
                            .collect();
                        Some(Alias::RegularAlias(alias_arguments))
                    }
                }
                None => {
                    None
                }
            })
    }

    pub fn list_aliases(&self) -> Vec<(String, String)> {
        match self.config.get("alias").and_then(|x| x.as_table()) {
            Some(table) => {
                let mut aliases: Vec<(String, String)> = table.iter()
                    .map(|(key, value)| (key.clone(), format!("{}", value)))
                    .collect();
                aliases.sort_by(|a, b| a.0.cmp(&b.0));
                aliases
            }
            _ => vec![]
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
        let config_file_path_str =
            match config_file_path.to_str() {
                None => Err("cannot convert path to string"),
                Some(v) => Ok(v),
            }?;

        let mut f = File::create(config_file_path)
            .map_err(|_| format!("Unable to create {} file", config_file_path_str))?;

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
                             config_file_path.to_str().unwrap()))?;

    let config = contents
        .parse::<Value>()
        .map_err(|e|
            format!("[ERROR] Cannot parse config file: {}. {}",
                    config_file_path.to_str().unwrap(),
                    e))?;

    Ok(Configuration { config })
}

pub fn empty_configuration() -> Configuration {
    Configuration { config: Table(Map::new()) }
}

pub fn get_configuration(environment: &Environment) -> Result<Configuration, String> {
    let executable_dir = environment.executable_dir();
    let config_file_path = get_config_path(executable_dir);
    create_config_if_needed(&config_file_path, &environment)
        .unwrap();
    let configuration = read_configuration(&config_file_path);
    if configuration.is_err() {
        return configuration;
    };

    let config_override_file_path = get_config_override_path(executable_dir);
    let override_configuration =
        if config_override_file_path.exists() {
            read_configuration(
                &config_override_file_path)
        } else {
            Ok(empty_configuration())
        };
    if override_configuration.is_err() {
        return override_configuration;
    }

    Ok(merge(
        &configuration.unwrap(),
        &override_configuration.unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let maybe_section = result.get("section");
        match maybe_section {
            None => {
                assert!(false, "'section' not found")
            }
            _ => {}
        }
        let section = maybe_section.unwrap();
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
}
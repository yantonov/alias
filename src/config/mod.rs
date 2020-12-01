use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use toml::Value;
use std::collections::BTreeMap;
use toml::value::Value::Table;

pub struct Configuration {
    config: Value
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

    pub fn get_executable(&self) -> Result<String, String> {
        let key = "executable";
        let value = self.get_key(key)?;
        let as_str = self.value_as_str(key, value)?;
        Ok(as_str)
    }

    pub fn get_alias(&self, command: &str) -> Result<Option<Alias>, String> {
        let alias = self.get_key("alias")?;

        Ok(
            match alias.get(command) {
                Some(a) => {
                    let alias_value = self.value_as_str(command, a)?;

                    if alias_value.starts_with("!") {
                        let shell_command: String = alias_value
                            .chars()
                            .skip(1)
                            .collect();
                        Some(Alias::ShellAlias(shell_command))
                    } else {
                        let alias_arguments: Vec<String> = alias_value
                            .split(" ")
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
                return aliases;
            }
            _ => vec![]
        }
    }
}

pub fn get_config_path(executable_dir: &PathBuf) -> PathBuf {
    let config_file_name = "config.toml";

    return executable_dir
        .as_path()
        .join(config_file_name);
}

pub fn create_config_if_needed(config_file_path: &PathBuf) -> Result<(), String> {
    if !config_file_path.exists() {
        let config_file_path_str =
            match config_file_path.to_str() {
                None => Err("cannot convert path to string"),
                Some(v) => Ok(v),
            }?;

        let mut f = File::create(config_file_path)
            .map_err(|_| format!("Unable to create {} file", config_file_path_str))?;

        let sample_config_content = [
            "executable=\"/bin/bash\"",
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

pub fn read_configuration(config_file_path: &PathBuf) -> Result<Configuration, String> {
    let contents = fs::read_to_string(config_file_path)
        .map_err(|_| format!("Something went wrong while reading the config file: {}",
                             config_file_path.to_str().unwrap()))?;

    let config = contents
        .parse::<Value>()
        .map_err(|e|
            format!("[ERROR] Cannot parse config file: {}. {}",
                    config_file_path.to_str().unwrap(),
                    e))?;

    return Ok(Configuration { config });
}

pub fn empty_configuration() -> Configuration {
    Configuration { config: Table(BTreeMap::new()) }
}


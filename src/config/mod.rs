use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use toml::Value;

pub struct Configuration {
    config: Value
}

pub enum Alias {
    ShellAlias(String),
    RegularAlias(Vec<String>),
}

impl Configuration {
    fn get_key(&self, key: &str) -> &Value {
        return self
            .config
            .get(key)
            .expect(&format!("key '{}' was not found at configuration file", key));
    }

    fn value_as_str<'a>(&self, key: &str, value: &'a Value) -> &'a str {
        return value
            .as_str()
            .expect(&format!("'{}' key has no string type", key));
    }

    pub fn get_executable(&self) -> String {
        let key = "executable";
        return self
            .value_as_str(
                key,
                self.get_key(key))
            .to_string();
    }

    pub fn get_alias(&self, command: &str) -> Option<Alias> {
        let alias = self
            .get_key("alias")
            .get(command);

        match alias {
            Some(a) => {
                let alias_value = self.value_as_str(command, a);
                if alias_value.starts_with("!") {
                    let shell_command: String = alias_value
                        .chars()
                        .skip(1)
                        .collect();
                    return Some(Alias::ShellAlias(shell_command));
                } else {
                    let alias_arguments: Vec<String> = alias_value
                        .split(" ")
                        .into_iter()
                        .map(|t| t.to_string())
                        .collect();
                    return Some(Alias::RegularAlias(alias_arguments));
                }
            }
            None => {
                return None;
            }
        }
    }
}

pub fn get_config_path(executable_dir: PathBuf) -> PathBuf {
    let config_file_name = "config.toml";

    return executable_dir
        .as_path()
        .join(config_file_name);
}

pub fn create_config_if_needed(config_file_path: &PathBuf) {
    if !config_file_path.exists() {
        let mut f = File::create(config_file_path)
            .expect(&format!(
                "Unable to create {} file",
                config_file_path.to_str().unwrap()));
        let sample_config_content = [
            "executable=\"/bin/bash\"",
            "",
            "[alias]",
            "test_alias1=\"--help\""
        ];
        for line in &sample_config_content {
            f.write_all(line.as_bytes()).expect("Unable to write data");
            f.write_all("\n".as_bytes()).expect("Unable to write data");
        }
    }
}

pub fn read_configuration(config_file_path: &PathBuf) -> Configuration {
    let contents = fs::read_to_string(config_file_path)
        .expect("Something went wrong reading the config file");

    let config = contents
        .parse::<Value>()
        .expect("Error while parsing config file");

    return Configuration { config };
}


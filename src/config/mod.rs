use std::fs;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

use toml::Value;

pub struct Configuration {
    config: Value
}

pub enum Alias {
    ShellAlias(String),
    RegularAlias(Vec<String>)
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
            .to_string()
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
                }
                else {
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

pub fn read_configuration(executable_dir: PathBuf) -> Configuration {
    let config_file_name = "config.toml";

    let config_file_path = executable_dir
        .as_path()
        .join(config_file_name);

    let p = config_file_path.as_path();
    if !p.exists() {
        let mut f = File::create(p).expect(format!("Unable to create {} file", config_file_name));
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

    let contents = fs::read_to_string(config_file_path)
        .expect("Something went wrong reading the config file");

    let config = contents
        .parse::<Value>()
        .expect("Error while parsing config file");

    return Configuration {config: config};
}


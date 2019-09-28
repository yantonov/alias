use std::fs;
use std::path::PathBuf;

use toml::Value;

pub struct Configuration {
    config: Value
}

impl Configuration {
    pub fn get_executable(&self) -> &str {
        return self.config["executable"].as_str().unwrap()
    }

    pub fn get_alias(&self, command: &str) -> Option<&Value> {
        return self.config["alias"].get(command);
    }
}

pub fn read_configuration(executable_dir: PathBuf) -> Configuration {
    let config_file_path = executable_dir.as_path().join("config.toml");

    let contents = fs::read_to_string(config_file_path)
        .expect("Something went wrong reading the config file");

    let config = contents
        .parse::<Value>()
        .unwrap();

    return Configuration {config: config};
}


use std::fs;
use std::path::PathBuf;

use toml::Value;

pub struct Configuration {
    config: Value
}

impl Configuration {
    fn get_key(&self, key: &str) -> &Value {
        return self
            .config
            .get(key)
            .expect(&format!("key '{}' was not found at configuration file", key));
    }

    fn get_string_key(&self, key: &str) -> &str {
        return self
            .get_key(key)
            .as_str()
            .expect(&format!("'{}' key has no string type", key));
    }

    pub fn get_executable(&self) -> &str {
        return self
            .get_string_key("executable");
    }

    pub fn get_alias(&self, command: &str) -> Option<&Value> {
        return self
            .get_key("alias")
            .get(command);
    }
}

pub fn read_configuration(executable_dir: PathBuf) -> Configuration {
    let config_file_path = executable_dir
        .as_path()
        .join("config.toml");

    if !config_file_path.exists() {
        panic!("config.toml file was not found")
    }

    let contents = fs::read_to_string(config_file_path)
        .expect("Something went wrong reading the config file");

    let config = contents
        .parse::<Value>()
        .expect("Error while parsing config file");

    return Configuration {config: config};
}


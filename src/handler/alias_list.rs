use crate::config::{AliasNode, Configuration, get_config_path};
use crate::environment::Environment;
use crate::handler::{Handler, passthrough};

fn print_tree(entries: &[(String, AliasNode)], indent: &str) {
    let mut printed = false;
    for (name, node) in entries {
        if let AliasNode::Leaf(value) = node {
            println!("{}{} = {}", indent, name, value);
            printed = true;
        }
    }
    for (name, node) in entries {
        if let AliasNode::Group(children) = node {
            if printed {
                println!();
            }
            println!("{}{}:", indent, name);
            print_tree(children, &format!("{}  ", indent));
            printed = true;
        }
    }
}

// The config file is created on the first launch, unless the directory the
// wrapper sits in cannot be written to, and then there is nowhere to define an
// alias in. The wrapper keeps forwarding commands either way, so this is the
// one place the difference between 'no aliases configured' and 'no config file
// at all' can be told.
fn missing_config_report(environment: &Environment) -> Option<String> {
    let config_file_path = get_config_path(environment.executable_dir());
    if config_file_path.exists() {
        return None;
    }
    Some(format!(
        "no aliases: {} does not exist and could not be created",
        config_file_path.display()
    ))
}

pub struct AliasListHandler {}

impl Handler for AliasListHandler {
    fn handle(&self, environment: &Environment, configuration: &Configuration) {
        let entries = configuration.list_alias_tree();
        if entries.is_empty() {
            // On stderr, so that the listing itself stays pipeable.
            if let Some(report) = missing_config_report(environment) {
                eprintln!("{}", report);
            }
        }
        print_tree(&entries, "");
        passthrough::try_passthrough(environment, configuration, &["--aliases"]);
    }
}

impl AliasListHandler {
    pub fn new() -> AliasListHandler {
        AliasListHandler {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_config_file_that_is_not_there_is_reported_by_its_path() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let environment = Environment::for_testing(directory.path().to_path_buf());

        let report = missing_config_report(&environment).expect("there is no config file");

        assert!(
            report.contains(&get_config_path(directory.path()).display().to_string()),
            "the path has to be named: {}",
            report
        );
    }

    #[test]
    fn a_config_file_that_exists_is_not_reported() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        std::fs::write(get_config_path(directory.path()), "[alias]\n").expect("a config file");
        let environment = Environment::for_testing(directory.path().to_path_buf());

        assert!(missing_config_report(&environment).is_none());
    }
}

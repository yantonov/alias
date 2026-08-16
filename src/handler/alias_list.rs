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
// at all' can be told. On stderr, so that the listing itself stays pipeable.
fn report_missing_config(environment: &Environment) {
    let config_file_path = get_config_path(environment.executable_dir());
    if !config_file_path.exists() {
        eprintln!(
            "no aliases: {} does not exist and could not be created",
            config_file_path.display()
        );
    }
}

pub struct AliasListHandler {}

impl Handler for AliasListHandler {
    fn handle(&self, environment: &Environment, configuration: &Configuration) {
        let entries = configuration.list_alias_tree();
        if entries.is_empty() {
            report_missing_config(environment);
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

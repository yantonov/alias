use crate::config::{AliasNode, Configuration};
use crate::environment::Environment;
use crate::handler::{passthrough, Handler};

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
            if printed { println!(); }
            println!("{}{}:", indent, name);
            print_tree(children, &format!("{}  ", indent));
            printed = true;
        }
    }
}

pub struct AliasListHandler {}

impl Handler for AliasListHandler {
    fn handle(&self,
              environment: &Environment,
              configuration: &Configuration) {
        print_tree(&configuration.list_alias_tree(), "");
        passthrough::try_passthrough(environment, configuration, &["--aliases"]);
    }
}

impl AliasListHandler {
    pub fn new() -> AliasListHandler {
        AliasListHandler {}
    }
}
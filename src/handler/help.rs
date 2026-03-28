use crate::config::Configuration;
use crate::environment::Environment;
use crate::handler::Handler;

pub struct HelpHandler {}

impl Handler for HelpHandler {
    fn handle(&self,
              _environment: &Environment,
              _configuration: &Configuration) {
        println!("alias v{}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("A thin wrapper that adds alias support to any command-line tool.");
        println!("Place this executable in PATH under the same name as the target program,");
        println!("then define aliases in config.toml next to the executable.");
        println!("Aliases are expanded transparently — use them just like built-in subcommands.");
        println!();
        println!("USAGE:");
        println!("    <tool> [ALIAS|ARGS...]");
        println!();
        println!("OPTIONS:");
        println!("    --aliases    List all configured aliases");
        println!("    --version    Print version");
        println!("    --help       Print this help message");
    }
}

impl HelpHandler {
    pub fn new() -> HelpHandler {
        HelpHandler {}
    }
}

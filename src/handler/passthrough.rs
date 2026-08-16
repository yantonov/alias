use crate::config::Configuration;
use crate::environment::Environment;
use crate::handler::get_executable;
use crate::process::{self, CallContext};

pub fn try_passthrough(environment: &Environment, configuration: &Configuration, args: &[&str]) {
    let executable = match get_executable(environment, configuration) {
        Ok(Some(exe)) => exe,
        _ => return,
    };

    let call_context = CallContext {
        executable,
        args: args.iter().map(|s| s.to_string()).collect(),
    };

    let _ = process::try_execute_captured(&call_context);
}

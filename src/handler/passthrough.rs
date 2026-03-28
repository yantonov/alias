use crate::config::Configuration;
use crate::environment::{expand_env, Environment};
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

fn get_executable(environment: &Environment, configuration: &Configuration) -> Result<Option<String>, String> {
    Ok(configuration.get_executable()?
        .map(|config| expand_env::expand_env_var(&config))
        .or_else(|| environment.try_detect_executable()))
}

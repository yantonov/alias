use std::ffi::OsStr;
use std::process::{Command};

pub struct CallContext {
    pub executable: String,
    pub args: Vec<String>,
}

fn exec<I, S>(executable: &str, args: I) -> Result<(), String>
    where I: IntoIterator<Item=S>,
          S: AsRef<OsStr>
{
    let mut output = Command::new(executable)
        .args(args)
        .spawn()
        .map_err(|_| "failed to execute process")?;

    output.wait()
        .map(|_| ())
        .map_err(|_| "failed to wait child process".to_owned())
}

pub fn execute(context: &CallContext) -> Result<(), String> {
    exec(&context.executable, &context.args)
}

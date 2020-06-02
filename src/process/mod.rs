use std::ffi::OsStr;
use std::io::{self, Write};
use std::process::{Command, Stdio};

pub struct CallContext {
    pub executable: String,
    pub args: Vec<String>,
}

fn exec<I, S>(executable: &str, args: I) -> Result<(), String>
    where I: IntoIterator<Item=S>,
          S: AsRef<OsStr>
{
    let output = Command::new(executable)
        .args(args)
        .stdout(Stdio::inherit())
        .output()
        .map_err(|_| "failed to execute process")?;

    io::stdout()
        .write_all(&output.stdout)
        .map_err(|_| "cannot redirect stdout")?;

    io::stderr()
        .write_all(&output.stderr)
        .map_err(|_| "cannot redirect stderr")?;

    let status = output.status;
    std::process::exit(status.code().unwrap())
}

pub fn execute(context: &CallContext) -> Result<(), String> {
    exec(&context.executable, &context.args)
}

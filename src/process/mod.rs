use std::process::{Command, Stdio};
use std::io::{self, Write};
use std::ffi::OsStr;

pub struct CallContext {
    pub executable: String,
    pub args: Vec<String>
}

fn exec<I, S>(executable: &str, args: I)
where I: IntoIterator<Item = S>,
      S: AsRef<OsStr>
{
    let output = Command::new(executable)
        .args(args)
        .stdout(Stdio::inherit())
        .output()
        .expect("failed to execute process");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    let status = output.status;
    std::process::exit(status.code().unwrap());
}

pub fn execute(context: &CallContext) {
    exec(&context.executable, &context.args);
}

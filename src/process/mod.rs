use std::ffi::OsStr;
use std::process::Command;

pub struct CallContext {
    pub executable: String,
    pub args: Vec<String>,
}

fn exec<I, S>(executable: &str, args: I) -> Result<Option<i32>, String>
    where I: IntoIterator<Item=S>,
          S: AsRef<OsStr>
{
    let mut output = Command::new(executable)
        .args(args)
        .spawn()
        .map_err(|_| "failed to execute process")?;

    output.wait()
        .map(|r| r.code())
        .map_err(|_| "failed to wait child process".to_string())
}

pub fn execute(context: &CallContext) -> Result<Option<i32>, String> {
    exec(&context.executable, &context.args)
}

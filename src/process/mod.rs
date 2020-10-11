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

pub fn exit(result: Result<Option<i32>, String>) {
    let unknown_exit_code = -1;
    match result {
        Ok(exit_code) => {
            std::process::exit(exit_code.unwrap_or(unknown_exit_code));
        }
        Err(_) => {
            std::process::exit(unknown_exit_code);
        }
    }
}

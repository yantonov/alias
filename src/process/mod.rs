use std::process::Command;

pub struct CallContext {
    pub executable: String,
    pub args: Vec<String>,
}

fn format_command(executable: &str, args: &[String]) -> String {
    std::iter::once(executable)
        .chain(args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ")
}

fn exec(executable: &str,
        args: &[String]) -> Result<Option<i32>, String>
{
    let mut output = Command::new(executable)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to execute process [{}]. {}",
                             format_command(executable, args), e))?;

    output.wait()
        .map(|r| r.code())
        .map_err(|e| format!("Failed to wait child process [{}]. {}",
                             format_command(executable, args), e))
}

pub fn execute(context: &CallContext) -> Result<Option<i32>, String> {
    exec(&context.executable, &context.args)
}

pub fn try_execute_captured(context: &CallContext) -> Result<Option<i32>, String> {
    let output = Command::new(&context.executable)
        .args(&context.args)
        .output()
        .map_err(|e| format!("Failed to execute process [{}]. {}",
                             format_command(&context.executable, &context.args), e))?;

    let code = output.status.code();
    if code == Some(0) {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(code)
}

pub fn exit(code: Option<i32>) -> ! {
    std::process::exit(code.unwrap_or(-1));
}

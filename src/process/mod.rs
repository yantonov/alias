use std::process::Command;

pub struct CallContext {
    pub executable: String,
    pub args: Vec<String>,
}

fn get_command(executable: &str,
               args: &[String]) -> String
{
    let mut tokens: Vec<String> = vec![
        executable.to_string()
    ];
    for arg in args {
        tokens.push(arg.clone());
    }
    tokens.join(" ")
}

fn exec(executable: &str,
        args: &[String]) -> Result<Option<i32>, String>
{
    let pretty_printed_command = get_command(executable, args);

    let mut output = Command::new(executable)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to execute process [{}]. {}",
                             pretty_printed_command,
                             e))?;

    output.wait()
        .map(|r| r.code())
        .map_err(|e| format!("Failed to wait child process [{}]. {}",
                             pretty_printed_command,
                             e))
}

pub fn execute(context: &CallContext) -> Result<Option<i32>, String> {
    exec(&context.executable, &context.args)
}

pub fn exit(code: Option<i32>) -> ! {
    std::process::exit(code.unwrap_or(-1));
}

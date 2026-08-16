use std::process::{Command, ExitStatus};

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

// A child terminated by a signal has no exit code of its own. Report it the way
// shells do, as 128 + signal number, instead of collapsing it into a failure
// that is indistinguishable from any other.
#[cfg(unix)]
fn exit_code(status: ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.code().or_else(|| status.signal().map(|signal| 128 + signal))
}

#[cfg(not(unix))]
fn exit_code(status: ExitStatus) -> Option<i32> {
    status.code()
}

// Replace this process with the target instead of spawning a child, which is
// what a thin wrapper should do: no extra link in the process tree, signals go
// straight to the target, and its exit status reaches the caller untouched.
// Returns only when exec itself failed.
#[cfg(unix)]
fn run(executable: &str,
       args: &[String]) -> Result<Option<i32>, String>
{
    use std::os::unix::process::CommandExt;

    let error = Command::new(executable)
        .args(args)
        .exec();

    Err(format!("Failed to execute process [{}]. {}",
                format_command(executable, args), error))
}

// Windows has no exec, so the target runs as a child process.
#[cfg(not(unix))]
fn run(executable: &str,
       args: &[String]) -> Result<Option<i32>, String>
{
    let mut output = Command::new(executable)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to execute process [{}]. {}",
                             format_command(executable, args), e))?;

    output.wait()
        .map(exit_code)
        .map_err(|e| format!("Failed to wait child process [{}]. {}",
                             format_command(executable, args), e))
}

pub fn execute(context: &CallContext) -> Result<Option<i32>, String> {
    run(&context.executable, &context.args)
}

pub fn try_execute_captured(context: &CallContext) -> Result<Option<i32>, String> {
    let output = Command::new(&context.executable)
        .args(&context.args)
        .output()
        .map_err(|e| format!("Failed to execute process [{}]. {}",
                             format_command(&context.executable, &context.args), e))?;

    let code = exit_code(output.status);
    if code == Some(0) {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(code)
}

pub fn exit(code: Option<i32>) -> ! {
    std::process::exit(code.unwrap_or(-1));
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;

    #[test]
    fn normal_exit_keeps_its_own_code() {
        assert_eq!(Some(3), exit_code(ExitStatus::from_raw(3 << 8)));
    }

    #[test]
    fn child_killed_by_signal_reports_128_plus_signal() {
        // SIGKILL, the way a shell would report it: 128 + 9
        assert_eq!(Some(137), exit_code(ExitStatus::from_raw(9)));
    }

    #[test]
    fn child_killed_by_sigint_reports_130() {
        assert_eq!(Some(130), exit_code(ExitStatus::from_raw(2)));
    }
}

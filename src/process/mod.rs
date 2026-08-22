use std::borrow::Cow;
use std::env;
use std::io::Read;
use std::process::{Command, ExitStatus, Stdio};

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

// A wrapper that ends up calling itself never stops. 'executable' can point
// back at it, two wrappers can name each other, and a shell alias can invoke
// the very alias it defines: st = "!git st" resolves to itself on every turn.
// That last one is both the easiest to write by accident and the most
// expensive, since every turn leaves a shell behind as well.
//
// Every call the wrapper makes carries the level it was made at, so a loop of
// any shape reaches the limit instead of running forever. Honest nesting stays
// shallow: a shell alias naming the wrapped program is two levels, and the
// limit leaves room for far more than anyone stacks in practice.
const NESTING: &str = "ALIAS_DEPTH";
const NESTING_LIMIT: u32 = 16;

// The variable belongs to the wrapper: a value it did not write says nothing
// about how deep the call really is.
fn nesting_level() -> u32 {
    env::var(NESTING)
        .ok()
        .and_then(|level| level.parse().ok())
        .unwrap_or(0)
}

pub fn check_nesting_limit() -> Result<(), String> {
    let level = nesting_level();
    if level < NESTING_LIMIT {
        return Ok(());
    }
    Err(format!(
        "{} levels of nested calls ({} is set): something calls this wrapper back. \
Check the 'executable' entry of the config and any shell alias that names the wrapped program.",
        level, NESTING
    ))
}

fn command(context: &CallContext) -> Command {
    let mut command = Command::new(&context.executable);
    command
        .args(&context.args)
        .env(NESTING, (nesting_level() + 1).to_string());
    command
}

// A child terminated by a signal has no exit code of its own, and collapsing
// that into a plain failure makes it indistinguishable from any other.
#[cfg(unix)]
fn exit_code(status: ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status
        .code()
        .or_else(|| status.signal().map(|signal| 128 + signal))
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
fn run(context: &CallContext) -> Result<Option<i32>, String> {
    use std::os::unix::process::CommandExt;

    let error = command(context).exec();

    Err(format!(
        "Failed to execute process [{}]. {}",
        format_command(&context.executable, &context.args),
        error
    ))
}

// Windows has no exec, so the target runs as a child process.
#[cfg(not(unix))]
fn run(context: &CallContext) -> Result<Option<i32>, String> {
    let mut output = command(context).spawn().map_err(|e| {
        format!(
            "Failed to execute process [{}]. {}",
            format_command(&context.executable, &context.args),
            e
        )
    })?;

    output.wait().map(exit_code).map_err(|e| {
        format!(
            "Failed to wait child process [{}]. {}",
            format_command(&context.executable, &context.args),
            e
        )
    })
}

pub fn execute(context: &CallContext) -> Result<Option<i32>, String> {
    run(context)
}

// The flag being forwarded may well be one the target knows nothing about
// (--aliases is ours, not its): the target then complains on stderr and exits
// non-zero, and that complaint is pure noise right after the wrapper printed
// its own answer. So stderr is held back until the exit code says whether the
// target agreed to the flag.
fn presentable_stderr(stderr: &[u8], code: Option<i32>) -> Cow<'_, str> {
    if code == Some(0) {
        String::from_utf8_lossy(stderr)
    } else {
        Cow::Borrowed("")
    }
}

// Only stderr is taken aside. Stdout stays the terminal the wrapper was given,
// so the target sees a tty on it and its help arrives paged and coloured, the
// way it does when the target is called directly.
pub fn try_execute_forwarded(context: &CallContext) -> Result<Option<i32>, String> {
    let mut child = command(context)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            format!(
                "Failed to execute process [{}]. {}",
                format_command(&context.executable, &context.args),
                e
            )
        })?;

    // Read to the end before waiting: a target with more to say than the pipe
    // holds would block on a buffer nobody is draining.
    let mut captured = Vec::new();
    if let Some(mut stream) = child.stderr.take() {
        let _ = stream.read_to_end(&mut captured);
    }

    let code = child.wait().map(exit_code).map_err(|e| {
        format!(
            "Failed to wait child process [{}]. {}",
            format_command(&context.executable, &context.args),
            e
        )
    })?;

    eprint!("{}", presentable_stderr(&captured, code));
    Ok(code)
}

pub const COULD_NOT_RUN: i32 = 127;

pub fn exit(code: Option<i32>) -> ! {
    std::process::exit(code.unwrap_or(COULD_NOT_RUN));
}

#[cfg(test)]
mod presentable_stderr_tests {
    use super::*;

    fn present(stderr: &str, code: Option<i32>) -> String {
        presentable_stderr(stderr.as_bytes(), code).into_owned()
    }

    #[test]
    fn stderr_of_a_target_that_succeeded_is_shown() {
        assert_eq!("a warning", present("a warning", Some(0)));
    }

    #[test]
    fn stderr_of_a_target_that_failed_is_dropped() {
        assert_eq!(
            "",
            present("error: unrecognized option '--aliases'", Some(2))
        );
    }

    #[test]
    fn a_target_that_said_nothing_shows_nothing() {
        assert_eq!("", present("", Some(0)));
        assert_eq!("", present("", None));
    }
}

#[cfg(all(test, unix))]
mod exit_code_tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;

    #[test]
    fn a_normal_exit_keeps_its_own_code() {
        assert_eq!(Some(3), exit_code(ExitStatus::from_raw(3 << 8)));
    }

    #[test]
    fn a_child_killed_by_a_signal_reports_128_plus_the_signal() {
        // SIGKILL, the way a shell would report it: 128 + 9
        assert_eq!(Some(137), exit_code(ExitStatus::from_raw(9)));
    }

    #[test]
    fn a_child_killed_by_sigint_reports_130() {
        assert_eq!(Some(130), exit_code(ExitStatus::from_raw(2)));
    }
}

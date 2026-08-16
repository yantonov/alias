use std::borrow::Cow;
use std::env;
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

// Anything that is not a number is treated as no nesting at all: the variable
// belongs to the wrapper, and a value it did not write says nothing about how
// deep the call really is.
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

// A child terminated by a signal has no exit code of its own. Report it the way
// shells do, as 128 + signal number, instead of collapsing it into a failure
// that is indistinguishable from any other.
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

// Decides what of a captured run is worth showing to the user.
//
// The flag being forwarded may well be one the target knows nothing about
// (--aliases is ours, not its): the target then complains on stderr and exits
// non-zero, and that complaint is pure noise right after the wrapper printed
// its own answer. So stderr is shown only when the target agreed to the flag.
//
// Stdout is shown whatever the exit code, because a tool that prints real help
// and still exits non-zero is common enough to matter, while a tool that has
// nothing to say prints nothing and the user sees nothing either way.
fn presentable_output<'a>(
    stdout: &'a [u8],
    stderr: &'a [u8],
    code: Option<i32>,
) -> (Cow<'a, str>, Cow<'a, str>) {
    let accepted = code == Some(0);
    (
        String::from_utf8_lossy(stdout),
        if accepted {
            String::from_utf8_lossy(stderr)
        } else {
            Cow::Borrowed("")
        },
    )
}

pub fn try_execute_captured(context: &CallContext) -> Result<Option<i32>, String> {
    let output = command(context).output().map_err(|e| {
        format!(
            "Failed to execute process [{}]. {}",
            format_command(&context.executable, &context.args),
            e
        )
    })?;

    let code = exit_code(output.status);
    let (stdout, stderr) = presentable_output(&output.stdout, &output.stderr, code);
    print!("{}", stdout);
    eprint!("{}", stderr);
    Ok(code)
}

pub const COULD_NOT_RUN: i32 = 127;

pub fn exit(code: Option<i32>) -> ! {
    std::process::exit(code.unwrap_or(COULD_NOT_RUN));
}

#[cfg(test)]
mod presentable_output_tests {
    use super::*;

    fn present(stdout: &str, stderr: &str, code: Option<i32>) -> (String, String) {
        let (out, err) = presentable_output(stdout.as_bytes(), stderr.as_bytes(), code);
        (out.into_owned(), err.into_owned())
    }

    #[test]
    fn target_that_accepted_the_flag_shows_both_streams() {
        let (stdout, stderr) = present("usage: git ...", "a warning", Some(0));
        assert_eq!("usage: git ...", stdout);
        assert_eq!("a warning", stderr);
    }

    // The target does not know --aliases: its 'unrecognized option' belongs to
    // the wrapper's own flag, not to the user, and is dropped.
    #[test]
    fn target_that_rejected_the_flag_keeps_stdout_and_drops_stderr() {
        let (stdout, stderr) = present("", "error: unrecognized option '--aliases'", Some(2));
        assert_eq!("", stdout);
        assert_eq!("", stderr);
    }

    // Help printed on stdout survives an unhelpful exit code.
    #[test]
    fn help_printed_with_a_non_zero_exit_code_is_still_shown() {
        let (stdout, _) = present("usage: tool [options]", "", Some(1));
        assert_eq!("usage: tool [options]", stdout);
    }

    #[test]
    fn silent_target_shows_nothing() {
        assert_eq!((String::new(), String::new()), present("", "", Some(0)));
        assert_eq!((String::new(), String::new()), present("", "", None));
    }
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

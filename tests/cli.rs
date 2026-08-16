// End to end tests: they run the built wrapper, not its internals.
//
// The whole point of the program is what ends up in the target's argv, and no
// unit test can see that. Each case gets a private directory holding a copy of
// the wrapper named after the target, a config beside it, and a target program
// that prints the arguments it received, one per line.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{PoisonError, RwLock};

use tempfile::TempDir;

// Writing an executable and spawning one race with each other on linux. Between
// fork and exec a child inherits every open descriptor, so while one test is
// still writing its copy of the wrapper, another test's fork holds that
// descriptor open for writing, and executing the copy fails with ETXTBSY
// ("Text file busy"). O_CLOEXEC does not help: it closes the descriptor at exec,
// after the window that matters.
//
// Setting the files up therefore takes this lock exclusively, running them
// shares it: spawns still overlap each other freely, they just never overlap a
// write.
static EXECUTABLES: RwLock<()> = RwLock::new(());

struct Wrapper {
    // Kept alive: dropping it removes the directory the wrapper lives in.
    _directory: TempDir,
    binary: PathBuf,
}

impl Wrapper {
    // 'aliases' is the part of the config below the executable line.
    fn fronting_argv_printer(aliases: &str) -> Wrapper {
        Wrapper::new(aliases, write_argv_printer)
    }

    fn fronting(aliases: &str, write_target: fn(&Path) -> PathBuf) -> Wrapper {
        Wrapper::new(aliases, write_target)
    }

    fn new(aliases: &str, write_target: fn(&Path) -> PathBuf) -> Wrapper {
        let _guard = EXECUTABLES.write().unwrap_or_else(PoisonError::into_inner);

        let directory = tempfile::tempdir().expect("a temporary directory");
        let target = write_target(&directory.path().join("target-program"));

        // The wrapper takes its identity from its own file name, so the copy is
        // named after the program it fronts rather than after the crate.
        let binary = directory.path().join(executable_file_name("frontend"));
        fs::copy(env!("CARGO_BIN_EXE_alias"), &binary).expect("the wrapper binary is copied");

        let config = format!(
            "executable={}\n\n{}\n",
            as_toml_string(&target.display().to_string()),
            aliases
        );
        fs::write(directory.path().join("config.toml"), config).expect("a config beside it");

        Wrapper {
            _directory: directory,
            binary,
        }
    }

    fn run(&self, arguments: &[&str]) -> Output {
        execute(self.command(arguments))
    }

    fn command(&self, arguments: &[&str]) -> Command {
        let mut command = Command::new(&self.binary);
        command.args(arguments);
        // Never inherit the shell of whoever runs the tests: on windows it may
        // be missing entirely, and the value only matters to shell aliases.
        command.env("SHELL", "/bin/sh");
        command
    }
}

// Every spawn goes through here, so that no fork happens while an executable is
// being written. Shared, not exclusive: spawns may overlap each other.
fn execute(mut command: Command) -> Output {
    let _guard = EXECUTABLES.read().unwrap_or_else(PoisonError::into_inner);
    command.output().expect("the wrapper starts")
}

fn executable_file_name(stem: &str) -> String {
    if cfg!(windows) {
        format!("{}.exe", stem)
    } else {
        stem.to_string()
    }
}

// Windows paths are full of backslashes, which a toml basic string reads as
// escapes. The app has a serializer for this; a test spells it out.
fn as_toml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\"))
}

fn stdout_lines(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

// The target programs. A script with no extension is not something windows can
// start, so there it is a .cmd file, and the wrapper is pointed at it by an
// explicit executable= entry rather than by autodetection.
#[cfg(windows)]
fn write_argv_printer(path: &Path) -> PathBuf {
    let target = path.with_extension("cmd");
    fs::write(
        &target,
        "@echo off\r\n\
               :loop\r\n\
               if \"%~1\"==\"\" goto end\r\n\
               echo %~1\r\n\
               shift\r\n\
               goto loop\r\n\
               :end\r\n",
    )
    .expect("a target program");
    target
}

#[cfg(unix)]
fn write_argv_printer(path: &Path) -> PathBuf {
    write_script(
        path,
        "for argument in \"$@\"; do echo \"$argument\"; done\n",
    )
}

#[cfg(windows)]
fn write_failing_target(path: &Path) -> PathBuf {
    let target = path.with_extension("cmd");
    fs::write(&target, "@echo off\r\nexit /b 3\r\n").expect("a target program");
    target
}

#[cfg(unix)]
fn write_failing_target(path: &Path) -> PathBuf {
    write_script(path, "exit 3\n")
}

#[cfg(unix)]
fn write_script(path: &Path, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let target = path.to_path_buf();
    fs::write(&target, format!("#!/bin/sh\n{}", body)).expect("a target program");
    fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).expect("an executable bit");
    target
}

#[test]
fn a_regular_alias_is_expanded_ahead_of_the_remaining_arguments() {
    let wrapper = Wrapper::fronting_argv_printer("[alias]\nco = \"checkout main\"");

    let output = wrapper.run(&["co", "topic"]);

    assert_eq!(vec!["checkout", "main", "topic"], stdout_lines(&output));
}

// The alias is split the way git splits its own, and the quoted part has to
// survive all the way into the target's argv as one argument.
#[test]
fn a_quoted_part_of_an_alias_arrives_as_a_single_argument() {
    let wrapper = Wrapper::fronting_argv_printer("[alias]\nci = 'commit -m \"work in progress\"'");

    let output = wrapper.run(&["ci"]);

    assert_eq!(
        vec!["commit", "-m", "work in progress"],
        stdout_lines(&output)
    );
}

#[test]
fn a_nested_group_resolves_to_the_alias_at_its_deepest_level() {
    let wrapper = Wrapper::fronting_argv_printer("[alias.docker.container]\nls = \"container ls\"");

    let output = wrapper.run(&["docker", "container", "ls", "--all"]);

    assert_eq!(vec!["container", "ls", "--all"], stdout_lines(&output));
}

#[test]
fn a_command_that_matches_no_alias_is_forwarded_untouched() {
    let wrapper = Wrapper::fronting_argv_printer("[alias]\nco = \"checkout main\"");

    let output = wrapper.run(&["status", "--short"]);

    assert_eq!(vec!["status", "--short"], stdout_lines(&output));
}

// A group name on its own is not an alias, so it goes to the target as typed.
#[test]
fn a_group_without_a_matching_member_is_forwarded_untouched() {
    let wrapper = Wrapper::fronting_argv_printer("[alias.docker]\nps = \"container ls\"");

    let output = wrapper.run(&["docker", "images"]);

    assert_eq!(vec!["docker", "images"], stdout_lines(&output));
}

#[test]
fn the_exit_code_of_the_target_is_the_exit_code_of_the_wrapper() {
    let wrapper = Wrapper::fronting("[alias]\nco = \"checkout\"", write_failing_target);

    let output = wrapper.run(&["co"]);

    assert_eq!(Some(3), output.status.code());
}

#[test]
fn configured_aliases_are_listed() {
    let wrapper = Wrapper::fronting_argv_printer(
        "[alias]\nco = \"checkout main\"\n\n[alias.docker]\nps = \"container ls\"",
    );

    let output = wrapper.run(&["--aliases"]);
    let listing = stdout(&output);

    assert!(
        listing.contains("co = checkout main"),
        "flat alias missing from:\n{}",
        listing
    );
    assert!(
        listing.contains("docker:"),
        "group missing from:\n{}",
        listing
    );
    assert!(
        listing.contains("ps = container ls"),
        "group member missing from:\n{}",
        listing
    );
}

#[test]
fn the_wrapper_reports_its_own_version_before_the_version_of_the_target() {
    let wrapper = Wrapper::fronting_argv_printer("[alias]\nco = \"checkout\"");

    let output = wrapper.run(&["--version"]);
    let printed = stdout_lines(&output);

    assert_eq!(
        format!("alias wrapper version {}", env!("CARGO_PKG_VERSION")),
        printed[0]
    );
    // What follows is the target's answer to --version, forwarded verbatim.
    assert_eq!(Some(&"--version".to_string()), printed.get(1));
}

// The shell is where aliases prefixed with ! are run, and the app refuses to
// start without one rather than guessing. This is what a windows user outside
// git bash runs into.
#[test]
fn a_missing_shell_is_reported_instead_of_being_guessed() {
    let wrapper = Wrapper::fronting_argv_printer("[alias]\nco = \"checkout\"");

    let mut command = wrapper.command(&["co"]);
    command.env_remove("SHELL");
    let output = execute(command);

    assert_eq!(Some(1), output.status.code());
    assert!(
        stderr(&output).contains("SHELL environment variable is not set"),
        "unexpected error: {}",
        stderr(&output)
    );
}

// A dry run has to show the argv the target would have received, and the target
// has to stay untouched: it would print those arguments bare, one per line.
#[test]
fn a_dry_run_prints_the_command_instead_of_running_it() {
    let wrapper = Wrapper::fronting_argv_printer("[alias]\nco = \"checkout main\"");

    let mut command = wrapper.command(&["co", "topic"]);
    command.env("ALIAS_DRY_RUN", "1");
    let printed = stdout(&execute(command));

    assert!(
        printed.contains("[1] checkout"),
        "missing from:\n{}",
        printed
    );
    assert!(printed.contains("[2] main"), "missing from:\n{}", printed);
    assert!(printed.contains("[3] topic"), "missing from:\n{}", printed);
    assert!(
        !printed.lines().any(|line| line == "checkout"),
        "the target ran after all:\n{}",
        printed
    );
}

// The shell alias path needs no shell for a dry run, so what a shell alias
// turns into can be checked on every platform, quoted "$@" included.
#[test]
fn a_dry_run_of_a_shell_alias_shows_what_the_shell_would_get() {
    let wrapper = Wrapper::fronting_argv_printer("[alias]\ntail = \"!docker logs -f\"");

    let mut command = wrapper.command(&["tail", "web"]);
    command.env("ALIAS_DRY_RUN", "1");
    let printed = stdout(&execute(command));

    assert!(printed.contains("[1] -c"), "missing from:\n{}", printed);
    assert!(
        printed.contains("[2] docker logs -f \"$@\""),
        "missing from:\n{}",
        printed
    );
    assert!(printed.contains("[4] web"), "missing from:\n{}", printed);
}

// The config written on the first launch has to be one the app can read on the
// second, which is not a given: the detected path lands inside it, and on
// windows such a path is full of backslashes.
#[test]
fn the_config_created_on_the_first_launch_is_read_back_on_the_second() {
    let wrapper_directory = tempfile::tempdir().expect("a temporary directory");
    let target_directory = tempfile::tempdir().expect("a temporary directory");

    // Same file name in both, the wrapper first: this is the layout the
    // installer produces and the one autodetection is meant to resolve.
    let name = executable_file_name("frontend");
    let binary = wrapper_directory.path().join(&name);
    {
        let _guard = EXECUTABLES.write().unwrap_or_else(PoisonError::into_inner);
        fs::copy(env!("CARGO_BIN_EXE_alias"), &binary).expect("the wrapper binary is copied");
        fs::copy(
            env!("CARGO_BIN_EXE_alias"),
            target_directory.path().join(&name),
        )
        .expect("a target to be detected");
    }

    let path = std::env::join_paths([wrapper_directory.path(), target_directory.path()])
        .expect("a PATH out of two directories");

    let run = || {
        let mut command = Command::new(&binary);
        command
            .arg("--version")
            .env("SHELL", "/bin/sh")
            .env("PATH", &path);
        execute(command)
    };

    let first = run();
    assert_eq!(
        Some(0),
        first.status.code(),
        "first launch failed: {}",
        stderr(&first)
    );

    let created = fs::read_to_string(wrapper_directory.path().join("config.toml"))
        .expect("the first launch creates a config");
    assert!(
        created.contains(&name),
        "the detected target should be in the config:\n{}",
        created
    );

    let second = run();
    assert!(
        !stderr(&second).contains("Cannot parse config file"),
        "the generated config does not parse:\n{}",
        stderr(&second)
    );
    assert_eq!(
        Some(0),
        second.status.code(),
        "second launch failed: {}",
        stderr(&second)
    );
}

// Shell aliases need a real POSIX shell, which is not something windows has by
// itself; the argv they are handed is covered by unit tests on every platform.
#[cfg(unix)]
#[test]
fn a_shell_alias_receives_the_arguments_that_follow_it() {
    let wrapper = Wrapper::fronting_argv_printer("[alias]\nshow = '!printf \"[%s]\\n\"'");

    let output = wrapper.run(&["show", "one", "two words"]);

    assert_eq!(vec!["[one]", "[two words]"], stdout_lines(&output));
}

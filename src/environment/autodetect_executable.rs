use std::env;
use std::path::Path;

pub fn autodetect_executable(executable_path: &Path,
                             executable_name: &str,
                             path_var: &str,
                             fs: &dyn FileSystemWrapper) -> Option<String> {
    let paths: Vec<_> = env::split_paths(path_var).collect();
    let candidates = candidate_names(executable_name);

    // Search only after the wrapper's own directory in PATH, so we skip the
    // wrapper itself and find the real target executable.
    let start = paths
        .iter()
        .position(|p| same_directory(p.as_path(), executable_path))
        .map(|i| i + 1)
        .unwrap_or(0);

    paths[start..].iter().find_map(|path_item| {
        // Never report the wrapper itself: running it would make the process
        // call itself again and again. Only reachable when the lookup above
        // fails to recognize our own directory (8.3 short names, subst drives).
        if same_directory(path_item, executable_path) {
            return None;
        }
        // A directory at a time, every candidate within it before moving on,
        // which is the order windows itself resolves a bare name in: a shim in
        // the first PATH entry wins over an executable in the fifth.
        candidates.iter().find_map(|candidate| {
            let target = path_item.join(candidate);
            if fs.exists(&target) && fs.is_file(&target) {
                target.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
    })
}

// The names the target can go by. The wrapper is always an .exe on windows,
// while the program it fronts often is not: npm and yarn are shipped as .cmd,
// gradle and maven as .bat, and looking for npm.exe alone finds nothing.
//
// PATHEXT is deliberately not consulted. It describes what a shell resolves,
// .vbs and .msc included, and those are not programs but scripts for an
// interpreter: finding one would move the failure from detection to startup.
// The list below is what a process can actually be started from, native first.
#[cfg(windows)]
fn candidate_names(executable_name: &str) -> Vec<String> {
    const EXECUTABLE_EXTENSIONS: [&str; 3] = [".exe", ".cmd", ".bat"];

    let stem = Path::new(executable_name)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(executable_name);

    // The name the wrapper carries comes first, so wherever the target is a
    // plain .exe the search runs exactly as it did before.
    let mut names = vec![executable_name.to_string()];
    for extension in EXECUTABLE_EXTENSIONS {
        let candidate = format!("{}{}", stem, extension);
        if !names.iter().any(|name| name.eq_ignore_ascii_case(&candidate)) {
            names.push(candidate);
        }
    }
    names
}

// Executable extensions are a windows notion. Everywhere else the target goes
// by the name the wrapper carries and by no other.
#[cfg(not(windows))]
fn candidate_names(executable_name: &str) -> Vec<String> {
    vec![executable_name.to_string()]
}

// Windows and the default macOS filesystem are case-insensitive, so the same
// directory can appear in PATH spelled differently from what current_exe()
// reports. Path comparison is case-sensitive, hence the lowercased forms are
// compared as paths, which keeps separator and trailing-slash handling intact.
#[cfg(any(windows, target_os = "macos"))]
fn same_directory(a: &Path, b: &Path) -> bool {
    fn lowercased(path: &Path) -> std::path::PathBuf {
        std::path::PathBuf::from(path.to_string_lossy().to_lowercase())
    }
    a == b || lowercased(a) == lowercased(b)
}

#[cfg(not(any(windows, target_os = "macos")))]
fn same_directory(a: &Path, b: &Path) -> bool {
    a == b
}

pub trait FileSystemWrapper {
    fn exists(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
}

pub struct OsFileSystemWrapper {}

impl FileSystemWrapper for OsFileSystemWrapper {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    // Symlinks are followed on purpose: package managers routinely put a link
    // in PATH pointing at the real binary elsewhere (every homebrew formula
    // does), and such a link is a perfectly good target. What must not be
    // accepted is a directory carrying the target's name.
    fn is_file(&self, path: &Path) -> bool {
        std::fs::metadata(path)
            .map(|metadata| metadata.is_file())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::collections::HashMap;

    #[derive(Clone)]
    struct TestFileDescriptor {
        is_file: bool,
    }

    impl TestFileDescriptor {
        pub fn file() -> TestFileDescriptor {
            TestFileDescriptor { is_file: true }
        }

        pub fn directory() -> TestFileDescriptor {
            TestFileDescriptor { is_file: false }
        }
    }

    struct TestFileSystemWrapper {
        // Keyed by PathBuf, not String: Path compares and hashes by components,
        // so '/bin/alias' and '/bin\alias' are the same key on Windows.
        path_to_descriptor: HashMap<PathBuf, TestFileDescriptor>,
    }

    impl TestFileSystemWrapper {
        pub fn create() -> TestFileSystemWrapper {
            TestFileSystemWrapper {
                path_to_descriptor: HashMap::new(),
            }
        }

        pub fn add(&mut self, path: &str, descriptor: &TestFileDescriptor) {
            self.path_to_descriptor.insert(PathBuf::from(path), (*descriptor).clone());
        }
    }

    impl FileSystemWrapper for TestFileSystemWrapper {
        fn exists(&self, path: &Path) -> bool {
            self.path_to_descriptor.contains_key(path)
        }

        fn is_file(&self, path: &Path) -> bool {
            self.path_to_descriptor.get(path)
                .map(|d| d.is_file)
                .unwrap_or(false)
        }
    }

    // Joins with the platform PATH separator, so the result can be split back
    // by env::split_paths on any operating system.
    fn make_path(entries: &[&str]) -> String {
        env::join_paths(entries)
            .expect("PATH entry contains the separator character")
            .into_string()
            .expect("PATH is not valid UTF-8")
    }

    fn detect(executable_path: &str,
              executable_name: &str,
              path_var: &str,
              fs: &dyn FileSystemWrapper) -> Option<String> {
        autodetect_executable(Path::new(executable_path), executable_name, path_var, fs)
    }

    #[test]
    fn target_executable_can_be_found_later_in_the_path() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/bin/alias", &TestFileDescriptor::file());
        fs.add("/usr/bin/alias", &TestFileDescriptor::file());
        let path = make_path(&["/bin", "/usr/bin"]);
        let autodetect = detect("/bin", "alias", &path, &fs).unwrap();
        assert_eq!(Path::new("/usr/bin/alias"), Path::new(&autodetect));
    }

    #[test]
    fn directory_named_after_the_target_is_skipped() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/bin/alias", &TestFileDescriptor::file());
        fs.add("/usr/bin/alias", &TestFileDescriptor::directory());
        fs.add("/usr/local/bin/alias", &TestFileDescriptor::file());
        let path = make_path(&["/bin", "/usr/bin", "/usr/local/bin"]);
        let autodetect = detect("/bin", "alias", &path, &fs).unwrap();
        assert_eq!(Path::new("/usr/local/bin/alias"), Path::new(&autodetect));
    }

    #[test]
    fn target_executable_cannot_be_found_later_in_the_path() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/home/username/alias_app/alias", &TestFileDescriptor::file());
        fs.add("/home/username/some_app/alias", &TestFileDescriptor::file());
        let path = make_path(&[
            "/home/username/some_app",
            "/home/username/alias_app",
            "/bin",
            "/usr/bin",
        ]);
        assert!(detect("/home/username/alias_app", "alias", &path, &fs).is_none());
    }

    #[test]
    fn alias_path_does_not_exist_in_path() {
        let fs = TestFileSystemWrapper::create();
        let path = make_path(&["/bin", "/usr/bin"]);
        assert!(detect("/home/username/app", "alias", &path, &fs).is_none());
    }

    #[test]
    fn alias_path_exists_but_target_executable_doesnt() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/home/username/app/alias", &TestFileDescriptor::file());
        let path = make_path(&["/home/username/app", "/bin", "/usr/bin"]);
        assert!(detect("/home/username/app", "alias", &path, &fs).is_none());
    }

    #[test]
    fn wrapper_doesnt_exist_in_path_try_to_find_first_executable_that_has_the_same_name() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/usr/bin/alias", &TestFileDescriptor::file());
        let path = make_path(&["/bin", "/usr/bin"]);
        let autodetect = detect("/home/username/app", "alias", &path, &fs).unwrap();
        assert_eq!(Path::new("/usr/bin/alias"), Path::new(&autodetect));
    }

    // The wrapper is always an .exe on windows, while the program it fronts
    // often is not.
    #[cfg(windows)]
    mod executable_extensions {
        use super::*;

        // The layout node ships: a shell script with no extension, a .cmd shim
        // and a .ps1 shim. Only the .cmd one can be started as a process.
        #[test]
        fn cmd_shim_is_found_among_the_shims_of_the_same_tool() {
            let mut fs = TestFileSystemWrapper::create();
            fs.add("/wrapper/npm.exe", &TestFileDescriptor::file());
            fs.add("/tools/npm", &TestFileDescriptor::file());
            fs.add("/tools/npm.ps1", &TestFileDescriptor::file());
            fs.add("/tools/npm.cmd", &TestFileDescriptor::file());
            let path = make_path(&["/wrapper", "/tools"]);
            let autodetect = detect("/wrapper", "npm.exe", &path, &fs).unwrap();
            assert_eq!(Path::new("/tools/npm.cmd"), Path::new(&autodetect));
        }

        #[test]
        fn bat_shim_is_found() {
            let mut fs = TestFileSystemWrapper::create();
            fs.add("/wrapper/gradle.exe", &TestFileDescriptor::file());
            fs.add("/tools/gradle.bat", &TestFileDescriptor::file());
            let path = make_path(&["/wrapper", "/tools"]);
            let autodetect = detect("/wrapper", "gradle.exe", &path, &fs).unwrap();
            assert_eq!(Path::new("/tools/gradle.bat"), Path::new(&autodetect));
        }

        #[test]
        fn the_executable_itself_wins_over_a_shim_beside_it() {
            let mut fs = TestFileSystemWrapper::create();
            fs.add("/wrapper/git.exe", &TestFileDescriptor::file());
            fs.add("/tools/git.cmd", &TestFileDescriptor::file());
            fs.add("/tools/git.exe", &TestFileDescriptor::file());
            let path = make_path(&["/wrapper", "/tools"]);
            let autodetect = detect("/wrapper", "git.exe", &path, &fs).unwrap();
            assert_eq!(Path::new("/tools/git.exe"), Path::new(&autodetect));
        }

        // Directory beats extension, the order windows resolves a bare name
        // in. Reversing the two loops would silently invert it.
        #[test]
        fn a_shim_nearby_wins_over_an_executable_further_along_the_path() {
            let mut fs = TestFileSystemWrapper::create();
            fs.add("/wrapper/npm.exe", &TestFileDescriptor::file());
            fs.add("/tools/npm.cmd", &TestFileDescriptor::file());
            fs.add("/other/npm.exe", &TestFileDescriptor::file());
            let path = make_path(&["/wrapper", "/tools", "/other"]);
            let autodetect = detect("/wrapper", "npm.exe", &path, &fs).unwrap();
            assert_eq!(Path::new("/tools/npm.cmd"), Path::new(&autodetect));
        }
    }

    // Executable extensions are a windows notion: a .cmd next to a unix target
    // is a file that happens to share its name, nothing more.
    #[cfg(not(windows))]
    #[test]
    fn extensions_are_not_appended_outside_windows() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/wrapper/npm", &TestFileDescriptor::file());
        fs.add("/tools/npm.cmd", &TestFileDescriptor::file());
        let path = make_path(&["/wrapper", "/tools"]);
        assert!(detect("/wrapper", "npm", &path, &fs).is_none());
    }

    #[test]
    fn same_directory_matches_identical_paths() {
        assert!(same_directory(Path::new("/usr/bin"), Path::new("/usr/bin")));
    }

    #[test]
    fn same_directory_rejects_different_paths() {
        assert!(!same_directory(Path::new("/usr/bin"), Path::new("/usr/local/bin")));
    }

    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    fn same_directory_ignores_case_on_case_insensitive_filesystems() {
        assert!(same_directory(Path::new("/Users/bob/BIN"), Path::new("/users/bob/bin")));
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    #[test]
    fn same_directory_keeps_case_on_case_sensitive_filesystems() {
        assert!(!same_directory(Path::new("/home/bob/BIN"), Path::new("/home/bob/bin")));
    }

    // The fake filesystem cannot express symlinks, and a link in PATH pointing
    // at the real binary is the normal case on macOS, where homebrew installs
    // every executable that way. Uses the real filesystem for that reason.
    #[cfg(unix)]
    #[test]
    fn symlink_to_the_target_executable_is_detected() {
        let dir = tempfile::tempdir().unwrap();
        let wrapper_dir = dir.path().join("git-aliases");
        let link_dir = dir.path().join("link");
        let real_dir = dir.path().join("real");
        std::fs::create_dir_all(&wrapper_dir).unwrap();
        std::fs::create_dir_all(&link_dir).unwrap();
        std::fs::create_dir_all(&real_dir).unwrap();
        std::fs::write(wrapper_dir.join("git"), b"wrapper").unwrap();
        std::fs::write(real_dir.join("git"), b"the real git").unwrap();
        std::os::unix::fs::symlink(real_dir.join("git"), link_dir.join("git")).unwrap();

        let path_var = env::join_paths([&wrapper_dir, &link_dir])
            .unwrap()
            .into_string()
            .unwrap();

        let detected = autodetect_executable(
            &wrapper_dir,
            "git",
            &path_var,
            &OsFileSystemWrapper {},
        ).expect("the symlink should have been detected");

        assert_eq!(link_dir.join("git"), PathBuf::from(detected));
    }

    // Regression test against self-detection, which makes the wrapper execute
    // itself in an endless chain. Uses the real filesystem, because the point
    // is the case-insensitive behaviour of the volume itself.
    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    fn wrapper_is_skipped_when_path_entry_case_differs_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let wrapper_dir = dir.path().join("uv-aliases");
        let real_dir = dir.path().join("real");
        std::fs::create_dir_all(&wrapper_dir).unwrap();
        std::fs::create_dir_all(&real_dir).unwrap();
        std::fs::write(wrapper_dir.join("uv"), b"wrapper").unwrap();
        std::fs::write(real_dir.join("uv"), b"the real uv").unwrap();

        // The installer prepends the wrapper directory to PATH; here it is
        // spelled with a different case than the one on disk.
        let miscased_wrapper_dir = dir.path().join("UV-ALIASES");
        let path_var = env::join_paths([&miscased_wrapper_dir, &real_dir])
            .unwrap()
            .into_string()
            .unwrap();

        let detected = autodetect_executable(
            &wrapper_dir,
            "uv",
            &path_var,
            &OsFileSystemWrapper {},
        ).expect("the real target should have been detected");

        assert_eq!(real_dir.join("uv"), PathBuf::from(detected));
    }
}
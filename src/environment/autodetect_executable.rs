use std::env;
use std::path::Path;

pub fn autodetect_executable(executable_path: &Path,
                             executable_name: &str,
                             path_var: &str,
                             fs: &dyn FileSystemWrapper) -> Option<String> {
    let paths: Vec<_> = env::split_paths(path_var).collect();

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
        let target = path_item.join(executable_name);
        if fs.exists(&target) && fs.is_file(&target) {
            target.to_str().map(|s| s.to_string())
        } else {
            None
        }
    })
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

    fn is_file(&self, path: &Path) -> bool {
        let metadata = std::fs::symlink_metadata(path);
        metadata.map(|x| !x.is_dir())
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

        pub fn symlink() -> TestFileDescriptor {
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

    #[test]
    fn target_executable_can_be_found_later_in_the_path() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/bin/alias", &TestFileDescriptor::file());
        fs.add("/usr/bin/alias", &TestFileDescriptor::file());
        let path = make_path(&["/bin", "/usr/bin"]);
        let autodetect = autodetect_executable(Path::new("/bin"), "alias", &path, &fs).unwrap();
        assert_eq!(Path::new("/usr/bin/alias"), Path::new(&autodetect));
    }

    #[test]
    fn symlink_to_target_executable_can_be_found_later_in_the_path() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/bin/alias", &TestFileDescriptor::file());
        fs.add("/usr/bin/alias", &TestFileDescriptor::symlink());
        let path = make_path(&["/bin", "/usr/bin"]);
        assert!(autodetect_executable(Path::new("/bin"), "alias", &path, &fs).is_none());
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
        assert!(autodetect_executable(
            Path::new("/home/username/alias_app"),
            "alias",
            &path,
            &fs,
        ).is_none());
    }

    #[test]
    fn alias_path_does_not_exist_in_path() {
        let fs = TestFileSystemWrapper::create();
        let path = make_path(&["/bin", "/usr/bin"]);
        assert!(autodetect_executable(Path::new("/home/username/app"), "alias", &path, &fs).is_none());
    }

    #[test]
    fn alias_path_exists_but_target_executable_doesnt() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/home/username/app/alias", &TestFileDescriptor::file());
        let path = make_path(&["/home/username/app", "/bin", "/usr/bin"]);
        assert!(autodetect_executable(Path::new("/home/username/app"), "alias", &path, &fs).is_none());
    }

    #[test]
    fn wrapper_doesnt_exist_in_path_try_to_find_first_executable_that_has_the_same_name() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/usr/bin/alias", &TestFileDescriptor::file());
        let path = make_path(&["/bin", "/usr/bin"]);
        let autodetect = autodetect_executable(
            Path::new("/home/username/app"),
            "alias",
            &path,
            &fs,
        ).unwrap();
        assert_eq!(Path::new("/usr/bin/alias"), Path::new(&autodetect));
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
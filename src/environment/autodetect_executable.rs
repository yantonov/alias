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
        .position(|p| p.as_path() == executable_path)
        .map(|i| i + 1)
        .unwrap_or(0);

    paths[start..].iter().find_map(|path_item| {
        let target = path_item.join(executable_name);
        if fs.exists(&target) && fs.is_file(&target) {
            target.to_str().map(|s| s.to_string())
        } else {
            None
        }
    })
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
    use std::path::Path;
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
        path_to_descriptor: HashMap<String, TestFileDescriptor>,
    }

    impl TestFileSystemWrapper {
        pub fn create() -> TestFileSystemWrapper {
            TestFileSystemWrapper {
                path_to_descriptor: HashMap::new(),
            }
        }

        pub fn add(&mut self, path: &str, descriptor: &TestFileDescriptor) {
            self.path_to_descriptor.insert(path.to_string(), (*descriptor).clone());
        }
    }

    impl FileSystemWrapper for TestFileSystemWrapper {
        fn exists(&self, path: &Path) -> bool {
            self.path_to_descriptor.contains_key(path.to_str().unwrap())
        }

        fn is_file(&self, path: &Path) -> bool {
            self.path_to_descriptor.get(path.to_str().unwrap())
                .map(|d| d.is_file)
                .unwrap_or(false)
        }
    }

    fn make_path(entries: &[&str]) -> String {
        entries.join(":")
    }

    #[test]
    fn target_executable_can_be_found_later_in_the_path() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/bin/alias", &TestFileDescriptor::file());
        fs.add("/usr/bin/alias", &TestFileDescriptor::file());
        let path = make_path(&["/bin", "/usr/bin"]);
        let autodetect = autodetect_executable(Path::new("/bin"), "alias", &path, &fs).unwrap();
        assert_eq!("/usr/bin/alias", autodetect);
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
        assert_eq!("/usr/bin/alias", autodetect);
    }
}
use std::env;
use std::path::Path;

pub fn autodetect_executable(executable_path: &Path,
                             executable_name: &str,
                             fs: &dyn FileSystemWrapper) -> Option<String> {
    let executable_name_as_path = Path::new(executable_name);
    match env::var("PATH") {
        Ok(path_var) => {
            let paths = env::split_paths(&path_var);
            let mut found_current = false;
            for path_item in paths {
                if !found_current {
                    if path_item.as_path() == executable_path {
                        found_current = true
                    }
                } else {
                    let target_path = Path::join(
                        &path_item,
                        &executable_name_as_path);
                    if fs.exists(&target_path) {
                        if fs.is_file(&target_path) {
                            return Some(target_path.to_str().unwrap().to_string());
                        }
                    }
                }
            }
            None
        }
        Err(_) => None
    }
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
        return metadata.map(|x| x.is_file())
            .unwrap_or(false);
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use std::path::Path;
    use std::collections::HashMap;

    #[derive(Clone)]
    struct TestFileDescriptor {
        is_file: bool,
    }

    impl TestFileDescriptor {
        pub fn file() -> TestFileDescriptor {
            return TestFileDescriptor {
                is_file: true
            };
        }
    }

    struct TestFileSystemWrapper {
        path_to_descriptor: HashMap<String, TestFileDescriptor>,
    }

    impl TestFileSystemWrapper {
        pub fn create() -> TestFileSystemWrapper {
            return TestFileSystemWrapper {
                path_to_descriptor: HashMap::new()
            };
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
            return self.path_to_descriptor.get(path.to_str().unwrap())
                .map(|d| d.is_file)
                .unwrap_or(false);
        }
    }

    #[test]
    fn target_executable_can_be_found_later_in_the_path() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/bin/alias", &TestFileDescriptor::file());
        fs.add("/usr/bin/alias", &TestFileDescriptor::file());
        set_path(vec![
            "/bin",
            "/usr/bin"]);
        let autodetect = autodetect_executable(
            Path::new("/bin"),
            "alias",
            &fs).unwrap();
        assert_eq!("/usr/bin/alias", autodetect);
    }

    #[test]
    fn target_executable_cannot_be_found_later_in_the_path() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/home/username/alias_app/alias", &TestFileDescriptor::file());
        fs.add("/home/username/some_app/alias", &TestFileDescriptor::file());
        set_path(vec![
            "/home/username/some_app",
            "/home/username/alias_app",
            "/bin",
            "/usr/bin"]);
        assert!(autodetect_executable(
            Path::new("/home/username/alias_app"),
            "alias",
            &fs)
            .is_none());
    }

    #[test]
    fn alias_path_does_not_exist_in_path() {
        let fs = TestFileSystemWrapper::create();
        set_path(vec![
            "/bin",
            "/usr/bin"]);
        assert!(autodetect_executable(
            Path::new("/home/username/app"),
            "alias",
            &fs)
            .is_none());
    }

    #[test]
    fn alias_path_exists_but_target_executable_doesnt() {
        let mut fs = TestFileSystemWrapper::create();
        fs.add("/home/username/app/alias", &TestFileDescriptor::file());
        set_path(vec![
            "/home/username/app",
            "/bin",
            "/usr/bin"]);
        assert!(autodetect_executable(
            Path::new("/home/username/app"),
            "alias",
            &fs)
            .is_none());
    }

    fn set_path(path_strings: Vec<&str>) {
        let paths: Vec<&Path> = path_strings
            .into_iter()
            .map(|p| Path::new(p))
            .collect();
        let path_os_string = env::join_paths(
            paths.iter()).unwrap();
        env::set_var("PATH", path_os_string);
    }
}
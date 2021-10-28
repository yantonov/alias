use std::env;
use std::path::{PathBuf, Path};

pub fn autodetect_executable(executable_path: &Path,
                             executable_name: &str,
                             checker: &dyn CheckFile) -> Option<String> {
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
                    if checker.exists(&target_path) {
                        let string = target_path.as_os_str().to_str().unwrap().to_string();
                        return Some(string);
                    }
                }
            }
            None
        }
        Err(_) => None
    }
}

pub trait CheckFile {
    fn exists(&self, path: &Path) -> bool;
}

pub struct OsCheckFile {}

impl CheckFile for OsCheckFile {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

struct DummyCheckFile {
    expected_path: PathBuf,
}

impl CheckFile for DummyCheckFile {
    fn exists(&self, path: &Path) -> bool {
        self.expected_path == path
    }
}

struct NoFile {}

impl CheckFile for NoFile {
    fn exists(&self, _path: &Path) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use std::path::Path;

    #[test]
    fn target_executable_can_be_found_later_in_the_path() {
        set_path(vec![
            "/bin",
            "/usr/bin"]);
        let autodetect = autodetect_executable(
            Path::new("/bin"),
            "alias",
            &DummyCheckFile {
                expected_path: Path::new("/usr/bin/alias").to_owned()
            }).unwrap();
        assert_eq!("/usr/bin/alias", autodetect);
    }

    #[test]
    fn target_executable_cannot_be_found_later_in_the_path() {
        set_path(vec![
            "/home/username/some_app",
            "/home/username/alias_app",
            "/bin",
            "/usr/bin"]);
        assert!(autodetect_executable(
            Path::new("/home/username/alias_app"),
            "alias",
            &NoFile {})
            .is_none());
    }

    #[test]
    fn alias_path_does_not_exist_in_path_wait_autodetect_executable_fail() {
        set_path(vec![
            "/bin",
            "/usr/bin"]);
        assert!(autodetect_executable(
            Path::new("/home/username/app"),
            "alias",
            &NoFile {})
            .is_none());
    }

    #[test]
    fn alias_path_exists_but_target_executable_doesnt_expect_autodetect_fail() {
        set_path(vec![
            "/home/username/app",
            "/bin",
            "/usr/bin"]);
        assert!(autodetect_executable(
            Path::new("/home/username/app"),
            "alias",
            &NoFile {})
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
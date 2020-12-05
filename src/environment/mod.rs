use std::env;
use std::path::{PathBuf, Path};

use regex::{Captures, Regex};

pub struct Environment {
    executable_name: String,
    executable_dir: PathBuf,
    args: Vec<String>,
    shell: String,
}

impl Environment {
    pub fn executable_name(&self) -> &String {
        &self.executable_name
    }

    pub fn executable_dir(&self) -> &PathBuf {
        &self.executable_dir
    }

    pub fn call_arguments(&self) -> &[String] {
        &self.args[1..self.args.len()]
    }

    pub fn shell(&self) -> String {
        self.shell.to_string()
    }
}

struct SystemEnvironment {}

impl SystemEnvironment {
    pub fn executable_name(&self) -> Result<String, String> {
        env::current_exe()
            .map(|x| x
                .file_name()
                .expect("cannot detect filename")
                .to_str()
                .expect("cannot convert filename to string")
                .to_string())
            .map_err(|_| "cannot get current executable".to_string())
    }

    pub fn executable_dir(&self) -> Result<PathBuf, String> {
        let executable = env::current_exe()
            .map_err(|_| "cannot get current executable")?;
        match executable
            .parent()
            .map(|x| x.to_path_buf()) {
            None => Err("cannot get parent directory".to_string()),
            Some(v) => Ok(v),
        }
    }

    pub fn call_arguments(&self) -> Vec<String> {
        return env::args().collect();
    }

    pub fn shell(&self) -> Result<String, &str> {
        return env::var("SHELL")
            .map_err(|_| "SHELL environment variable is not defined");
    }
}

pub fn system_environment() -> Environment {
    let sys_env = SystemEnvironment {};
    return Environment {
        executable_name: sys_env.executable_name().unwrap(),
        executable_dir: sys_env.executable_dir().unwrap(),
        args: sys_env.call_arguments(),
        shell: sys_env.shell().unwrap(),
    };
}

pub fn expand_env_var(path: &str) -> String {
    let re = Regex::new(r"(\$\{[^{}]+\})").unwrap();
    let expanded = re.replace_all(
        path,
        |captures: &Captures| -> String {
            let env_var = captures
                .get(1)
                .unwrap()
                .as_str()
                .to_string();
            return env::var(&env_var[2..(env_var.len() - 1)])
                .unwrap_or(env_var);
        });
    return expanded.into_owned();
}

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
    fn exists(&self, path: &PathBuf) -> bool;
}

pub struct OsCheckFile {}

impl CheckFile for OsCheckFile {
    fn exists(&self, path: &PathBuf) -> bool {
        path.exists()
    }
}

struct DummyCheckFile {
    expected_path: PathBuf
}

impl CheckFile for DummyCheckFile {
    fn exists(&self, path: &PathBuf) -> bool {
        self.expected_path.as_path() == path.as_path()
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use std::path::Path;

    #[test]
    fn expand_existing_var() {
        env::set_var("ENV_VAR", "yes");
        assert_eq!("yes/replaced",
                   expand_env_var("${ENV_VAR}/replaced"));
    }

    #[test]
    fn not_existing_var_wait_unmodified_string() {
        assert_eq!("${NOT_EXISTING_VAR}/not_replaced",
                   expand_env_var("${NOT_EXISTING_VAR}/not_replaced"));
    }

    #[test]
    fn target_executable_can_be_found_later_in_the_path() {
        let paths = [
            Path::new("/bin"),
            Path::new("/usr/bin")];
        let path_os_string = env::join_paths(paths.iter()).unwrap();
        env::set_var("PATH", path_os_string);
        let autodetect = autodetect_executable(
            Path::new("/bin"),
            "alias",
            &DummyCheckFile {
                expected_path: Path::new("/usr/bin/alias").to_owned()
            }).unwrap();
        assert_eq!("/usr/bin/alias", autodetect);
    }
}

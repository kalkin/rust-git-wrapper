use crate::AbsoluteDirPath;
use crate::ConfigReadError;
use crate::GenericRepository;
use std::path::Path;
use std::process::Command;

/// Represents a bare repository
#[derive(Debug)]
pub struct BareRepository(AbsoluteDirPath);

impl BareRepository {
    /// # Panics
    ///
    /// When git execution fails
    ///
    /// # Errors
    ///
    /// Returns a string output when something goes horrible wrong
    ///
    #[inline]
    pub fn create(path: &Path) -> Result<Self, String> {
        let mut cmd = Command::new("git");
        let out = cmd
            .arg("init")
            .arg("--bare")
            .current_dir(&path)
            .output()
            .expect("Execute git-init(1)");

        if out.status.success() {
            let git_dir = path.try_into().map_err(|e| format!("{}", e))?;
            Ok(Self(git_dir))
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string())
        }
    }

    /// Return config value for specified key
    ///
    /// # Errors
    ///
    /// When given invalid key or an invalid config file is read.
    ///
    /// # Panics
    ///
    /// Will panic if git exits with an unexpected error code. Expected codes are 0, 1 & 3.
    #[inline]
    pub fn config(&self, key: &str) -> Result<String, ConfigReadError> {
        self.gen_config(key)
    }

    /// Returns a prepared git `Command` struct
    #[must_use]
    #[inline]
    pub fn git(&self) -> Command {
        let mut cmd = Command::new("git");
        let git_dir = self.0 .0.to_str().expect("Convert to string");
        cmd.env("GIT_DIR", git_dir);
        cmd
    }
}

impl GenericRepository for BareRepository {
    fn gen_git(&self) -> Command {
        self.git()
    }
}

#[cfg(test)]
mod test {
    use tempfile::TempDir;
    #[test]
    fn bare_repo() {
        let tmp_dir = TempDir::new().unwrap();
        let repo_path = tmp_dir.path();
        let _repo = crate::BareRepository::create(repo_path).expect("Created a bare repo");
    }
}

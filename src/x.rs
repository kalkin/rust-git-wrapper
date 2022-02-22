use crate::Repository;
use posix_errors::PosixError;

/// # Errors
///
/// Returns an error when git reset --hard fails
#[inline]
pub fn reset_hard(repo: &Repository, sha: &str) -> Result<(), PosixError> {
    let mut cmd = repo.git();
    let out = cmd
        .args(&["reset", "--hard", "--quiet", sha])
        .output()
        .expect("Failed to execute git-reset(1)");

    if !out.status.success() {
        let message = String::from_utf8_lossy(&out.stderr).to_string();
        let code = out.status.code().unwrap_or(1);
        return Err(PosixError::new(code, message));
    }
    Ok(())
}

//Copyright (c) 2021 Bahtiar `kalkin` Gadimov <bahtiar@gadimov.de>
//
//This file is part of git-wrapper.
//
//This program is free software: you can redistribute it and/or modify
//it under the terms of the GNU Lesser General Public License as published by
//the Free Software Foundation, either version 3 of the License, or
//(at your option) any later version.
//
//This program is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU Lesser General Public License for more details.
//
//You should have received a copy of the GNU Lesser General Public License
//along with this program. If not, see <http://www.gnu.org/licenses/>.

//! A wrapper around [git(1)](https://git-scm.com/docs/git) inspired by
//! [`GitPython`](https://github.com/gitpython-developers/GitPython).

#![allow(unknown_lints)]
#![warn(clippy::all)]

pub use posix_errors::PosixError;
use std::collections::HashMap;
use std::process::Command;
use std::process::Output;

macro_rules! cmd {
    ($args:expr) => {
        Command::new("git").args($args).output()
    };
    ($name:expr, $args:expr) => {
        Command::new("git").arg($name).args($args).output()
    };
}

macro_rules! cmd_in_dir {
    ( $working_dir:expr, $args:expr ) => {
        Command::new("git")
            .args(&["-C", $working_dir])
            .args($args)
            .output()
    };
    ($working_dir:expr, $name:expr, $args: expr) => {
        Command::new("git")
            .args(&["-C", $working_dir])
            .arg($name)
            .args($args)
            .output()
    };
}

/// Helper function executing git in the specified working directory and returning
/// [`std::process::Output`].
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn git_cmd_out(working_dir: &str, args: Vec<&str>) -> Result<Output, PosixError> {
    let result = cmd_in_dir!(working_dir, args);
    if let Ok(value) = result {
        return Ok(value);
    }

    Err(PosixError::from(result.unwrap_err()))
}

/// Helper function executing git *without* a working directory and returning
/// [`std::process::Output`].
///
/// Useful for git commands not needing a working directory like e.g. `git ls-remote`.
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn git_cmd(args: Vec<&str>) -> Result<Output, PosixError> {
    let result = cmd!(args);
    if let Ok(value) = result {
        return Ok(value);
    }

    Err(PosixError::from(result.unwrap_err()))
}

/// Wrapper around [git-ls-remote(1)](https://git-scm.com/docs/git-ls-remote)
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn ls_remote(args: &[&str]) -> Result<Output, PosixError> {
    let result = cmd!("ls-remote", args);

    if let Ok(value) = result {
        return Ok(value);
    }

    Err(PosixError::from(result.unwrap_err()))
}

/// Returns all tags from a remote
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn tags_from_remote(url: &str) -> Result<Vec<String>, PosixError> {
    let mut vec = Vec::new();
    let output = ls_remote(&["--refs", "--tags", url])?;
    if output.status.success() {
        let tmp = String::from_utf8(output.stdout).expect("Expected UTF-8");
        for s in tmp.lines() {
            let mut split = s.splitn(3, '/');
            split.next();
            split.next();
            let split_result = split.next();
            if let Some(value) = split_result {
                vec.push(String::from(value));
            }
        }
        Ok(vec)
    } else {
        Err(PosixError::from(output))
    }
}

/// Return the path for the top level repository directory for current working dir.
///
/// This function will fail if the CWD is not a part of a git repository.
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn top_level() -> Result<String, PosixError> {
    let output = git_cmd(vec!["rev-parse", "--show-toplevel"])?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)
            .expect("UTF-8 encoding")
            .trim_end()
            .to_string())
    } else {
        Err(PosixError::from(output))
    }
}

/// Set a config value via [git-config(1)](https://git-scm.com/docs/git-config)
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn config_set(
    working_dir: &str,
    file: &str,
    key: &str,
    value: &str,
) -> Result<bool, PosixError> {
    let output = cmd_in_dir!(working_dir, "config", vec!["--file", file, key, value])
        .expect("Failed to execute git config");
    if output.status.success() {
        Ok(true)
    } else {
        Err(PosixError::from(output))
    }
}

/// Update the sparse-checkout file to include additional patterns.
///
/// See also [git-sparse-checkout(1)](https://git-scm.com/docs/git-sparse-checkout)
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn sparse_checkout_add(working_dir: &str, pattern: &str) -> Result<bool, PosixError> {
    let output = cmd_in_dir!(working_dir, "sparse-checkout", vec!["add", pattern])
        .expect("Failed to execute git sparse-checkout");
    if output.status.success() {
        Ok(true)
    } else {
        Err(PosixError::from(output))
    }
}

/// Return `true` if the repository is sparse
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
#[must_use]
pub fn is_sparse(working_dir: &str) -> bool {
    let output = cmd_in_dir!(working_dir, "config", vec!["core.sparseCheckout"])
        .expect("Failed to execute git config");

    String::from_utf8(output.stdout).expect("UTF-8 encoding") == "true"
}

/// Create the `prefix` subtree by importing its contents from the given `remote`
/// and remote `git_ref`.
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn subtree_add(
    working_dir: &str,
    prefix: &str,
    url: &str,
    git_ref: &str,
    msg: &str,
) -> Result<bool, PosixError> {
    let output = cmd_in_dir!(
        working_dir,
        "subtree",
        vec!["add", "-P", prefix, url, git_ref, "-m", msg]
    )
    .expect("Failed to execute git subtree");
    if output.status.success() {
        Ok(true)
    } else {
        Err(PosixError::from(output))
    }
}

/// Return all `.gitsubtrees` files in the working directory.
///
/// Uses [git-ls-files(1)](https://git-scm.com/docs/git-ls-files)
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn subtree_files(working_dir: &str) -> Result<Vec<String>, PosixError> {
    let output = git_cmd_out(working_dir, vec!["ls-files", "--", "*.gitsubtrees"])?;
    if output.status.success() {
        let tmp = String::from_utf8(output.stdout).expect("UTF-8 encoding");
        Ok(tmp.lines().map(str::to_string).collect())
    } else {
        Err(PosixError::from(output))
    }
}

/// Return `true` if the working dir index is clean.
///
/// Uses [git-diff(1)](https://git-scm.com/docs/git-diff)
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn is_working_dir_clean(working_dir: &str) -> Result<bool, PosixError> {
    let output = git_cmd_out(working_dir, vec!["diff", "--quiet"]);
    Ok(output?.status.success())
}

/// Figure out the default branch for given remote.
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn resolve_head(remote: &str) -> Result<String, PosixError> {
    let proc =
        cmd!("ls-remote", vec!["--symref", remote, "HEAD"]).expect("Failed to execute git command");
    if proc.status.success() {
        let stdout = String::from_utf8(proc.stdout).expect("UTF-8 encoding");
        let mut lines = stdout.lines();
        let first_line = lines.next().expect("Failed to parse HEAD from remote");
        let mut split = first_line
            .split('\t')
            .next()
            .expect("Failed to parse HEAD from remote")
            .splitn(3, '/');
        split.next();
        split.next();
        return Ok(split
            .next()
            .expect("Failed to parse default branch")
            .to_string());
    }

    Err(PosixError::from(proc))
}

/// Resolve hash id of the given branch/tag at the remote.
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn remote_ref_to_id(remote: &str, name: &str) -> Result<String, PosixError> {
    let proc = cmd!("ls-remote", vec![remote, name]).expect("Failed to execute git ls-remote");
    if proc.status.success() {
        let stdout = String::from_utf8(proc.stdout).expect("UTF-8 encoding");
        let mut lines = stdout.lines();
        let first_line = lines.next().expect("Failed to parse first line");
        return Ok(first_line
            .split('\t')
            .next()
            .expect("Failed to parse id")
            .to_string());
    }
    Err(PosixError::from(proc))
}

/// Convert a long hash id to a short one.
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn short_ref(working_dir: &str, long_ref: &str) -> Result<String, PosixError> {
    let proc = git_cmd_out(working_dir, vec!["rev-parse", "--short", long_ref])?;
    if proc.status.success() {
        return Ok(String::from_utf8(proc.stdout)
            .expect("UTF-8 encoding")
            .trim_end()
            .to_string());
    }
    Err(PosixError::from(proc))
}

/// Convert a symbolic ref like HEAD to an git id
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn ref_to_id(working_dir: &str, git_ref: &str) -> Result<String, PosixError> {
    let proc = git_cmd_out(working_dir, vec!["rev-parse", git_ref])?;
    if proc.status.success() {
        return Ok(String::from_utf8(proc.stdout)
            .expect("UTF-8 encoding")
            .trim_end()
            .to_string());
    }
    Err(PosixError::from(proc))
}

/// Clone a remote
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn clone(url: &str, directory: &str) -> Result<bool, PosixError> {
    let proc = git_cmd(vec!["clone", "--", url, directory])?;
    if proc.status.success() {
        return Ok(true);
    }
    Err(PosixError::from(proc))
}

/// Lists commit objects in reverse chronological order
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn rev_list(working_dir: &str, args: Vec<&str>) -> Result<String, PosixError> {
    let proc = cmd_in_dir!(working_dir, "rev-list", args).expect("Failed to run rev-list");
    if proc.status.success() {
        return Ok(String::from_utf8_lossy(&proc.stdout).trim_end().to_string());
    }
    Err(PosixError::from(proc))
}

/// Check if the first <commit> is an ancestor of the second <commit>.
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn is_ancestor(working_dir: &str, first: &str, second: &str) -> Result<bool, PosixError> {
    let args = vec!["--is-ancestor", first, second];
    let proc = cmd_in_dir!(working_dir, "merge-base", args).expect("Failed to run rev-list");
    Ok(proc.status.success())
}

enum RemoteDir {
    Fetch,
    Push,
}

struct RemoteLine {
    name: String,
    url: String,
    dir: RemoteDir,
}

/// Represents a git remote
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Remote {
    pub name: String,
    pub push: Option<String>,
    pub fetch: Option<String>,
}

/// Return a map of all remotes
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn remotes(working_dir: &str) -> Result<HashMap<String, Remote>, PosixError> {
    let mut my_map: HashMap<String, Remote> = HashMap::new();
    let mut remote_lines: Vec<RemoteLine> = vec![];

    let proc = cmd_in_dir!(working_dir, "remote", &["-v"]).expect("failed to run remote -v");
    let text = String::from_utf8(proc.stdout).expect("UTF-8 encoding");

    for line in text.lines() {
        let mut split = line.trim().split('\t');
        let name = split.next().expect("Remote name").to_string();
        let rest = split.next().expect("Remote rest");
        let mut rest_split = rest.split(' ');
        let url = rest_split.next().expect("Remote url").to_string();
        let dir = if rest_split.next().expect("Remote direction") == "(fetch)" {
            RemoteDir::Fetch
        } else {
            RemoteDir::Push
        };
        remote_lines.push(RemoteLine { name, url, dir });
    }
    for remote_line in remote_lines {
        let mut remote = my_map.remove(&remote_line.name).unwrap_or(Remote {
            name: remote_line.name.to_string(),
            push: None,
            fetch: None,
        });
        match remote_line.dir {
            RemoteDir::Fetch => remote.fetch = Some(remote_line.url.to_string()),
            RemoteDir::Push => remote.push = Some(remote_line.url.to_string()),
        }
        my_map.insert(remote_line.name.clone(), remote);
    }

    Ok(my_map)
}

/// Try to guess the main url for the repo
///
/// ⒈ Upstream
/// ⒉ Origin
/// ⒊ Random
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
pub fn main_url(working_dir: &str) -> Result<Option<String>, PosixError> {
    let remote_map = remotes(working_dir)?;
    if let Some(remote) = remote_map.get("upstream") {
        Ok(remote.fetch.clone().or_else(|| remote.push.clone()))
    } else if let Some(remote) = remote_map.get("origin") {
        Ok(remote.fetch.clone().or_else(|| remote.push.clone()))
    } else if remote_map.is_empty() {
        Ok(None)
    } else {
        let remotes: Vec<Remote> = remote_map.into_iter().map(|(_, v)| v).collect();
        let remote = remotes.first().expect("At least one remote");
        Ok(remote.fetch.clone().or_else(|| remote.push.clone()))
    }
}

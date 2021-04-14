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
//! [GitPython](https://github.com/gitpython-developers/GitPython).

#![allow(unknown_lints)]
#![warn(clippy::all)]

use posix_errors::{error_from_output, to_posix_error, PosixError};
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
/// [std::process::Output].
pub fn git_cmd_out(working_dir: String, args: Vec<&str>) -> Result<Output, PosixError> {
    let result = cmd_in_dir!(&working_dir, args);
    if let Ok(value) = result {
        return Ok(value);
    }

    Err(to_posix_error(result.unwrap_err()))
}

/// Helper function executing git *without* a working directory and returning
/// [std::process::Output].
///
/// Useful for git commands not needing a working directory like e.g. `git ls-remote`.
pub fn git_cmd(args: Vec<&str>) -> Result<Output, PosixError> {
    let result = cmd!(args);
    if let Ok(value) = result {
        return Ok(value);
    }

    Err(to_posix_error(result.unwrap_err()))
}

/// Wrapper around [git-ls-remote(1)](https://git-scm.com/docs/git-ls-remote)
pub fn ls_remote(args: &[&str]) -> Result<Output, PosixError> {
    let result = cmd!("ls-remote", args);

    if let Ok(value) = result {
        return Ok(value);
    }

    Err(to_posix_error(result.unwrap_err()))
}

/// Returns all tags from a remote
pub fn tags_from_remote(url: &str) -> Result<Vec<String>, PosixError> {
    let mut vec = Vec::new();
    let output = ls_remote(&["--refs", "--tags", &url])?;
    if output.status.success() {
        let tmp = String::from_utf8(output.stdout).unwrap();
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
        Err(error_from_output(output))
    }
}

/// Return the path for the top level repository directory in current working dir.
///
/// This function will fail if the CWD is not a part of a git repository.
pub fn top_level() -> Result<String, PosixError> {
    let output = git_cmd(vec!["rev-parse", "--show-toplevel"])?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)
           .unwrap()
           .trim_end()
           .to_string())
    } else {
        Err(error_from_output(output))
    }
}

/// Set a config value via [git-config(1)](https://git-scm.com/docs/git-config)
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
        Err(error_from_output(output))
    }
}

/// Update the sparse-checkout file to include additional patterns.
///
/// See also [git-sparse-checkout(1)](https://git-scm.com/docs/git-sparse-checkout)
pub fn sparse_checkout_add(working_dir: &str, pattern: &str) -> Result<bool, PosixError> {
    let output = cmd_in_dir!(working_dir, "sparse-checkout", vec!["add", pattern])
        .expect("Failed to execute git sparse-checkout");
    if output.status.success() {
        Ok(true)
    } else {
        Err(error_from_output(output))
    }
}

/// Return `true` if the repository is sparse
pub fn is_sparse(working_dir: &str) -> bool {
    let output = cmd_in_dir!(working_dir, "config", vec!["core.sparseCheckout"])
        .expect("Failed to execute git config");

    String::from_utf8(output.stdout).unwrap() == "true"
}

/// Create the `prefix` subtree by importing its contents from the given `remote`
/// and remote `git_ref`.
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
        Err(error_from_output(output))
    }
}

/// Return all `.gitsubtrees` files in the working directory.
///
/// Uses [git-ls-files(1)](https://git-scm.com/docs/git-ls-files)
pub fn subtree_files(working_dir: &str) -> Result<Vec<String>, PosixError> {
    let output = git_cmd_out(
        working_dir.to_string(),
        vec!["ls-files", "--", "*.gitsubtrees"],
    )?;
    if output.status.success() {
        let tmp = String::from_utf8(output.stdout).unwrap();
        Ok(tmp.lines().map(str::to_string).collect())
    } else {
        Err(error_from_output(output))
    }
}

/// Return `true` if the working dir index is clean.
///
/// Uses [git-diff(1)](https://git-scm.com/docs/git-diff)
pub fn is_working_dir_clean(working_dir: &str) -> Result<bool, PosixError> {
    let output = git_cmd_out(working_dir.to_string(), vec!["diff", "--quiet"]);
    Ok(output?.status.success())
}

/// Figure out the default branch for given remote.
pub fn resolve_head(remote: &str) -> Result<String, PosixError> {
    let proc =
        cmd!("ls-remote", vec!["--symref", remote, "HEAD"]).expect("Failed to execute git command");
    if proc.status.success() {
        let stdout = String::from_utf8(proc.stdout).unwrap();
        let mut lines = stdout.lines();
        let first_line = lines.next().expect("Failed to parse HEAD from remote");
        let mut split = first_line
            .split('\t')
            .next()
            .expect("Failed to parse HEAD from remote")
            .splitn(3, '/');
        split.next();
        split.next();
        return Ok(split.next().unwrap().to_string());
    }

    Err(error_from_output(proc))
}

/// Resolve hash id of the given branch/tag at the remote.
pub fn remote_ref_to_id(remote: &str, name: &str) -> Result<String, PosixError> {
    let proc = cmd!("ls-remote", vec![remote, name]).expect("Failed to execute git ls-remote");
    if proc.status.success() {
        let stdout = String::from_utf8(proc.stdout).unwrap();
        let mut lines = stdout.lines();
        let first_line = lines.next().expect("Failed to parse id from remote");
        return Ok(first_line.split('\t').next().unwrap().to_string());
    }
    Err(error_from_output(proc))
}

/// Convert a long hash id to a short one.
pub fn short_ref(working_dir: &str, long_ref: &str) -> Result<String, PosixError> {
    let proc = git_cmd_out(
        working_dir.to_string(),
        vec!["rev-parse", "--short", long_ref],
    )?;
    if proc.status.success() {
        return Ok(String::from_utf8(proc.stdout)
                  .unwrap()
                  .trim_end()
                  .to_string());
    }
    Err(error_from_output(proc))
}

/// Clone a remote
pub fn clone(url: &str, directory: &str) -> Result<bool, PosixError> {
    let proc = git_cmd(vec!["clone", "--", url, directory])?;
    if proc.status.success() {
        return Ok(true);
    }
    Err(error_from_output(proc))
}

pub fn rev_list(working_dir: &str, args: Vec<&str>) -> Result<String, PosixError> {
    let proc = cmd_in_dir!(working_dir, "rev-list", args).expect("Failed to run rev-list");
    if proc.status.success() {
        return Ok(String::from_utf8(proc.stdout).unwrap().trim_end().to_string());
    }
    Err(error_from_output(proc))
}

// Check if the first <commit> is an ancestor of the second <commit>.
pub fn is_ancestor(working_dir: &str, first: &str, second: &str) -> Result<bool, PosixError> {
    let args = vec!["--is-ancestor", first, second];
    let proc = cmd_in_dir!(working_dir, "merge-base", args).expect("Failed to run rev-list");
    Ok(proc.status.success())
}

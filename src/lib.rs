// Copyright (c) 2021 Bahtiar `kalkin` Gadimov <bahtiar@gadimov.de>
//
// This file is part of git-wrapper.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! A wrapper around [git(1)](https://git-scm.com/docs/git) inspired by
//! [`GitPython`](https://github.com/gitpython-developers/GitPython).

#![allow(unknown_lints)]
#![warn(clippy::all)]

pub use posix_errors::{PosixError, EACCES, EINVAL, ENOENT};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Output;

pub mod x;

macro_rules! cmd {
    ($args:expr) => {
        Command::new("git").args($args).output()
    };
    ($name:expr, $args:expr) => {
        Command::new("git").arg($name).args($args).output()
    };
}

/// Wrapper around [git-ls-remote(1)](https://git-scm.com/docs/git-ls-remote)
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
#[inline]
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
#[inline]
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

pub enum ConfigSetError {
    InvalidSectionOrKey(String),
    InvalidConfigFile(String),
    WriteFailed(String),
}

/// # Errors
///
/// Throws [`ConfigSetError`] on errors
///
/// # Panics
///
/// When git-config(1) execution fails
#[inline]
pub fn config_file_set(file: &Path, key: &str, value: &str) -> Result<(), ConfigSetError> {
    let args = &["--file", file.to_str().expect("UTF-8 encoding"), key, value];
    let mut cmd = Command::new("git");
    cmd.arg("config").args(args);
    let out = cmd.output().expect("Failed to execute git-config(1)");
    if out.status.success() {
        Ok(())
    } else {
        let msg = String::from_utf8(out.stdout).expect("UTF-8 encoding");
        match out.status.code().unwrap() {
            1 => Err(ConfigSetError::InvalidSectionOrKey(msg)),
            3 => Err(ConfigSetError::InvalidConfigFile(msg)),
            4 => Err(ConfigSetError::WriteFailed(msg)),
            _ => panic!("Unexpected error:\n{}", msg),
        }
    }
}

/// Return all `.gitsubtrees` files in the working directory.
///
/// Uses [git-ls-files(1)](https://git-scm.com/docs/git-ls-files)
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
/// Figure out the default branch for given remote.
///
/// # Errors
///
/// Will return [`PosixError`] if command exits with an error code.
/// TODO Return a custom error type
#[inline]
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
            .to_owned());
    }

    Err(PosixError::from(proc))
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

fn cwd() -> Result<PathBuf, RepoError> {
    if let Ok(result) = std::env::current_dir() {
        Ok(result)
    } else {
        Err(RepoError::FailAccessCwd)
    }
}

/// A path which is canonicalized and exists.
#[derive(Debug, Clone)]
pub struct AbsoluteDirPath(PathBuf);

impl AbsoluteDirPath {
    fn new(path: &Path) -> Result<Self, RepoError> {
        let path_buf;
        if path.is_absolute() {
            path_buf = path.to_path_buf();
        } else if let Ok(p) = path.canonicalize() {
            path_buf = p;
        } else {
            return Err(RepoError::AbsolutionError(path.to_path_buf()));
        }

        Ok(Self(path_buf))
    }
}

/// The main repository object.
///
/// This wrapper allows to keep track of optional *git-dir* and *work-tree* directories when
/// executing commands. This functionality was needed for `glv` & `git-stree` project.

#[derive(Clone, Debug)]
pub enum Repository {
    Bare {
        git_dir: AbsoluteDirPath,
    },
    Normal {
        git_dir: AbsoluteDirPath,
        work_tree: AbsoluteDirPath,
    },
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum RepoError {
    #[error("GIT_DIR Not found")]
    GitDirNotFound,
    #[error("Invalid directory: `{0}`")]
    InvalidDirectory(PathBuf),
    #[error("Failed to canonicalize the path buffer: `{0}`")]
    AbsolutionError(PathBuf),
    #[error("Failed to access current working directory")]
    FailAccessCwd,
}

impl From<RepoError> for PosixError {
    #[inline]
    fn from(e: RepoError) -> Self {
        let msg = format!("{}", e);
        match e {
            RepoError::GitDirNotFound | RepoError::InvalidDirectory(_) => Self::new(ENOENT, msg),
            RepoError::AbsolutionError(_) => Self::new(EINVAL, msg),
            RepoError::FailAccessCwd => Self::new(EACCES, msg),
        }
    }
}

fn search_git_dir(start: &Path) -> Result<AbsoluteDirPath, RepoError> {
    let path;
    if start.is_absolute() {
        path = start.to_path_buf();
    } else {
        match start.canonicalize() {
            Ok(path_buf) => path = path_buf,
            Err(_) => return Err(RepoError::InvalidDirectory(start.to_path_buf())),
        }
    }

    match (
        path.join("HEAD").canonicalize(),
        path.join("objects").canonicalize(),
    ) {
        (Ok(head_path), Ok(objects_path)) => {
            if head_path.is_file() && objects_path.is_dir() {
                return AbsoluteDirPath::new(&path);
            }
        }
        (_, _) => {}
    }

    for parent in path.ancestors() {
        let candidate = parent.join(".git");
        if candidate.is_dir() && candidate.exists() {
            return Ok(AbsoluteDirPath::new(candidate.as_ref()))?;
        }
    }
    Err(RepoError::GitDirNotFound)
}

fn work_tree_from_git_dir(git_dir: &AbsoluteDirPath) -> Result<Option<AbsoluteDirPath>, RepoError> {
    let gd = git_dir.0.to_str().unwrap();
    let mut cmd = Command::new("git");
    cmd.args(&["--git-dir", gd, "rev-parse", "--is-bare-repository"]);
    let output = cmd.output().expect("failed to execute rev-parse");
    if output.status.success() {
        let tmp = String::from_utf8_lossy(&output.stdout);
        if tmp.trim() == "true" {
            return Ok(None);
        }
    }

    match git_dir.0.parent() {
        Some(dir) => Ok(Some(AbsoluteDirPath::new(dir)?)),
        None => Ok(None),
    }
}

fn git_dir_from_work_tree(work_tree: &AbsoluteDirPath) -> Result<AbsoluteDirPath, RepoError> {
    let result = work_tree.0.join(".git");
    AbsoluteDirPath::new(result.as_ref())
}

#[derive(Debug, PartialEq)]
pub struct InvalidRefError;

/// Getters
impl Repository {
    #[must_use]
    #[inline]
    pub fn is_bare(&self) -> bool {
        matches!(self, Self::Bare { .. })
    }

    /// # Panics
    ///
    /// Panics of executing git-diff(1) fails
    #[must_use]
    #[inline]
    pub fn is_clean(&self) -> bool {
        let output = self.git().args(&["diff", "--quiet"]).output().unwrap();
        if !output.status.success() {
            return false;
        }

        let output2 = self.git().args(&["rev-parse", "HEAD"]).output().unwrap();
        output2.status.success()
    }

    #[must_use]
    #[inline]
    pub fn remotes(&self) -> Option<HashMap<String, Remote>> {
        let args = &["remote", "-v"];
        let mut cmd = self.git();
        let out = cmd
            .args(args)
            .output()
            .expect("Failed to execute git-remote(1)");
        if !out.status.success() {
            eprintln!(
                "Failed to execute git-remote(1):\n{}",
                String::from_utf8_lossy(&out.stderr)
            );
            return None;
        }

        let text = String::from_utf8_lossy(&out.stdout);
        let mut my_map: HashMap<String, Remote> = HashMap::new();
        let mut remote_lines: Vec<RemoteLine> = vec![];
        for line in text.lines() {
            let mut split = line.trim().split('\t');
            let name = split.next().expect("Remote name").to_owned();
            let rest = split.next().expect("Remote rest");
            let mut rest_split = rest.split(' ');
            let url = rest_split.next().expect("Remote url").to_owned();
            let dir = if rest_split.next().expect("Remote direction") == "(fetch)" {
                RemoteDir::Fetch
            } else {
                RemoteDir::Push
            };
            remote_lines.push(RemoteLine { name, url, dir });
        }
        for remote_line in remote_lines {
            let mut remote = my_map.remove(&remote_line.name).unwrap_or(Remote {
                name: remote_line.name.clone(),
                push: None,
                fetch: None,
            });
            match remote_line.dir {
                RemoteDir::Fetch => remote.fetch = Some(remote_line.url.clone()),
                RemoteDir::Push => remote.push = Some(remote_line.url.clone()),
            }
            my_map.insert(remote_line.name.clone(), remote);
        }

        Some(my_map)
    }

    // TODO return a Result with custom error type
    #[must_use]
    #[inline]
    pub fn head(&self) -> Option<String> {
        let args = &["rev-parse", "HEAD"];
        let mut cmd = self.git();
        let out = cmd
            .args(args)
            .output()
            .expect("Failed to execute git-rev-parse(1)");
        if !out.status.success() {
            return None;
        }
        let result = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        Some(result)
    }

    #[must_use]
    #[inline]
    pub fn work_tree(&self) -> Option<PathBuf> {
        match self {
            Self::Normal { work_tree, .. } => Some(work_tree.0.clone()),
            Self::Bare { .. } => None,
        }
    }

    /// Return true if the repo is sparse
    #[must_use]
    #[inline]
    pub fn is_sparse(&self) -> bool {
        let path = self.git_dir_path().join("info").join("sparse-checkout");
        path.exists()
    }

    fn git_dir(&self) -> &AbsoluteDirPath {
        match self {
            Self::Normal { git_dir, .. } | Self::Bare { git_dir } => git_dir,
        }
    }

    fn git_dir_path(&self) -> &PathBuf {
        match self {
            Self::Normal { git_dir, .. } | Self::Bare { git_dir } => &git_dir.0,
        }
    }

    /// # Errors
    ///
    /// Will return [`InvalidRefError`] if invalid reference provided
    #[inline]
    pub fn short_ref(&self, long_ref: &str) -> Result<String, InvalidRefError> {
        let args = vec!["rev-parse", "--short", long_ref];
        let mut cmd = self.git();
        let out = cmd
            .args(args)
            .output()
            .expect("Failed to execute git-commit(1)");
        if !out.status.success() {
            return Err(InvalidRefError);
        }

        let short_ref = String::from_utf8_lossy(&out.stderr).to_string();
        Ok(short_ref)
    }
}

/// Constructors
impl Repository {
    /// # Errors
    ///
    /// Will return [`RepoError`] when fails to find repository
    #[inline]
    pub fn discover(path: &Path) -> Result<Self, RepoError> {
        let git_dir = search_git_dir(path)?;
        if let Some(work_tree) = work_tree_from_git_dir(&git_dir)? {
            return Ok(Self::Normal { git_dir, work_tree });
        }
        Ok(Self::Bare { git_dir })
    }

    /// # Errors
    ///
    /// Will return [`RepoError`] when fails to find repository
    #[inline]
    pub fn default() -> Result<Self, RepoError> {
        Self::from_args(None, None, None)
    }

    #[must_use]
    #[inline]
    pub fn new(git_dir: AbsoluteDirPath, work_tree: Option<AbsoluteDirPath>) -> Self {
        match work_tree {
            Some(work_tree) => Self::Normal { git_dir, work_tree },
            None => Self::Bare { git_dir },
        }
    }

    /// # Panics
    ///
    /// When git execution fails
    ///
    /// # Errors
    ///
    /// Returns a string output when something goes horrible wrong
    #[inline]
    pub fn create(path: &Path) -> Result<Self, String> {
        let mut cmd = Command::new("git");
        let out = cmd.arg("init").current_dir(&path).output().unwrap();

        if out.status.success() {
            let work_tree = AbsoluteDirPath::new(path).unwrap();
            let git_dir = AbsoluteDirPath::new(&path.join(".git")).unwrap();
            Ok(Self::Normal { git_dir, work_tree })
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string())
        }
    }

    /// # Panics
    ///
    /// When git execution fails
    ///
    /// # Errors
    ///
    /// Returns a string output when something goes horrible wrong
    #[inline]
    pub fn create_bare(path: &Path) -> Result<Self, String> {
        let mut cmd = Command::new("git");
        let out = cmd
            .arg("init")
            .arg("--bare")
            .current_dir(&path)
            .output()
            .unwrap();

        if out.status.success() {
            let git_dir = AbsoluteDirPath::new(path).unwrap();
            Ok(Self::Bare { git_dir })
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string())
        }
    }

    /// # Errors
    ///
    /// Will return [`RepoError`] when fails to find repository
    #[inline]
    pub fn from_args(
        change: Option<&str>,
        git: Option<&str>,
        work: Option<&str>,
    ) -> Result<Self, RepoError> {
        match (change, git, work) {
            (None, None, None) => {
                let git_dir = if let Ok(gd) = std::env::var("GIT_DIR") {
                    AbsoluteDirPath::new(gd.as_ref())?
                } else {
                    search_git_dir(&cwd()?)?
                };

                let work_tree = if let Ok(wt) = std::env::var("GIT_WORK_TREE") {
                    Some(AbsoluteDirPath::new(wt.as_ref())?)
                } else {
                    work_tree_from_git_dir(&git_dir)?
                };

                Ok(Self::new(git_dir, work_tree))
            }
            (_, _, _) => {
                let root = change.map_or_else(PathBuf::new, PathBuf::from);
                match (git, work) {
                    (Some(g_dir), None) => {
                        let git_dir = AbsoluteDirPath::new(&root.join(g_dir))?;
                        let work_tree = work_tree_from_git_dir(&git_dir)?;
                        Ok(Self::new(git_dir, work_tree))
                    }
                    (None, Some(w_dir)) => {
                        let work_tree = AbsoluteDirPath::new(&root.join(w_dir))?;
                        let git_dir = git_dir_from_work_tree(&work_tree)?;
                        Ok(Self::Normal { git_dir, work_tree })
                    }
                    (Some(g_dir), Some(w_dir)) => {
                        let git_dir = AbsoluteDirPath::new(&root.join(g_dir))?;
                        let work_tree = AbsoluteDirPath::new(&root.join(w_dir))?;
                        Ok(Self::Normal { git_dir, work_tree })
                    }
                    (None, None) => {
                        let git_dir = search_git_dir(&root)?;
                        let work_tree = work_tree_from_git_dir(&git_dir)?;
                        Ok(Self::new(git_dir, work_tree))
                    }
                }
            }
        }
    }

    #[must_use]
    #[inline]
    pub fn git(&self) -> Command {
        let mut cmd = Command::new("git");
        let git_dir = self.git_dir().0.to_str().expect("Convert to string");
        cmd.env("GIT_DIR", git_dir);

        if let Self::Normal { work_tree, .. } = self {
            let work_tree = work_tree.0.to_str().expect("Convert to string");
            cmd.env("GIT_WORK_TREE", work_tree);
            cmd.current_dir(work_tree);
        }
        cmd
    }

    #[inline]
    pub fn git_in_dir<P: AsRef<Path>>(&self, path: P) -> Command {
        let mut cmd = self.git();
        cmd.current_dir(path);
        cmd
    }
}

#[derive(Debug, PartialEq)]
pub enum SubtreeAddError {
    BareRepository,
    WorkTreeDirty,
    Failure(String, i32),
}

#[derive(Debug, PartialEq)]
pub enum SubtreePullError {
    BareRepository,
    WorkTreeDirty,
    Failure(String, i32),
}

#[derive(Debug, PartialEq)]
pub enum SubtreePushError {
    BareRepository,
    Failure(String, i32),
}

#[derive(Debug, PartialEq)]
pub enum SubtreeSplitError {
    BareRepository,
    WorkTreeDirty,
    Failure(String, i32),
}

#[derive(Debug)]
pub enum ConfigReadError {
    InvalidSectionOrKey(String),
    InvalidConfigFile(String),
}

#[derive(thiserror::Error, Debug)]
pub enum StagingError {
    #[error("Bare repository")]
    BareRepository,
    #[error("`{0}`")]
    Failure(String, i32),
    #[error("File does not exist: `{0}`")]
    FileDoesNotExist(PathBuf),
}

impl From<StagingError> for PosixError {
    #[inline]
    fn from(e: StagingError) -> Self {
        let msg = format!("{}", e);
        match e {
            StagingError::BareRepository => Self::new(1, msg),
            StagingError::FileDoesNotExist(_) => Self::new(ENOENT, msg),
            StagingError::Failure(_, code) => Self::new(code, msg),
        }
    }
}

#[derive(Debug)]
pub enum CommitError {
    Failure(String, i32),
    BareRepository,
}

#[derive(Debug)]
pub enum RefSearchError {
    /// Thrown when `git-ls-remote(1)` fails to execute.
    Failure(String),
    /// Generic IO error
    IOError(std::io::Error),
    /// Failed to find reference
    NotFound,
    /// `git-ls-remote(1)` returned garbage on `STDOUT`
    ParsingFailure(String),
    /// Failed to decode strings
    UTF8Decode(std::string::FromUtf8Error),
}

impl From<std::io::Error> for RefSearchError {
    #[inline]
    fn from(prev: std::io::Error) -> Self {
        Self::IOError(prev)
    }
}

impl From<std::string::FromUtf8Error> for RefSearchError {
    #[inline]
    fn from(prev: std::string::FromUtf8Error) -> Self {
        Self::UTF8Decode(prev)
    }
}

/// Functions
impl Repository {
    /// Return config value for specified key
    ///
    /// # Errors
    ///
    /// See [`CommitError`]
    ///
    /// # Panics
    ///
    /// When `git-commit(1)` fails to execute
    #[inline]
    pub fn commit(&self, message: &str) -> Result<(), CommitError> {
        if self.is_bare() {
            return Err(CommitError::BareRepository);
        }
        let out = self
            .git()
            .args(&["commit", "-m", message])
            .output()
            .unwrap();
        if out.status.code().unwrap() != 0 {
            let msg = String::from_utf8_lossy(out.stderr.as_ref()).to_string();
            let code = out.status.code().unwrap_or(1);
            return Err(CommitError::Failure(msg, code));
        }
        Ok(())
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
        let out = self
            .git()
            .arg("config")
            .arg(key)
            .output()
            .expect("Failed to execute git-config(1)");
        if out.status.success() {
            Ok(String::from_utf8(out.stdout)
                .expect("UTF-8 encoding")
                .trim()
                .to_owned())
        } else {
            match out.status.code().unwrap() {
                1 => Err(ConfigReadError::InvalidSectionOrKey(key.to_owned())),
                3 => {
                    let msg = String::from_utf8_lossy(out.stderr.as_ref()).to_string();
                    Err(ConfigReadError::InvalidConfigFile(msg))
                }
                _ => {
                    let msg = String::from_utf8_lossy(out.stderr.as_ref());
                    panic!("Unexpected git-config(1) failure:\n{}", msg);
                }
            }
        }
    }

    /// Read file from workspace or use `git-show(1)` if bare repository
    ///
    /// # Panics
    ///
    /// When UTF-8 encoding path fails
    ///
    /// # Errors
    ///
    /// When fails throws [`std::io::Error`]
    #[inline]
    pub fn hack_read_file(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        match self {
            Self::Normal { work_tree, .. } => {
                let absolute_path = work_tree.0.join(path);
                std::fs::read(absolute_path)
            }
            Self::Bare { .. } => {
                let path = format!(":{}", path.to_str().expect("UTF-8 string"));
                let mut cmd = self.git();
                cmd.args(&["show", &path]);
                let out = cmd.output().expect("Failed to execute git-show(1)");
                if out.status.success() {
                    Ok(out.stdout)
                } else if out.status.code().unwrap() == 128 {
                    let msg = format!("Failed to read file: {:?}", path);
                    Err(std::io::Error::new(std::io::ErrorKind::NotFound, msg))
                } else {
                    let msg = String::from_utf8_lossy(out.stderr.as_ref());
                    Err(std::io::Error::new(std::io::ErrorKind::Other, msg))
                }
            }
        }
    }

    #[must_use]
    #[inline]
    pub fn is_ancestor(&self, first: &str, second: &str) -> bool {
        let args = vec!["merge-base", "--is-ancestor", first, second];
        let mut cmd = self.git();
        cmd.args(args);
        let proc = cmd.output().expect("Failed to run git-merge-base(1)");
        proc.status.success()
    }

    /// # Errors
    ///
    /// See [`RefSearchError`]
    #[inline]
    pub fn remote_ref_to_id(&self, remote: &str, git_ref: &str) -> Result<String, RefSearchError> {
        let proc = self.git().args(&["ls-remote", remote, git_ref]).output()?;
        if !proc.status.success() {
            let msg = String::from_utf8_lossy(proc.stderr.as_ref()).to_string();
            return Err(RefSearchError::Failure(msg));
        }
        let stdout = String::from_utf8(proc.stdout)?;
        if let Some(first_line) = stdout.lines().next() {
            if let Some(id) = first_line.split('\t').next() {
                return Ok(id.to_owned());
            }
            return Err(RefSearchError::ParsingFailure(first_line.to_owned()));
        }

        Err(RefSearchError::NotFound)
    }

    /// # Errors
    ///
    /// When fails will return a String describing the issue.
    ///
    /// # Panics
    ///
    /// When git-sparse-checkout(1) execution fails
    #[inline]
    pub fn sparse_checkout_add(&self, pattern: &str) -> Result<(), String> {
        let out = self
            .git()
            .args(["sparse-checkout", "add"])
            .arg(pattern)
            .output()
            .expect("Failed to execute git sparse-checkout");

        if out.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(out.stderr.as_ref()).to_string())
        }
    }

    /// # Errors
    ///
    /// See [`StagingError`]
    ///
    /// # Panics
    ///
    /// Panics if fails to execute `git-add(1)`
    #[inline]
    pub fn stage(&self, path: &Path) -> Result<(), StagingError> {
        if self.is_bare() {
            return Err(StagingError::BareRepository);
        }

        let path = if path.is_absolute() {
            path.strip_prefix(self.work_tree().unwrap()).unwrap()
        } else {
            path
        };

        let file = path.as_os_str();
        let out = self.git().args(&["add", "--"]).arg(file).output().unwrap();
        match out.status.code().unwrap() {
            0 => Ok(()),
            128 => Err(StagingError::FileDoesNotExist(path.to_path_buf())),
            e => {
                let msg = String::from_utf8_lossy(&out.stdout).to_string();
                Err(StagingError::Failure(msg, e))
            }
        }
    }

    /// # Errors
    ///
    /// Fails if current repo is bare or dirty. In error cases see the provided string.
    ///
    /// # Panics
    ///
    /// When git-subtree(1) execution fails
    #[inline]
    pub fn subtree_add(
        &self,
        url: &str,
        prefix: &str,
        revision: &str,
        message: &str,
    ) -> Result<(), SubtreeAddError> {
        if self.is_bare() {
            return Err(SubtreeAddError::BareRepository);
        }

        if !self.is_clean() {
            return Err(SubtreeAddError::WorkTreeDirty);
        }

        let args = vec!["-q", "-P", prefix, url, revision, "-m", message];
        let mut cmd = self.git();
        cmd.arg("subtree").arg("add").args(args);
        let out = cmd.output().expect("Failed to execute git-subtree(1)");
        if out.status.success() {
            Ok(())
        } else {
            let msg = String::from_utf8_lossy(out.stderr.as_ref()).to_string();
            let code = out.status.code().unwrap_or(1);
            Err(SubtreeAddError::Failure(msg, code))
        }
    }

    /// # Errors
    ///
    /// Fails if current repo is bare or dirty. In error cases see the provided string.
    ///
    /// # Panics
    ///
    /// When git-subtree(1) execution fails
    #[inline]
    pub fn subtree_split(&self, prefix: &str) -> Result<(), SubtreeSplitError> {
        if self.is_bare() {
            return Err(SubtreeSplitError::BareRepository);
        }

        if !self.is_clean() {
            return Err(SubtreeSplitError::WorkTreeDirty);
        }

        let args = vec!["-P", prefix, "--rejoin", "HEAD"];
        let mut cmd = self.git();
        cmd.arg("subtree").arg("split").args(args);
        let result = cmd
            .spawn()
            .expect("Failed to execute git-subtree(1)")
            .wait();
        match result {
            Ok(code) => {
                if code.success() {
                    Ok(())
                } else {
                    Err(SubtreeSplitError::Failure(
                        "git-subtree split failed".to_owned(),
                        1,
                    ))
                }
            }
            Err(e) => {
                let msg = format!("{}", e);
                Err(SubtreeSplitError::Failure(msg, 1))
            }
        }
    }

    /// # Errors
    ///
    /// Fails if current repo is bare or dirty. In error cases see the provided string.
    ///
    /// # Panics
    ///
    /// When git-subtree(1) execution fails
    #[inline]
    pub fn subtree_pull(
        &self,
        remote: &str,
        prefix: &str,
        git_ref: &str,
        message: &str,
    ) -> Result<(), SubtreePullError> {
        if self.is_bare() {
            return Err(SubtreePullError::BareRepository);
        }

        if !self.is_clean() {
            return Err(SubtreePullError::WorkTreeDirty);
        }

        let args = vec!["-q", "-P", prefix, remote, git_ref, "-m", message];
        let mut cmd = self.git();
        cmd.arg("subtree").arg("pull").args(args);
        let out = cmd.output().expect("Failed to execute git-subtree(1)");
        if out.status.success() {
            Ok(())
        } else {
            let msg = String::from_utf8_lossy(out.stderr.as_ref()).to_string();
            let code = out.status.code().unwrap_or(1);
            Err(SubtreePullError::Failure(msg, code))
        }
    }

    /// # Errors
    ///
    /// Fails if current repo is bare. In other error cases see the provided message string.
    #[inline]
    pub fn subtree_push(
        &self,
        remote: &str,
        prefix: &str,
        git_ref: &str,
    ) -> Result<(), SubtreePushError> {
        if self.is_bare() {
            return Err(SubtreePushError::BareRepository);
        }

        let args = vec!["subtree", "push", "-q", "-P", prefix, remote, git_ref];
        let mut cmd = self.git();
        cmd.args(args);
        let out = cmd.output().expect("Failed to execute git-subtree(1)");
        if out.status.success() {
            Ok(())
        } else {
            let msg = String::from_utf8_lossy(out.stderr.as_ref()).to_string();
            let code = out.status.code().unwrap_or(1);
            Err(SubtreePushError::Failure(msg, code))
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum InvalidCommitishError {
    #[error("Invalid reference or commit id: `{0}`")]
    One(String),
    #[error("One or Multiple invalid reference or commit ids: `{0:?}`")]
    Multiple(Vec<String>),
}

/// Commit Functions
impl Repository {
    ///  Find best common ancestor between to commits.
    ///
    /// # Errors
    ///
    /// Will return `InvalidCommitishError::Multiple` when one or multiple provided ids do not
    /// exist
    ///
    /// # Panics
    ///
    /// When exit code of git-merge-base(1) is not 0 or 128
    #[inline]
    pub fn merge_base(&self, ids: &[&str]) -> Result<Option<String>, InvalidCommitishError> {
        let output = self
            .git()
            .arg("merge-base")
            .args(ids)
            .output()
            .expect("Executing git-merge-base(1)");
        if output.status.success() {
            let tmp = String::from_utf8_lossy(&output.stdout);
            if tmp.is_empty() {
                return Ok(None);
            }
            let result = tmp.trim_end();
            return Ok(Some(result.to_owned()));
        }
        match output.status.code().expect("Getting status code") {
            128 => {
                let tmp = ids.to_vec();
                let e_ids = tmp.iter().map(ToString::to_string).collect();
                Err(InvalidCommitishError::Multiple(e_ids))
            }
            1 => Ok(None),
            code => {
                panic!("Unexpected error code for merge-base: {}", code);
            }
        }
    }
}

#[cfg(test)]
mod test {

    // https://stackoverflow.com/a/63904992
    macro_rules! function {
        () => {{
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);

            // Find and cut the rest of the path
            match &name[..name.len() - 3].rfind(':') {
                Some(pos) => &name[pos + 1..name.len() - 3],
                None => &name[..name.len() - 3],
            }
        }};
    }

    mod repository_initialization {
        use crate::{RepoError, Repository};
        use tempdir::TempDir;

        #[test]
        fn git_dir_not_found() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let discovery_path = tmp_dir.path();
            let actual = Repository::discover(discovery_path);
            assert!(actual.is_err(), "Fail to find repo in an empty directory");
            let actual = actual.err().unwrap();
            assert!(actual == RepoError::GitDirNotFound);
            tmp_dir.close().unwrap();
        }

        #[test]
        fn bare_repo() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create_bare(repo_path).unwrap();
            assert!(
                match repo {
                    Repository::Normal { .. } => false,
                    Repository::Bare { .. } => true,
                },
                "Expected a bare repo"
            );

            assert_eq!(
                repo.work_tree(),
                None,
                "Bare repo should not have a work-tree"
            );
            tmp_dir.close().unwrap();
        }

        #[test]
        fn normal_repo() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create(repo_path).unwrap();

            assert!(
                match repo {
                    Repository::Normal { .. } => true,
                    Repository::Bare { .. } => false,
                },
                "Expected a non-bare repo"
            );

            assert_ne!(
                repo.work_tree(),
                None,
                "Non-Bare repo should have a work-tree"
            );
            tmp_dir.close().unwrap();
        }
    }

    mod is_bare {
        use crate::Repository;
        use tempdir::TempDir;

        #[test]
        fn yes() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create_bare(repo_path).unwrap();
            assert!(repo.is_bare(), "Expected a bare repository");

            tmp_dir.close().unwrap();
        }

        #[test]
        fn no() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create(repo_path).unwrap();
            assert!(!repo.is_bare(), "Expected a non-bare repository");

            tmp_dir.close().unwrap();
        }
    }

    mod config {
        use crate::Repository;
        use tempdir::TempDir;

        #[test]
        fn config() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create_bare(repo_path).unwrap();
            let actual = repo.config("core.bare").unwrap();
            assert_eq!(actual, "true".to_string(), "Expected true");

            tmp_dir.close().unwrap();
        }
    }

    mod sparse_checkout {
        use crate::Repository;
        use std::process::Command;
        use tempdir::TempDir;

        #[test]
        fn is_sparse() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create(repo_path).unwrap();
            let mut cmd = Command::new("git");
            let out = cmd
                .args(&["sparse-checkout", "init"])
                .current_dir(repo_path)
                .output()
                .unwrap();
            assert!(out.status.success(), "Try to make repository sparse");
            assert!(repo.is_sparse(), "Not sparse repository")
        }

        #[test]
        fn not_sparse() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create(repo_path).unwrap();
            assert!(!repo.is_sparse(), "Not sparse repository")
        }

        #[test]
        fn add() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create(repo_path).unwrap();
            repo.git()
                .args(["sparse-checkout", "init"])
                .output()
                .unwrap();
            let actual = repo.sparse_checkout_add("foo/bar");
            assert!(actual.is_ok(), "Expected successfull execution");

            tmp_dir.close().unwrap();
        }
    }

    mod subtree_add {
        use crate::{Repository, SubtreeAddError};
        use tempdir::TempDir;

        #[test]
        fn bare_repo() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create_bare(repo_path).unwrap();
            let actual =
                repo.subtree_add("https://example.com/foo/bar", "bar", "HEAD", "Some Message");
            assert!(actual.is_err(), "Expected an error");
            let actual = actual.unwrap_err();
            assert_eq!(actual, SubtreeAddError::BareRepository);
            tmp_dir.close().unwrap();
        }

        #[test]
        fn dirty_work_tree() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create(repo_path).unwrap();
            let actual =
                repo.subtree_add("https://example.com/foo/bar", "bar", "HEAD", "Some Message");
            assert!(actual.is_err(), "Expected an error");
            let actual = actual.unwrap_err();
            assert_eq!(actual, SubtreeAddError::WorkTreeDirty);
            tmp_dir.close().unwrap();
        }

        #[test]
        fn successfull() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create(repo_path).unwrap();
            let readme = repo_path.join("README.md");
            std::fs::File::create(&readme).unwrap();
            std::fs::write(&readme, "# README").unwrap();
            repo.stage(&readme).unwrap();
            repo.commit("Test").unwrap();
            let actual = repo.subtree_add(
                "https://github.com/kalkin/file-expert",
                "bar",
                "HEAD",
                "Some Message",
            );
            assert!(actual.is_ok(), "Failure to add subtree");
        }
    }

    mod subtree_pull {
        use crate::{Repository, SubtreePullError};
        use tempdir::TempDir;

        #[test]
        fn bare_repo() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create_bare(repo_path).unwrap();
            let actual =
                repo.subtree_pull("https://example.com/foo/bar", "bar", "HEAD", "Some Message");
            assert!(actual.is_err(), "Expected an error");
            let actual = actual.unwrap_err();
            assert_eq!(actual, SubtreePullError::BareRepository);
            tmp_dir.close().unwrap();
        }

        #[test]
        fn dirty_work_tree() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create(repo_path).unwrap();
            let actual =
                repo.subtree_pull("https://example.com/foo/bar", "bar", "HEAD", "Some Message");
            assert!(actual.is_err(), "Expected an error");
            let actual = actual.unwrap_err();
            assert_eq!(actual, SubtreePullError::WorkTreeDirty);
            tmp_dir.close().unwrap();
        }

        #[test]
        fn successfull() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create(repo_path).unwrap();
            let readme = repo_path.join("README.md");
            std::fs::File::create(&readme).unwrap();
            std::fs::write(&readme, "# README").unwrap();
            repo.stage(&readme).unwrap();
            repo.commit("Test").unwrap();
            repo.subtree_add(
                "https://github.com/kalkin/file-expert",
                "bar",
                "v0.10.1",
                "Some Message",
            )
            .unwrap();

            let actual = repo.subtree_pull(
                "https://github.com/kalkin/file-expert",
                "bar",
                "v0.13.1",
                "Some message",
            );
            assert!(actual.is_ok(), "Failure to pull subtree");
        }
    }

    mod remote_ref_resolution {
        use crate::RefSearchError;
        use crate::Repository;
        use tempdir::TempDir;

        #[test]
        fn not_found() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create(repo_path).unwrap();
            let result =
                repo.remote_ref_to_id("https://github.com/kalkin/file-expert", "v230.40.50");
            assert!(result.is_err());
            let actual = matches!(result.unwrap_err(), RefSearchError::NotFound);
            assert!(actual, "should not find v230.40.50")
        }

        #[test]
        fn failure() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create(repo_path).unwrap();
            let result = repo.remote_ref_to_id("https://example.com/asd/foo", "v230.40.50");
            assert!(result.is_err());
            let actual = matches!(result.unwrap_err(), RefSearchError::Failure(_));
            assert!(actual, "should not find any repo")
        }

        #[test]
        fn successfull_search() {
            let tmp_dir = TempDir::new(function!()).unwrap();
            let repo_path = tmp_dir.path();
            let repo = Repository::create(repo_path).unwrap();
            let result = repo.remote_ref_to_id("https://github.com/kalkin/file-expert", "v0.9.0");
            assert!(result.is_ok());
            let actual = result.unwrap();
            let expected = "24f624a0268f6cbcfc163abef5f3acbc6c11085e".to_string();
            assert_eq!(expected, actual, "Find commit id for v0.9.0")
        }
    }
}

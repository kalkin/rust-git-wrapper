# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [0.6.1] - 2022-10-01

### Added

- Add `Repository::is_shallow()`
- Add `setup_test_author()` helper function

## [0.6.0] - 2022-04-21

### Changed

- Fix `clippy::panic-in-result-fn`
- Remove most `unwrap_used()` calls
- Use `thiserror::Error`
- Split into `Repository` & `BareRepository`
- Replace `AbsoluteDirPath::new` with `TryFrom`

## [0.5.2] - 2022-03-06

### Added

- Stashing functions
- `Repository::commit_extended()`
- From trait for `RepoError` & `StagingError`
- experimental `function x::reset_hard()`

### Changed

- `CommitError` derive from `thiserror::Error`
- Simplify `StagingError`
- `subtree_split()` show progress
- Provide error messages for each enum value

### Fixed

- `Repository::stash_pop()` restore index
- `Repository::is_clean()` now also checks for staged content
- Trim whitespace in `Repository::head()` output
- `clippy::to-string-in-format-args`
- `merge_base` handle exit code 1

## [0.5.1] - 2022-01-17

### Fixed

- fix(git-stree-push): Pushing to remote

## [0.5.0] - 2022-01-05

### Changed

- ! Introduce a new OO API. This allows keeping track of `GIT_DIR` and
  `GIT_WORK_TREE` properly

## [0.4.1] - 2021-09-07

### Added

- Reexport PosixError
- Add Clone,Debug,Eq,PartialEq traits for Remote

## [0.4.0] - 2021-04-28

### Added

- `is_ancestor()`
- `ref_to_id()`
- `remotes()`
- `main_url()`

### Changed

- Fix `clippy::needless-pass-by-value`
- Fix `clippy::unwrap_used`
- Drop usage of `deprecated error_from_output()`
- Drop usage of `deprecated to_posix_error()`

### Fixed

- docs: Fix `clippy::doc-markdown`
- docs: Fix `clippy::missing-errors-doc`
- `or_fun_call` in `main_url()`
- rev-list handle broken UTF-8 encoding
- style: Fix `clippy::explicit-iter-loop`

## [0.2.0] - 2021-04-10

### Added

- Add `clone()`
- Add `git_cmd()`
- Add internal `cmd!` & `cmd_in_dir!` macros
- Implement parsing a commit range

### Changed

- `breaking_change`: `git_cmd` & `git_cmd_in_out` accept Vec<&str>

### Fixed

- docs: Fix typo

### Refactored

- Use the `cmd!` & `cmd_in_dir!` macros
- `top_level`() use `git_cmd()`

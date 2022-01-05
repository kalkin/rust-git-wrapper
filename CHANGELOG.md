# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

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

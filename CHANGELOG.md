# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

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

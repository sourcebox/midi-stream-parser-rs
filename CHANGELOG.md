# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - [Unreleased]

### Changed

- *BREAKING:* `parse` function returns a `ParserOutput` enum instead of `&[u8]`.
- Set Rust edition to 2024.

### Removed

- Internal fixed size buffer for SysEx messages.

## [0.1.1] - 2026-06-25

### Added

- Tests for more message types.

### Fixed

- Parsing of *Tune Request* message.

## [0.1.0] - 2022-12-18

Initial release.

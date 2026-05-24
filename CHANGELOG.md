# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [0.1.17] - 2026-05-24

### Fixed

- Fixed release note synchronization failing when helper scripts are checked out without executable permissions

## [0.1.16] - 2026-05-24

### Added

- Added deterministic GitHub Release description synchronization from `CHANGELOG.md`

### Changed

- Changed standalone asset publishing to upload files without regenerating GitHub release notes
- Changed standalone releases to publish only the `tauri-release` executable

### Fixed

- Fixed duplicated `What's Changed` sections in GitHub Release descriptions
- Fixed the unexpected `cargo-tauri-release` standalone asset being published

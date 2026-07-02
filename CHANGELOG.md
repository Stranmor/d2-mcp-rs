# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project uses semantic versioning once releases are tagged.

## [Unreleased]

### Added

- `d2_layouts` MCP tool for discovering available D2 layout engines.
- `d2_themes` MCP tool for discovering available D2 themes.
- Public demo diagram rendered from `docs/demo.d2`.
- Expanded README, client configuration docs, tool reference, and security model.
- GitHub issue templates and pull request template.

### Changed

- D2 theme validation now accepts the current official theme ID range through `303`.

### Verified

- CI runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` with D2 installed.

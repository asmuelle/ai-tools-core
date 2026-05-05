# Changelog

All notable changes to `ai-tools-core` are documented in this file.

## [0.1.0] - 2026-05-05

### Added

- Unified `Finding`, `Severity`, `Confidence`, `Location`, `RuleId`, and attribution types shared by the Rust AI coding toolchain.
- SARIF v2.1.0 renderer for GitHub Code Scanning and other SARIF consumers.
- Git helpers for changed-file discovery, unified diffs, blame attribution, and repository detection.
- Cargo helpers for metadata loading, workspace discovery, and Rust source file traversal.
- Secret scrubbing pipeline with built-in patterns for AWS keys, GitHub tokens, OpenAI keys, PEM private keys, JWTs, and path-level redaction.
- `.cargo-vibe.toml` configuration model and project config discovery helpers.

[0.1.0]: https://github.com/asmuelle/ai-tools-core/releases/tag/v0.1.0

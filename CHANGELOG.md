# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog.

## [Unreleased]

### Added

- Rust-based `opencli` core with command dispatch through `clap`
- Provider abstraction plus `openai-compatible` and `anthropic` backends
- Real SSE streaming text rendering for supported providers
- Tool system with `read_file`, `list_dir`, `search_files`, and `run_shell`
- Tool policy controls using `allowedToolKinds` and `allowedTools`
- Shell approval policies for interactive and non-interactive execution
- Audit logging with list, tail, export, and clear commands
- Session persistence with list, resume, rename, and delete commands
- Config diagnostics via `config doctor`
- Terminal UI chat flow via `tui`
- Shell completion generation and install helpers
- CI workflow and tagged release workflow
- Stable exit code mapping for common failure classes

### Changed

- Evolved the project from a minimal CLI into a modular runtime with explicit boundaries for app, provider, tools, output, approval, audit, and session layers

### Notes

- Versioning and release notes can move historical entries into dedicated release sections once tags are cut.

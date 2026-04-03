# Contributing

## Setup

1. Install stable Rust.
2. Clone the repository.
3. Build once with `cargo build`.
4. Run tests with `cargo test`.

## Daily Commands

- Build: `cargo build`
- Format: `cargo fmt`
- Check formatting: `cargo fmt -- --check`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Test all: `cargo test`
- Test one unit test: `cargo test renames_session`
- Test one integration file: `cargo test --test cli_smoke`
- Test one integration case: `cargo test --test cli_smoke completions_smoke`

## Project Structure

- `src/cli.rs`: CLI definitions
- `src/app.rs`: command orchestration
- `src/runtime.rs`: runtime dependency assembly
- `src/provider.rs`: provider implementations
- `src/tools.rs`: tool registry and tool logic
- `src/session.rs`: session persistence
- `src/audit.rs`: audit logging and queries
- `src/approval.rs`: command approval policies
- `src/tui.rs`: terminal UI flow

## Coding Guidelines

- Keep changes small and local to the correct module.
- Prefer extending traits and factories over adding branching in `main.rs`.
- Use `cargo fmt` instead of manual formatting.
- Use `anyhow::Result` in application code.
- Add `.context(...)` around IO and parsing failures.
- Avoid `unwrap()` in production code.
- Add or update tests when behavior changes.
- Update `README.md` for user-facing changes.
- Update `AGENTS.md` when repository workflow or conventions change.

## Provider Changes

- Keep provider-specific translation in `src/provider.rs`.
- Route provider selection through `src/provider_factory.rs`.
- Preserve support for `openai-compatible` and `anthropic` unless intentionally changing scope.
- Add serialization or behavior-focused tests for provider message changes.

## Tool Changes

- New tools should implement `tools::Tool`.
- Register tools through `ToolRegistry`.
- Every tool must define metadata: `name`, `description`, `kind`, `requires_approval`, and parameter schema.
- Respect `allowedToolKinds`, `allowedTools`, workspace path restrictions, approval policy, and audit logging.

## Before Opening a PR

1. Run `cargo fmt`.
2. Run `cargo clippy --all-targets --all-features -- -D warnings`.
3. Run `cargo test`.
4. Update docs if command surface or config changed.

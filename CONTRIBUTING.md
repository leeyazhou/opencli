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

- `crates/opencli/src/main.rs`: thin binary entrypoint
- `crates/opencli/src/cli.rs`: CLI definitions
- `crates/opencli-core/src/app/`: command orchestration
- `crates/opencli-core/src/runtime/`: runtime dependency assembly
- `crates/opencli-provider/src/provider/`: provider implementations
- `crates/opencli-tools/src/`: generic tool registry and filesystem/shell tools
- `crates/opencli-audit/src/`: audit logging and queries
- `crates/opencli-session/src/`: session persistence
- `crates/opencli-core/src/tools/`: core tool wrapper and delegation tools
- `crates/opencli-core/src/tui/`: terminal UI flow

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

- Keep provider-specific translation in `crates/opencli-provider/src/provider/`.
- Route provider selection through `crates/opencli-provider/src/provider_factory.rs`.
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

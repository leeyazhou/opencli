# AGENTS.md

This file is for coding agents operating in this repository.
It summarizes build/test commands, repository conventions, and implementation expectations.

## Scope

- Repository: `ai-cli`
- Language: Rust
- Package manager/build tool: Cargo
- Primary binary: `ai-cli`
- Runtime model: async CLI application using `tokio`

## Rule Files

- No `.cursorrules` file was found.
- No `.cursor/rules/` directory was found.
- No `.github/copilot-instructions.md` file was found.
- If any of those files are added later, treat them as higher-priority local instructions and update this file.

## Primary Commands

- Build debug binary: `cargo build`
- Build release binary: `cargo build --release`
- Run full test suite: `cargo test`
- Run a specific unit test by name: `cargo test renames_session`
- Run a specific integration test file: `cargo test --test cli_smoke`
- Run a single integration test by name: `cargo test --test cli_smoke completions_smoke`
- Run tests with captured output shown: `cargo test -- --nocapture`
- Check one binary command help: `cargo run -- audit --help`
- Run the CLI locally: `cargo run -- "Explain this repository"`

## Locked/CI Commands

- CI build: `cargo build --locked`
- CI tests: `cargo test --locked`
- Release build for local verification: `cargo build --release --locked`

## Formatting and Linting

- Format code: `cargo fmt`
- Check formatting without writing: `cargo fmt -- --check`
- Lint with Clippy: `cargo clippy --all-targets --all-features -- -D warnings`

Notes:

- `cargo fmt` is the canonical formatter even if formatting has not yet been wired into CI.
- Clippy is not currently enforced in CI, but agents should run it for non-trivial changes.

## Install and Release Helpers

- Unix-like install script: `./scripts/install.sh`
- Windows install script: `./scripts/install.ps1`
- Generate shell completions: `cargo run -- completions bash`
- Release automation lives in `.github/workflows/release.yml`

## Repository Architecture

Keep changes aligned with the current boundaries.
Do not collapse these layers back together.

- `main`: CLI entrypoint and command dispatch
- `app`: use-case orchestration and interactive command flows
- `runtime`: assembled dependencies for one app instance
- `provider_factory`: provider selection and creation
- `provider`: provider trait and provider implementations (directory module)
- `message`: shared chat message model
- `output`: renderer abstraction
- `tools`: tool trait, registry, policy checks, execution (directory module)
- `approval`: approval strategy abstraction
- `audit`: audit sink and audit query helpers
- `session`: session persistence
- `safety`: path and command classification logic
- `agent`: agent loop and autonomous execution logic
- `context`: workspace context gathering and injection
- `markdown`: terminal markdown rendering and formatting
- `tui`: terminal UI flow (directory module)

## File Map

Use this as a quick routing guide before making changes.

- `src/main.rs`: process entrypoint, CLI parsing, top-level dispatch
- `src/cli.rs`: clap command definitions and argument shapes
- `src/app.rs`: application orchestration for commands and interactive flows
- `src/runtime.rs`: constructs runtime dependencies from config
- `src/config.rs`: config schema, defaults, env override merging, config file loading
- `src/provider/`: provider trait and implementations (directory module)
  - `mod.rs`: `Provider` trait definition and exports
  - `openai_compatible.rs`: OpenAI-compatible provider implementation
  - `anthropic.rs`: Anthropic provider implementation
  - `types.rs`: shared provider request/response types
  - `util.rs`: SSE parsing and provider utilities
- `src/provider_factory.rs`: provider selection from config
- `src/message.rs`: shared chat message model passed through providers/sessions/tools
- `src/tools/`: tool trait, registry, policy enforcement, execution (directory module)
  - `mod.rs`: `Tool` trait definition, exports, policy enforcement
  - `registry.rs`: `ToolRegistry` construction and lookup
  - `types.rs`: shared tool metadata and schema types
  - `read_file.rs`: read_file tool implementation
  - `list_dir.rs`: list_dir tool implementation
  - `search_files.rs`: search_files tool implementation
  - `run_shell.rs`: run_shell tool implementation
- `src/approval.rs`: interactive and non-interactive shell approval behavior
- `src/audit.rs`: audit log writes, reads, export, and clear helpers
- `src/session.rs`: session persistence, lookup, rename, delete
- `src/output.rs`: terminal rendering abstraction and default renderer
- `src/agent.rs`: agent loop and autonomous execution logic
- `src/context.rs`: workspace context gathering and injection
- `src/markdown.rs`: terminal markdown rendering and formatting
- `src/tui/`: terminal UI chat loop (directory module)
  - `mod.rs`: TUI main loop and event handling
  - `state.rs`: TUI state management
- `src/safety.rs`: path normalization and command risk classification
- `src/errors.rs`: stable exit code inference
- `src/completions.rs`: shell completion generation
- `tests/cli_smoke.rs`: basic CLI integration smoke tests
- `scripts/install.sh`: Unix-like local install helper
- `scripts/install.ps1`: Windows local install helper
- `.github/workflows/ci.yml`: CI build and test workflow
- `.github/workflows/release.yml`: tagged release packaging workflow

## Implementation Strategy

- Prefer small changes inside the correct module instead of broad rewrites.
- Extend traits and factories rather than branching in `main.rs` or `app.rs`.
- New model backends should go through `provider::Provider` and `provider_factory`.
- New tools should go through `tools::Tool` and `ToolRegistry`.
- New rendering modes should go through `output::Renderer`.
- Approval logic should stay in `approval`, not inside tool implementations.
- Audit writes and queries should stay in `audit`, not inside app orchestration.

## Code Style

## Imports

- Follow rustfmt ordering.
- Prefer grouped imports from the same crate.
- Keep standard library imports first, external crates next, local crate imports last.
- Remove unused imports immediately.

## Formatting

- Use `cargo fmt`; do not hand-format against project style.
- Keep lines readable; let rustfmt make the final decision.
- Prefer trailing commas in multi-line structs, enums, and function calls.

## Types

- Prefer explicit domain structs over loose `serde_json::Value` unless dealing with provider payload edges or tool argument passthrough.
- Use `String` for owned text and `&str` for borrowed inputs.
- Use `Result<T>` from `anyhow` for application-layer errors.
- Use enums for stable categories such as command risk or exit code.
- Avoid introducing generics unless they materially improve reuse.

## Naming

- Types and traits: `UpperCamelCase`
- Functions and modules: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Keep names concrete and domain-specific, for example `run_audit_export`, `PolicyApproval`, `ProviderFactory`.
- Do not use vague names like `Manager`, `Helper`, or `Util` unless the abstraction is genuinely broad.

## Error Handling

- Bubble errors with `anyhow::Result` unless a local enum is clearly better.
- Add context with `.context(...)` or `.with_context(...)` around IO, JSON parsing, and process execution.
- Use actionable error messages; they should tell the operator what to fix.
- Preserve stable exit code behavior by letting errors reach `main`.
- Do not `unwrap()` or `expect()` in production code.
- `unwrap()` in tests is acceptable when it keeps tests concise.

## Async and Concurrency

- Use `tokio` for async entrypoints and provider calls.
- Keep async traits using the existing `async-trait` pattern.
- Avoid unnecessary background tasks.
- If a function does not need to be async, keep it synchronous.

## CLI Conventions

- Add new commands in `src/cli.rs`.
- Route command behavior through `app.rs`, not directly from `main.rs`.
- Help text should be concise and user-facing.
- If a command affects persisted state, ensure audit/session/config behavior remains coherent.

## Provider Conventions

- Preserve support for both `openai-compatible` and `anthropic`.
- Keep provider-specific request/response translation inside `provider.rs`.
- Shared provider selection belongs in `provider_factory.rs`.
- For streaming, prefer real SSE/event parsing when the upstream supports it.
- Add serialization tests when changing provider message formats.

## Tooling and Safety Conventions

- All tool execution must respect `allowedToolKinds` and `allowedTools`.
- Shell execution must go through approval policy and workspace path checks.
- File-system access must remain constrained by `workspaceRoot`.
- New tools need clear metadata: `name`, `description`, `kind`, `requires_approval`, and schema.

## Testing Expectations

- Add or update unit tests for behavior changes.
- Add CLI smoke tests for new top-level commands where practical.
- For provider work, add serialization or factory tests even if full network integration is not possible.
- For stateful features, prefer temp directories and isolated files.

## When Adding Features

- Update `README.md` if the user-facing surface changes.
- Update `AGENTS.md` if build/test/style expectations change.
- Keep release scripts, install scripts, and CI in sync with new commands or dependencies.

## Things to Avoid

- Do not put provider-specific branching in `app.rs` if it belongs in provider implementations.
- Do not bypass `ToolRegistry` for tool execution.
- Do not bypass `Approval` or `AuditLogger` in shell-like flows.
- Do not silently widen filesystem or shell permissions.
- Do not add unrelated dependencies without a concrete need.

## Open-Source Maturity Roadmap

This section tracks gaps identified during a full project audit.
Items are grouped by priority. Resolve high-priority items before tagging the first stable release.

### High Priority -- Foundational Issues

1. **Cargo.toml metadata is incomplete.**
   `[package]` only declares `name`, `version`, and `edition`. The package cannot be published to crates.io.
   Add: `description`, `license = "MIT"`, `repository`, `homepage`, `authors`, `keywords`, `categories`, `readme = "README.md"`, and `rust-version` (MSRV). Edition 2024 implies a minimum of Rust 1.85.0; make this explicit with `rust-version = "1.85.0"`.

2. **CI is missing formatting and linting checks.**
   `.github/workflows/ci.yml` only runs `cargo build` and `cargo test`. Code style regressions can land uncaught.
   Add steps: `cargo fmt -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`. Consider a separate `lint` job so build failures and style failures are reported independently.

3. **`.gitignore` is too minimal.**
   Currently only ignores `/target`. The repository already contains an `.idea/` directory and a stray `~/` directory (accidental artifact from a dev run).
   Add at minimum: `.idea/`, `.vscode/`, `~/`, `.DS_Store`, `*.swp`, `*.swo`, `*~`. Remove the tracked `.idea/` and `~/` directories from the repository.

4. **`LICENSE` file is missing the copyright holder.**
   The file reads `Copyright (c) 2026` with no person or organization name. Fill in the actual copyright holder.

5. **`.github/CODEOWNERS` uses a placeholder.**
   The file references `@example-owner`. Replace with the actual GitHub username(s) of the project maintainers.

6. **`SECURITY.md` has no actionable contact method.**
   The file says to report vulnerabilities "privately to the maintainers" but does not specify how. Add a contact email address or point to GitHub Security Advisories.

7. **Zero API documentation in source code.**
   No `///` or `//!` doc comments exist anywhere in the codebase. `cargo doc` produces empty output. All public traits, structs, enums, and functions should have doc comments. Prioritize the core abstractions: `Provider`, `Tool`, `Renderer`, `Runtime`, `Config`, and the message types.

### Medium Priority -- Open-Source Standards

8. **No `CODE_OF_CONDUCT.md`.**
   Most open-source projects of this scope adopt Contributor Covenant or a similar standard. Add one to set community expectations.

9. **No automated dependency updates.**
   There is no `.github/dependabot.yml` or `renovate.json`. Stale dependencies accumulate security vulnerabilities and compatibility debt. Add Dependabot configuration for Cargo with a weekly schedule.

10. **No supply-chain auditing (`cargo-deny`).**
    There is no `deny.toml`. License conflicts, known advisories, and duplicate dependencies go undetected. Add a `deny.toml` covering `[advisories]`, `[licenses]`, `[bans]`, and `[sources]` sections, and add a `cargo deny check` step to CI.

11. **README lacks badges.**
    No CI status badge, no license badge, no crates.io version badge. Add badges at the top of `README.md` for quick project health visibility.

12. **Release workflow is missing ARM targets.**
    Only `x86_64` targets are built for Linux, macOS, and Windows. Add `aarch64-apple-darwin` (Apple Silicon) and `aarch64-unknown-linux-gnu` (ARM Linux) to the release matrix.

13. **CI has no security audit step.**
    Neither `cargo audit` nor `cargo deny check advisories` runs in CI. Known CVEs in dependencies will go unnoticed. Add a dedicated audit job or integrate it into the existing CI workflow.

14. **Test coverage is low.**
    Only 3 integration smoke tests exist. At least 13 source files (`app.rs`, `approval.rs`, `audit.rs`, `cli.rs`, `completions.rs`, `config.rs`, `errors.rs`, `main.rs`, `output.rs`, `runtime.rs`, `agent.rs`, and most files under `tools/` and `tui/`) have no unit tests. Prioritize tests for `config.rs` (parsing, merging, defaults), `audit.rs` (write/read/export round-trips), `approval.rs` (policy decisions), and `tools/` (execution and policy enforcement).

### Low Priority -- Polish and Best Practices

15. **No `.editorconfig` file.**
    Cross-editor consistency for indentation, line endings, and trailing whitespace is not enforced. Add an `.editorconfig` specifying UTF-8, LF line endings, 4-space indentation for Rust, and trim trailing whitespace.

16. **No `rustfmt.toml`.**
    The project relies on default rustfmt behavior. Even if defaults are acceptable, an explicit (possibly empty) `rustfmt.toml` signals intent and prevents drift if defaults change in future Rust editions.

17. **No `.github/FUNDING.yml`.**
    If the project accepts sponsorships or donations, add a funding configuration so GitHub displays the Sponsor button.

18. **No `examples/` directory.**
    Example programs help new users and contributors understand typical usage patterns. Consider adding examples for common workflows: basic prompt, streaming chat, tool execution, custom provider configuration.

19. **No `benches/` directory.**
    No performance benchmarks exist. For a CLI tool, consider benchmarking: config parsing, session serialization/deserialization, markdown rendering, and provider response parsing.

20. **Stray `~/` directory in repository root.**
    A literal tilde directory exists at the repo root containing `.config/ai-cli/audit.jsonl`. This is an accidental artifact from a development run. Remove it from the repository and add `~/` to `.gitignore`.

21. **No pre-commit hook enforcement.**
    `CONTRIBUTING.md` recommends running `cargo fmt` and `cargo clippy` before committing, but this is not enforced. Consider adding a `.pre-commit-config.yaml` or `lefthook.yml` to automate these checks locally.

22. **No `[profile]` or `[features]` configuration in `Cargo.toml`.**
    There are no release profile optimizations (`lto`, `codegen-units`, `strip`) and no feature gates for optional functionality. For a CLI binary, adding `[profile.release]` with `lto = true`, `codegen-units = 1`, and `strip = true` can significantly reduce binary size. Feature gates can be deferred until the project grows.

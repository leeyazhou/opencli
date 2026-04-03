# ai-cli

 Rust implementation of a code-focused AI CLI with multi-provider support.

The codebase is organized around explicit boundaries:

- `app`: command orchestration and interactive flows
- `runtime`: assembled dependencies for one app instance
- `provider_factory`: provider selection and creation
- `provider`: provider trait and OpenAI-compatible implementation
- `message`: shared chat message model
- `output`: terminal renderer abstraction
- `tools`: tool trait, registry, and tool execution
- `approval`: shell approval policy abstraction
- `audit`: audit logging abstraction
- `session`: persistence
- `safety`: path and shell policies

This structure is intended to keep future changes localized:

- add a new model backend by implementing `provider::Provider`
- change provider selection by extending `provider_factory`
- add a new output mode by implementing `output::Renderer`
- add a new built-in tool by implementing `tools::Tool`
- change approval behavior by implementing `approval::Approval`
- change audit sink by implementing `audit::AuditLogger`
- assemble different runtime combinations in `runtime`

## Features

- Single prompt execution
- True SSE streaming for `openai-compatible` and `anthropic` text responses
- Interactive chat mode
- Basic terminal UI chat mode
- Local A2A delegation via `delegate_agent` and `a2a`
- Concurrent batch A2A delegation via `delegate_agents` and `a2a-batch`
- `run --file/--dir` context injection
- Local session save, list, and resume
- JSON config at `~/.config/ai-cli/config.json`
- Model-driven tool calls
- Built-in tools: `read_file`, `list_dir`, `search_files`, `run_shell`
- Shell approval modes with default `on-write`
- `models` and `config show` commands
- `completions` command
- `audit list` command
- `audit tail` command
- `audit clear/export` commands
- `session delete` command
- `session rename` command
- `config doctor` command
- Multi-provider support: `openai-compatible`, `anthropic`
- Tool kind allowlist and non-interactive approval policy

## Build

```bash
cargo build
```

## Install

Local install into `~/.local/bin`:

```bash
./scripts/install.sh
```

Windows PowerShell install:

```powershell
./scripts/install.ps1
```

Generate shell completions manually:

```bash
cargo run -- completions bash
cargo run -- completions zsh
cargo run -- completions fish
```

## Initialize config

```bash
cargo run -- config init
```

Then edit `~/.config/ai-cli/config.json` and set `apiKey`, `baseUrl`, and `model`.

Example config:

```json
{
  "provider": "openai-compatible",
  "baseUrl": "https://api.openai.com/v1",
  "apiKey": "",
  "model": "gpt-4.1",
  "anthropicVersion": "2023-06-01",
  "temperature": 0.2,
  "maxTokens": 4096,
  "approvalMode": "on-write",
  "nonInteractiveApproval": "deny",
  "workspaceRoot": ".",
  "sessionDir": "~/.config/ai-cli/sessions",
  "auditLogPath": "~/.config/ai-cli/audit.jsonl",
  "requestTimeoutMs": 120000,
  "shellTimeoutMs": 120000,
  "agentMaxSteps": 8,
  "a2aEnabled": true,
  "a2aMaxDepth": 2,
  "a2aMaxConcurrency": 4,
  "allowedToolKinds": [
    "filesystem-read",
    "filesystem-search",
    "shell",
    "agent"
  ],
  "allowedTools": []
}
```

## Usage

```bash
cargo run -- "Explain this repository"
cargo run -- chat
cargo run -- tui
cargo run -- a2a --role researcher "Summarize the repository architecture"
cargo run -- a2a-batch --file tasks.json --concurrency 3
cargo run -- models
cargo run -- completions zsh
cargo run -- config show
cargo run -- config doctor
cargo run -- audit list --limit 20
cargo run -- audit tail --lines 20 --follow
cargo run -- audit stats
cargo run -- audit graph
cargo run -- audit export --output /tmp/audit.json
cargo run -- audit clear
cargo run -- run --file src/main.rs "Explain this file"
cargo run -- run --dir src "Summarize this codebase"
cargo run -- session list
cargo run -- session resume <session-id>
cargo run -- session delete <session-id>
cargo run -- session rename <session-id> "New title"
```

## Tool behavior

The model can call these built-in tools:

- `read_file`
- `list_dir`
- `search_files`
- `run_shell`
- `delegate_agent`
- `delegate_agents`

`run_shell` is constrained to the configured workspace and uses `approvalMode`:

- `on-write`: read-like commands run directly, write-like commands ask for approval
- `always-ask`: every shell command asks for approval
- `never-ask`: shell commands run without approval except blocked dangerous commands

Dangerous commands such as `sudo`, `rm -rf /`, `mkfs`, `shutdown`, and `reboot` are blocked.

Shell commands are terminated when they exceed `shellTimeoutMs`.

`allowedToolKinds` controls which tool groups can run:

- `filesystem-read`
- `filesystem-search`
- `shell`
- `agent`

`allowedTools` can further restrict execution to specific tool names. An empty array means "allow all tools that pass kind checks".

`nonInteractiveApproval` controls shell behavior when there is no TTY:

- `deny`
- `allow-read-only`
- `allow-all`

## Providers

Supported providers:

- `openai-compatible`
- `anthropic`

For `anthropic`, set:

- `provider` to `anthropic`
- `baseUrl` to `https://api.anthropic.com/v1`
- `anthropicVersion` to a supported API version

## A2A

The CLI supports local agent-to-agent delegation.

Direct sub-agent invocation:

```bash
cargo run -- a2a --role researcher "Summarize the project structure"
cargo run -- a2a-batch --file tasks.json --concurrency 3
```

Example `tasks.json` for batch delegation:

```json
[
  {
    "role": "researcher",
    "task": "Summarize the provider module"
  },
  {
    "role": "reviewer",
    "task": "Identify risks in the TUI flow"
  }
]
```

Model-driven delegation uses the `delegate_agent` tool. Delegation is controlled by:

- `a2aEnabled`
- `a2aMaxDepth`
- `a2aMaxConcurrency`
- `allowedToolKinds`
- `allowedTools`

Sub-agent results are returned as structured JSON including:

- `agent_id`
- `parent_agent_id`
- `role`
- `task`
- `success`
- `output`
- `error`
- `duration_ms`

Tool audit events also include parent/child agent correlation fields when delegation is involved.

For concurrent batch delegation, one sub-agent failure does not abort the whole batch. Each result item reports its own `success` and `error` fields.

## Audit

Tool executions are written to `auditLogPath` as JSONL.

Query recent audit records:

```bash
cargo run -- audit list --limit 50
cargo run -- audit list --tool run_shell
cargo run -- audit list --event tool_finish
cargo run -- audit tail --lines 20 --follow
cargo run -- audit stats
cargo run -- audit graph
cargo run -- audit export --output /tmp/audit.json
cargo run -- audit clear
```

`audit graph` prints a readable agent delegation tree based on recorded A2A events.

## Session Management

Supported session commands:

- `session list`
- `session resume <id>`
- `session delete <id>`
- `session rename <id> <title>`

## Config Doctor

Run a basic local config validation:

```bash
cargo run -- config doctor
```

It checks for missing API keys, missing base URLs, empty tool policy, and invalid workspace paths.

## TUI

Launch the terminal UI chat mode:

```bash
cargo run -- tui
```

Controls:

- `Enter`: send prompt
- `Backspace`: delete character
- `Esc`: exit

## Exit Codes

`ai-cli` returns stable non-zero exit codes for common failure classes:

- `1`: generic failure
- `2`: config error
- `3`: authentication error
- `4`: permission or approval error
- `5`: not found
- `6`: validation error
- `7`: network or timeout error

## Logging

Structured tracing logs can be enabled with `RUST_LOG`.

Examples:

```bash
RUST_LOG=info cargo run -- chat
RUST_LOG=debug cargo run -- config doctor
```

Current logging focuses on:

- provider requests and streaming lifecycle
- tool execution and policy blocking
- config doctor endpoint probing

## Features

Compile-time feature flags:

- `anthropic`: enable Anthropic provider support
- `tui`: enable terminal UI support
- `benchmarks`: reserve benchmark-focused builds

Examples:

```bash
cargo build --no-default-features
cargo build --features anthropic,tui
```

## Benchmarks

Compile benchmark targets:

```bash
cargo bench --no-run
```

Run the markdown benchmark:

```bash
cargo bench --bench markdown_render
```

The repository also includes mock-server provider tests and fixture-based rendering tests in `tests/`.

## CI

GitHub Actions CI is included at `.github/workflows/ci.yml` and runs:

- `cargo build --locked`
- `cargo test --locked`

## Release

GitHub Actions release automation is included at `.github/workflows/release.yml`.

Pushing a tag like `v0.1.0` builds release archives for:

- Linux `x86_64-unknown-linux-gnu`
- macOS `x86_64-apple-darwin`
- Windows `x86_64-pc-windows-msvc`

## Repository Metadata

The repository also includes:

- `AGENTS.md` for coding agents
- `CONTRIBUTING.md` for contributors
- `CHANGELOG.md` for release tracking
- `docs/architecture.md` for module/layer overview
- `SECURITY.md` for vulnerability reporting guidance
- `.github/CODEOWNERS` for ownership rules
- issue and pull request templates under `.github/`

## Testing Strategy

The project currently uses multiple test layers:

- unit tests for core modules
- CLI smoke tests in `tests/cli_smoke.rs`
- fixture-based rendering tests in `tests/fixtures_*`
- mock-server provider integration tests in provider modules

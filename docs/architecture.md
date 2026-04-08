# Architecture

## Workspace Crates

1. Interface crates
- `opencli`

2. Orchestration crate
- `opencli-core`

3. Protocol and adapter crates
- `opencli-provider`
- `opencli-tools`
- `opencli-output`

4. State and config crates
- `opencli-config`
- `opencli-audit`
- `opencli-session`

## Execution Flow

1. `opencli` parses CLI arguments
2. `opencli-core::runtime` assembles dependencies
3. `opencli-core::app` runs a use case
4. `opencli-provider::ProviderFactory` constructs the selected provider
5. `opencli-core::agent` coordinates model/tool loops
6. `opencli-output` renders terminal and markdown output
7. `opencli-session` and `opencli-audit` persist state and trace history

## Quality System

- Unit tests for core modules
- CLI smoke tests
- Fixture-based rendering tests
- Mock-server provider tests
- Criterion benchmark entrypoints
- Structured tracing with request correlation ids

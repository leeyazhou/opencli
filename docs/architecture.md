# Architecture

## Layers

1. Interface layer
- `cli`
- `tui`

2. Application layer
- `app`

3. Service layer
- `services/config_doctor`

4. Runtime assembly layer
- `runtime`
- `provider_factory`

5. Adapter layer
- `provider`
- `tools`
- `output`
- `audit`

6. Shared models
- `message`
- `session`

7. Rendering layer
- `markdown`
- `output`
- `tui/view`

## Execution Flow

1. `main` parses CLI arguments
2. `runtime` assembles dependencies
3. `app` runs a use case
4. `provider_factory` constructs the selected provider
5. `agent` coordinates model/tool loops
6. `output`/`markdown` render output
7. `session`/`audit` persist state and trace history

## Quality System

- Unit tests for core modules
- CLI smoke tests
- Fixture-based rendering tests
- Mock-server provider tests
- Criterion benchmark entrypoints
- Structured tracing with request correlation ids

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${HOME}/.local/bin"
COMPLETION_DIR_BASE="${HOME}/.local/share"

mkdir -p "$TARGET_DIR"

cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"
install "$ROOT_DIR/target/release/opencli" "$TARGET_DIR/opencli"

mkdir -p "$COMPLETION_DIR_BASE/bash-completion/completions" "$COMPLETION_DIR_BASE/zsh/site-functions" "$COMPLETION_DIR_BASE/fish/vendor_completions.d"
"$ROOT_DIR/target/release/opencli" completions bash > "$COMPLETION_DIR_BASE/bash-completion/completions/opencli"
"$ROOT_DIR/target/release/opencli" completions zsh > "$COMPLETION_DIR_BASE/zsh/site-functions/_opencli"
"$ROOT_DIR/target/release/opencli" completions fish > "$COMPLETION_DIR_BASE/fish/vendor_completions.d/opencli.fish"

printf 'Installed opencli to %s\n' "$TARGET_DIR/opencli"
printf 'Installed shell completions under %s\n' "$COMPLETION_DIR_BASE"

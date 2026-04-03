#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${HOME}/.local/bin"
COMPLETION_DIR_BASE="${HOME}/.local/share"

mkdir -p "$TARGET_DIR"

cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"
install "$ROOT_DIR/target/release/ai-cli" "$TARGET_DIR/ai-cli"

mkdir -p "$COMPLETION_DIR_BASE/bash-completion/completions" "$COMPLETION_DIR_BASE/zsh/site-functions" "$COMPLETION_DIR_BASE/fish/vendor_completions.d"
"$ROOT_DIR/target/release/ai-cli" completions bash > "$COMPLETION_DIR_BASE/bash-completion/completions/ai-cli"
"$ROOT_DIR/target/release/ai-cli" completions zsh > "$COMPLETION_DIR_BASE/zsh/site-functions/_ai-cli"
"$ROOT_DIR/target/release/ai-cli" completions fish > "$COMPLETION_DIR_BASE/fish/vendor_completions.d/ai-cli.fish"

printf 'Installed ai-cli to %s\n' "$TARGET_DIR/ai-cli"
printf 'Installed shell completions under %s\n' "$COMPLETION_DIR_BASE"

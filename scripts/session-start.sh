#!/usr/bin/env bash
# Claude Code SessionStart hook: make the toolchain ready in cloud sessions.
# Local machines are left alone; run scripts/setup-toolchain.sh yourself.
set -euo pipefail

if [[ "${CLAUDE_CODE_REMOTE:-}" != "true" ]]; then
  exit 0
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Host toolchain from rust-toolchain.toml (installed once, then a no-op).
rustup show active-toolchain >/dev/null 2>&1 || rustup toolchain install
# Agave + platform-tools; also adds cargo-build-sbf to PATH via CLAUDE_ENV_FILE.
scripts/setup-toolchain.sh

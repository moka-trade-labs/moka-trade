#!/usr/bin/env bash
# Install the pinned SBF toolchain: Agave cargo-build-sbf + platform-tools.
#
# Downloads only from GitHub releases (release.anza.xyz may be blocked) and
# pre-installs platform-tools, because Agave 3.x cargo-build-sbf otherwise
# tries to fetch them itself. Idempotent: a second run only re-links.
#
# Usage: scripts/setup-toolchain.sh
# Then:  export PATH="$(scripts/setup-toolchain.sh --print-bin):$PATH"
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=pins.env
source "$SCRIPT_DIR/pins.env"

MOKA_TOOLS_DIR="${MOKA_TOOLS_DIR:-$HOME/.cache/moka-toolchain}"
AGAVE_DIR="$MOKA_TOOLS_DIR/agave-$AGAVE_VERSION"
AGAVE_BIN="$AGAVE_DIR/solana-release/bin"
PT_DIR="$HOME/.cache/solana/$PLATFORM_TOOLS_VERSION/platform-tools"

if [[ "${1:-}" == "--print-bin" ]]; then
  echo "$AGAVE_BIN"
  exit 0
fi

log() { echo "[setup-toolchain] $*" >&2; }

if [[ "$(uname -s)-$(uname -m)" != "Linux-x86_64" ]]; then
  log "only Linux x86_64 is scripted; see docs/architecture/p1-upstream-reproduction.md §2"
  exit 1
fi

download() { # url dest
  curl --fail --location --silent --show-error --retry 4 --retry-delay 2 -o "$2.part" "$1"
  mv "$2.part" "$2"
}

if [[ ! -x "$AGAVE_BIN/cargo-build-sbf" ]]; then
  log "installing Agave $AGAVE_VERSION into $AGAVE_DIR"
  mkdir -p "$AGAVE_DIR"
  tarball="$AGAVE_DIR/solana-release.tar.bz2"
  download "https://github.com/anza-xyz/agave/releases/download/$AGAVE_VERSION/solana-release-x86_64-unknown-linux-gnu.tar.bz2" "$tarball"
  tar -xjf "$tarball" -C "$AGAVE_DIR"
  rm -f "$tarball"
fi

if [[ ! -x "$PT_DIR/llvm/bin/llvm-objcopy" ]]; then
  log "installing platform-tools $PLATFORM_TOOLS_VERSION into $PT_DIR"
  mkdir -p "$PT_DIR"
  tarball="$PT_DIR.tar.bz2"
  download "https://github.com/anza-xyz/platform-tools/releases/download/$PLATFORM_TOOLS_VERSION/platform-tools-linux-x86_64.tar.bz2" "$tarball"
  tar -xjf "$tarball" -C "$PT_DIR"
  rm -f "$tarball"
fi

# cargo-build-sbf looks for platform-tools inside the SDK; its strip step can
# delete this link, so scripts/build-sbf.sh re-creates it before every build.
"$SCRIPT_DIR/build-sbf.sh" --link-only

log "cargo-build-sbf: $("$AGAVE_BIN/cargo-build-sbf" --version | head -1)"
log "platform-tools rustc: $("$PT_DIR/rust/bin/rustc" --version)"

# Expose the toolchain to GitHub Actions and Claude Code SessionStart hooks.
if [[ -n "${GITHUB_PATH:-}" ]]; then echo "$AGAVE_BIN" >> "$GITHUB_PATH"; fi
if [[ -n "${CLAUDE_ENV_FILE:-}" ]]; then echo "export PATH=\"$AGAVE_BIN:\$PATH\"" >> "$CLAUDE_ENV_FILE"; fi

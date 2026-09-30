#!/usr/bin/env bash
# Build one SBF program with the pinned platform-tools.
#
# Agave 3.0.10's SDK scripts (install.sh, run by the strip step) hardcode
# platform-tools v1.51: without help they download it, re-link rustup
# toolchains and replace our v1.52 link. We pre-create the SDK dependency
# links and the markers install.sh checks, so it finds everything installed
# and strips with the pinned v1.52 llvm-objcopy. See
# docs/architecture/p1-upstream-reproduction.md §2.
#
# Usage: scripts/build-sbf.sh <crate-dir> [extra cargo-build-sbf args...]
#        scripts/build-sbf.sh --link-only
# Output: <target-dir>/deploy/<name>.so (stripped)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=pins.env
source "$SCRIPT_DIR/pins.env"

MOKA_TOOLS_DIR="${MOKA_TOOLS_DIR:-$HOME/.cache/moka-toolchain}"
AGAVE_BIN="$MOKA_TOOLS_DIR/agave-$AGAVE_VERSION/solana-release/bin"
PT_DIR="$HOME/.cache/solana/$PLATFORM_TOOLS_VERSION/platform-tools"
SBF_SDK="$AGAVE_BIN/platform-tools-sdk/sbf"

link_tools() {
  local deps="$SBF_SDK/dependencies" sdk_tools sdk_criterion
  sdk_tools="$(sed -n 's/^tools_version=//p' "$SBF_SDK/scripts/install.sh")"
  sdk_criterion="$(sed -n 's/^  version=//p' "$SBF_SDK/scripts/install.sh" | head -1)"
  mkdir -p "$deps" "$deps/criterion"
  ln -sfn "$PT_DIR" "$deps/platform-tools"
  touch "$deps/platform-tools-$sdk_tools.md" "$deps/criterion-$sdk_criterion.md"
}

link_tools
if [[ "${1:-}" == "--link-only" ]]; then
  exit 0
fi

crate_dir="${1:?usage: build-sbf.sh <crate-dir> [args...]}"
shift
cd "$crate_dir"
"$AGAVE_BIN/cargo-build-sbf" --tools-version "$PLATFORM_TOOLS_VERSION" --skip-tools-install "$@"

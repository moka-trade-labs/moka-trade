#!/usr/bin/env bash
# Fetch the pinned Percolator sources into vendor/, build the SBF programs the
# tests need, and verify their sha256 against scripts/upstream-hashes.sha256.
#
# Usage: scripts/build-upstream.sh            build and verify
#        scripts/build-upstream.sh --record   build and rewrite the hash file
#                                             (a pin bump: execution-plan.md §5)
# Output: vendor/artifacts/*.so
# Requires: scripts/setup-toolchain.sh has run.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
# shellcheck source=pins.env
source "$SCRIPT_DIR/pins.env"

VENDOR="$ROOT/vendor"
ARTIFACTS="$VENDOR/artifacts"
HASHES="$SCRIPT_DIR/upstream-hashes.sha256"
record=false
[[ "${1:-}" == "--record" ]] && record=true

log() { echo "[build-upstream] $*" >&2; }

# checkout <repo-url> <commit> <dir>: exact commit, detached, clean tree.
checkout() {
  local url="$1" commit="$2" dir="$3"
  if [[ ! -d "$dir/.git" ]]; then
    git init -q "$dir"
    git -C "$dir" remote add origin "$url"
  fi
  if [[ "$(git -C "$dir" rev-parse -q --verify HEAD 2>/dev/null || true)" != "$commit" ]]; then
    log "fetching $url@$commit"
    git -C "$dir" fetch -q --depth 1 origin "$commit"
    git -C "$dir" checkout -q --detach "$commit"
  fi
  if [[ -n "$(git -C "$dir" status --porcelain --untracked-files=no)" ]]; then
    log "$dir has local modifications; vendored code must stay unmodified"
    exit 1
  fi
}

checkout "$WRAPPER_REPO" "$WRAPPER_COMMIT" "$VENDOR/percolator-prog"
checkout "$ENGINE_REPO" "$ENGINE_COMMIT" "$VENDOR/percolator"
# The wrapper tests load ../percolator-match/target/deploy/percolator_match.so.
checkout "$MATCH_REPO" "$MATCH_COMMIT" "$VENDOR/percolator-match"

# EXE-01: the wrapper must declare exactly the pinned engine revision.
declared="$(grep -oE 'aeyakovenko/percolator", rev = "[0-9a-f]{40}"' "$VENDOR/percolator-prog/Cargo.toml" | grep -oE '[0-9a-f]{40}' | sort -u)"
if [[ "$declared" != "$ENGINE_COMMIT" ]]; then
  log "wrapper declares engine '$declared', pin says $ENGINE_COMMIT"
  exit 1
fi

build() { # crate-dir
  log "building $1"
  # --locked: upstream Cargo.lock files are part of the pin.
  "$SCRIPT_DIR/build-sbf.sh" "$1" -- --locked >/dev/null
}
build "$VENDOR/percolator-prog"
build "$VENDOR/percolator-prog/tests/fixtures/auth_matcher"
build "$VENDOR/percolator-prog/tests/fixtures/hostile_matcher"
build "$VENDOR/percolator-match"

mkdir -p "$ARTIFACTS"
cp "$VENDOR/percolator-prog/target/deploy/percolator_prog.so" \
   "$VENDOR/percolator-prog/tests/fixtures/auth_matcher/target/deploy/auth_matcher.so" \
   "$VENDOR/percolator-prog/tests/fixtures/hostile_matcher/target/deploy/hostile_matcher.so" \
   "$VENDOR/percolator-match/target/deploy/percolator_match.so" \
   "$ARTIFACTS/"

cd "$ARTIFACTS"
if $record; then
  sha256sum -- *.so > "$HASHES"
  log "recorded hashes in $HASHES"
fi
sha256sum -c "$HASHES"
log "all artifacts match the pinned hashes"

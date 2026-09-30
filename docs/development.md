# Local development and testing

How to build, test and continue development on your own machine (or a cloud session). The pinned versions live in `scripts/pins.env`; the reasons for them are in `architecture/p1-upstream-reproduction.md`.

## 1. Prerequisites

| Tool | Version | Notes |
| --- | --- | --- |
| OS | Linux x86_64 | The scripts are tested there. On macOS, use a Linux VM, a devcontainer or Docker (`ubuntu:24.04`); see §6 |
| rustup | any recent | `rust-toolchain.toml` pins host Rust `1.94.1` and installs it on first use |
| git, curl, tar, bzip2 | any | Used by the scripts |
| Disk | ~6 GB free | Agave release (~0.5 GB), platform-tools (~2 GB unpacked), upstream `target/` dirs |
| Node 24 + pnpm 10 | later | Only from Phase 7 (SDK/keeper/web) |

You do **not** need the Solana CLI installed separately; `setup-toolchain.sh` fetches the Agave release that contains `cargo-build-sbf`.

## 2. First-time setup

```bash
git clone https://github.com/moka-trade-labs/moka-trade.git
cd moka-trade

# 1. SBF toolchain: Agave v3.0.10 + platform-tools v1.52, sha256-verified (~2 min, once).
scripts/setup-toolchain.sh
export PATH="$(scripts/setup-toolchain.sh --print-bin):$PATH"   # add to your shell rc

# 2. Pinned upstream: clone into vendor/, build the wrapper and matcher
#    fixtures for SBF, verify sha256 against scripts/upstream-hashes.sha256 (~4 min).
scripts/build-upstream.sh
```

Success looks like four `OK` lines followed by `all artifacts match the pinned hashes`. The verified programs are in `vendor/artifacts/`.

Where things go:

| Path | Contents | Committed? |
| --- | --- | --- |
| `~/.cache/moka-toolchain/agave-v3.0.10/` | Agave release (`cargo-build-sbf`) | no |
| `~/.cache/solana/v1.52/platform-tools/` | platform-tools (SBF rustc, LLVM) | no |
| `vendor/percolator-prog`, `vendor/percolator`, `vendor/percolator-match` | Pinned upstream checkouts | no (`vendor/` is ignored) |
| `vendor/artifacts/*.so` | Verified program binaries loaded by tests | no |
| `scripts/upstream-hashes.sha256` | Expected hashes (the pin manifest) | **yes** |

## 3. Everyday commands

Run these before every push; CI runs the same:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`cargo test` works without the SBF toolchain: tests that need `vendor/artifacts` skip with a note. CI sets `MOKA_REQUIRE_ARTIFACTS=1` so a missing artifact fails instead of skipping. After `build-upstream.sh` they also verify every loaded binary against the pin manifest (`tests/tests/pins.rs`).

Other useful commands:

| Task | Command |
| --- | --- |
| Build one SBF crate (ours or upstream) | `scripts/build-sbf.sh <crate-dir>` → `<target>/deploy/<name>.so` |
| Engine host tests (197 tests, ~1 min) | `(cd vendor/percolator && cargo test --release)` |
| Upstream wrapper suite (~45 min; known failures, see P1 record §2.1–2.2) | `(cd vendor/percolator-prog && cargo test --release --lib --tests --no-fail-fast)` |
| Shell script lint | `shellcheck scripts/*.sh` |

## 4. Workspace layout (what exists today)

| Path | What it is |
| --- | --- |
| `crates/moka-types` | Pins (`pins.rs`) and fixed-point units (`units.rs`). `no_std`, no Solana dependency |
| `crates/moka-math` | Checked `mul_div`/bps helpers with explicit rounding (CAP-07) |
| `tests/` (`moka-tests`) | Integration tests. `src/svm.rs` is the LiteSVM harness (`Harness::new`, `init_market_group`, `send_wrapper`, and CAP-13 `snapshot`/`trace`/`reconcile`); `load_pinned_program(name)` is the only way to load an upstream `.so`. Wrapper instructions are encoded by the pinned `percolator-prog` crate (git dependency at `WRAPPER_COMMIT`) |
| `programs/*` | Empty until Phases 3–5 |
| `scripts/` | Toolchain, upstream build, SessionStart hook |

Workspace lints (`Cargo.toml`) forbid `unsafe`, deny unchecked arithmetic, lossy casts, float arithmetic and `unwrap()`. Use `checked_*`, `try_from` and `moka-math`.

## 5. Continuing development

1. Read `docs/HANDOFF.md` (state, decisions, next step) and pick the next open issue of the current phase milestone on GitHub (labels `phase:N`).
2. Branch from `main`, one issue per PR. Use the PR template: list spec IDs and how to verify.
3. Put the spec requirement ID in every protocol test name or doc comment (e.g. `fn sec_09_market_groups_are_isolated()`).
4. Update `docs/HANDOFF.md` §2/§8 before you stop if state or decisions changed.

### Writing a Phase 2 test

The harness uses `litesvm 0.1` and `solana-sdk 1.18` to match upstream's own tests. A new test looks like `tests/tests/market_group.rs`:

```rust
#[test]
fn cap_13_deposit_reconciles() {
    require_artifacts!();                        // skip when vendor/artifacts is missing
    let mut h = Harness::new([1; 32]).unwrap();   // pinned wrapper + auth_matcher, USDC mint
    let group = h.init_market_group(MarketProfile::default()).unwrap();
    let before = h.snapshot("start", &[&group]).unwrap();
    // h.send_wrapper(WrapperIx::..., accounts, &[&signer]) for each step, then:
    h.reconcile(&[&group]).unwrap();              // conservation, custody, solvency
    println!("{}", before.trace(&h.snapshot("step", &[&group]).unwrap()));
}
```

Run one file with `cargo test -p moka-tests --test market_group -- --nocapture`. For account lists and argument choices, read the matching helper in `vendor/percolator-prog/tests/support/v16_svm.rs` (e.g. `init_primary_portfolio`, `deposit_primary`, `trade_cpi`).

### Bumping a pin

Follow `execution-plan.md` §5. Mechanically: edit `scripts/pins.env` and `crates/moka-types/src/pins.rs` (a test checks they agree), run `scripts/build-upstream.sh --record`, re-run the upstream suites, update the P1 record, and commit the new `upstream-hashes.sha256` in the same PR.

## 6. Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| `sha256sum: WARNING: 1 computed checksum did NOT match` | Different toolchain, dirty `vendor/` checkout, or a changed pin. Run `git -C vendor/<repo> status`, re-run `setup-toolchain.sh`, then `build-upstream.sh`. Never re-record hashes to make this pass without a pin bump |
| `sha256 mismatch ... refusing to install` from `setup-toolchain.sh` | The downloaded Agave or platform-tools tarball does not match `scripts/pins.env`. Do not edit the hash to make it pass; retry the download, and treat a repeat mismatch as a possible supply-chain problem |
| `MOKA_REQUIRE_ARTIFACTS is set but ... has no percolator_prog.so` | CI mode: artifact-dependent tests fail instead of skipping. Run `scripts/build-upstream.sh`, or unset the variable locally |
| `cargo-build-sbf` panics validating `--tools-version` | You ran `cargo build-sbf` directly without `--skip-tools-install`, so it tried the GitHub API. Use `scripts/build-sbf.sh`, which works offline |
| Build downloads platform-tools v1.51 | The SDK's `install.sh` markers are missing. `scripts/build-sbf.sh --link-only` recreates them |
| `edition2024` errors | Agave 2.x (platform-tools v1.48) is on `PATH` first. Put `$(scripts/setup-toolchain.sh --print-bin)` first |
| rustup "detected conflict" installing components | Two rustup installs raced (e.g. an editor and a terminal). Re-run `rustup toolchain install` alone |
| macOS | Not scripted. Run inside Docker: `docker run -it -v "$PWD":/w -w /w ubuntu:24.04`, then install `curl git bzip2 build-essential` and rustup, then §2 |

## 7. Cloud sessions (Claude Code on the web)

`.claude/settings.json` runs `scripts/session-start.sh` at session start in cloud sessions (`CLAUDE_CODE_REMOTE=true`). It installs the host toolchain and runs `setup-toolchain.sh`, and adds `cargo-build-sbf` to `PATH`. Run `scripts/build-upstream.sh` yourself when a task needs the upstream binaries. The environment's network policy must allow `github.com`, `objects.githubusercontent.com`/`release-assets.githubusercontent.com` and `crates.io`/`static.crates.io`; `release.anza.xyz` is not needed.

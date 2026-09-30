# Session handoff

Read this first in any new session (human or agent). Update it before ending a session that changed state or decisions. Newest session log at the bottom.

## 1. What Moka Trade is (one paragraph)

A Solana perpetuals DEX for freshly launched tokens that have a graduated DEX pool but no perp market. Any token passing onchain listing rules is listed permissionlessly. Each token gets its **own** Percolator market group (own USDC vault, insurance and backing). The mark price comes onchain from its **deepest DEX pool**. Traders send signed IOC orders to a real-capital **maker tranche**; a **backing tranche** earns Percolator's native, stress-responsive backing yield. Winners are paid in USDC.

## 2. Current state (2026-09-30, end of session 2)

- **Planning PR merged:** [moka-trade-labs/moka-trade#1](https://github.com/moka-trade-labs/moka-trade/pull/1).
- **Tracking:** backlog issues [#2](https://github.com/moka-trade-labs/moka-trade/issues/2)–[#35](https://github.com/moka-trade-labs/moka-trade/issues/35) created with `phase:N`, `area:*`, `type:*` labels. **Milestones and the project board are not created yet** (the session tooling cannot); the owner creates them in the GitHub UI and bulk-assigns by `phase:N` label.
- **Phase 1 (repo tooling and CI):** all deliverables on branch `claude/compassionate-shannon-43swup`, covering issues #2–#7: `scripts/setup-toolchain.sh`, `scripts/build-upstream.sh`, `scripts/build-sbf.sh`, `scripts/pins.env`, `scripts/upstream-hashes.sha256`, the Cargo workspace (`crates/moka-types`, `crates/moka-math`, `tests` = `moka-tests`), `.github/workflows/ci.yml`, issue/PR templates, and the SessionStart hook (`.claude/settings.json` → `scripts/session-start.sh`). Local guide: `docs/development.md`. Phase 1 is done when CI is green on `main`; CI has not run yet.
- **Verified in a cloud container:** a clean `setup-toolchain.sh && build-upstream.sh` (about 100 s + 4 min) reproduced all four pinned hashes, with the wrapper at `558778ee…546c` (P1 record §2). `cargo fmt/clippy -D warnings/test` pass (7 tests).
- **Upstream pin (unchanged):** wrapper `aeyakovenko/percolator-prog@5cb331dd` + engine `aeyakovenko/percolator@4db11a8c`, used unmodified.

## 3. Owner decisions (all dated 2026-09-30)

| # | Decision |
| --- | --- |
| 1 | Pin wrapper `5cb331dd` + declared engine `4db11a8c`; never pin an engine older than the wrapper declares |
| 2 | Phased roadmap: Track A MVP first, then Track B brings back everything deferred (`execution-plan.md`) |
| 3 | One market group per token; nothing shared between tokens (SEC-09) |
| 4 | Price = deepest supported DEX pool, onchain adapter, no trusted price bot on any cluster (ORC-08/09) |
| 5 | Dynamic APY = engine-native backing utilization fee (BST-07) |
| 6 | Two LP share classes (maker + backing) from the first release; devnet/fork design identical to mainnet (LP-08, environment parity) |
| 7 | Portfolio rent (~0.067 SOL per token per user, swept to the market on close) is disclosed, not subsidized (API-06A) |
| 8 | Sub-cent tokens: contract multiplier `10^k`; re-denominate by listing a new market group |
| 9 | Our programs use only the SEC-10 route allowlist; no vendored patches for the MVP; a maintained fork is a Phase 14 decision |
| 10 | Track progress with GitHub milestones (phases), issues and a project board; open issues after the planning PR merges |
| 11 | The public repo holds no named competitor analysis, deployment details of other projects, positioning slogans or ranked roadmap strategy; those live in the private repo `moka-trade-labs/strategy` |

## 4. Key findings to remember

- The wrapper is a multi-asset "market group"; a one-asset group is **3,003 bytes** (~0.022 SOL). A portfolio is **9,563 bytes** (~0.067 SOL). `ClosePortfolio` sweeps rent to the market account.
- Upstream is research code run by an automated invariant-hunting loop. `tests/invariants/invariant_status.tsv` shows 0/89 proven and 8 `REFUTED_CURRENT`; `open_findings.tsv` lists ~165 open public-route findings. A red suite on `main` is its documented bug list.
- The core-route risks to test in Phase 2 are listed in P1 record §2.2. The top one is CPI backing-fee consent, #223/#224 (an "LP siphon").
- Other Percolator-based venues exist; some run maintained forks rather than upstream pins, so their versions are not comparable to ours. Competitive analysis is kept outside this public repository by owner decision (2026-09-30).
- Native dynamic yield: `rate = base + slope·util/kink` below the kink, steeper above it. The unit is 1e-9 per slot (≈7.9% APR per unit at 400 ms slots). It is charged to lien holders and paid to the backing bucket.
- `mark_e6 ∈ [1, 1e12]` explains the contract multiplier. A single-leg `TradeCpi` measured ~94k CU, of which the matcher took 694 CU.
- `SetMatcherConfig` payload changed between `2b1d025` and `5cb331dd`; encode from the head decoder.

## 5. Build recipe (cloud container notes)

- Agave `v3.0.10` `cargo-build-sbf` + platform-tools **v1.52** extracted to `~/.cache/solana/v1.52/platform-tools` and symlinked as `<agave>/bin/platform-tools-sdk/sbf/dependencies/platform-tools`. Run `cargo build-sbf --tools-version v1.52 --skip-tools-install`, then strip with the v1.52 `llvm-objcopy` (the SDK strip script tries to reinstall v1.51 and removes the symlink). Agave 2.x silently uses platform-tools v1.48 and fails.
- Upstream tests also need `tests/fixtures/hostile_matcher` built and a sibling `../percolator-match` checkout (`60aac3a9…`) built; the README omits both.
- `release.anza.xyz` is blocked by the container's network policy; GitHub release downloads work.
- **All of the above is now scripted** (`scripts/setup-toolchain.sh`, `scripts/build-sbf.sh`, `scripts/build-upstream.sh`; see `docs/development.md`). `build-sbf.sh` pre-creates the SDK's v1.51 markers so the strip step never downloads v1.51 or unlinks v1.52. The repo's `rust-toolchain.toml` (1.94.1) also applies inside `vendor/`; never run two rustup installs at once (they race and fail with "detected conflict").

## 6. Document map

| Need | File |
| --- | --- |
| Local setup, build, test, troubleshooting | `docs/development.md` |
| Required behavior | `docs/spec.md` |
| Phases, verification, order | `docs/execution-plan.md` |
| Pins, test results, route allowlist, avoided-route needs | `docs/architecture/p1-upstream-reproduction.md` |
| Oracle adapter and contract sizing | `docs/architecture/dex-oracle-design.md` |
| Goal vs plan, decisions, LP classes explained | `docs/architecture/mvp-assessment.md` |
| Test environments (LiteSVM/Mollusk/Surfpool) | `docs/architecture/test-environments.md` |
| Performance/CU strategy | `docs/architecture/performance-strategy.md` |
| Repo layout, program split, tracking | `docs/architecture/repo-and-tracking.md` |
| Maintained fork explainer | `docs/architecture/fork-strategy.md` |
| Product feature ideas | `docs/research/feature-ideas.md` |
| Owner curriculum and progress | `docs/research/learning-curriculum.md`, `docs/research/learning-progress.md` |
| Issue backlog (created as #2–#35) | `docs/roadmap/issue-backlog.md` |

## 7. Open questions

- Exact CU of `PushAuthMark` and of our adapter (measure in Phase 5).
- Surfpool cheatcode names for Pyth refresh and time travel (verify in Phase 6).
- Initial parameter profile (spreads, caps `α/β`, kink curve, leverage): calibration blocker (TEST-06), needed before Phase 8.
- Whether to add the maintained fork before mainnet (Phase 14 criteria in `fork-strategy.md`).

## 8. Session log

- **2026-09-30** (Claude Code on the web): analyzed docs; reproduced upstream builds and tests at `2b1d025` and `5cb331dd`; researched the upstream finding ledger, Surfpool, DEX pool layouts and oracle performance; recorded owner decisions 1–10; wrote the phased plan, architecture notes, feature ideas and curriculum; opened the planning PR. Competitor-specific analysis was removed from the public repo at the owner's request and the PR history was squashed. Named competitor and adjacent-project details (including the older thesis and validation-review benchmarks) now live in the private `moka-trade-labs/strategy` repo; public docs keep neutral category descriptions.
- **2026-09-30, session 2** (Claude Code on the web): PR #1 merged. Created issues #2–#35 from the backlog (labels only, no milestones). Implemented Phase 1 (#2–#7): toolchain and upstream build scripts with a committed hash manifest, a strict-lint Cargo workspace, CI, templates, the SessionStart hook and `docs/development.md`. Reproduced all pinned hashes from a clean build. Found that upstream's own LiteSVM harness (`vendor/percolator-prog/tests/support/v16_svm.rs`, 6.2k lines, `litesvm 0.1` + `solana-sdk 1.18`) already drives every Phase 2 flow.
- **Next session starts here:** (1) get Phase 1 merged with CI green; the owner creates the milestones/board. (2) Phase 2 (#9 first). Decide the harness approach: **recommended**, `moka-tests` takes `percolator-prog` as a **git** dependency at `WRAPPER_COMMIT` (`default-features = false`, `no-entrypoint`; not a `vendor/` path, so `cargo test` still works without `vendor/`) for instruction encoders and state decoders (the ABI source of truth), uses `litesvm 0.1`/`solana-sdk 1.18` to match upstream, and keeps a thin harness of our own that follows `v16_svm.rs` rather than including it. Load programs only via `moka_tests::load_pinned_program`.

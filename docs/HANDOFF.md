# Session handoff

Read this first in any new session (human or agent). Update it before ending a session that changed state or decisions. Newest session log at the bottom.

## 1. What Moka Trade is (one paragraph)

A Solana perpetuals DEX for freshly launched tokens that have a graduated DEX pool but no perp market. Any token passing onchain listing rules is listed permissionlessly. Each token gets its **own** Percolator market group (own USDC vault, insurance and backing). The mark price comes onchain from its **deepest DEX pool**. Traders send signed IOC orders to a real-capital **maker tranche**; a **backing tranche** earns Percolator's native, stress-responsive backing yield. Winners are paid in USDC.

## 2. Current state (2026-09-30)

- **Code:** none yet. Docs, plans and research only. The next step is Phase 1 (repo tooling/CI), then Phase 2 (LiteSVM slice).
- **Upstream pin (accepted):** wrapper `aeyakovenko/percolator-prog@5cb331dde354517c6371a8acf92cecb194f3bb73` + engine `aeyakovenko/percolator@4db11a8cb0053815e23a35d3a7d3edc265d8d866`, used unmodified.
- **Verified in a cloud container:** engine 197/197 tests pass. Wrapper builds (sha256 `558778ee…546c`), and its suite runs 1,876 pass / 173 fail. The failures are upstream's documented open-finding tests plus certification guards pinned to engine `94979ede`, classified in the P1 record §2.2.
- **Branch/PR:** planning work is on `claude/brave-meitner-ii42xc`; PR [moka-trade-labs/moka-trade#1](https://github.com/moka-trade-labs/moka-trade/pull/1) targets `main`. After merge, create issues from `docs/roadmap/issue-backlog.md`.

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
| 11 | The public repo holds no named competitor analysis, deployment details of other projects, positioning slogans or ranked roadmap strategy; those live in a private location chosen by the owner |

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

## 6. Document map

| Need | File |
| --- | --- |
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
| Issue backlog to create after merge | `docs/roadmap/issue-backlog.md` |

## 7. Open questions

- Exact CU of `PushAuthMark` and of our adapter (measure in Phase 5).
- Surfpool cheatcode names for Pyth refresh and time travel (verify in Phase 6).
- Initial parameter profile (spreads, caps `α/β`, kink curve, leverage): calibration blocker (TEST-06), needed before Phase 8.
- Whether to add the maintained fork before mainnet (Phase 14 criteria in `fork-strategy.md`).

## 8. Session log

- **2026-09-30** (Claude Code on the web): analyzed docs; reproduced upstream builds and tests at `2b1d025` and `5cb331dd`; researched the upstream finding ledger, Surfpool, DEX pool layouts and oracle performance; recorded owner decisions 1–10; wrote the phased plan, architecture notes, feature ideas and curriculum; opened the planning PR. Competitor-specific analysis was removed from the public repo at the owner's request and the PR history was squashed.

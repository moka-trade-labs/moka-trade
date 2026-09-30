# Execution plan: phased roadmap

- Date: 2026-09-30 (supersedes the 2026-09-12 P0–P8 plan and the same-day M0–M4 draft).
- Status: owner-approved direction (see [MVP assessment](architecture/mvp-assessment.md) §6 and [HANDOFF](HANDOFF.md)). Not evidence of completed engineering, security review or commercial validation.
- Requirements: [specification](spec.md). Pins and route allowlist: [P1 record](architecture/p1-upstream-reproduction.md). Oracle: [DEX-pool oracle design](architecture/dex-oracle-design.md). Learning track: [curriculum](research/learning-curriculum.md).

## 1. How to read this plan

Development is split into **small, sequential phases**. Each phase has one goal, a short deliverable list, a "how you verify" checklist you can run yourself, and the curriculum modules it depends on.

- **Track A (Phases 0–8): the MVP.** Low-hanging fruit that produces a public devnet product whose design matches the intended mainnet design (spec "Environment parity").
- **Track B (Phases 9–15): production hardening.** Everything removed from the MVP for speed comes back here, in order of value. Nothing is dropped.

Rules for every phase:

1. One phase = one GitHub milestone; one deliverable = one issue; one issue = one small PR.
2. A phase is done only when its verification checklist passes and `docs/HANDOFF.md` records what changed.
3. No phase starts while the previous phase has an open blocker. Learning modules can run ahead of the build.

## 2. Track A — MVP (public devnet)

### Phase 0 — Foundations ✅ (done 2026-09-30)

- **Goal:** know exactly what we build on.
- **Delivered:** upstream pin (wrapper `5cb331dd` + engine `4db11a8c`); reproduction recipe and results; instruction matrix; SEC-10 route allowlist mapped to upstream open findings; owner decisions; oracle design.
- **Verify:** read `architecture/p1-upstream-reproduction.md` §1–§5 and `HANDOFF.md`.

### Phase 1 — Repo tooling and CI (small) — 🟡 delivered on branch, CI pending (2026-09-30)

- **Status:** all deliverables exist (issues #2–#7); a clean cloud build reproduced every pinned hash. Done when CI is green on `main`. How to run it: [development.md](development.md).
- **Goal:** anyone can build and test the pinned upstream with one command.
- **Deliverables:** `scripts/setup-toolchain.sh` (Agave 3.0.10 + platform-tools v1.52, including the strip workaround); `scripts/build-upstream.sh` (clones pins into `vendor/`, builds wrapper and fixtures, checks sha256); a Rust workspace skeleton (`crates/`, `tests/`); a GitHub Actions workflow running `cargo fmt`, `clippy` and tests; a SessionStart hook for Claude Code on the web.
- **Verify:** on a fresh machine, `scripts/setup-toolchain.sh && scripts/build-upstream.sh` prints hashes matching the P1 record; the CI badge is green.
- **Learn first:** curriculum R1 (Cargo/workspaces), S1 (Solana accounts and programs).

### Phase 2 — Localnet vertical slice with upstream only (no programs of ours)

- **Goal:** prove the Percolator accounting path we depend on, before writing any program.
- **Deliverables:** LiteSVM tests in `tests/` against the pinned `.so` for one single-asset market group:
  1. `InitMarket` → maker and trader portfolios → deposit.
  2. `TradeNoCpi`, then `TradeCpi` through the upstream `auth_matcher`.
  3. `TopUpBackingBucket` → `PushAuthMark` shock → `PermissionlessCrank` liquidation → `SyncBackingDomainLedger` → `WithdrawBackingBucketEarnings` → permitted withdrawals.
  4. A two-market-group isolation test (SEC-09).
  5. The core-route attack list from P1 record §2.2, including the #223 LP-siphon attempt.
  6. Token and ledger reconciliation after every step (CAP-13).
- **Verify:** `cargo test -p moka-tests` passes; a printed trace shows balances before and after each step; each §2.2 core-route case is marked pass, avoided or blocker in the P1 record.
- **Learn first:** S2 (transactions/CPI/PDAs), P1 (Percolator engine concepts), T1 (LiteSVM).

### Phase 3 — Matcher program

- **Goal:** our pricing, onchain.
- **Deliverables:** `programs/matcher`: the spec §8.3 inventory-aware quote, oracle band, depth-scaled caps read from the registry account, stale-mark refusal, fee and backing-fee caps, golden vectors shared with the SDK; LiteSVM tests replacing `auth_matcher` in the Phase 2 slice.
- **Verify:** the golden-vector test passes in Rust; the Phase 2 slice passes with our matcher; the malicious-response tests reject.
- **Learn first:** R2 (ownership, zero-copy, `bytemuck`), M1 (fixed-point maths), S3 (compute budget).

### Phase 4 — LP vault program (maker and backing classes)

- **Goal:** LPs can deposit into, and redeem from, both tranches.
- **Deliverables:** `programs/lp-vault`: the maker share class (a PDA-owned maker portfolio, `SetMatcherConfig`), then the backing share class (backing authority, `TopUpBackingBucket`/`WithdrawBackingBucket`/`WithdrawBackingBucketEarnings`, `SyncBackingDomainLedger`); conservative NAV per class; pending-redemption exclusion; first-depositor protection.
- **Verify:** property tests show deposit→redeem round trips never create value; the loss-then-deposit case charges old LPs, not new ones; the backing class's earnings equal `utilization_fee_earnings` deltas.
- **Learn first:** D2 (vault share maths, ERC-4626 lessons), M2 (rounding and invariants).

### Phase 5 — Market registry and DEX-pool oracle

- **Goal:** permissionless listing with onchain prices.
- **Deliverables:** `programs/market-registry`: onchain listing rules (ORC-09), `InitMarket` as `marketauth`, role hand-off to lp-vault/adapter PDAs, candidate pool registration, the PumpSwap adapter first, then Raydium CPMM and Meteora DAMM v2, Pyth SOL/USD conversion, smoothing/clamp/cross-check, the contract multiplier and re-denomination.
- **Verify:** golden-account fixtures from mainnet decode correctly; the flash-liquidity, one-slot spike, stale Pyth and disagreeing-pool tests reject; a listing end-to-end test creates a fully isolated market.
- **Learn first:** D3 (AMM maths, oracle manipulation), S4 (reading foreign program accounts safely).

### Phase 6 — Mainnet-fork rehearsal (Surfpool)

- **Goal:** run our real programs against real mainnet pools, Pyth accounts and token mints with zero real funds.
- **Deliverables:** `ops/surfpool/` config and runbook; a scripted scenario that lists a real graduated token read from mainnet state, trades, shocks the price by swapping in the forked pool, liquidates and redeems LP shares. See `architecture/test-environments.md`.
- **Verify:** one command replays the scenario and prints the same reconciliation report as Phase 2.
- **Learn first:** T2 (Surfpool, account dumps).

### Phase 7 — SDK and keeper

- **Goal:** anyone can use and run the protocol without our servers.
- **Deliverables:** `packages/sdk` (hand-encoded instructions with golden bytes checked against the pinned decoder, account decoders, unit conversions, the API-06A rent disclosure helper); `services/keeper` (mark cranks, liquidation hints, ledger syncs; restart-safe).
- **Verify:** the SDK round-trip test matches Rust golden bytes; killing and restarting the keeper mid-scenario causes no double action.
- **Learn first:** F1 (TypeScript + `@solana/kit`), O1 (keeper design).

### Phase 8 — Web app and public devnet launch (MVP)

- **Goal:** the MVP is live on devnet.
- **Deliverables:** `apps/web` (list a token, trade, positions, LP deposit/redeem per class with live utilization, rate, dollars earned and principal at risk, rent disclosure); a devnet deployment manifest; at least two tokens listed from real devnet PumpSwap pools; a scripted demo; a short user-interview round with the first devnet users.
- **Verify:** follow `ops/devnet-demo.md` end to end in a browser wallet; the manifest hashes match the CI build.
- **Learn first:** F2 (wallet UX, transaction lifecycle).

## 3. Track B — production hardening (the items deferred from the MVP)

Ordered from cheapest and most valuable first. Each maps to the curriculum that makes it achievable.

| Phase | Brings back | Deliverables | Curriculum |
| --- | --- | --- | --- |
| **9. Indexer and public API** | Postgres indexer, `/v1` API, WebSocket stream (API-03..05, API-08) | Rebuildable indexer with finality handling, reconciliation job, rate-limited read API | O2 |
| **10. Operations** | Dual independent keepers, monitoring, incident drills (OPS-03/04) | Second keeper operator (or an open bounty), alerts, runbooks, drill records | O1, O3 |
| **11. Economics** | Simulator, equal-capital comparison, custom backstop controller and reward escrow (TEST-05, BST-02/03, LP-07) | `sim/` with scenario library; controller only if it beats native BST-07 | M3, D4 |
| **12. Oracle breadth** | Pyth/Hybrid path for large tokens; CLMM, Whirlpool and DLMM adapters; offchain risk attestations where onchain rules can't reach (ORC-07) | New adapters with fixtures; attestation format if still needed | D3 |
| **13. Performance** | Pinocchio/assembly hot paths, CU budgets (see `architecture/performance-strategy.md`) | Rewritten matcher/adapter hot paths with before/after CU tables | R4, S5 |
| **14. Assurance** | Independent review sign-offs, audit, maintained fork decision, bug bounty | Audit report and fixes; fork per `architecture/fork-strategy.md` if needed | P2, S6 |
| **15. Mainnet pilot** | Capped real-money launch (OPS-05) | Explicit authorization, multisig/upgrade policy, capped markets | All |

The interview sprint returns in two lightweight forms: Phase 8 (first devnet users) and continuously after launch. The product feature ideas (`research/feature-ideas.md`) slot in after Phase 8 or in parallel with Track B, one feature per phase.

## 4. Continuous gates (every PR)

- Build our programs with the pinned toolchain; run `cargo test` and the LiteSVM suite against the pinned wrapper binary.
- Put spec requirement IDs in every protocol test name or doc comment and in every security-sensitive PR description.
- A test over instruction tags checks that no wrapper route outside the SEC-10 allowlist is invoked.
- Checked arithmetic only; no floating point in onchain or SDK financial paths (CAP-07).
- No secrets, keypairs or `.env` values committed.

## 5. Upstream pin policy

- Pin a wrapper commit together with the engine revision it declares; never mix them.
- On every bump: rebuild, re-run the full upstream suite, and diff `invariant_status.tsv` and `open_findings.tsv`. Any new failure on an allowlisted route blocks the bump until assessed.
- Do not patch vendored code for the MVP. If a fix is needed, prefer an upstream commit; our own fork is a Phase 14 decision.

## 6. Scope reminder

This plan does **not** authorize a mainnet deployment or movement of real funds. Phase 15 requires a separate explicit authorization.

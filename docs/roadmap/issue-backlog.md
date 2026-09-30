# Initial issue backlog

**Created 2026-09-30** as GitHub issues [#2](https://github.com/moka-trade-labs/moka-trade/issues/2)–[#35](https://github.com/moka-trade-labs/moka-trade/issues/35) (backlog item *n* is issue *n + 1*). The session tooling could create issues and labels but not milestones, so each issue carries a `phase:N` label and names its milestone in the body. Create the milestones in the GitHub UI and bulk-assign by label.

Each line becomes one GitHub issue under the named milestone. The labels and template are in `docs/architecture/repo-and-tracking.md` §3. Track B phases get their issues when Track A reaches Phase 8.

## Milestone: Phase 1 — Repo tooling and CI

1. `scripts/setup-toolchain.sh`: install Agave 3.0.10 + platform-tools v1.52 reproducibly (incl. strip workaround) — area:ops
2. `scripts/build-upstream.sh`: fetch pinned wrapper/engine/percolator-match into `vendor/`, build wrapper + fixtures, verify sha256 against the P1 record — area:ops — EXE-01, TEST-08
3. Cargo workspace skeleton: `crates/moka-types`, `crates/moka-math`, `tests/` crate — area:programs
4. GitHub Actions CI: fmt, clippy, tests, upstream build cache — area:ops
5. Issue/PR templates with spec-ID and verification fields — area:docs
6. SessionStart hook so Claude Code web sessions have the toolchain ready — area:ops
7. Learning: complete R1 and S1 — area:learning

## Milestone: Phase 2 — Localnet vertical slice (upstream only)

8. LiteSVM harness: load pinned wrapper + `auth_matcher`, create USDC mint, `InitMarket` for one single-asset market group — T1
9. Portfolio lifecycle test: `InitPortfolio`/`Deposit`/`Withdraw` with reconciliation — CAP-13
10. `TradeNoCpi` then `TradeCpi` via `auth_matcher`: equal and opposite exposure, fees — EXE-02, EXE-04
11. Backing flow: `TopUpBackingBucket` → shock via `PushAuthMark` → liens → `SyncBackingDomainLedger` → `WithdrawBackingBucketEarnings` — BST-04, BST-05, BST-07
12. Liquidation under shock via `PermissionlessCrank` — EXE-11
13. Two-market-group isolation test — SEC-09
14. Core-route attack list from P1 §2.2, incl. the #223 LP-siphon attempt; record pass/avoided/blocker — SEC-10
15. Measure CU for `TradeCpi`, `PushAuthMark`, crank — MAT-04
16. Learning: S2, P1, T1

## Milestone: Phase 3 — Matcher program

17. `crates/moka-math`: §8.3 quote in integers + golden vectors — CAP-07, MAT-03
18. `programs/matcher` (Pinocchio): ABI v3 return, oracle band, caps, stale-mark refusal — MAT-01..04
19. Malicious-response and boundary tests; replace `auth_matcher` in the Phase 2 slice — TEST-03

## Milestone: Phase 4 — LP vault

20. Maker share class: PDA-owned maker portfolio, deposit/redeem, NAV — LP-02..06, LP-08
21. Backing share class: backing authority, top-up/withdraw/earnings, ledger sync — BST-04..07, LP-08
22. Vault property tests: first depositor, rounding, loss-then-deposit — LP-03, LP-06

## Milestone: Phase 5 — Market registry and oracle

23. Listing rules and `InitMarket` as `marketauth`; role hand-off to PDAs — ORC-09, SEC-09
24. PumpSwap adapter + mainnet golden fixtures — ORC-08
25. Pyth SOL/USD conversion — ORC-02, CAP-14
26. Depth window, hysteresis, cross-check, EMA/clamp, `PushAuthMark` — ORC-03..05, ORC-08
27. Contract multiplier and re-denomination flow — CAP-12
28. Raydium CPMM and Meteora DAMM v2 adapters — ORC-08

## Milestones: Phases 6–8

29. Surfpool rehearsal scenario and runbook — T2
30. TS SDK with golden-byte tests; rent disclosure helper — API-02, API-06A
31. Keeper service (marks, cranks, syncs), restart-safe — OPS-04
32. Web app: list/trade/LP/positions with disclosures — API-06, API-07
33. Devnet deployment manifest and demo runbook — OPS-01, TEST-07
34. First devnet user interviews — product

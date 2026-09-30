# P1 upstream reproduction and ABI compatibility matrix (draft)

- Date: 2026-09-30
- Status: **partial P1 evidence**. Upstream builds and host/LiteSVM tests were reproduced in a cloud container. The application-level integration harness (deposit → IOC → backing → loss/recovery → withdrawal) is not yet written. Nothing here makes the candidates release pins.
- Requirement IDs: EXE-01, TEST-08, BST-04, BST-05, MAT-01.

## 1. Pin relationship

| Item | Docs candidate | Actual finding |
| --- | --- | --- |
| Wrapper | `aeyakovenko/percolator-prog@2b1d025c004f92d3f89bac00113be90a0cbbcf63` (2026-09-12) | Reproduced (§2) |
| Engine | `aeyakovenko/percolator@8eb7142aada316f6c476f5c4fa815d3a806706d5` (2026-09-03) | **Not what the wrapper uses.** Wrapper `Cargo.toml` pins `percolator` rev `394fd0bf2cb7d73df425eb3754dc3be1a0c44336` (2026-09-08) for both SBF and host targets |
| Engine diff `8eb7142..394fd0bf` | — | One commit, `src/v16.rs` only: "fix(v16): reject side OI cap overflow on attach". Adds a `MAX_OI_SIDE_Q` bound check in `add_open_interest_for_new_position` |

Owner decision 1 (2026-09-30): pin the engine revision the wrapper declares, never an older one. For wrapper `2b1d025` that is `394fd0bf`.

### 1.0 Upstream pins vs. maintained forks

Other Percolator-based projects may deploy a maintained fork (their own engine and wrapper branches that adopt upstream changes selectively) rather than an upstream commit. Version numbers from such forks are not comparable to an upstream pin. We pin upstream `5cb331dd` + `4db11a8c`. A maintained fork of our own is a later option, explained in `docs/architecture/fork-strategy.md`.

### 1.1 MVP pin (owner-accepted 2026-09-30; supersedes `2b1d025` + `394fd0bf`)

| Component | Pin | Why |
| --- | --- | --- |
| Wrapper | `percolator-prog@5cb331dde354517c6371a8acf92cecb194f3bb73` (2026-09-22, head at inspection) | Contains 20 wrapper fixes merged after `2b1d025`, including side-OI cap enforcement, taker base-fee consent (#411/#432), matcher-grant binding to position episodes and asset generations (#412/#414), cold-admin oracle takeover (#416) and first-risk fee crystallization (#434) |
| Engine | `percolator@4db11a8cb0053815e23a35d3a7d3edc265d8d866` (declared by the wrapper) | `394fd0bf` plus 3 fixes, all in resolved/terminal payout paths (#417 late-backing receipts, resolved residual entitlements, late resolved debt) |
| Matcher fixture | `percolator-match@60aac3a996d0264c47fbefe9a2625d6ee6fb6a47` | Unpinned upstream; test-only; pinned here |
| Toolchain | Agave `v3.0.10` `cargo-build-sbf` + platform-tools `v1.52`, host `rustc 1.94.1` | See §2 notes |

Caveat recorded, not hidden: upstream bumped the engine to `4db11a8c` on 2026-09-17 (wrapper commit `8fc4787c`) without re-certifying every proof-composition guard. 18 test references still certify `94979ede`, and those guards fail by design ("must be reviewed on every engine pin change"). The engine delta sits in resolved/terminal payout code, which the MVP only reaches through exit paths.

Upstream `main` moves daily (674 wrapper commits in the 10 days after `2b1d025`, mostly tests). Bumps follow the pin policy in `execution-plan.md` §5.

License: wrapper is Apache-2.0 (`LICENSE`). Both repos carry an "educational, not audited, do not use with real funds" disclaimer.

## 2. Reproduction record

Environment: Linux x86_64 container, 4 vCPU, host `rustc 1.94.1`.

| Step | Command | Result |
| --- | --- | --- |
| Engine host tests @ `394fd0bf` | `cargo test --release` | **PASS**: 197 tests across 7 targets (lib 50, `backing_double_claim_fuzz` 13, `resolved_insolvent_fuzz` 2, `v16_spec_tests` 132; `proofs_v16*` targets contain Kani-only harnesses, 0 run), 48 s |
| Wrapper SBF build @ `2b1d025` | `cargo build-sbf --tools-version v1.52` (see toolchain notes) | **PASS**: compiled in 1m00s, no stack-offset diagnostics. Stripped `percolator_prog.so` = 1,258,664 bytes, sha256 `5029cc3419b928c0db2660d4da0f82f021cb3347bde14c32738c04c4b042e83e` |
| `auth_matcher` fixture | `(cd tests/fixtures/auth_matcher && cargo build-sbf --tools-version v1.52)` | **PASS**: 29,216 bytes, sha256 `50e53226…9ddf93` |
| `hostile_matcher` fixture | same, in `tests/fixtures/hostile_matcher` | **PASS**: 84,472 bytes, sha256 `e0c20fad…d6240a`. **Not listed in upstream README build steps**, but required by 70+ tests |
| `percolator-match` sibling | `git clone aeyakovenko/percolator-match ../percolator-match && cargo build-sbf` | **PASS** at `60aac3a996d0264c47fbefe9a2625d6ee6fb6a47` (2026-06-03, head at inspection): 47,064 bytes, sha256 `15ca5d63…e6dc3ea`. **Unpinned**: the wrapper tests load `../percolator-match/target/deploy/percolator_match.so` with no commit reference. Must be added to our pin manifest |
| Wrapper all-target tests | `cargo test --release --all-targets` | **FAIL TO COMPILE**: `examples/dump_layout.rs` references removed field `WrapperConfigV16::_reserved_insurance_withdraw_deposit_remaining` (E0609). Upstream defect in a diagnostic example, not program code |
| Wrapper lib + tests, first run (fixtures missing) | `cargo test --release --lib --tests` | lib 8/8 pass; `v16_cu` 1097 pass / 74 fail. All inspected failures are fixture-path assertions (`tests/v16_cu.rs:572` percolator-match, `:7679` hostile_matcher); cargo stopped before the fuzz targets |
| Wrapper lib + tests, all fixtures | `cargo test --release --lib --tests --no-fail-fast` | **FAIL: 1,597 pass / 10 fail of 1,607**, 44 min wall-clock. See §2.1 |
| Kani (engine + wrapper) | `cargo kani ...` | **NOT RUN**: Kani not installed; required before any release pin |

### Toolchain notes (reproducibility)

- Agave `v2.3.13` `cargo-build-sbf` silently fell back to platform-tools **v1.48** (cargo 1.84). That fails on the `wincode 0.5.3` dependency (`edition2024`). **Do not use Agave 2.x for this wrapper.**
- Agave `v3.0.10` `cargo-build-sbf` queries the GitHub API to validate `--tools-version` and panics when it is unreachable, **unless** `--skip-tools-install` is passed with the tools pre-installed. Re-checked 2026-09-30: `scripts/build-sbf.sh` builds with all network access broken and produces identical hashes, so CI has no GitHub API or rate-limit dependency.
- Working recipe: extract `platform-tools v1.52` (`platform-tools-linux-x86_64.tar.bz2` from `anza-xyz/platform-tools` releases) to `~/.cache/solana/v1.52/platform-tools`, symlink it as `<agave>/bin/platform-tools-sdk/sbf/dependencies/platform-tools`, and run `cargo build-sbf --tools-version v1.52 --skip-tools-install`. The SDK's `strip.sh`/`env.sh` then tries to install v1.51 and removes the symlink, so strip with `~/.cache/solana/v1.52/platform-tools/llvm/bin/llvm-objcopy --strip-all target/sbpf-solana-solana/release/<name>.so target/deploy/<name>.so`.
- The `--release` flag on host tests is a speed deviation from upstream's documented debug `cargo test --all-targets`.
- **Scripted (2026-09-30):** `scripts/setup-toolchain.sh` + `scripts/build-upstream.sh` implement this recipe. Instead of stripping by hand, `scripts/build-sbf.sh` pre-creates the SDK's `platform-tools-v1.51.md`/criterion markers so `install.sh` skips its v1.51 download and the SDK strips with the linked v1.52 tools. A clean build in a fresh cloud container (different checkout path from the original run) reproduced all four hashes below with `--locked`, so the build is path-independent. The committed manifest is `scripts/upstream-hashes.sha256`:

  | Artifact | Bytes | sha256 |
  | --- | --- | --- |
  | `percolator_prog.so` @ `5cb331dd` | 1,272,152 | `558778ee9b0d02ca95d9c1f430e5b0e786d1a27a02484301dd60d6489a87546c` |
  | `auth_matcher.so` | 29,216 | `50e532267926e180f013200c1799e26127dd23dc150866cffd491424629ddf93` |
  | `hostile_matcher.so` | 84,472 | `e0c20fad34a7822cc6ce42a3c77ff08a8591977102f0c497a339d66a9dd6240a` |
  | `percolator_match.so` @ `60aac3a9` | 47,064 | `15ca5d638e7911157ef075b60170c06d82ef2208f6565002cf6620315e6dc3ea` |

- A container image or verified-build (`solana-verify`) flow is still needed before any deployment manifest (Phase 8).

### 2.1 Wrapper test results

| Target | Passed | Failed |
| --- | --- | --- |
| lib unit tests (`src/v16_program.rs`) | 8 | 0 |
| `tests/v16_cu.rs` (LiteSVM BPF + invariant suites) | 1,169 | 2 |
| `tests/v16_program_fuzz_regressions.rs` | 121 | 2 |
| `tests/v16_program_stateful_fuzz.rs` (default case counts) | 303 | 6 |

Failures and assessment:

| Test | Message | Assessment |
| --- | --- | --- |
| `inv_058::v16_program_recreated_counterparty_preserves_post_transition_cumulative_limits` | `TradeCpi` size `-99,999,999,999,999` → custom error `0x12` = `EngineInvalidLeg` | **Engine/wrapper test drift.** Engine `394fd0bf` added the `MAX_OI_SIDE_Q = 1e14` side-OI check; this wrapper test predates or ignores it. The engine behavior is the safer one |
| `inv_079::v16_machine_invariant_status_is_authoritative_and_nonoverclaiming`, `inv_079::v16_post_pr135_counterexamples_reopen_every_affected_invariant` | "stale counterexample projection for INV-058" / "only unresolved gaps belong here" | Follows from the INV-058 drift: upstream's machine-readable invariant status ledger disagrees with results |
| `inv_056::v16_program_recovery_and_resolved_dispatch_treat_hints_as_discovery_only` | "duplicate Recovery hints must reject atomically" | **Open.** Crank hint handling in recovery dispatch. Relevant to keeper design |
| `inv_047::v16_program_nonzero_fee_trade_routes_are_byte_exact_after_transport_normalization`, `inv_047::v16_program_generated_nonzero_fee_trade_routes_are_economically_equivalent` | "INV-047 Cpi diverged from NoCpi for lots=1, account_a_long=true, fee_bps=1" | **Open, MVP-critical.** `TradeCpi` with a nonzero fee is not route-equivalent to `TradeNoCpi`; the MVP trade path is `TradeCpi` with fees. Possibly caused by the unpinned `percolator-match` build; must be root-caused before our matcher relies on fee accounting |
| `inv_010::v16_program_conflicting_matcher_controls_and_trade_exhaust_all_landing_orders` | out-of-order matcher-control/trade landing matrix | **Open.** Matcher config vs trade ordering; relevant to `SetMatcherConfig` handling |
| `inv_008::v16_program_retry_operation_matrix_rejects_every_stale_retry` | assertion `left == right` | **Open.** Stale-retry rejection matrix |
| `inv_014::v16_program_fee_consent_operation_matrix_discovers_unsigned_debits` | fee-consent matrix | **Open.** Fee-consent semantics; relevant to fee caps (EXE-05) |
| `inv_086::v16_program_reference_model_matches_deployed_public_sequence` | reference model vs deployed sequence | **Open.** Reference-model equivalence |

### 2.2 Results at the recommended pin (`5cb331dd` + `4db11a8c`)

| Step | Result |
| --- | --- |
| Wrapper SBF build | **PASS**, no stack-offset diagnostics; stripped `percolator_prog.so` sha256 `558778ee9b0d02ca95d9c1f430e5b0e786d1a27a02484301dd60d6489a87546c` |
| Fixtures | `auth_matcher` and `hostile_matcher` byte-identical to `2b1d025` builds (sources unchanged) |
| Engine host tests @ `4db11a8c` | **PASS** 197 / 197 (same targets as §2), 49 s |
| lib unit tests | 8 / 8 pass |
| `tests/v16_cu.rs` | **1,412 pass / 145 fail** (more tests exist at head: 1,557 vs 1,171) |
| `tests/v16_program_fuzz_regressions.rs` + `tests/v16_program_stateful_fuzz.rs` | 146 / 2 and 310 / 26: **28 fail**. Of these, 2 are metadata guards, 15 are in forbidden/exit families, and 11 are retained-grant, receipt or reference-model cases. The one squarely on an MVP core route is `inv_036::v16_program_pr223_cpi_backing_fee_consent_fuzz` (CPI backing-fee consent, #223/#224 "LP siphon" family) |
| Total | **1,876 pass / 173 fail of 2,049**, 68 min wall-clock |

Why head has more failures than `2b1d025`, and why that is not a regression signal. Upstream runs an automated invariant-hunting loop (`scripts/loop.md`, `scripts/worker_blind.md`). A finding lands as a red exploit test; its fix lands as a separate PR, and ~165 such PRs are open. `main`'s failing tests are therefore upstream's **documented, unfixed bug list**, which grows as the loop finds more. The 145 failures break down as:

| Group | Count | Relevance to the MVP |
| --- | --- | --- |
| Proof-composition/roster guards pinned to engine `94979ede` or requiring source completeness | 33 | Process guards; no behavior claim |
| Behavioral failures in insurance, terminal/resolved payout, Hybrid/EWMA/composite oracles, batch, authority handoff/succession, shutdown/restart, recovery forfeit | 87 | Forbidden routes, or exit-only routes whose known liveness issues the UI must disclose |
| Max-shape compute (14 legs / 28 sources / multi-asset siblings), charter/index completeness | 14 | Not reachable in single-asset, single-leg market groups (SEC-09), or metadata |
| **Core-route behavioral failures** | **11** | Must be reproduced against our configuration in M1: stale matcher grant after CPI prefix (INV-012), liquidation-reward rollback and keeper-reward recertification (INV-020), mark catch-up reward price and fractional carry (INV-045), latent source after owner reduction (INV-028), hint-independent B budget (INV-056), exit liveness across backing-expiry prerequisites (INV-071), fee-ingress witnessing (INV-040), alternate-entrypoint lane selection (INV-023) |

Plus from the fuzz targets: CPI backing-fee consent (INV-036 / #223), conflicting matcher controls vs trade landing order (INV-010), and retained matcher-grant expiry/context binding (INV-012/047).

MVP mitigations for the core-route list: `backing_fee_cap_bps` in every `TradeCpi` is set by our SDK and the matcher to the fixed market-profile value, and M1 includes an explicit LP-siphon attack (a third-party taker plus our matcher, asserting maker and backing NAV never fall except by priced fills and published fees); disable liquidation keeper rewards (`liquidation_cranker_fee_share_bps = 0`; our keeper runs anyway); one leg per portfolio; long backing expiry renewed well before lapse; the matcher config is set once and never re-granted; M1 reproduces each listed test's scenario against our exact market profile and records pass/fail/avoided.

Conclusion at `2b1d025` (kept for the record): the candidate wrapper **does not pass its own suite** at its declared engine pin in this environment. It is not yet acceptable as a release pin. Next actions:

1. Determine the `percolator-match` revision upstream used for `2b1d025`, re-run, and separate matcher-fixture effects from wrapper defects.
2. Check whether a later upstream wrapper commit fixes these (upstream `main` has moved to `5cb331dd`) and evaluate re-pinning to a commit whose full suite is green, rather than patching vendored code.
3. File or track upstream issues for any failure that reproduces at a pinned matcher.
4. Record every remaining failure as a documented exclusion with an MVP impact assessment (TEST-08).

## 3. Architecture facts that change the plan's mapping

1. **Market group, not market-per-token.** One program-owned market-group account holds many assets settling in one base collateral. Asset 0 is created at `InitMarket`. Assets 1..N are created (optionally permissionlessly, for a fee) via `UpdateAssetLifecycle`. Each asset has a monotonic `u64 market_id` generation; reused slots never reuse a `market_id`.
2. **Isolation unit = `(asset, side)` source domain.** Backing, insurance and claims are domain-bound; upstream BPF attack tests cover cross-asset drains.
3. **Portfolios are cross-margin across ≤ 14 assets.** Spec's "no cross-market margin" can only be enforced by our clients (one portfolio per asset), not by the wrapper.
4. **Per-asset authority set:** `asset_admin`, `oracle_authority`, `insurance_authority`, `insurance_operator`, `backing_bucket_authority`, rotated with `UpdateAssetAuthority` (tag 65). One market-level `marketauth`.
5. **Owners are plain signers.** `InitPortfolio`/`Deposit`/`SetMatcherConfig` require only `is_signer` on the owner, so a program PDA can be the maker/LP owner via `invoke_signed` (SEC-03 feasible without wrapper changes).
6. **No IDL.** Instruction encoding lives in `src/v16_program.rs` (`Instruction::decode`), with Kani encode/decode round-trip proofs. The SDK must hand-mirror it with golden-byte tests against the pinned source.
7. **README drift:** the upstream README documents `SetMatcherConfig` without `expiry_slot` and lists tag 41 `WithdrawInsurance`. Decoder source has `expiry_slot` and no tag-41 arm. **Source is authoritative.**
8. **Native dynamic backing yield exists**: see `mvp-assessment.md` §3.5.

## 4. Instruction/ABI compatibility matrix (from decoder source @ `2b1d025`; tag set re-verified identical @ `5cb331dd`)

At `5cb331dd`, `SetMatcherConfig` (68) moved to a body decoder because its payload gained generation/episode bindings (#412/#414). The SDK must encode it from the head decoder, not from the `2b1d025` layout below. `TradeCpi` (10) carries `account_{a,b}_portfolio_id`, `account_{a,b}_position_epoch`, `account_b_matcher_sequence`, `asset_index`, `market_id`, `size_q: i128`, `fee_bps`, `limit_price` and `backing_fee_cap_bps`, which is what EXE-03/EXE-05 need onchain.

Status legend: **Mapped** = instruction located in source, spec role assigned, not yet exercised by our harness. **Needed-MVP** = on the MVP critical path.

| Spec operation / requirement | Wrapper instruction (tag) | Notes | MVP |
| --- | --- | --- | --- |
| Create market group (Propose/configure market) | `InitMarket` (0) | Sets `marketauth`, collateral mint, risk params, asset 0 | Needed-MVP |
| Per-token market create/activate/shutdown/retire | `UpdateAssetLifecycle` (40) | Permissionless create iff `permissionless_market_init_fee > 0` (`UpdateMarketInitFeePolicy`, 59) | Needed-MVP |
| Trader/maker registration | `InitPortfolio` (1) | Assigns monotonic `portfolio_id` | Needed-MVP |
| Account deposit / withdrawal | `Deposit` (3) / `Withdraw` (4) | Both carry `portfolio_id`, `expected_sequence`, `u128 amount` | Needed-MVP |
| Matcher opt-in (MAT-01) | `SetMatcherConfig` (68) | `portfolio_id, expected_sequence, enabled, trade_fee_cap_bps, expiry_slot` | Needed-MVP |
| IOC trade via matcher (EXE-03/04, MAT-01) | `TradeCpi` (10) | Matcher ABI v3 return prefix: abi, flags (VALID, backing-fee-cap<<8), exec_price, i128 exec_size, req_id, lp id, oracle price, asset; see `tests/fixtures/auth_matcher` | Needed-MVP |
| Bilateral test trade | `TradeNoCpi` (6) | Harness bootstrap only | Harness |
| Batch trades | `BatchTradeNoCpi` (66) / `BatchTradeCpi` (67) | Post-MVP | Later |
| Crank / liquidation / settlement (EXE-11/12) | `PermissionlessCrank` (5) | Candidate accounts are hints; optional reward portfolio | Needed-MVP |
| Oracle: authority mark (ORC-02/04) | `ConfigureAuthMark` (62) / `PushAuthMark` (63) | Binds `market_id` + `observation_sequence` | Needed-MVP |
| Oracle: EWMA mark | `ConfigureEwmaMark` (35) / `PushEwmaMark` (36) | Alternative | Later |
| Oracle: external/hybrid (Pyth) | `ConfigureHybridOracle` (34) | For tokens with a Pyth feed | Later |
| Domain backing top-up (BST-03/04, CAP-09) | `TopUpBackingBucket` (24) | Binds generation, authority epoch, monotonic `intent_id`, future `expiry_slot` | Needed-MVP |
| Backing withdrawal (BST-06) | `WithdrawBackingBucket` (50) | Native encumbrance checks | Needed-MVP |
| Backing yield withdrawal | `WithdrawBackingBucketEarnings` (52) | Utilization-fee earnings | Needed-MVP |
| Backing fee policy | `UpdateBackingFeePolicy` (51) | Kink curve + trade fee + insurance share | Needed-MVP |
| Loss/recovery ledger (BST-05) | `SyncBackingDomainLedger` (53) | `cumulative_loss_atoms` / `cumulative_recovery_atoms` | Needed-MVP |
| Insurance top-ups | `TopUpInsurance` (9) / `TopUpInsuranceDomain` (56) | Separate tranche only (BST-04) | Later |
| Insurance ledger / live withdrawal | `SyncInsuranceLedger` (54) / `WithdrawInsuranceAsset` (57) | | Later |
| Fee policies | `UpdateTradeFeePolicy` (55), `UpdateFeeRedirectPolicy` (58), `UpdateLiquidationFeePolicy` (37), `UpdateMaintenanceFeePolicy` (49), `SyncMaintenanceFee` (48) | | Config |
| Authority rotation | `UpdateAuthority` (32), `UpdateAssetAuthority` (65) | Controller PDA becomes `backing_bucket_authority` here | Needed-MVP |
| Recovery / risk reduction | `CureAndCancelClose` (42), `ForfeitRecoveryLeg` (43), `RebalanceReduce` (44), `FinalizeResetSide` (45), `ForceCloseAbandonedAsset` (64), `RestartAssetOracle` (69) | Native RECOVERY lifecycle | Later (demo 1 path) |
| Resolution / wind-down | `ResolveMarket` (19), `ConfigurePermissionlessResolve` (38), `ResolveStalePermissionless` (39), `CloseResolved` (30), `ClaimResolvedPayoutTopup` (46), `ConvertReleasedPnl` (28), `ClosePortfolio` (8), `CloseSlab` (13) | | Later |
| Base-unit mints | `UpdateBaseUnitMints` (60), `SwapSecondaryForPrimary` (61) | | Later |

Next P1 step: exercise every **Needed-MVP** row in one LiteSVM scenario against the hashes in §2, asserting token and ledger reconciliation after each transition (CAP-13).

## 5. Upstream open findings and the MVP route allowlist (spec SEC-10)

Upstream tracks its own security state in `tests/invariants/invariant_status.tsv` and `tests/invariants/open_findings.tsv`. At wrapper `5cb331dd` (2026-09-22) the ledger reports **0 of 89 invariants PROVEN**, 8 `REFUTED_CURRENT` (INV-005, 014, 028, 045, 058, 067, 070, 073), 34 `OPEN_EVIDENCE` and 45 `SUPPORTED` (sampled, conditional). The open-finding benchmark lists **~165 public-route findings** (loss-of-funds and DoS; most labeled BLOCKER) raised as upstream PRs through 2026-09-13. Some were later fixed on `main` (for example #411/#432 taker fee consent, #412/#414 matcher-grant binding, #416 cold-admin oracle takeover, #417 late-backing-expiry receipts, #422 Hybrid reward provenance, #429 shutdown reserves, #433 terminal reserve payouts, #434 flat-fee settlement). **There is no fully green Percolator build.** Reliability therefore comes from three things together:

1. **Newest consistent pin.** Use the latest wrapper commit and the engine revision it declares. Each wrapper commit after `2b1d025` fixes findings on routes we use. Never pin an older engine than the wrapper declares.
2. **Narrow routes.** Our programs and SDK may use only the routes below. Most open findings sit in routes we never configure or call.
3. **Per-token blast radius.** One market group per token (SEC-09) means a wrapper defect exploited in one token's market can reach only that token's vault.

| Class | Routes (tag) | Relevant open findings (benchmark PR #) | MVP mitigation / required test |
| --- | --- | --- | --- |
| **Core: allowed** | `InitMarket` (0), `InitPortfolio` (1), `Deposit` (3), `Withdraw` (4), `ClosePortfolio` (8) | 299, 305, 307, 309, 350, 355 (incarnation/replay binding) | Maker/backing owners are PDAs; sequences one-shot; replay tests in M1 |
| **Core: allowed** | `TradeCpi` (10), `SetMatcherConfig` (68) | 223, 224 (CPI caller-fee LP siphon), 310, 313, 334, 393, 411/432, 412, 414 | Our matcher is the only configured matcher; fee caps set explicitly; M1 attack test: a third-party caller cannot extract LP value through fee fields |
| **Core: allowed** | `ConfigureAuthMark` (62), `PushAuthMark` (63) | 264, 275, 332, 335 (mark staging/replay/generation) | Adapter PDA is the sole authority; monotonic `observation_sequence`; M1 test of push-then-trade ordering |
| **Core: allowed** | `TopUpBackingBucket` (24), `WithdrawBackingBucket` (50), `WithdrawBackingBucketEarnings` (52), `SyncBackingDomainLedger` (53) | 267, 283, 286, 321, 339, 351, 361, 363, 367 (backing locality, expiry, fee terms, replay) | Only the lp-vault PDA tops up/withdraws; fee terms fixed at listing; long expiry with scheduled renewal; M1 expiry tests |
| **Core: allowed** | `TopUpInsurance` (9) (listing deposit only), `UpdateAssetAuthority` (65) (listing only, to hand roles to PDAs) | 320, 344, 345, 346, 375 | Called only inside the registry's listing transaction |
| **Keeper: allowed** | `PermissionlessCrank` (5), `SyncMaintenanceFee` (48), `FinalizeResetSide` (45) | 190, 212, 220, 246, 280, 366 (crank progress, liquidation marks) | Keeper hints are advisory; M1 liquidation-under-shock test |
| **Exit/recovery: allowed, known DoS risk** | `CureAndCancelClose` (42), `ForfeitRecoveryLeg` (43), `RebalanceReduce` (44), `ConvertReleasedPnl` (28), `UpdateAssetLifecycle` SHUTDOWN (40, registry only), `ForceCloseAbandonedAsset` (64), `ConfigurePermissionlessResolve` (38), `ResolveStalePermissionless` (39), `CloseResolved` (30), `ClaimResolvedPayoutTopup` (46) | Many INV-073/067/028 DoS findings (e.g. 203, 216, 217, 252, 266, 268, 368, 371, 376, 420, 421) | Needed so users can always exit; the UI discloses that recovery paths have known liveness issues; exercised in M1/M4 shock demos |
| **Forbidden in MVP** | `BatchTradeNoCpi` (66), `BatchTradeCpi` (67); EWMA/Hybrid oracle modes (34, 35, 36); live fee-policy updates (37, 49, 51, 55, 58, 59); `WithdrawInsuranceAsset` (57), `SyncInsuranceLedger` (54), `TopUpInsuranceDomain` (56); base-unit mint changes (60, 61); `UpdateAuthority` (32); `RestartAssetOracle` (69); `ResolveMarket` (19); `CloseSlab` (13) | 225, 256, 260, 265, 282, 284, 325, 326, 329, 331, 336–340, 347, 349, 369, 380, 381, 422, 426 and others | Registry never calls them; policy fixed at `InitMarket`; a CI test asserts that our programs and SDK encode no forbidden tag |

### 5.1 How much do we need the avoided areas?

| Avoided route(s) | What it gives | Needed for the MVP? | When it would be needed | Cost of living without it |
| --- | --- | --- | --- | --- |
| `BatchTradeNoCpi`/`BatchTradeCpi` (66/67) | Atomic multi-asset, multi-leg trades | No: one asset per market group | Only if we ever add cross-asset spreads | None for single-token perps |
| EWMA/Hybrid oracle (34/35/36) | Pyth-driven or smoothed internal marks | No: memecoins have no Pyth feed; our DEX adapter pushes AuthMark | Phase 12, for large tokens with Pyth feeds | Large tokens use the same DEX-pool adapter until then |
| Live fee-policy updates (37, 49, 51, 55, 58, 59) | Change fees/curves on a live market | No | Mainnet tuning (Phase 14/15), behind a timelock | Retuning a market means listing a new market generation |
| Live insurance withdrawal and domain insurance (54, 56, 57) | Operator takes surplus insurance out of a live market | No: insurance stays until wind-down | Protocol revenue sharing, Track B | Insurance surplus is locked while the market lives |
| Base-unit mint changes (60, 61) | Migrate collateral (e.g. USDC → another stablecoin) | No | Only on a collateral migration | New market generation instead |
| `UpdateAuthority` (32) | Rotate `marketauth` | No: the registry PDA is `marketauth` forever | Registry program migration | Upgradeable registry program covers it |
| `RestartAssetOracle` (69) | Restart a shut-down asset in place | No | — | Re-list as a new market group (also used for re-denomination) |
| `ResolveMarket` (19) | Admin-priced resolution | No: we prefer not to hold an admin price power | Never by choice; permissionless stale resolve and shutdown/force-close cover exits | Delisting a rugged token uses SHUTDOWN + force-close |
| `CloseSlab` (13) | Reclaim market account rent after full wind-down | No | Track B: reclaiming rent (could fund future rent rebates, API-06A) | ~0.02 SOL plus accumulated portfolio rent stays locked per dead market |

Conclusion: **none is needed for the MVP**. Three become useful later: live fee tuning, `CloseSlab`, and the Pyth/Hybrid oracle. Each re-enters only after its upstream open findings are re-checked at the pin current at that time.

The benchmark is a snapshot of PR titles, not fix status. At each pin, the fix status of every finding on an allowed route is the result of upstream's per-finding regression test in the full suite (§2). A failing regression on an allowed route blocks M1 until it is mitigated or avoided.

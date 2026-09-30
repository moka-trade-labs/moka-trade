# Senior protocol engineer curriculum (owner's learning track)

- Date: 2026-09-30
- Purpose: take the owner from "can read the code" to "can design, build, audit and maintain a Percolator-based perp protocol, including a maintained fork and assembly-level hot paths".
- How it is used: when the owner says **"give me my daily lesson"** (or similar) in this repo, the agent follows §9 using `docs/research/learning-progress.md`.
- Module IDs (R1, S2, …) are referenced by `docs/execution-plan.md`, so each build phase says which modules come first.

## 0. Map

| Track | Modules | Ends with the ability to… |
| --- | --- | --- |
| **R: Rust** | R1–R4 | write safe, zero-copy, no-alloc, auditable Rust; read `unsafe` critically |
| **S: Solana** | S1–S6 | build Anchor and Pinocchio programs, reason about CPI/PDAs/compute, write sBPF assembly, spot Solana-specific vulnerabilities |
| **M: Maths** | M1–M3 | do fixed-point arithmetic with provable rounding; model perps, liquidation and LP risk |
| **D: DeFi** | D1–D4 | design perps, vaults, AMMs and oracles; analyze manipulation economics |
| **P: Percolator** | P1–P2 | explain every engine concept we use; operate a maintained fork |
| **T: Testing & verification** | T1–T3 | write property tests and fuzzers, fork mainnet, write Kani proofs |
| **F: Frontend & SDK** | F1–F2 | build a typed SDK and a correct wallet UX |
| **O: Operations** | O1–O3 | run keepers, indexers, monitoring and incident response |

Recommended order (runs slightly ahead of the build phases): R1 → S1 → M1 → R2 → S2 → P1 → T1 → D2 → S3 → M2 → D3 → S4 → T2 → F1 → O1 → F2 → then Track-B material (R3, R4, S5, S6, M3, D4, T3, P2, O2, O3).

Each module = about 1–2 weeks at 1–2 hours a day. Every module ends with an exercise and a "done when" test.

## 1. Rust

**R1 — Rust fundamentals for protocol code.** Ownership/borrowing, enums and `match`, `Result`/`?`, traits, generics, modules, Cargo workspaces, features, `cargo test`/`clippy`/`fmt`.
- Read: *The Rust Programming Language* (ch. 1–11, 13); *Rust by Example*; do Rustlings.
- Exercise: a CLI fixed-point calculator (`u128` scaled integers, checked ops, explicit rounding modes) with unit tests.
- Done when: it passes `clippy -D warnings` and a proptest showing `round_down(x) ≤ exact ≤ round_up(x)`.

**R2 — Data layout and zero-copy.** `#[repr(C)]`, alignment and padding, `bytemuck::Pod`, slices and borrow splitting, lifetimes in account views, `no_std`/no-alloc mindset.
- Read: *Rust for Rustaceans* (Gjengset) ch. 1–3; `bytemuck` docs; the upstream `percolator` engine's `V16Pod*` types and `examples/dump_sizes.rs`.
- Exercise: define a 1 KB account layout, cast it zero-copy from a byte slice, and write a test proving the size and offsets are stable.
- Done when: you can explain why every field in the wrapper's `WrapperConfigV16` sits at its offset.

**R3 — Correctness engineering.** Error design, newtypes for units (`Atoms`, `Lots`, `PriceE6`), `#[must_use]`, const generics, invariants in types, macro hygiene (reading Anchor's macros).
- Read: *Effective Rust* (Drysdale); Anchor macro expansion via `cargo expand`.
- Exercise: refactor your R1 calculator so a price cannot be added to an amount (compile error).

**R4 — `unsafe`, performance and low level.** Raw pointers, aliasing rules, `MaybeUninit`, why Pinocchio is fast, profiling CU hot spots.
- Read: *The Rustonomicon* (selected chapters); the Pinocchio source (`anza-xyz/pinocchio`).
- Exercise: port a small Anchor instruction to Pinocchio and compare CU with Mollusk.

## 2. Solana

**S1 — Accounts and programs.** The account model, rent, owners, system/token programs, SPL Token vs Token-2022, lamports, transactions and signatures.
- Read: solana.com/docs core concepts; the Solana Cookbook; the SPL Token docs.
- Exercise: with the CLI, create a mint and token accounts on devnet, then decode their raw bytes by hand.

**S2 — Programs, CPI and PDAs.** Instruction processing, CPI, `invoke_signed`, PDA derivation and bumps, account validation checklists, re-entrancy limits.
- Read: the Anchor docs (1.0); `coral-xyz/sealevel-attacks` (every example); Neodyme's "Solana smart contracts: common pitfalls".
- Exercise: an escrow program in Anchor with LiteSVM tests covering every sealevel-attack class that applies.
- Done when: you can list the SEC-04 checks for each instruction from memory.

**S3 — Compute, transactions and fees.** The compute budget, CU measurement, priority fees, account locks and parallelism, address lookup tables, transaction size limits.
- Read: the Solana transaction-pipeline docs; the Helius blog on priority fees.
- Exercise: rewrite your escrow in Pinocchio; produce a CU table (Anchor vs Pinocchio) with Mollusk.

**S4 — Reading foreign programs safely.** Owner/discriminator/layout checks on third-party accounts (AMM pools, Pyth), versioning, golden fixtures.
- Read: the PumpSwap and Raydium CPMM IDLs; the Pyth receiver `PriceUpdateV2` docs; our `dex-oracle-design.md`.
- Exercise: a read-only program that validates a PumpSwap pool fixture and returns its spot price with exact integer maths.

**S5 — sBPF and assembly.** The sBPF ISA, registers, the calling convention, syscalls, ELF layout, writing and testing assembly programs.
- Read: the Helius blog "How to Write Solana Programs with SBPF Assembly"; `blueshift-gg/doppler` (a ~21-CU oracle); Blueshift's assembly challenges.
- Exercise: an assembly program that stores a price with a sequence check; measure its CU; compare with a Pinocchio version.

**S6 — Security and audits.** Common vulnerability classes, audit reports, upgrade authority and multisig, verified builds.
- Read: public Solana audit reports (OtterSec, Neodyme, Sec3, Zellic); the Mango Markets oracle-manipulation postmortem; any public protocol audit-scope document as a model for writing one.
- Exercise: write an audit-style review of your S2 escrow and of `programs/matcher` once it exists.

## 3. Maths

**M1 — Fixed-point and integer arithmetic.** Scales, units, overflow, widening (`u128`, U256), rounding direction as a security property, dust.
- Read: spec CAP-07/08/12; the upstream `percolator/src/wide_math.rs`.
- Exercise: implement spec §8.3's quote with integers; match the rational test vector exactly.

**M2 — Invariants and conservation.** Conservation laws, monotonic counters, share accounting, rounding-exploit proofs.
- Read: OpenZeppelin on ERC-4626 inflation attacks; the upstream `INVARIANTS.md` (skim sections 0–3).
- Exercise: property tests proving your vault can't be drained through repeated deposit/redeem rounding.

**M3 — Derivatives and risk maths.** Perp payoffs, funding, margin and liquidation thresholds, gap risk, basic stochastic models (random walks, jumps), stress scenarios, LVR.
- Read: Hull, *Options, Futures and Other Derivatives* (futures chapters); `docs/research/perp-futures-literature-review.md`; Milionis et al., "Automated Market Making and Loss-Versus-Rebalancing"; Chitra's ADL paper (cited in the thesis).
- Exercise: a simulator of one market with jump shocks, comparing static backing with native-rate backing (the Phase 11 seed).

## 4. DeFi design

**D1 — Perp exchange architectures.** CLOB vs AMM vs pool-as-maker vs vAMM; oracles vs mark prices; funding; ADL.
- Read: Drift v2 docs and `drift-labs/protocol-v2` (liquidation and AMM modules); Phoenix docs (matching engine, splines, liquidations); GMX docs; our thesis and review docs.
- Exercise: a one-page comparison of how Drift, Phoenix, GMX and Percolator pay a winning trader after a gap.

**D2 — Vaults and share accounting.** NAV, deposits/redemptions, queues, first-depositor attacks, tranching (junior/senior).
- Read: the ERC-4626 spec; Yearn and Morpho vault docs; spec LP-01..08.
- Exercise: a Solana vault program with share mint/burn, conservative NAV and inflation-attack tests (feeds Phase 4).

**D3 — AMMs and oracle manipulation.** Constant product, concentrated liquidity, TWAP/EMA, manipulation cost vs extractable value, flash liquidity.
- Read: the Uniswap v2 and v3 whitepapers; Angeris & Chitra, "Improved Price Oracles: Constant Function Market Makers"; Uniswap's oracle-manipulation writing; the Pyth best-practices docs.
- Exercise: an AMM program plus a LiteSVM test that manipulates it and measures the attacker's cost vs a perp payout (feeds Phase 5).

**D4 — Mechanism and incentive design.** Funded rewards, wash-trading resistance, keeper economics, auctions.
- Read: Paradigm research on MEV and auctions; Archer and Drift JIT docs; spec BST/LP/CAP-10.
- Exercise: design memo for the Phase 11 backstop controller with an equal-capital test plan.

## 5. Percolator

**P1 — Engine and wrapper concepts.** Market groups, portfolios, source domains, liens, H/backing, A/K/B coefficients, side modes, recovery, the matcher ABI.
- Read: the upstream `percolator` README and `spec.md` (v16.9.1); the `percolator-prog` README; our P1 record.
- Exercise: trace one `TradeCpi` end to end through `src/v16_program.rs` and the engine, writing the call graph (feeds Phase 2).

**P2 — Operating a fork.** Git mechanics (rebase/merge/cherry-pick/subtree), patch logs, sync rituals, test-diffing against the invariant ledger, reproducible and verified builds.
- Read: `docs/architecture/fork-strategy.md`; the fork/sync history of any public Percolator-based fork on GitHub, as a case study (search forks of `aeyakovenko/percolator-prog`).
- Exercise: fork a small open-source Solana program, add one patch, sync it twice with upstream, and keep a `PATCHES.md`.

## 6. Testing and verification

**T1 — Unit, property and integration tests.** `proptest`, LiteSVM, Mollusk, golden vectors.
- Read: the proptest book; the LiteSVM and Mollusk READMEs; upstream `tests/support/v16_svm.rs`.
- Exercise: the Phase 2 slice itself.

**T2 — Mainnet state and forks.** Account dumps, fixtures, Surfpool.
- Read: `docs/architecture/test-environments.md`; the Surfpool docs.
- Exercise: dump a real PumpSwap pool and replay a swap in LiteSVM.

**T3 — Fuzzing and formal methods.** `cargo-fuzz`, Trident stateful fuzzing, Kani model checking, what a proof does and doesn't cover.
- Read: the Kani book; the Trident docs; upstream `kani/v16_kani.rs` and `tests/invariants/README.md`.
- Exercise: a Kani proof that your M1 rounding function never rounds in the user's favor.

## 7. Frontend and SDK

**F1 — TypeScript SDK.** `@solana/kit`, instruction encoding, account decoding, exact bigint maths, golden tests against Rust.
- Exercise: an SDK encoder for `Deposit` whose bytes match the Rust decoder test.

**F2 — Wallet UX.** The wallet standard, transaction lifecycle states, simulation, error mapping, disclosures (API-06/06A).
- Exercise: a minimal page that deposits and shows every lifecycle state.

## 8. Operations

**O1 — Keepers.** Idempotency, retries, priority fees, restart safety, competition between keepers.
**O2 — Indexers.** Finality, reorg handling, replay, reconciliation, Postgres schemas.
**O3 — Monitoring and incidents.** Alerts, runbooks, key management, multisig, postmortems.
- Read: Helius/Triton RPC docs; Jito docs; Squads multisig docs; public incident postmortems.

## 9. Daily lesson protocol (for the agent)

When the owner asks for a daily lesson:

1. Read `docs/research/learning-progress.md` for the current module and step. If none, start at R1.
2. Deliver one ~45–60 minute lesson with:
   - the concept, taught briefly and concretely;
   - **one code walk-through from this repo or the pinned upstream source** (file and line references), tied to the current build phase where possible;
   - one exercise with a precise "done when";
   - 3–5 quiz questions, answers hidden until requested.
3. Prefer primary sources (official docs, source code, papers). Don't invent links; say when a resource needs verifying.
4. Update `docs/research/learning-progress.md` with the date, the module/step covered and the next step. Record exercise results when the owner shares them.
5. If the current build phase needs a module the owner hasn't done, say so and suggest doing it next.

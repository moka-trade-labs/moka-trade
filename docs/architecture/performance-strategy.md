# Performance strategy: how far can we optimize, and does it matter?

- Date: 2026-09-30
- Answers: "Phoenix beats CEXes with sharp bps and cheap oracle updates; ORDR-style teams write oracle programs in assembly (~20–30 CU). How much can we optimize to compete?"
- Requirement IDs: MAT-04, OPS-03.

## 1. Where compute actually goes in our design

| Path | Who pays | Measured / expected cost | Who controls it |
| --- | --- | --- | --- |
| Trader IOC (`TradeCpi`, one leg) | Trader | **~94k CU** observed in the upstream suite for a single-leg `TradeCpi`, of which the matcher CPI was **694 CU** (`auth_matcher`) | ~99% is the pinned Percolator wrapper and engine; our matcher is a rounding error |
| Mark update (our DEX-pool adapter → `PushAuthMark`) | Keeper (anyone) | To be measured in Phase 5: pool read + Pyth read + EMA + a wrapper CPI | Our adapter is small; the wrapper's `PushAuthMark` handler is not ours |
| Liquidation/crank | Keeper | Upstream-bounded, tens to hundreds of k CU | Wrapper/engine |
| LP deposit/redeem | LP | Our vault + wrapper deposit/backing CPIs | Shared |

Consequence: **we cannot win a CU race on the trade path while Percolator executes the trade.** Phoenix's and ORDR's numbers come from owning the whole matching engine and optimizing it end to end. Blueshift's `doppler` oracle updates in about 21 CU in sBPF assembly; ORDR stores maker orders as offsets from mid so a whole book reprices in O(1). Those are maker-quote update costs, which our design doesn't have: our maker is an onchain pool that prices off the oracle, so it never sends quote updates.

## 2. What "sharp" means for our users

Memecoin traders compare us with the **spot pool** they already trade: PumpSwap charges 25 bps per swap, plus price impact on thin pools. Winning there means:

1. **Tight quoted spread relative to the pool.** Our matcher's half-spread and impact parameters (spec §8.3) can price inside the spot pool's effective cost for small sizes, because the maker's inventory absorbs flow instead of moving the AMM curve.
2. **Fresh marks at low cost.** A cheap mark crank lets keepers update often, so the matcher's oracle band can be tighter without refusing trades.
3. **No failed transactions.** CU headroom, correct priority fees and account-lock minimization matter more to UX than raw CU.

## 3. Optimization levels we can realistically own

| Level | What | Target | Phase |
| --- | --- | --- | --- |
| L0 | Anchor for lp-vault and registry, zero-copy accounts where large | Correctness first | Phases 3–5 |
| L1 | Matcher in **Pinocchio** (no Anchor), zero-copy context, no heap | ≤ 1,000 CU per quote | Phase 3 |
| L2 | Mark crank adapter in Pinocchio: fixed-offset pool reads (layouts pinned + golden-tested), integer EMA, one CPI | Adapter overhead ≤ 5–10k CU excluding the wrapper CPI (measure first) | Phase 5, refined in 13 |
| L3 | Transaction engineering: address lookup tables, exact CU limits from simulation, dynamic priority fees, keeper batching | Fewer failed or overpaying transactions | Phases 7, 10 |
| L4 | sBPF assembly for the matcher quote or the pool read | Tens to hundreds of CU; only if L1/L2 measurements show it matters | Phase 13, optional |
| L5 | Own the matching engine (a maintained Percolator fork with a trimmed trade path) | Large trade-path savings | Only after audit and the fork decision (Phase 14) |

L4 and L5 are where the "assembly oracle" skills from the curriculum (S5, R4) pay off. Before that point they would slow the MVP down without changing user outcomes, because the wrapper dominates the trade path.

## 4. Measurement discipline

- Every program PR records CU for its hot instructions (Mollusk measures exact CU) in a table in the PR description.
- CI fails if a hot path regresses more than 10% without a note.
- Report p50/p99 end-to-end devnet transaction landing time for trades and mark cranks from Phase 8 on (MAT-04).

## 5. Sources

- Doppler oracle: https://github.com/blueshift-gg/doppler
- sBPF assembly: https://www.helius.dev/blog/sbpf-assembly
- Proprietary AMMs and cheap oracle updates: https://www.helius.dev/blog/solanas-proprietary-amm-revolution
- ORDR: https://www.ordrtrade.com/ ; Phoenix matching engine: https://docs.phoenix.trade/phoenix/matching-engine/matching-engine

# Product feature ideas

- Date: 2026-09-30
- Status: ideas, not committed scope. A feature enters the roadmap only as an issue in a phase of `docs/execution-plan.md`. Every feature must respect spec invariants: no unfunded rewards, no parallel PnL ledger, onchain enforcement.

## 1. User needs we design for

People active around fresh token launches (graduation snipers, spot liquidity providers, short-term traders) repeatedly want to:

1. **Take a short view** on a newly launched token, where no borrow market exists.
2. **Hedge** a spot holding or a spot LP position against a fast move.
3. **Earn yield from volatility** without picking a direction.
4. **Be paid in a stable asset** when they are right, even if the token collapses.

## 2. Design choices that serve those needs

- USDC collateral and settlement (spec §1), so winners are paid in a stable asset.
- A real-capital maker tranche and a backing tranche (LP-08) instead of virtual liquidity, so payouts are backed by money.
- Prices from the deepest supported DEX pool, with manipulation-resistant selection (ORC-08).
- Full per-token isolation (SEC-09).

## 3. Feature ideas

| Feature | What the user gets | How it fits the design | Rough effort |
| --- | --- | --- | --- |
| **LP hedge** | One click shorts the perp against a spot AMM LP position, so the LP mainly keeps fee income instead of directional exposure | SDK/web composition only: read the LP position, compute token exposure, place a short IOC | Low |
| **Graduation feed** | Live list of tokens graduating from launchpads, a countdown to listing eligibility (ORC-09 minimum pool age), and a notice when a market opens | Registry enforces the rules; the feed is an indexer/web feature | Low–medium |
| **Stress-yield board** | Tokens ranked by live backing utilization and by **dollars** paid to backing LPs over the last hour/day | Reads native `utilization_fee_earnings`; no new economics (BST-07, API-06) | Low |
| **Shareable trade cards and Blinks** | An image of a trade result, plus a Solana Action/Blink that opens the same trade | Off-chain rendering; Blinks build the IOC through the SDK | Low–medium |
| **Community-sponsored backing** | A token's community seeds the backing tranche of its perp and earns the native backing yield | Ordinary deposits into the backing class, with an attribution label | Low |
| **Chat-bot trading** | Trade from the chat apps where these users already coordinate | Bot builds unsigned transactions via the SDK; signing design needed | Medium |
| **Keeper bounties** | Public keeper leaderboard and funded bounties | Needs liquidation keeper rewards, which stay disabled until the relevant upstream findings are resolved (P1 record §2.2) | Medium |
| **Seasons/points** | Retention | Only with funded budgets and wash-trade analysis (LP-07, CAP-10) | Medium, risky |

## 4. What to learn at the first devnet release (Phase 8)

- Do users take short positions on fresh tokens at our fees and caps?
- Do spot LPs use the hedge flow?
- Does anyone deposit into the backing tranche without subsidies, and which display (utilization, dollars earned) influences that?

# MVP gap assessment: goal vs. current plan

- Date: 2026-09-30
- Status: assessment and proposal. It changes no normative requirement in `spec.md`; the decisions in §6 must be resolved and written back into the spec before implementation relies on them.
- Inputs: `spec.md`, `execution-plan.md`, `dynamic-jit-liquidity-perp.md`, `idea-validation-review.md`, and direct inspection of upstream `percolator` and `percolator-prog` source (see [p1-upstream-reproduction.md](p1-upstream-reproduction.md)).

## 1. Stated product goal (owner, 2026-09-30)

1. A perpetuals DEX on Solana for freshly launched tokens, ideally any SPL token.
2. Percolator as the risk engine, so each token market is isolated with its own risk management.
3. LPs take the counterparty/backing risk and are paid for it.
4. Dynamic LP yield: when a market is volatile and its backing ratio (Percolator `H`) is under pressure, the yield rises to attract more LP capital.
5. LPs should be able to earn a decent return for running a market, even when the token's price collapses.
6. An end-to-end product, reaching MVP as fast as possible.

## 2. Verdict

The documents point in the right direction, but they will not reach the goal quickly as written.

| Goal | Does the current plan deliver it? | Gap |
| --- | --- | --- |
| Perp DEX with Percolator risk | Yes, same core architecture (pinned wrapper + LP-scoped matcher + pool-as-maker) | The candidate engine/wrapper pair is not a compatible pair (§3.1) |
| Isolated per-token markets | Yes, by design | The plan does not use the wrapper's native multi-asset isolation (§3.2) |
| "Any fresh token" | **No.** The plan targets one admitted market and defers permissionless activation indefinitely | There is no oracle strategy for tokens without a Pyth/Switchboard feed (§3.3) |
| LPs take risk and earn | Yes, conceptually | Two LP roles (maker equity vs. backing) are specified but not productized (§3.4) |
| Dynamic APY under stress | Partially. The plan designs a new controller with precommitments, epochs and a reward escrow, gated on simulation | The engine **already** has a trader-paid, utilization-kinked backing yield (§3.5); the plan underuses it |
| LPs earn through a dump | Not addressed as a product property; no mechanism can guarantee it | Needs honest reframing (§4) |
| MVP ASAP | **No.** 8 phases, 10–18 weeks by the plan's own estimate, many sign-off gates | A compressed critical path is proposed in §5 |

The plan is safety-first: separate ledgers, onchain enforcement, no parallel PnL ledger, pre-shortfall triggering. These principles are correct and should be kept. What slows it down is process and scope (risk certificates, an independent-review role, a controller with commitment epochs, two keepers, Postgres indexer) that a devnet MVP does not need.

## 3. Findings from docs + upstream source

### 3.1 The candidate pins are not a compatible pair (blocker; easy fix)

- Wrapper `percolator-prog@2b1d025` declares `percolator = { rev = "394fd0bf2cb7d73df425eb3754dc3be1a0c44336" }` in its `Cargo.toml`, **not** the docs' engine candidate `8eb7142`.
- `8eb7142` is the direct parent of `394fd0bf`. The only difference is a safety fix: `394fd0bf` rejects per-side OI overflow above `MAX_OI_SIDE_Q` when attaching a position (`src/v16.rs`, `add_open_interest_for_new_position`).
- Reproduction (see [p1-upstream-reproduction.md](p1-upstream-reproduction.md)): the engine suite passes (197 tests). The wrapper builds with platform-tools v1.52, but its own suite has **10 failures out of 1,607**. These include a `TradeCpi`-vs-`TradeNoCpi` fee divergence (INV-047) on our exact trade path. The suite also depends on an unpinned sibling repo (`percolator-match`). The candidate wrapper is therefore **not yet release-pin quality**, and the first engineering task is to root-cause these failures or move to a later upstream commit whose suite is green.
- Recommendation: adopt engine `394fd0bf` as the engine candidate, because it is what the wrapper actually builds and tests against. `execution-plan.md` §5 already instructs "record the manifest's actual dependency commit". `AGENTS.md`, `spec.md` EXE-01, the execution plan, the thesis and `.superstack/build-context.md` all repeat `8eb7142` and need updating once the owner accepts this.

### 3.2 Upstream already provides per-token isolation inside one market group

The v16 wrapper is a **market group** ("slab") of up to ~5,800 assets settling in one base collateral (USDC):

- Asset 0 is created at `InitMarket` and receives a configurable share of all trading fees (`fee_redirect_to_market_0_bps`) plus permissionless-create fees.
- Assets 1..N can be created **permissionlessly for a fee** (`UpdateAssetLifecycle`, tag 40; `permissionless_market_init_fee`). Each asset has its own `market_id` generation, oracle profile, per-asset admin keys, insurance and backing.
- Claims are bound to `(asset, side)` source domains. Upstream BPF attack tests assert that one asset's insolvency cannot touch another asset's insurance, backing or winners.
- A single portfolio can hold legs in up to 14 assets (cross-margin). The spec forbids cross-market margin. The wrapper cannot enforce one-asset-per-portfolio, but isolation of backing and insurance holds regardless. The app should create one portfolio per asset and disclose this.

**Superseded by owner decision 3 (§6):** each token gets its own market group, because a shared group shares one physical USDC vault. The default market-group capacity is one asset, so the account stays small. The original analysis follows.

**Consequence:** "one isolated market per token" maps to "one asset (plus its two domains) per token inside one USDC market group". It does not require one deployment or one slab per token. This matches the "any token" goal far better than the plan's one-off single-market framing.

### 3.3 The real blocker for "any fresh token" is price, not code

Freshly launched tokens have no Pyth/Switchboard feed. The docs correctly identify manipulation as the critical risk (ORC-03, ORC-07), but they never choose an oracle adapter. Upstream offers:

- **AuthMark** (tags 62/63): the configured per-asset `oracle_authority` pushes a mark. The README explicitly suggests an adapter PDA that verifies a feed or custom policy and signs `PushAuthMark`.
- **EwmaMark** (tags 35/36): an authority-pushed, EWMA-smoothed mark with premium-based funding.
- A per-slot effective-price movement cap (the "meltdown brake") set at market init, plus `permissionless_resolve_stale_slots` for oracle death.

**Superseded by owner decision 4 (§6):** the onchain adapter in `dex-oracle-design.md` is used from the first devnet release; there is no trusted price bot. Original options:

1. **Devnet MVP:** a keeper-held `oracle_authority` pushes AuthMark from an aggregator price. This is trusted and must be disclosed as such (SEC-06).
2. **Pre-mainnet:** an onchain `oracle-adapter` program whose PDA is the `oracle_authority`. It reads reserves from a named AMM pool (PumpSwap / Raydium CPMM / Meteora), keeps a slot-bounded EMA/TWAP, and pushes AuthMark. It is combined with the engine price cap and OI caps derived from that pool's depth. Anyone can crank it.

"Any token" is therefore feasible as **permissionless creation with capacity bounded by measured spot depth**, not unlimited leverage on every mint. This is what the spec already says ("permissionless creation is not permissionless activation of unlimited leverage"), made concrete.

### 3.4 Two LP roles, both natively supported

| Role | Native mechanism | Earns | Loses when |
| --- | --- | --- | --- |
| **Maker LP** (pool as counterparty) | An LP portfolio with `SetMatcherConfig` to our matcher; fills via `TradeCpi` | Spread, inventory skew, trading fees (minus the asset-0 redirect) | Traders win net against the pool |
| **Backing LP** | `TopUpBackingBucket` into `(asset, side)` domain buckets; the per-asset `backing_bucket_authority` can be a program PDA | Backing utilization rent + backing trade fee (§3.5) | Backing is consumed to pay winners that the losing side could not cover |

Both can be owned by a program PDA without wrapper changes. `InitPortfolio`, `Deposit` and `SetMatcherConfig` only require the owner to sign (`invoke_signed` works), and the backing authority is a rotatable per-asset key. This satisfies SEC-03.

### 3.5 "Dynamic APY" already exists natively, paid by traders

`percolator@394fd0bf`, `backing_utilization_rate_e9_for_source_state`:

```text
util = liened_backing / fresh_reserved_backing              (per source domain)
rate = base + slope_at_kink * util/kink                     if util <= kink
     = base + slope_at_kink + slope_above * (util-kink)/(1-kink)  otherwise
fee  = lien * rate * slots / 1e9   (charged to the trader account holding the lien,
                                    credited to the bucket's utilization_fee_earnings,
                                    withdrawn via WithdrawBackingBucketEarnings, tag 52)
```

In addition, `backing_trade_fee_bps_{long,short}` charges a one-time fee on lien growth, split between the provider and insurance.

This is what the owner asked for. As more trader profit leans on backing (the precondition for `H` falling), backing utilization rises and the yield on that backing steepens past the kink. It is paid by the traders whose profits consume the backing, not by an unfunded subsidy. It also triggers **before** `H < 1`, which the docs rightly require: capital added after claims are haircut cannot restore them.

Calibration notes:

- The rate unit is 1e-9 per slot. At ~400 ms slots, 1 unit ≈ 7.9% APR, so the low end of the curve is coarse, while the stress end (for example 50 units ≈ 390% APR) is expressible. The engine cap `MAX_BACKING_FEE_RATE_E9_PER_SLOT = 1e9` is far above any sane setting; our market profile must bound it.
- Rent is charged only while a lien exists, only if the account's PnL is non-negative, and only up to its capital. Idle backing earns only the trade fee, so the headline yield is low in calm markets and high under stress. The UI must show dollars earned and principal at risk, not an annualized headline (API-06).

**Recommendation:** ship the native curve as the MVP's dynamic yield. Treat the plan's precommitment/epoch/reward-escrow controller as a post-MVP experiment, still subject to the equal-capital comparison the docs require (TEST-05). An optional creator-funded boost escrow is the smallest useful extension.

## 4. Reframing "LPs earn even when the price dumps"

No design can guarantee LP profit. The economics are:

- **Price dump, traders net long** (typical for memecoin flow): the maker pool is net short and **profits**. Backing is rarely touched. LPs earn spread, fees and directional gains.
- **Price dump, traders net short:** the maker pool is net long and loses. Losses are bounded by entry notional (a long cannot lose more than it paid).
- **Price pump, traders net long:** the dangerous case. The maker pool is short and losses are unbounded in price. Backing gets liened, then consumed, and backing LPs earn peak rent right before potential loss.

The protections that actually make the dump case work for LPs are **inventory skew in the matcher** (spec §8.3 `k`, discouraging a lopsided book), **per-side OI caps** tied to spot depth and pool capital (CAP-03/05), the **engine price-movement cap**, and **fast liquidation keepers**. The dynamic yield is compensation for bearing pump risk. It is not insurance for LPs.

## 5. Proposed compressed MVP path

The target is an end-to-end devnet demo with one engineer + Claude, in about 6–8 weeks. All spec safety principles are kept; process gates marked "deferred" move to the pre-mainnet track.

| Step | Deliverable | Spec IDs exercised |
| --- | --- | --- |
| **M0** (days) | Reproduce the upstream build/tests at the corrected pin pair; write the ABI compatibility matrix. **Done in draft**, except for the 10 open wrapper test failures; resolve them by root-causing or re-pinning | EXE-01, TEST-08 |
| **M1** (1–2 wk) | `tests/` LiteSVM harness in Rust against the pinned `.so`: `InitMarket` → permissionless asset create → AuthMark → LP + trader portfolios → deposit → `TradeNoCpi`, then `TradeCpi` with the upstream `auth_matcher` fixture → backing top-up → price shock → liquidation crank → backing loss/recovery ledger sync → earnings withdraw → permitted withdrawal. This is the AGENTS.md "first task" and R0/R1 | CAP-01/02, EXE-02/04, BST-04/05, TEST-02 |
| **M2** (2–3 wk) | `programs/matcher`: fork of `auth_matcher` + §8.3 inventory-aware quote, oracle band, per-side caps, golden vectors. `programs/lp-vault`: PDA-owned maker portfolio + backing authority; two share classes (maker, backing) with loss-aware NAV from native state; deposit/redeem with pending-redemption exclusion | MAT-01..04, LP-02..06, CAP-07/08/12/13, SEC-03/04 |
| **M3** (2 wk) | `packages/sdk` (hand-encoded wrapper instructions, since there is no IDL), `services/keeper` (mark push, `PermissionlessCrank` with liquidation hints, `SyncBackingDomainLedger`), `apps/web` (market list, IOC trade, LP deposit with live utilization/yield and principal-at-risk). The indexer starts as direct RPC reads | API-02/06, OPS-04 |
| **M4** (1 wk) | Devnet deployment with test USDC + a test token, deployment manifest, demo of trader and LP lifecycles including a shock and recovery | TEST-07, OPS-01 |

Deferred to post-MVP / pre-mainnet: risk certificates, the precommitment controller and reward escrow, the GUARDED state machine beyond native side modes (`DrainOnly`/`ResetPending`) and asset shutdown/RECOVERY, a Postgres indexer, dual independent keepers, persistent orders, independent audit. (The onchain DEX-pool oracle moved *into* the MVP by decision 4 below.) The authoritative milestone plan is now `docs/execution-plan.md`.

## 6. Owner decisions (2026-09-30)

| # | Question | Decision | Where recorded |
| --- | --- | --- | --- |
| 1 | Engine pin | **Accepted:** wrapper `5cb331dd` + declared engine `4db11a8c` | `spec.md` EXE-01, P1 record §1.1 |
| 2 | Scope | Phased roadmap: low-hanging MVP first, then the deferred items as later phases (not dropped) | `execution-plan.md` |
| 3 | Topology | Each token fully isolated: **one market group per token**, own USDC vault, insurance and backing (stricter than one shared market group) | `spec.md` SEC-09 |
| 4 | Oracle | The deepest supported DEX pool is the price source when no Pyth feed exists; onchain adapter from day one, **no trusted price bot even on devnet** | `spec.md` ORC-08/09, `dex-oracle-design.md` |
| 5 | Dynamic APY | Engine built-in backing utilization curve | `spec.md` BST-07 |
| 6 | LP share classes | **Both classes from the first release**, identical to the intended mainnet design (simulated/devnet capital only) | `spec.md` LP-08, §7 |
| 7 | Portfolio rent | Not subsidized; disclosed and acknowledged before the account is created | `spec.md` API-06A |
| 8 | Sub-cent tokens | Synthetic contract multiplier `10^k`; re-denominate by listing a new market group | `dex-oracle-design.md` §6 |

## 7. LP share classes explained (decision 6)

Percolator gives LPs two different ways to put money at risk in a token's market. They behave like two tranches.

| | **Maker shares** (first loss) | **Backing shares** (second loss) |
| --- | --- | --- |
| What the money does | Sits in the pool's maker portfolio and is the counterparty to every trade | Sits in the token's long/short backing buckets and stands behind winning traders' profits when the losing side can't pay in time |
| Earns | Spread and inventory skew from the matcher, trading fees, and **directional PnL**: when traders lose, maker shares win | The engine's **dynamic backing yield** (BST-07): near zero when calm, rising steeply as utilization passes the kink, i.e. exactly when the market is stressed. Also the backing trade-fee share |
| Loses | When traders win on net (for memecoins, typically a pump against a crowd of longs) | Only when native loss allocation actually consumes backing, after maker equity and loser collateral are exhausted |
| Risk profile | High risk, high variance, earns in calm and dump markets | Lower frequency of loss; paid most when risk is highest |
| Who it suits | LPs who want to "be the house" | LPs who want to be paid for standing by in stress, the "dynamic APY" product |

Options:

- **Maker-only first demo.** Smallest build, but **there is no dynamic APY at all**: the built-in yield is paid only to backing, and with no backing capital there is nothing to accrue. This contradicts decision 5.
- **One blended share class.** Rejected: it hides which risk an LP holds, makes the displayed rate meaningless, and violates CAP-02 (one dollar in one role).
- **Two share classes in one `lp-vault` program (recommended).** Same program, same accounting code, two independent ledgers and NAVs per token. It adds roughly one week versus maker-only. It is the only option that delivers your dynamic-APY feature. It can be sequenced: maker class first in M2, backing class right after, both in the M4 demo.

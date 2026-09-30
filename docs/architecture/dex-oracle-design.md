# DEX-pool oracle adapter (`programs/oracle-adapter`)

- Date: 2026-09-30
- Status: proposed design, owner-approved direction ("use the pool with the highest liquidity as the price source when no Pyth/oracle feed exists"). Not implemented.
- Requirement IDs: ORC-02, ORC-03, ORC-04, ORC-05, ORC-07, CAP-05, CAP-12, CAP-14, SEC-02, SEC-06.

## 1. Why this exists

Freshly launched tokens have no Pyth, Switchboard or Chainlink feed. Their only price is the spot price of the AMM pools they trade in. Percolator's wrapper supports **AuthMark** markets, where one configured per-asset `oracle_authority` signs `PushAuthMark` (tag 63; accounts `[authority (signer), market]`; `mark_e6` must be `1..=1e12`). We make that authority a PDA of our own `oracle-adapter` program. The only way a mark reaches the market is then a permissionless crank that reads DEX pool state onchain and applies the rules below. No human key and no offchain server sets prices, on devnet or mainnet.

Tokens that *do* have a Pyth feed can use the wrapper's native Hybrid oracle (`ConfigureHybridOracle`, tag 34) instead. That path is post-MVP.

## 2. Supported pool programs

MVP supports the venues where launchpads graduate tokens. All three expose reserves onchain in the same transaction, so there is no stale-observation choice for the caller (ORC-04).

| Venue | Why | Program | Price and depth from |
| --- | --- | --- | --- |
| PumpSwap | pump.fun graduations since March 2025 (canonical pool = `index 0`) | `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA` (mainnet **and devnet**) | Constant product. Base = `pool_base_token_account.amount`; quote = `pool_quote_token_account.amount + Pool.virtual_quote_reserves` |
| Raydium CPMM | Raydium LaunchLab / bonk.fun graduations | `CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C` (devnet id differs) | Constant product. `vault_amount_without_fee()`: vault balances minus `protocol_fees_*`, `fund_fees_*`, `creator_fees_*` |
| Meteora DAMM v2 | Meteora DBC graduations | `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG` | Price from `sqrt_price` (Q64). Depth from `token_a_amount`/`token_b_amount` net of `protocol_*_fee`. Reject pools whose `[sqrt_min_price, sqrt_max_price]` range is narrower than a configured minimum |

Post-MVP adapters: Raydium AMM v4, Raydium CLMM, Orca Whirlpool and Meteora DLMM. These are concentrated or orderbook-hybrid, so depth must be measured in-range, not from vault balances. Bonding-curve (pre-graduation) tokens are **not eligible**.

Every adapter decoder checks the account owner equals the allowlisted program ID, the Anchor discriminator, the layout version/size, the pool status (enabled, past `open_time`/`activation_point`), and that the vault accounts are the ones named in the pool and owned by the expected token program. Layouts are pinned to exact upstream commits with golden-account fixtures dumped from mainnet (SEC-04).

## 3. Quote asset conversion

Memecoin pools are usually quoted in SOL (WSOL), sometimes in USDC or USDT.

- **USDC-quoted pool:** price is used directly in USDC units (CAP-14: settlement is USDC, not USD).
- **SOL-quoted pool:** `token/USDC = token/SOL × SOL/USD`. SOL/USD comes from Pyth's sponsored SOL/USD `PriceUpdateV2` account, verified by owner (Pyth Solana Receiver), feed ID, full verification level, freshness (`publish_time` within `max_age`) and confidence (`conf/price ≤ max_conf_bps`). The wrapper already depends on `pythnet-sdk` for this format, so we reuse the same typed decode. A USDC/USD depeg guard is optional in the MVP and documented as the CAP-14 parity assumption.
- USDT-quoted pools are unsupported in the MVP.

## 4. Choosing the highest-liquidity pool

A program cannot enumerate every pool for a mint, so selection is **permissionless registration + onchain comparison**.

- `SourceRegistry` PDA per `(market group, token mint)` holds up to `K = 4` candidate pools.
- `register_pool(pool)` is permissionless. It validates §2 and requires the pair to be `(token, WSOL | USDC)`. A small refundable spam bond prevents slot griefing. A new candidate can replace the lowest-depth one only if it is deeper.
- **Depth metric** `D` = quote-side reserve valued in USDC atoms (for constant product this equals half the pool TVL). This is the capital an attacker must fight to move the price.
- **Flash-liquidity resistance:** each crank records every candidate's depth. The selection metric is the **minimum** observed depth over a rolling window `W_depth` (e.g., 30 min of crank samples, stored as a small ring buffer). Liquidity added and removed within a window never counts.
- **Switching with hysteresis:** the primary changes only when a challenger's windowed-minimum depth exceeds the incumbent's by `h_switch` (e.g., 25%) for `N_switch` consecutive cranks. Every switch emits an event and resets the price EMA seed to the new pool's price only if the two pools agree within the deviation band (§5). Otherwise the market goes to `PRICE_UNCERTAIN`.
- The owner-facing summary: **the price comes from the deepest supported pool, measured conservatively over time, and changes only when another pool is persistently deeper.**

## 5. Price computation (`crank_mark`)

Permissionless. Anyone (our keeper, or any third party) can call it. Per call:

1. Read the primary pool, compute spot `p_spot` in USDC atoms per contract (§6), and read SOL/USD if needed.
2. **Cross-check:** if a secondary candidate has windowed depth ≥ `min_secondary_depth_ratio` of the primary, its spot must be within `max_cross_dev_bps` of `p_spot`. Otherwise do not push a mark; set `uncertain_since` and emit an event (ORC-01/05).
3. **Smoothing:** update a slot-time-weighted EMA, `p_ema += (p_spot - p_ema) × min(1, Δslot / H)`, with half-life `H` (e.g., 150 slots ≈ 60 s). A single-slot pool manipulation then moves the mark only by `Δslot/H` of the displacement, and holding it costs arbitrage losses every slot.
4. **Per-update clamp:** limit `|p_mark_new - p_mark_old| ≤ max_move_bps_per_slot × Δslot`. This is on top of the engine's own per-slot effective-price cap, which remains the final "meltdown brake".
5. CPI `PushAuthMark { asset_index, market_id, now_slot, mark_e6, observation_sequence, authority_epoch }`, signed by the adapter PDA, with a monotonic `observation_sequence` stored in the adapter state (ORC-02 replay/regression rejection).

Chain time is taken from `Clock`, never from the caller.

## 6. Contract sizing for sub-cent tokens (CAP-12)

**The problem.** `mark_e6` is a `u64` price in millionths of a USDC with range `1..=1e12` (`MAX_ORACLE_PRICE`). A token trading at $0.0000123 would get `mark_e6 = 12`. One tick is then about 8% of the price, and a price of $0.0000004 rounds to zero and is rejected. Memecoins routinely trade below $0.0001.

**The fix: a synthetic contract.** A perp never delivers the token, so the traded unit need not be one token. Each market fixes a **contract multiplier** `m = 10^k` tokens per contract at listing. The perp's price is the price of `m` tokens:

```text
mark_e6 = floor(p_token_usdc × m × 1e6)        (computed in u128 with checked math, rounded down)
k       = the smallest integer with  p_token_usdc × 10^k ≥ 1 USDC   (clamped to 0..=12)
```

Worked example: a token at $0.0000123 gets `k = 5`, so 1 contract = 100,000 tokens = $1.23 and `mark_e6 = 1,230,000`. One tick is now 0.00008% of the price.

**Precision band.** We require `1e4 ≤ mark_e6 ≤ 1e12` for new risk; `1e4` means a tick is ≤ 0.01% of the price. The formula for `k` puts the initial mark between `1e6` and `1e7`, so the token can fall **100–1,000×** or rise **100,000–1,000,000×** before leaving the band.

**Position sizes.** Wrapper sizes are `size_q` with `POS_SCALE = 1e6`, so position granularity is `m / 1e6` tokens (0.1 tokens in the example). The SDK converts `tokens ↔ contracts ↔ size_q` with exact integer maths and rejects unrepresentable sizes rather than rounding them to zero (CAP-12).

**Leaving the band (re-denomination).** The engine cannot rescale open positions in place, and `RestartAssetOracle` is not on the SEC-10 allowlist. When `mark_e6` leaves the band:

1. The matcher refuses new risk: only reductions fill (reduce-only by policy).
2. The registry permissionlessly lists a **new market group** for the same mint with the new multiplier. The UI calls it the same token and shows both generations until the old one is empty.
3. The old market keeps getting marks while it winds down through ordinary closes. Anyone can then run native resolution/close once it is empty.

This is the same idea as a stock split implemented as a new ticker. It never mutates open positions, and it reuses routes we already need.

**What users see.** The UI always shows the per-token price and token quantities. "Contracts" appear only in an advanced details panel ("1 contract = 100,000 PEPE"). `m` is part of the immutable market profile and the SDK unit descriptor; a different `m` always means a different market.

## 7. Capacity tied to pool depth (ORC-03, ORC-07, CAP-05)

Moving a constant-product pool's price by a fraction `δ` requires trading roughly `D × δ` of quote capital against it. Our EMA forces the attacker to hold the distortion for many slots against arbitrageurs. The extractable perp profit is at most `OI_side × δ`. We therefore cap:

```text
max_oi_per_side_usdc  ≤ α × D_primary_windowed_min      (α calibrated, e.g. 0.02–0.05)
max_order_usdc        ≤ β × D_primary_windowed_min
```

The matcher reads the adapter account and enforces these caps on every fill. If the adapter is stale (`now - last_push_slot > max_mark_age_slots`) or uncertain, the matcher returns a rejection (a zero fill), so no pool-maker risk is added without a fresh, cross-checked mark. `α`, `β`, `H`, `W_depth`, `h_switch` and the deviation bounds are market-profile parameters and remain calibration blockers (TEST-06). The values above are placeholders for simulation, not launch settings.

## 8. Permissionless listing rule ("any token")

`list_token` succeeds for any mint only if all of these are verifiable onchain at listing time:

1. A registered primary pool from §2 with windowed depth ≥ `min_listing_depth`, observed for ≥ `min_pool_age` (for example, 24 h of samples; listing is a two-step `propose` → `activate`).
2. The mint uses SPL Token, or Token-2022 without transfer-hook, permanent-delegate, pausable or confidential-transfer extensions.
3. **Mint authority is `None`** and **freeze authority is `None`**, so the issuer can't print tokens or freeze pool vaults.
4. A creator/lister deposit funds the market's initial insurance (and optionally a first maker allocation). There is no free listing.

Tokens that fail stay listable later. There is no discretionary allowlist in the path, which makes "any token that meets the published onchain rules" permissionless (SEC-06). Concentrated-holder and wash-volume checks cannot be verified onchain; the UI discloses this.

## 9. Devnet and local testing

- PumpSwap is deployed on devnet, so the devnet demo creates a test token and a real PumpSwap pool and prices from it with the same code as mainnet. No trusted price bot is needed. A scripted "market mover" wallet trades the pool to create shocks.
- LiteSVM tests load pool/vault account snapshots dumped from mainnet as golden fixtures (TEST-08), plus synthetic pools for manipulation tests: a one-slot spike, a sustained push, flash liquidity to hijack the primary, disagreeing secondary pools, a stale Pyth SOL/USD, and zero or near-zero reserves.

## 10. Known limits (disclosed, not solved)

- A token whose deepest pool is thin is cheap to manipulate. The depth-scaled caps shrink capacity but cannot make it safe at any size.
- A coordinated, sustained move of the real spot price is not manipulation. The perp follows it, and LPs bear the directional risk (see `mvp-assessment.md` §4).
- A rug that drains the pool shows up as a genuine price collapse. Longs lose and shorts win against the pool, bounded by caps and the engine's price-movement cap.

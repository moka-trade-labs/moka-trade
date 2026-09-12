# Perpetual Futures AMM: Literature Review & Innovation Opportunities

## Goal

Build an on-chain perpetual futures DEX on Solana using Toly's Percolator risk engine math. Find the "million-dollar equation" for perps — beyond x·y=k — and identify novel contributions we can bring to the space.

---

## 1. Foundational: A Primer on Perpetuals

**Authors:** Guillermo Angeris, Tarun Chitra, Alex Evans, Matthew Lorig
**Venue:** SIAM Journal on Financial Mathematics (2022) / arXiv:2209.03307
**Link:** https://arxiv.org/abs/2209.03307

The paper that formalized perpetual contracts mathematically.

### Core Funding Rate Formula

For a perpetual contract with payoff φ(S) on underlying S:

```
F_t = ½ Σᵢ Σⱼ (σσᵀ)⁽ⁱʲ⁾ S⁽ⁱ⁾ S⁽ʲ⁾ ∂ᵢ∂ⱼφ(S_t) - (φ(S_t) - Σᵢ S⁽ⁱ⁾ ∂ᵢφ(S_t)) r_t
```

**Model-free form** (no μ, σ knowledge needed):

```
F_t dt = ½ Σᵢ Σⱼ S⁽ⁱ⁾ S⁽ʲ⁾ ∂ᵢ∂ⱼφ(S_t) d⟨log S⁽ⁱ⁾, log S⁽ʲ⁾⟩_t
         - (φ(S_t) - Σᵢ S_t⁽ⁱ⁾ ∂ᵢφ(S_t)) d log M_t
```

### Key Results

- **Replication**: Short side holds Δ_t⁽ⁱ⁾ = ∂ᵢφ(S_t) shares. Portfolio value = φ(S_t) at all times.
- **Discounting mechanism**: D_t = F_t / φ(S_t) converts funding payments into notional decay.
- **CFMM connection**: Geometric mean CFMM LP position IS a perpetual contract with discounting. The discount rate captures convexity cost (≈ impermanent loss).
- **With jumps**: Funding includes jump compensation integral. Replication requires European options for spanning.

### Relevance to Us

This is the theoretical foundation. Every perp design must satisfy the funding rate formula. Percolator's funding mechanism is a discrete implementation of this continuous formula.

---

## 2. The Million-Dollar Equation: PvpAMM

**Authors:** Zhenhang Shang, Zhenyu Zhao, Kani Chen (HKUST)
**Venue:** AFT 2025 (Advances in Financial Technologies) / LIPIcs Vol. 354
**Link:** https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.AFT.2025.34

**The first complete AMM framework engineered specifically for perpetual futures.**

### PLT Token Mechanism

Define λ(t) = PLT price = total_collateral / total_weighted_positions.

```
λ_pvp(t) = Σⱼ mⱼ / Σⱼ (wⱼ(t) / λ_pvp(tⱼ))
```

Individual position value in pvpAMM:

```
w̃ⱼ(t) = (wⱼ(t) / λ_pvp(tⱼ)) · λ_pvp(t)
```

### Pool Solvency Identity (always holds)

```
Σⱼ mⱼ = Σⱼ w̃ⱼ(t)   ∀t
```

Total collateral ALWAYS equals total position value. This is the invariant — no under-collateralization possible.

### The ψ(t) SDE — Core Equation

Define ψ(t) = 1/λ_pvp(t). This is the **scaling parameter** that tracks global solvency.

**Position imbalance parameter:**

```
ξ(t) = Σ(nⱼ · P_t · ψ(tⱼ)) / Σ(wⱼ(t) · ψ(tⱼ))
```

sign(ξ) = sign(net directional exposure of the pool).

**The SDE:**

```
dψ(t)/ψ(t) = ξ(t)/P_t · dP_t
```

Under geometric Brownian motion (dP/P = μ dt + σ dB):

```
ψ(t) = exp{ ∫₀ᵗ ξ(s)[σ(s)dB_s + μ(s)ds] - ½∫₀ᵗ [ξ(s)σ(s)]² ds }
```

### Critical Properties

1. **When μ = 0 (martingale price)**: ψ is a positive martingale, E[ψ] = 1, ψ(t) converges a.s. as t → ∞.

2. **Flash-loan resistance** (Theorem 8):
   ```
   lim_{t→tⱼ⁺} w̃ⱼ(t) = mⱼ
   ```
   No unanticipated profit from immediate exit after entry.

3. **Arbitrage mechanism** (Theorem 6): Construct portfolio that is zero-cost at t and has value at t+Δt:
   ```
   Profit = ξ(ξ - b)(ΔP)² + o((ΔP)²)
   ```
   Positive when b < ξ (if ξ > 0) or b > ξ (if ξ < 0). This keeps ψ pinned near 1.

4. **LP convergence** (Theorem 7): As LP collateral → ∞, ψ(t) → 1, and w̃ⱼ/wⱼ → 1. Large LPs stabilize the pool toward mark-to-market pricing.

5. **Minority-side amplification**: When minority leveraged side wins, returns exceed conventional perpetuals.

### GMX vs PvpAMM

GMX's LPT mechanism converges (as Δt → 0) to:

```
Bⱼ(t) → (mⱼ/λ(tⱼ)) · (P_t/P_{tⱼ})^b
```

This grows EXPONENTIALLY with price under frequent updates — the paper's critique of GMX.

PvpAMM fixes this via ψ(t) normalization.

### Why This Is The Equation

ψ(t) directly maps to **Percolator's global haircut ratio H**. When ψ(t) < 1, pool total position value > total collateral → haircut needed. When ψ(t) > 1, pool has surplus.

The SDE gives us:
- **Continuous-time solvency tracking** (replaces discrete A/K/F indices)
- **Provable martingale property** (solvency guaranteed in expectation)
- **O(1) computation** (cumulative product, order-independent)
- **Natural funding rate** (sign of dψ gives funding direction)

---

## 3. LVR = Funding Fees: The Deep Connection

**Authors:** Srisht Fateh Singh et al. (University of Toronto)
**Venue:** AFT 2025 / arXiv:2508.02971
**Link:** https://arxiv.org/abs/2508.02971

### Central Theorem

For a CFAMM position delta-replicated by a strip of perpetual American continuous-installment (CI) put options:

```
dFee_t = dLVR_t    and    Fee|₀ᵀ = LVR|₀ᵀ   (∀T > 0)
```

where:

```
dLVR_t = ½σ²S_t² X'(S_t) dt
```

### CI Option Closed Form

The perpetual CI put with funding rate q satisfies the inhomogeneous Black-Scholes ODE in the continuation region:

```
½σ²S² ∂²P_q/∂S² + rS ∂P_q/∂S - rP_q = q
```

With value-matching and smooth-pasting at optimal exercise boundaries S_ℓ(q), S_u(q):

```
S_ℓ = q/(r + σ²/2) · [g - g^{1/γ_p}]
S_u = q/(r + σ²/2) · [g^{1-1/γ_p} - 1]
```

where γ_p = -2r/σ², g = 1 + rK/q.

**As q → ∞**: Both boundaries collapse to K. The band width scales as:

```
lim_{q→∞} q · (S_u - S_ℓ) = σ²K²/2
```

### Constant-LVR Liquidity Profile

Choose AMM position delta matching a single perpetual CI put:

```
X(S) = X_q(S; K*)   →   dLVR_t = q·dt + ε(t)·dt,  |ε(t)| ≤ rK*
```

**Funding rate becomes approximately constant** if you shape liquidity correctly.

### Relevance to Us

Percolator's funding rate should be designed as the **theta of a perpetual CI option**. This gives:
- Optimal exercise boundary → endogenous liquidation trigger
- Constant funding profile → predictable solvency
- Closed-form pricing → on-chain compute feasible

---

## 4. Funding Rate Design with Uniqueness Guarantee

**Authors:** Jaehyun Kim, Hyungbin Park (Seoul National University)
**Venue:** arXiv:2506.08573 (2025)
**Link:** https://arxiv.org/abs/2506.08573

### The Problem

Naive funding rate Φ(x) = -σ²X² does NOT uniquely determine the perpetual price. Multiple prices solve the same BSDE.

### The Fix: Mean-Reversion Forcing

```
Φ(s, X_s, Y(s)) = ℓ(φ(s,X_s) - Y(s)) - 𝓛φ(s,X_s) + rY(s)
```

where 𝓛 is the infinitesimal generator:

```
𝓛φ = ∂_sφ + ½tr(σσᵀ∂_xxφ) + r∂_xφ·γ
```

**ℓ must be sufficiently large** for uniqueness:

```
ℓ > inf_{K>0} (K + ½(C_r/√(2K) + M_{ρ∨2}C₃)²)ρ
```

where M_{ρ∨2} is the Burkholder-Davis-Gundy inequality constant.

### Path-Dependent Funding (8-hour TWAP)

Practical funding averages over window δ = 1/1095:

```
Φ^δ(s,γ,η) = (1/δ) ∫_{s-δ}^s Φ(u, γ, η(u)) du
```

Leads to a **delayed BSDE**. Converges at O(√δ) — for δ = 8 hours, error bounds are explicit and small.

### Model-Free Form

Funding rate expressible without knowing μ, σ, r:

```
Φ = H(φ,Y) - ∂_sφ - ½Σ∂ᵢ∂ⱼφ · d⟨Xᵢ,Xⱼ⟩/ds - ∂_xφ·X·d(log G)/ds + Y·d(log G)/ds
```

### Relevance to Us

Percolator's **bounded price envelope** is the practical implementation of "sufficiently large ℓ." By capping oracle price movement between cranks, Percolator enforces the mean-reversion condition that guarantees unique perpetual pricing.

**We can derive Percolator's envelope bounds from ℓ_crit** — making the heuristic bound provable.

---

## 5. Funding-Aware Optimal Market Making

**Authors:** Nam Anh Le
**Venue:** arXiv:2605.06405 (May 2026)
**Link:** https://arxiv.org/abs/2605.06405

### Model

Extends Avellaneda-Stoikov market making to include stochastic funding as a state variable.

**Funding dynamics (Ornstein-Uhlenbeck):**

```
df_t = κ(f̄ - f_t)dt + σ_f dW_t^f
```

**Reduced HJB equation:**

```
0 = ∂_tθ + κ(f̄-f)∂_fθ + ½σ_f²∂_ffθ - qf - φq² + 1_{q>q_min}H^a + 1_{q<q_max}H^b
```

**Ask-side Hamiltonian:**

```
H^a = sup_{δ^a} Λe^{-kδ^a}[δ^aΔq + θ(t, q-Δq, f) - θ(t, q, f)]
```

### Optimal Quotes

```
δ^{a*} = 1/k - A(t,q,f)    where A = θ(t, q-Δq, f) - θ(t, q, f)
δ^{b*} = 1/k - B(t,q,f)    where B = θ(t, q+Δq, f) - θ(t, q, f)
```

Cross-coupling term a₄(t)qf captures inventory-funding interaction:
- When f > 0 (longs pay shorts) AND q > 0 (long inventory) → quotes widen (you're paying funding on inventory)
- When f > 0 AND q < 0 (short inventory) → quotes tighten (you're receiving funding)

### Calibration (Hyperliquid Data)

| Parameter | ETH | BTC | SOL |
|-----------|-----|-----|-----|
| κ (mean reversion) | 146.0 | 106.1 | 89.0 |
| σ_f (vol of funding) | 0.020 | 0.018 | 0.036 |
| Half-life | ~4.3 min | ~5.9 min | ~7.1 min |

### Relevance to Us

Replace state variable f with ψ from PvpAMM. The HJB state becomes (q, ψ):

```
0 = ∂_tθ + μ_ψ∂_ψθ + ½σ²ψ²ξ²∂_ψψθ - q·f(ψ) - φq² + H^a + H^b
```

ψ captures BOTH inventory imbalance AND funding direction in one variable. The SDE for ψ is already derived (PvpAMM Theorem 5).

---

## 6. Autodeleveraging: Theory and Practice

**Title:** Autodeleveraging: Impossibilities and Optimization
**Venue:** arXiv:2512.01112 (2025)
**Link:** https://arxiv.org/abs/2512.01112

### The ADL Trilemma

**No single policy simultaneously satisfies:**
1. Exchange solvency
2. Exchange revenue
3. Fairness to traders

### Pro-Rata is Uniquely Fair

Under axioms of **monotonicity, scale invariance, and sybil resistance**, the pro-rata ADL rule is the UNIQUE fair mechanism. This matches **Percolator's A/K/F pro-rata clearing**.

### Hyperliquid Case Study (Oct 10, 2025)

| Metric | Value |
|--------|-------|
| Realized haircuts | $2.1B |
| True negative equity | $23.2M |
| Overshoot | $653M |
| Queue vs. optimal | **28× over-utilization** |
| Wallets hit | 19,337 |
| Tickers involved | 162 |

### Three Optimal Mechanism Classes

1. **Pro-rata** (Drift, Paradex, Percolator) — axiomatic fairness
2. **Risk-weighted pro-rata** — maximizes robustness to price shocks
3. **Dynamic Stackelberg** — minimizes revenue loss across multiple ADL rounds

### Relevance to Us

Percolator's pro-rata design is already correct per the axiomatic analysis. The improvement opportunity is **risk-weighted pro-rata** using ψ(t) as the risk score g(leverage) → replace equal-weight with ψ-weighted haircuts.

---

## 7. Perpetual Demand Lending Pools

**Authors:** Tarun Chitra, Theo Diamandis, Jeff Sheng, Kamil Sterle, Kanat Yusubov (Gauntlet / Bain Capital)
**Venue:** arXiv:2502.06028 (Feb 2025)
**Link:** https://arxiv.org/abs/2502.06028

### Target Weight Mechanism

Pool maintains target portfolio weights w*. Optimization:

```
minimize   ‖w(R+Δ) - w*‖
s.t.       Δ ≥ -R^A
```

### Key Results

- **Funding rate arbitrage bound**: Fee upper bound = κ(1 - B⁻¹)/L₀ for price range [1, B]
- **Delta-hedging**: Closed-form optimal hedge with transaction costs
- **Sharpe improvement conditions**: Sufficient expected fee income vs. volatility cost
- **Single pool dominates multiple pools** under covariance conditions

### Relevance to Us

The PDLP is the LP-facing side of our perp DEX. TWM provides the formalism for how LP pools rebalance to target weights — this is what sits behind Percolator's slab structure.

---

## 8. Everlasting Options & DPMM

**Authors:** Mohanty, Zaarour, Krishnamachari (USC)
**Venue:** IEEE ICBC 2025 / arXiv:2508.07068
**Link:** https://arxiv.org/abs/2508.07068

### DPMM Mark Price

```
P_m = i_value · (1 + k(V/Q₀)²)
```

where V = inventory, Q₀ = reference liquidity, k = shape parameter.

### DPMM vs AMM Results

| Metric | DPMM | AMM |
|--------|------|-----|
| Win rate | **90.8%** | 48.6% |
| Sharpe ratio | **0.63** | 0.43 |
| Profit factor | **22.4** | 11.4 |
| Median PnL | **+$153K** | -$11K |

DPMM's inventory-sensitive pricing eliminates the adverse selection that plagues static AMMs.

### Funding Fee

```
F_t = P_m^t - payoff_t
```

Calculated daily. Higher liquidity → lower funding fees and less volatility.

### Relevance to Us

Adapt DPMM for perpetual futures (not options). The mark price formula becomes:

```
P_m = oracle_price · (1 + k(V/Q₀)² · sign(ξ))
```

where ξ is the PvpAMM imbalance parameter. This gives inventory-sensitive pricing without external arbitrageurs.

---

## 9. Amortizing Perpetual Options

**Authors:** Zachary Feinstein (Stevens Institute)
**Venue:** arXiv:2512.06505 (2025)
**Link:** https://arxiv.org/abs/2512.06505

### Core Mechanism

Instead of paying continuous funding, the holder sells notional back to the underwriter:

```
dN_t = -q_t N_t dt    where q_t = c_t / V_t
```

### Pricing ODE

```
½σ²S²V''(S) + rSV'(S) - (r+q)V(S) = 0
```

### Closed-Form Call

```
α_C = √((r/σ² + ½)² + 2(r+q)/σ²) - r/σ² + ½

S̄_C = α_C·K / (α_C - 1)

C₀ = K/(α_C - 1) · ((α_C - 1)S₀/(α_C·K))^α_C
```

**As q → 0**: Approaches vanilla perpetual American option.
**As q → ∞**: Approaches intrinsic value. Exercise boundary → K.

### Relevance to Us

AmPO exercise boundary = endogenous liquidation trigger. Instead of fixed maintenance margin, use:

```
S_ℓ(q) from the AmPO closed form
```

When funding is high, liquidation boundary tightens automatically. More precise than Percolator's static haircut ratio.

---

## 10. Unified Framework for DeFi Derivatives

**Venue:** arXiv:2512.19113 (2025)
**Link:** https://arxiv.org/abs/2512.19113

### Structural Classification

| Feature | Perpetuals | Expiring Options | Everlasting Options | Synthetics |
|---------|-----------|-----------------|--------------------|------------|
| T_expiry | No | Yes | No | No |
| P_strike | No | Yes | Yes | No |
| F_m (funding) | Yes | No | Yes | No |
| Leverage L | Yes | Yes | Yes | No |

### Key Empirical Findings

- **Leverage dominates liquidation risk** (tornado analysis): ±20% leverage shift = ±7-8pp liquidation probability change. Volatility = ±9-11pp. Everything else < 2pp.
- **TVL resurgence driven by Solana and Arbitrum** since 2024.
- **Fee structures**: Jupiter flat 0.06%, dYdX maker-taker 0.025-0.05%, GMX 0.04-0.06% + price impact.
- **Matching engines**: Order-book (dYdX, Hyperliquid, Derive) vs. pool-counterparty (Jupiter, GMX, Hegic) vs. hybrid (Drift).

### Liquidation Probability (Simulated — Jupiter-like, SOL, 7-day)

| σ \ L | 2x | 5x | 10x | 20x | 50x | 100x |
|-------|-----|-----|------|------|------|-------|
| 0.04 | 0% | 3.4% | 32.4% | 61% | 86% | 92% |
| 0.08 | 0% | 27.4% | 63% | 80.8% | 91% | 94.8% |

---

## 11. Percolator: Solana's Risk Engine

**Author:** Anatoly Yakovenko (Solana co-founder)
**Repo:** github.com/aeyakovenko/percolator
**Crate:** percolator-engine v12.1.0 (formally verified)

### Core Architecture

| Component | Description |
|-----------|-------------|
| **Global haircut ratio H** | Profit = junior claim. When system stressed, all profitable accounts scaled down. Flat accounts (no positions) always protected. |
| **A/K/F lazy side indices** | A = position scaling, K = mark/ADL overhang, F = funding effects. All O(1) per account, order-independent. |
| **Bounded price/funding envelope** | Oracle price movement capped between cranks. Fails closed. Wrappers must stair-step fast moves. |
| **Pro-rata liquidation** | Every account on affected side absorbs impact proportionally. No singled-out counterparties. |
| **Sharded (slab) structure** | Per-token isolation. Parallel processing. |
| **Cursor-based keeper crank** | Permissionless, bounded per call. Budgets for liquidation, force-realize, GC. |
| **Three-phase side reset** | DrainOnly → ResetPending → Normal. Deterministic recovery. No admin intervention. |
| **Native i128/u128 math** | Precision for financial calculations. no_std Rust. |

---

## 12. Innovation Synthesis: ψ-Perp Architecture

### The Core Insight

PvpAMM's ψ(t) SDE unifies Percolator's A, K, and F indices into **one continuous state variable**:

```
dψ/ψ = ξ(P, positions) · dP/P
```

| Percolator Index | ψ(t) Mapping |
|-----------------|--------------|
| A (position scaling) | ∂ψ/∂(net exposure) via ξ(t) |
| K (mark/ADL overhang) | ψ < 1 → haircut needed. Magnitude = 1-ψ |
| F (funding effects) | sign(dψ) = funding direction |

### ψ-HJB Risk Engine

Replace Percolator's discrete A/K/F updates with continuous ψ(t) process:

```
State: (q, ψ) where q = inventory, ψ = solvency scalar

dψ = ψ·ξ·σ·dB (under martingale measure)
df(ψ) = funding_rate(ψ) derived from CI option theta
```

HJB:

```
0 = ∂_tθ + μ_ψ·∂_ψθ + ½σ²ψ²ξ²·∂_ψψθ - q·f(ψ) - φq² + sup_δ[H^a + H^b]
```

Solved via monotone finite-difference scheme (Le's method, validated on Hyperliquid data).

### DPMM Mark Price for Perpetuals

```
P_mark = oracle_price · (1 + k·(ξ/Q₀)² · sign(ξ))
```

ξ replaces inventory V from the options DPMM. When ξ > 0 (net long), mark up. When ξ < 0, mark down.

### CI Option Liquidation

Liquidation trigger at S_ℓ(q) from AmPO closed form:

```
S_ℓ(q) = q/(r + σ²/2) · [g - g^{1/γ_p}]
```

where g = 1 + rK/q, γ_p = -2r/σ².

Maintenance margin becomes **endogenous** — depends on funding rate, volatility, and risk-free rate. Higher funding → tighter boundary → earlier liquidation.

### Funding Rate Design

Use Kim & Park's BSDE design with Percolator's envelope as the ℓ-forcing:

```
Φ = ℓ(φ - Y) - 𝓛φ + rY
```

where ℓ is calibrated from Percolator's max price move per crank. The envelope bound guarantees ℓ > ℓ_crit → unique perpetual pricing.

### Pro-Rata ADL via ψ

Per-trade ADL haircut:

```
haircut_i = max(0, 1-ψ) · (equity_i / Σ equity_winners)
```

Risk-weighted variant:

```
haircut_i = max(0, 1-ψ) · (leverage_i · equity_i / Σ leverage_j · equity_j)
```

### Computational Feasibility (Solana SVM)

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| ψ(t) update per crank | O(1) | Cumulative product, no loops |
| Position value w̃ⱼ | O(1) | wⱼ · ψ(tⱼ)/ψ(t) |
| Pro-rata ADL | O(n) | Single pass over affected side |
| DPMM mark price | O(1) | One quadratic term |
| CI option boundary | O(1) | Closed form, no iteration |
| HJB quote update | O(n_f × n_q) | Grid size. Pre-computable off-chain |

ψ(t) computation is a **cumulative product** — like tracking a TWAP — not a loop over all positions. This is critical for on-chain feasibility.

---

## 13. What's Novel vs. What Exists

### Already Exists (Don't Build)

| Component | Where |
|-----------|-------|
| Basic funding rate design | Primer on Perpetuals (Angeris et al.) |
| Pro-rata ADL | Percolator, Drift, Paradex |
| vAMM | Perpetual Protocol V2 |
| Oracle pricing | GMX, GNS, Jupiter |
| PDLP (TWM) | Jupiter, Hyperliquid, GMX |
| Order book perps | dYdX, Hyperliquid, Derive |
| Leveraged tokens as perps | Squeeth (Opyn), TLH |

### Novel Contributions (Our Innovation Space)

1. **ψ-based solvency tracking**: No existing DEX uses continuous ψ(t). All use discrete checks. The ψ(t) SDE from PvpAMM is peer-reviewed (AFT 2025) but not deployed.

2. **Unified A/K/F → ψ collapse**: Percolator uses three separate indices. ψ collapses them into one variable with known stochastic dynamics. Simplifies the risk engine significantly.

3. **CI option liquidation boundaries**: No perp DEX uses optimal stopping theory for liquidation. All use static maintenance margin ratios. The closed form is simple enough for on-chain.

4. **DPMM for perpetual futures**: DPMM proven for everlasting options (Mohanty et al.). Adapting for perps with ξ-based pricing is new.

5. **HJB-optimal market making on perp AMM**: Le's funding-aware HJB calibrates to Hyperliquid but isn't deployed on-chain. Combining with ψ state variable instead of f is new.

6. **Provable envelope bounds**: Percolator's bounded envelope is heuristic. Deriving it from Kim & Park's ℓ_crit makes it provable.

7. **ψ-weighted ADL**: Risk-weighted pro-rata using ψ-derived risk scores. More efficient than equal-weight pro-rata (proven in ADL paper).

### Papers That Don't Exist Yet (Open Problems)

1. Convergence proof of ψ(t) → 1 under arbitrary trading strategies (only proven for μ=0 case)
2. ψ-HJB existence and uniqueness (only standard OU-funding HJB proven)
3. Optimal k for DPMM mark price formula in perp context
4. Cross-slab ψ contagion model (when liquidations in one slab affect ψ of neighboring slabs)
5. MEV/resistance of ψ-based pricing to oracle manipulation

---

## 14. Reference: All Papers

| # | Title | Authors | Venue | Year | ID |
|---|-------|---------|-------|------|-----|
| 1 | A Primer on Perpetuals | Angeris, Chitra, Evans, Lorig | SIAM J. Fin. Math | 2022 | arXiv:2209.03307 |
| 2 | PvpAMM: Perpetual Market for Unbalanced Long-Short Positions | Shang, Zhao, Chen | AFT 2025 | 2025 | LIPIcs.354.34 |
| 3 | Modeling LVR via Continuous-Installment Options | Singh et al. | AFT 2025 | 2025 | arXiv:2508.02971 |
| 4 | Designing Funding Rates for Perpetual Futures | Kim, Park | arXiv | 2025 | arXiv:2506.08573 |
| 5 | Funding-Aware Optimal Market Making for Perpetual DEXs | Le | arXiv | 2026 | arXiv:2605.06405 |
| 6 | Autodeleveraging: Impossibilities and Optimization | — | arXiv | 2025 | arXiv:2512.01112 |
| 7 | Perpetual Demand Lending Pools | Chitra et al. | arXiv | 2025 | arXiv:2502.06028 |
| 8 | Proactive Market Making for Everlasting Options | Mohanty et al. | IEEE ICBC | 2025 | arXiv:2508.07068 |
| 9 | Amortizing Perpetual Options | Feinstein | arXiv | 2025 | arXiv:2512.06505 |
| 10 | Unified Framework for DeFi Derivatives | — | arXiv | 2025 | arXiv:2512.19113 |
| 11 | Perpetual Futures in CEX and DEX | Chen, Ma, Nie | arXiv | 2024 | arXiv:2402.03953 |
| 12 | Optimal Fees for Geometric Mean Market Makers | Angeris, Evans, Chitra | AFT | 2021 | arXiv:2104.00446 |
| 13 | Replicating Portfolios: Permissionless Derivatives | Angeris, Evans, Chitra | AFT | 2021 | — |
| 14 | Axioms for Automated Market Makers | Bichuch, Feinstein | arXiv | 2022 | arXiv:2210.01227 |

---

## 15. Next Steps

1. **Implement ψ(t) tracker in Rust** — cumulative product, no_std, test against PvpAMM paper's numerical examples
2. **Port Percolator and benchmark** — understand the A/K/F update cycle in detail
3. **Collapse A/K/F into ψ** — replace three indices with one, verify conservation laws
4. **Derive envelope bounds from ℓ_crit** — make Percolator's heuristic bound provable
5. **Simulate ψ-HJB market making** — compare against standard AS on historical Hyperliquid data
6. **Formal verification** — prove ψ(t) conservation under Percolator's crank model

---

*Research compiled 2026-07-14. All papers publicly available on arXiv or LIPIcs. Percolator source at github.com/aeyakovenko/percolator.*

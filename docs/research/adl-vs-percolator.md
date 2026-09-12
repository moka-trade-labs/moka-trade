# Research Report: Autodeleveraging (ADL) vs. Toly's Percolator

## Overview

This report synthesizes the insights from Tarun Chitra's paper ["Autodeleveraging: Impossibilities and Optimization" (arXiv:2512.01112)](https://arxiv.org/abs/2512.01112) and Toly's [Percolator](https://github.com/aeyakovenko/percolator) risk engine. Both address the critical problem of exchange solvency during catastrophic market crashes when routine liquidations fail and bad debt accrues. However, they approach the problem from different architectural paradigms.

---

## 1. The Core Problem: Bad Debt and Exchange Insolvency

In perpetual futures markets, positions become "liquidatable" when a trader's equity falls below a maintenance margin threshold. The exchange attempts to close these positions on the open market.
* **The Failure State:** During severe liquidity crunches or price gapping, the executed liquidation price may be worse than the bankruptcy price (where equity is exactly zero). This results in a shortfall (bad debt).
* **The Threat:** If this bad debt exceeds the exchange's insurance fund, the exchange becomes insolvent.

---

## 2. Comparing the Approaches

### The ADL Paper: Optimizing Loss Socialization
The paper provides a formal, venue-agnostic mathematical model for **Autodeleveraging (ADL)**—the last-resort mechanism that algorithmically socializes bad debt by forcibly reducing or closing the positions of highly profitable traders ("winners") to cover the shortfall.

* **Key Thesis (The ADL Trilemma):** The paper proves that no ADL policy can simultaneously maximize (1) Exchange Solvency, (2) Fairness to traders, and (3) Long-term Exchange Revenue.
* **Critique of Current Systems:** It mathematically proves that **Queue-based ADL** (used by Binance and Hyperliquid, which greedily seizes profits from the highest PNL/leverage winners) is the worst possible policy. It punishes the exchange's best, highest-fee-generating traders, leading them to churn and destroying the exchange's long-term revenue.
* **Proposed Solution (RAP):** The paper proposes a **Risk-Aware Pro-Rata (RAP)** mechanism that distributes haircuts proportionally across all winners, slightly tilted toward high-leverage positions. It models multi-round ADL as a dynamic Stackelberg game to optimally balance solvency and trader retention.

### Toly's Percolator: Mathematical Containment & Isolation
Percolator is a zero-copy perpetual-futures risk engine built for the Solana Virtual Machine (SVM). Rather than figuring out how to socialize global bad debt, Percolator focuses on **preventing systemic bad debt from spreading across markets**.

* **Source-Domain Aware Cross-Margining:** In Percolator, positive PnL (profit) is treated as **junior**. A trader's paper profit is strictly bound by **Source-Credit Liens** to the actual realizable backing (real tokens or insurance) of the losing side in *that specific isolated market*.
* **Failing Closed (Localized ADL):** If an isolated market fails (e.g., an oracle goes stale or the losing counterparties go bankrupt), the source-credit liens tied to that domain automatically become **Impaired**. The uncollectible paper profits of the winners in that specific market are haircut.
* **Provable Solvency:** Losses are never socialized across unrelated profitable accounts, all short positions globally, or a global base-asset index. This ensures that toxic flow in a single illiquid asset mathematically cannot breach the solvency of the wider vault.

---

## 3. How They Differ & How They Connect

**Are they the same approach?**
No. They are complementary approaches applied at different levels of the risk stack.
* **Percolator** acts at the architectural level. It strictly isolates risk domains so that a massive failure in Asset A cannot cause the exchange to steal funds from traders in Asset B.
* **ADL (The Paper)** acts at the allocation level *within* a failing domain. When a market fails and bad debt must be socialized among the winning counterparties, the paper provides the mathematical framework for *how* to distribute those losses.

### Synergy: Improving Percolator with the ADL Paper

While Percolator guarantees that bad debt won't spread across different asset domains, it still must execute a form of localized ADL (via "impaired liens") on the winners within the specific failing market.

We can take the following insights from the paper to build a better product on top of Percolator:

1. **Replace Queue-Based Lien Impairment with RAP:** 
   If Percolator (or a wrapper built around it) currently determines which winners get their profits haircut using a queue-based system (e.g., highest PNL first), it risks driving away the best liquidity providers for that specific market. By integrating the paper's **Risk-Aware Pro-Rata (RAP)** mechanism, Percolator can distribute the impaired liens proportionally across all winners in the isolated market. This ensures fairness and retains the top traders.
   
2. **Implement Stackelberg Dynamic Tuning:**
   The paper models multi-round liquidations as an optimal control problem. Percolator could dynamically adjust its `certified_maintenance_req` or the speed at which it impairs liens based on real-time order book depth and volatility. By acting as the "Stackelberg leader," the risk engine can proactively minimize the severity ($\theta$) of the required ADL before a complete domain failure occurs.

3. **Optimizing Hyperliquid's Overshoot:**
   The paper proves that non-optimized ADL causes massive "overshoots" (destroying more trader profit than strictly necessary). Percolator's deterministic `MaxCloseSlot` liquidation plans can be fine-tuned using the paper's Mirror Descent controller to minimize the required haircut size for impaired liens, preserving maximum possible value for the winners while maintaining the strict invariant of zero bad debt.

---

## Conclusion
Toly's Percolator provides an unbreakable vault that prevents cross-market contagion, while the ADL paper provides the optimal economic formula for fairly distributing the unavoidable losses within a contained market failure. Combining Percolator's *Source-Credit Liens* with the paper's *Risk-Aware Pro-Rata (RAP)* algorithm would create a provably solvent perpetuals exchange that is also mathematically optimized to retain its top traders during black swan events.

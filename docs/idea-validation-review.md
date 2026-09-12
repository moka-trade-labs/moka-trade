# Architecture and Validation of Low-Cap Perpetual Markets

## Executive assessment

The most appropriate initial architecture is **a fully onchain, isolated, real-capital-funded pool acting as the trading counterparty, with Percolator handling risk accounting through a compatible Solana wrapper**. This is the strongest bootstrap hypothesis for tokens that have defensible spot liquidity but few professional derivative market makers. It is not a recommendation to open unrestricted leveraged markets for every newly created token.

The earlier competitive-auction recommendation optimized the execution implementation and alignment with a particular Foundation brief. The clarified customer segment changes the priority: a small auction still fails if nobody is willing to quote. A pool can make an explicit underwriting allocation available without a dealer responding to each request. Its central weakness is that LPs become the counterparty to informed traders and potentially manipulated prices.

The verdict is **go validate, with a narrower launch cohort**. There is insufficient evidence for “go build a production exchange”: no committed underwriters, paid customer trials, or measured manipulation-resistant target-market capacity have been established. The appropriate immediate work is an economic simulator, a compatible-engine integration spike, and a small demand-validation sprint. Public real-money access remains a separate decision.

Evidence is assessed through September 11, 2026. Protocol descriptions below are attributed documentation claims, not independent proof of current volume, profitable LP returns, deployed-bytecode equivalence, or security. The implementation requirements are in [spec.md](spec.md); the dependency-ordered delivery plan is in [execution-plan.md](execution-plan.md).

## 1. The market that should actually be served

“Newly launched or low-cap, without a perp market” is a useful discovery filter, but it combines materially different risks. A token without perps may be underserved, or it may lack reliable prices, executable spot depth, borrow inventory, durable demand, or anyone willing to warehouse its risk. The absence of a listing cannot distinguish these explanations.

The initial promise should be: **earlier access to long/short exposure when a market becomes economically supportable, with transparent backing and payout constraints**. It should not be a universal listing promise. Existing CEX coverage is also not binary: a nominal listing can have poor size, access or execution. Conversely, a small token may already have a derivative or spot-leverage substitute. Candidate screening must inspect actual alternatives.

| Cohort | Typical constraint | Appropriate treatment |
|---|---|---|
| Pre-graduation bonding curve | Synthetic versus real reserves, concentrated control, venue migration | Registration/observation only; conventional leveraged perp disabled |
| Recently graduated, thin single pool | Cheap reference manipulation, removable depth, unreliable exits | Not automatically eligible; require an explicit economic case at practical size |
| Established spot liquidity, no useful perp | Sparse derivative makers but observable pricing and executable depth | Primary validation cohort for an isolated underwriting pool |
| Established spot plus funded independent makers | Continuous execution demand, inventory management and integration | Compare RFQ, pool-plus-makers and reusable CLOB |

Pump's public program documentation distinguishes virtual and real reserves and describes migration into PumpSwap. That supports treating graduation as a lifecycle event to reassess, not proof of price safety.[^1] No specific market-cap, token-age or TVL threshold is universally justified by those mechanics.

Market admission should evaluate the identity and control of the mint, accessible trading venues, liquidity concentration, withdrawability, executable size, source dependence, and manipulated-price exposure. A token/SOL market valued in dollars has two conversion legs; a robust SOL/USD feed does not make the token/SOL leg robust. Multiple data services can reproduce the same underlying pool price.

## 2. What actually causes the ADL problem

The product must distinguish three outcomes. **Execution failure** means a position cannot find an acceptable closing trade. **Backing failure** means there is insufficient economic capital for claims. **Settlement delay** means funds or claims cannot yet be released under accounting, oracle or lifecycle rules. One mechanism may improve one outcome while leaving the others unchanged.

Consider a simplified linear USDC-settled market. A long and a short each open $100,000 notional at $1 and post $20,000 collateral. The price jumps to $2 before liquidation succeeds. The long earns $100,000, but the short can contribute only $20,000. There is an $80,000 gap. Returning the long's own $20,000 collateral is not another source for paying its profit.

A fall from $1 to zero reverses the winners and losers but produces the same $80,000 gap in this example. The assumptions are failed liquidation, no hedges, no fees and no other backing. A pool with more genuinely committed equity can absorb more loss than a trader posting $20,000; it would be misleading to give every counterparty the same small balance in a comparison.

This explains why matched order flow is not equivalent to fully collateralized payouts. Gross exposure, counterparty margin and liquidation latency matter even when long and short notional match. It also explains why a CLOB is not an ADL cure: matching identifies counterparties but cannot force an insolvent counterparty to pay.

There is an important asymmetry. Loss on a long down to zero is bounded by its entry notional; loss on a short under an arbitrarily large upward move is not. A finite USDC pool cannot promise full payment for every possible upward gap in net customer longs. A chosen stress envelope can constrain risk, but it is not a mathematical guarantee outside that envelope. The residual fallback must be visible to traders and LPs.

## 3. Architecture comparison for the target cohort

| Architecture | What it contributes | Why it can fail here | Decision |
|---|---|---|---|
| Fully onchain CLOB | Persistent participant quotes, familiar order types and price discovery | Thin books, maker acquisition, cancellation pressure, margin reservations and integration complexity | Defer until makers and resting-order demand justify it |
| Competitive RFQ/short auction | Request-specific price competition, smaller initial order-management surface | No quotes without willing dealers; bounded full-size requests cannot combine fragmented liquidity | Conditional option, not baseline liquidity |
| Isolated funded pool | A pre-funded counterparty and predictable quote rules within capacity | Oracle extraction, adverse selection, directional losses, LP capital cost | Best bootstrap hypothesis for eligible markets |
| vAMM | A deterministic virtual pricing curve | Virtual depth is not payout backing; convergence and liquidation still need economic support | No advantage merely from “creating liquidity” |
| Pool plus optional independent makers | Baseline pool capacity plus potential price/inventory improvement | More routing and MEV complexity; the pool may inherit the most toxic residual flow | Later extension after measuring the pool |
| Offchain matching, onchain settlement | Different latency and order-management tradeoffs | Adds sequencing trust and does not create counterparties or safe prices | Not selected under the onchain-execution objective |
| Spot-backed leverage | Finances real assets and uses existing spot execution | Borrow supply, collateral valuation, liquidity removal and liquidation losses remain | Separate pivot if leverage matters more than a perp contract |

This comparison is analytical judgment, not a measured ranking. It can change if the team obtains committed dealers or finds a reusable CLOB that reduces integration effort. Phoenix Legacy provides public evidence that an atomically settling onchain orderbook is feasible on Solana; it does not establish drop-in compatibility with Percolator's credit and custody semantics.[^2]

The selected pool is not a spot constant-product pool and is not a virtual-reserve claim to unlimited liquidity. It is a funded maker position governed by a quote function and risk budget. The logical separation is: reference-price validation, executable quote calculation, risk admission, native accounting, and token custody. A single price must not silently serve all five purposes.

A conservative first implementation uses bounded onchain order intents followed by permissionless execution. User limits, expiry, fresh eligible oracle observations and capacity checks are enforced at execution. This creates latency and still permits selection or censorship risks; the exact timing and cancellation rules require adversarial tests. An instant pool trading against any recently signed price is not the safe default merely because it is simple.

Liquidation and risk reduction should not wait for an optional retail auction. However, they cannot be promised when valid prices or usable capital are absent. A reduction of one trader's position may also worsen the pool's remaining inventory, so “reduce-only” is a property to verify, not permission to bypass every market-level check.

## 4. Hedging and capital supply

Lack of another perp venue does not mean there is no possible hedge. When a customer goes long, the maker becomes short and can buy spot. When a customer goes short, the maker becomes long and can sell owned inventory or borrow and sell the token. These routes have different funding and inventory requirements. Newly launched assets may have no reliable token-borrow market.

A spot hedge is only credible when it is actually acquired, controlled, valued conservatively, and monetizable at useful size. Marking inventory at the latest thin-pool print is not the same as having withdrawable USDC. Creator-provided tokens can help some inventory strategies but are poor protection against the creator's own token collapsing. Creator USDC is a more direct source of financial loss absorption, while insider control remains an adverse-selection concern.

For the first devnet design, use a single collateral denomination and do not credit automated spot hedges that have not been implemented. This reduces custody and reconciliation scope, but also restricts economically safe capacity. A later underlying-backed design is a separate engineering and economic decision. GMX's documentation distinguishes backing that tracks the index asset from synthetic markets whose trader profits can outrun the backing assets; the general distinction is relevant, not its numerical settings.[^3]

The pool therefore needs actual underwriting conversations. It is not enough to find depositors who like an APY headline. Providers must understand market-specific drawdowns, principal loss, lock terms, recovery priority and their alternatives. The team should ask for written terms at concrete allocation sizes, without soliciting or taking funds during the research phase.

## 5. Oracle safety precedes architecture

An auction answers which participant will trade at a price. It does not prove that the price is suitable for liquidating everyone else. The same applies to a last CLOB fill or an inventory-adjusted pool quote. Small self-trades must not control valuation of much larger exposures.

Pyth's current guidance identifies stale-price selection, latency and size-insensitive executable pricing as hazards for derivatives. It discusses delayed execution, confidence handling and exposure limits.[^4] Those are useful integration principles, not certification that any particular low-cap feed is economically safe.

TWAPs also involve a tradeoff. A longer averaging window can increase the work needed to sustain a manipulation but delays response to a legitimate discontinuous move. Uniswap's oracle discussion connects attack economics to liquidity and arbitrage losses.[^5] A token with only one thin reference venue may lack the independent arbitrage mechanism assumed in a comfortable TWAP story. Several publishers using that venue do not create several independent sources of liquidity.

The relevant test is conservative **attacker net profit**, not just the capital required to move a spot price. Attackers can unwind inventories, own liquidity positions, split accounts or influence the issuer. The simulation must include the funds an accepted attack can extract from trader collateral, maker equity, deployable backstop and rewards, net of capital actually lost and other costs.

Adding a backstop can therefore make a manipulation more attractive by enlarging the payout available to an attacker. This is a design inference from the proposed mechanism. Admission limits and reserve activation cannot be developed independently. A fund that pays manipulated claims more reliably has improved neither market integrity nor sustainable user protection.

The MVP should enforce a narrow set of supported oracle adapters and onchain checks, while disclosing that source-independence and manipulation assessments initially rely on versioned risk attestations. A signature authenticates who made that assessment; it does not prove the assessment correct. Expiry must remove new-risk capacity. The research has not established a fully permissionless onchain proof of these offchain economic facts.

## 6. What remains of the dynamic-JIT innovation

The original post-shortfall pitch is economically weak: immediately after a known loss, a new depositor may be paying already-created claims. A large annualized rate does not necessarily compensate the actual loss. At 1,000% simple annualized APR, $100,000 earns about $114.16 for one hour. That is far below a predictable $20,000 principal loss.

Precommitment improves availability by moving the LP's decision to an earlier time. It does not remove the premium needed to compensate for the risk. The refined mechanism is an underwriting contract with funded availability rewards, explicit deployment conditions and loss-aware ownership—not a guarantee that strangers arrive during a wick.

Most importantly, $100,000 locked for one market and activated later is not more money than $100,000 continuously available as backing. Dynamic activation may add delay. Its possible value is better commitment terms, less unnecessary encumbrance, or greater sustainably attracted capital—not an accounting increase in solvency. In a single-market, no-rehypothecation MVP, some of these benefits may be small. That is a reason to test rather than assume the feature must survive.

The pinned Percolator source already has a kinked backing-utilization rate, and separate paths for ordinary account deposits and dedicated source backing. The wrapper and engine snapshots previously inspected also do not share the same declared dependency pin.[^6] These facts make two gates mandatory: demonstrate the exact accounting path on a compatible pair, and demonstrate value beyond a rate curve.

Use separate ledgers for trader margin, maker equity, dedicated domain backing, undrawn committed reserve and reward escrow. An internal activation transfers value from one use to another; it must not increase total assets. Requested LP redemptions reduce capacity for new exposure even while existing liens may prevent immediate payment. Reward liabilities consume funded resources; future trading fees do not count as cash today.

The simplest valid fallback is a static reserve with transparent risk limits. If the dynamic policy does not improve outcomes or underwriting terms after fair comparison, retain the exchange controls and remove that complexity. Reducing LP losses, payout impairment and execution failures together is more meaningful than reporting fewer “ADL events” while hiding losses in LP NAV.

## 7. Competitive and substitute evidence

The opportunity is not empty. The strongest direct overlap is no longer limited to historical hackathon submissions.

| Evidence | What the primary source establishes | What it does not establish |
|---|---|---|
| derp.trade | Describes a mainnet beta, long-tail AMM derivatives and a published program address | Independent adoption, safe parameters or profitable underwriting |
| derp.trade payout rules | Distinguishes displayed/sale PnL from realizable PnL constrained by pool resources | Full-profit exits on every price path |
| Perk | Versioned docs describe permissionless markets, vAMM execution and a Percolator-derived engine | Current economic safety or compatibility with the selected upstream engine |
| Wasabi | Documents underlying-backed spot leverage funded by vaults; API docs list Solana | Every advertised market or leverage limit being available on Solana |
| Omnipair | Documents isolated Solana spot/margin pools combining swaps and lending | Elimination of valuation, write-off or LP risk |

derp.trade's own risk and position-value pages disclose pool payout constraints.[^7] This is directly relevant to the intended promise: simply enabling a closing instruction is not equivalent to paying the profit shown on a chart. Its documentation calls the application a mainnet beta and publishes a program address, but no independent activity or bytecode verification was performed here.[^8]

Perk's documentation overlaps strongly with the “permissionless + Percolator” pitch. Its security page distinguishes internal reviews from independent audits.[^9] Claims about its oracle independence, automatic recovery or proof coverage should not be imported as demonstrated guarantees for a new implementation.

Wasabi explicitly describes spot-leverage contracts rather than synthetic perpetual exposure, while Omnipair describes an integrated spot-and-lending pool with internal pricing and debt write-offs.[^10][^11] These substitutes strengthen the need to interview users about the job they want done: leveraged buying, shorting, hedging holdings, or holding a transferable perpetual position. They do not establish demand for this particular app or backstop policy.

The September 9 Colosseum benchmark remains useful historical evidence: Perc-o-dex, derp.trade, Squeeze, Archer, InsureOS, Uranus and Reflect overlap with components of the concept. Awards and project submissions are not customer commitments. The record and exact links remain in the [idea file](dynamic-jit-liquidity-perp.md).

The defensible differentiation hypothesis is narrower: **admit supportable underserved markets, publish credible capacity, and buy loss-bearing capital at a price that improves net payout outcomes**. No first-ever claim is established. A reusable backstop/accounting component may also be a better entry point than another complete trading venue if distribution proves difficult.

## 8. Foundation alignment and trust boundaries

Fully onchain execution means the authoritative order, quote calculation, execution checks, cancellation and settlement are enforced on Solana. Offchain indexers and keepers may observe state and submit transactions, but must not decide balances or hold unilateral custody authority.

The Foundation's June 1 brief separately prefers participant-driven pricing and explicitly names competing-maker RFQ and orderbooks. An oracle-priced pool is not compliant with that pricing preference merely because its transactions are onchain.[^12] Grant fit is a real tradeoff, not a naming problem.

If that entire brief is a hard business constraint, either obtain competing makers and narrow supported markets accordingly, or position the capital controller as complementary infrastructure. If product-market fit is the priority, the pool-first hypothesis can proceed to validation without claiming that grant alignment. The current documents preserve onchain execution and disclose the pricing tradeoff; they do not claim funding eligibility.

Onchain execution also does not remove other trust dependencies: oracle publishers, risk-attestation signers, upgrade authorities, stablecoin controls, RPC availability and keeper inclusion. Permissionless market registration can coexist with gated activation. The interface and specification must make that distinction explicit rather than advertise a fully autonomous listing engine before one exists.

## 9. Validation verdict and sprint

### Evidence scorecard

| Dimension | Assessment | Reason |
|---|---|---|
| Demand for this app and fee structure | Weak; 1/3 | Motivating anecdote and adjacent products, no customer commitment |
| Underwriting supply | Unvalidated | No accepted size, lock, loss and premium terms |
| Competition | Material direct and substitute overlap | Long-tail derivatives, Percolator apps and spot leverage already described |
| Technical feasibility | Hard, conditional | Risk engine reuse helps; oracle economics, wrapper integration and loss accounting remain |
| Two-week outcome | Validation and bounded simulation, not production exchange | Existing program is a counter scaffold |
| Founder fit / distribution | Unknown | No reliable evidence supplied; no invented aggregate score |
| Blockchain necessity | Strong for the stated trust model | Shared custody rules, transparent balances and non-custodial execution are core |

The verdict is **go validate / narrow the cohort**, not an unconditional go. Confidence is moderate in the pool-first conditional architecture and low in commercial viability until actual terms and behavior are observed. Removing the blockchain would not make leverage impossible, but it would replace the intended public custody and permissionless transaction model with a trusted operator.

### A concrete ten-business-day validation sprint

These are proposed research targets, not completed interviews or commitments.

1. **Build a candidate register.** Start with 10–20 tokens that appear to lack a useful perp. Record exact mints, launch/graduation status, real venues, source overlap, liquidity control, spot size at several impacts, candidate feed and existing leverage substitutes. Identify at least two plausible candidates for a narrow pilot; reject the rest with reasons. No token is admitted from its market cap alone.
2. **Interview 8–12 target traders.** Ask for their last actual missed trade or hedge, position size, holding period, current workaround, unacceptable spread, and tolerance for payout delay. Separate longs, shorts and hedgers. Ask them to attempt the proposed flow at disclosed fees on a prototype; do not count a generic expression of interest as a conversion.
3. **Interview 3–5 independent potential underwriters.** Present concrete jump/rug scenarios and loss-aware redemption terms. Seek at least two written, non-binding indications of acceptable allocation, premium and duration. Distinguish paid trading revenue from issuer subsidy. Do not collect deposits during interviews.
4. **Test dealer availability separately.** Ask prospective makers what size and uptime they could supply without an external perp hedge. If useful commitments exist, reopen the RFQ/CLOB decision; do not assume the pool must always be the right answer.
5. **Run matched-budget simulation.** Compare static backing, native-rate behavior, reactive deposits and precommitted activation with the same total resources, oracle rules, liquidation policy and fees. Separate extra-capital effects from allocation-policy effects.
6. **Demonstrate adversarial outcomes.** Include permanent +100% and +500% moves, near-zero collapse, liquidity withdrawal, failed keepers, stale/malicious prices, exhausted rewards, no new LP arrivals, withdrawal races and a price move beyond the reserve's capacity.
7. **Review the decision.** Proceed to a limited integrated prototype only if an eligible cohort and economically plausible counterparties remain. This sprint does not replace independent security or legal review before any real-money launch.

Proposed pass criteria are substantive, not a magic score: at least two credible customer problems with observable follow-through; written underwriting terms that fit plausible collected fees; one market whose practical caps withstand the selected economic attack model; and exact accounting reconciliation in the integration spike. If a failure is masked by subsidy or by refusing almost all useful trades, the architecture is not validated.

### Kill, narrow or pivot rules

- If meaningful trade sizes make manipulation profitable, shrink capacity; if the remaining size has no user value, reject that market.
- If no LP will underwrite at trader-affordable fees, reject or change the payoff/hedging model rather than invent rewards.
- If dynamic activation loses its advantage against static backing, ship the simpler reserve policy or stop that feature.
- If users primarily need financed spot exposure, evaluate spot-backed leverage as a separate product decision.
- If grant-aligned price discovery is mandatory but makers are absent, narrow assets or build supporting infrastructure.
- If no reliable settlement reference exists, a bounded-payoff derivative still needs a credible resolution rule; observation or spot-only is the appropriate fallback, not a renamed unsafe perp.

## 10. Architecture-to-MVP boundary

The local program currently exposes `initialize` and `increment`, not trading, custody or liquidation. The production specification must therefore be implemented and tested, not treated as a description of an existing exchange. The engine and wrapper pin, account mapping and native recovery behavior are architecture-freeze gates.

The execution plan separates an instrumented simulator, program integration, SDK, backend/indexer, frontend, independent review and public devnet acceptance. It also separates the MVP from an optional capped mainnet pilot. Reusing Percolator is justified by the protocol's scope, but its mathematical guarantees do not extend automatically to a new oracle adapter, LP share ledger or CPI boundary.

Integration-first assessment: reuse verified token interfaces, appropriate oracle adapters, testing infrastructure and generated client bindings. A spot CLOB or generic lending SDK is not automatically a substitute for the required perp risk interface. Novel capital allocation and policy enforcement warrant custom program logic; a full custom matching engine does not yet warrant its additional scope.

Production work must include checked fixed-point arithmetic, defined rounding, bounded account work, invariant and differential tests, snapshot-based external-state tests, key separation, reproducible builds, recovery drills and independent review. No live deployment, paid API subscription, customer outreach, asset selection or capital allocation is authorized by this document.

## Sources

[^1]: Pump, [Pump program documentation](https://github.com/pump-fun/pump-public-docs/blob/main/docs/PUMP_PROGRAM_README.md) and [PumpSwap program documentation](https://github.com/pump-fun/pump-public-docs/blob/main/docs/PUMP_SWAP_README.md), living repository documentation; accessed September 11, 2026. Used for reserve and migration distinctions, not market safety thresholds.
[^2]: Ellipsis Labs, [Phoenix Legacy repository](https://github.com/Ellipsis-Labs/phoenix-v1), living README/source; accessed September 11, 2026. Feasibility and public implementation reference, not a proposed perp integration certification.
[^3]: GMX, [Providing liquidity](https://docs.gmx.io/docs/providing-liquidity/), living documentation; accessed September 11, 2026. Index-matched versus synthetic backing and isolated market mechanics.
[^4]: Pyth Network, [Best practices](https://docs.pyth.network/price-feeds/core/best-practices), living documentation referencing the August 26, 2026 upgrade; accessed September 11, 2026. Executable-price and latency risks.
[^5]: Uniswap, [Uniswap v2 Oracles](https://developers.uniswap.org/docs/protocols/v2/concepts/oracles), version-specific documentation; accessed September 11, 2026. TWAP economic rationale; Ethereum timing assumptions are not transferred to Solana.
[^6]: Anatoly Yakovenko et al., [Percolator source at 8eb7142](https://github.com/aeyakovenko/percolator/blob/8eb7142aada316f6c476f5c4fa815d3a806706d5/src/v16.rs) and [wrapper manifest at d5e2ec6](https://github.com/aeyakovenko/percolator-prog/blob/d5e2ec6fa727a1049a62851c3be19c638815b4e2/Cargo.toml), inspected September 9 and rechecked in the retained snapshot September 11, 2026. The wrapper pins engine `495a5590c97055bd71c6f94d849ff0298f243145`; compatibility is not assumed.
[^7]: derp.trade, [Risk overview](https://docs.derp.trade/docs/risk) and [Position value](https://docs.derp.trade/docs/protocol/value), living documentation; accessed September 11, 2026. Attributed payout-limit and oracle-model disclosures, not an endorsement of all safety claims on those pages.
[^8]: derp.trade, [Introduction](https://docs.derp.trade/) and [Contract addresses](https://docs.derp.trade/docs/dev/contracts), living documentation; accessed September 11, 2026. Mainnet-beta and deployment claims are publisher-reported.
[^9]: Perk, [Introduction](https://docs.perk.fund/introduction), labeled documentation version March 29, 2026 / protocol v1.4.1, and [Security](https://docs.perk.fund/security), accessed September 11, 2026. Product and review-scope claims, not independent proof verification.
[^10]: Wasabi, [Leverage Trade](https://docs.wasabi.xyz/_/overview/leverage-trade) and [API Integration](https://docs.wasabi.xyz/_/overview/technical-documentation/api-integration), living documentation; accessed September 11, 2026. Spot-leverage mechanics and listed chain support. The page contains different headline and step-level leverage limits; no numerical limit is relied upon.
[^11]: Omnipair, [Introduction](https://docs.omnipair.fi/), living documentation; accessed September 11, 2026. Attributed isolated spot/margin design; “oracle-less” means no external oracle in that design, not no valuation or loss risk.
[^12]: Solana Foundation, [Build Fully Onchain Perps on Solana](https://solana.com/news/build-onchain-perps), June 1, 2026; accessed September 11, 2026. Onchain-execution and participant-pricing preferences are distinct.

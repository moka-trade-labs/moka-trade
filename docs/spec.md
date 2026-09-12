# Isolated Long-Tail Perpetuals — Production-Intent Specification

- Date: 2026-09-12
- Status: design requirements, not an audited implementation or mainnet authorization.
- Scope: one-market devnet MVP; extension to gated permissionless market creation after validation.
- Related: [idea review](idea-validation-review.md), [execution plan](execution-plan.md), [product thesis](dynamic-jit-liquidity-perp.md).

Document precedence: this specification defines required behavior; the execution plan orders delivery; the idea and review explain rationale. Unresolved numerical profiles and native ABI mappings below are explicit architecture-freeze/release blockers. They are not permission for an implementer to choose silent defaults.

## 1. Product and non-goals

Serve post-launch Solana assets with defensible spot liquidity and price observation but inadequate perp access. Permissionless creation is not permissionless activation of unlimited leverage. Market cap alone is not a listing test.

The MVP is a USDC-collateralized, isolated, synthetic linear perp using a real-capital pool as its maker, a small LP-scoped matcher, a compatible pinned Percolator engine/wrapper, and a separately accounted backstop controller. The pool quotes within explicit risk capacity. It does not provide guaranteed exits, guaranteed full PnL, or infinite liquidity. Finite stablecoin capital cannot guarantee an uncapped long's profits under an unbounded price increase. That remains a disclosed economic limitation, not an implicit payoff cap.

No external maker participation, spot hedge, token borrow, future fee revenue, or uncommitted emergency deposit counts as available protection. Optional competitive makers, persistent orders and a CLOB are deferred. No cross-market margin, shared insurance rescue, governance token, bridge, arbitrary mint activation, user-defined oracle executable, or backend-authorized matching is included.

“Production-intent” means the requirements include production failure modes. It does not mean parameters are calibrated, upstream integration is compatible, contracts are implemented, or deployment is safe.

## 2. Normative terminology and release states

MUST is a required invariant or acceptance criterion. SHOULD is a preference requiring documented rationale if omitted. A requirement marked BLOCKED is not satisfied by writing this document.

Release states are `DESIGN`, `SIMULATOR`, `LOCALNET`, `DEVNET`, and `MAINNET_ELIGIBLE`. These are project gates, not protocol account enum values. The initial target is DEVNET. Mainnet eligibility requires every applicable blocker in section 16 to be closed with evidence and an explicit deployment authorization.

| Prefix | Requirement domain |
| --- | --- |
| CAP | Capital, capacity and accounting |
| ORC | Oracle, observation and recovery |
| EXE | Orders, execution and liquidations |
| MAT | Matcher pricing, capacity and CPI response safety |
| BST | Backstop authority, deployment, rewards and loss ownership |
| LP | LP shares, commitments and withdrawals |
| SEC | Authorization and contract security |
| API | SDK, backend, indexing and frontend |
| OPS | Deployment and operations |
| TEST | Verification |

## 3. Logical architecture and trust boundaries

```text
Trader wallet -- signed market/limit IOC --> pinned Percolator wrapper
Oracle account --------------------------->       |       |
                                                   | CPI   | linked library
                                                   v       v
                                            LP-scoped   Percolator
                                              matcher     engine
                                                   |       |
                                                   +--> trader + maker portfolios
                                                        collateral, claims, domains

Backstop LP --> controller vault/epochs --> atomic domain top-up CPI --> wrapper
Funded reward escrow --------------------> controller
Permissionless keepers --> crank / liquidate / activate; programs revalidate all inputs
Onchain accounts/events --> rebuildable indexer --> read API --> frontend
```

Logical modules MAY share a program if authority and accounting boundaries remain enforceable. The engine is a Rust library, not a CPI target; CPI, if used, targets its Solana wrapper. Program count is an implementation decision, not a security boundary by itself.

SEC-01: authoritative custody, order state, risk admission, execution price validation, settlement and loss allocation MUST be onchain. Bots may construct transactions and calculate candidate inputs; they cannot authorize exceptions. Backend downtime MUST NOT prevent a user from submitting a valid direct instruction.

SEC-02: every reachable risk-increasing native entrypoint MUST enforce market policy, including paths bypassing the application frontend. If an unmodified wrapper exposes bypasses, the adapter architecture MUST change before release. A frontend-only cap is not a cap.

SEC-03: the pool PDA MUST be the constrained authority over its maker account, not a human keeper wallet. Depositors receive economic shares; they cannot directly sign native maker withdrawals or change its positions. A keeper is a transaction submitter, not custodian.

SEC-03A: the deployment SHOULD consist of a minimally modified pinned wrapper fork plus two application-owned programs: an LP-scoped matcher and backstop controller. A separate registry is permitted only if its policy cannot be bypassed through direct wrapper entrypoints. Splitting programs does not relax atomicity, owner, signer or CPI validation.

### 3.1 Engine compatibility contract

EXE-01: begin P1 with engine `8eb7142aada316f6c476f5c4fa815d3a806706d5` and wrapper `2b1d025c004f92d3f89bac00113be90a0cbbcf63` as candidate pins. Pin the final compatible pair, deployed program build, compiler/toolchain, feature flags and account layout only after reproducing upstream builds/tests and this project's matrix. Upstream is explicitly educational and unaudited; a commit pin is reproducibility evidence, not a safety endorsement.

The matrix MUST identify: trader and maker registration; account deposit/withdrawal; `SetMatcherConfig`; `TradeCpi` request/response binding and `exec_size`; funding accrual and settlement; source-domain allocation; `TopUpInsuranceDomain`; `TopUpBackingBucket`; `WithdrawBackingBucket`; `SyncBackingDomainLedger`; liquidation/position transfer; realization/recovery; market generation and authority epochs; lifecycle; authority and crank requirements. Each adapter action must map to real upstream instructions or an explicitly reviewed wrapper modification. This specification invents no upstream ABI.

EXE-02: pool-as-maker means a trade creates equal-and-opposite base exposure in trader and pool accounts through the native supported matching path, with conservation checked by integration tests. If the pinned engine does not expose an admissible path, pool integration is BLOCKED. Do not implement an independent PnL ledger pretending to be Percolator settlement.

CAP-01: engine claims, liens, funding, realization, source-side backing and native loss waterfall remain authoritative. The application MUST NOT recreate extinguished claims, overstate realizable PnL using one global H, or override native loss ordering with an LP receipt promise.

### 3.2 Reference-derived boundaries

- Phoenix demonstrates that order liquidity and virtual maker liquidity can share one priority model, and that liquidation should progress through cancellation, partial liquidation, backstop transfer and only then ADL. For this MVP it is a design reference, not a CPI dependency.
- Percolator Trade already occupies the permissionless long-tail Percolator category. The product MUST be evaluated on admitted-market quality, published executable capacity and payout/ADL reduction—not a first-to-market claim.
- ORDR's maker-local books and offset repricing inform the future maker ABI. Its protocol implementation was not available in the public repository inspected for this review, so no security or integration claim may rely on it.
- Velocity's permissionless fillers/keepers and explicit reduce-only-to-settlement lifecycle inform operations. Its archived public Drift-derived repository MUST NOT be treated as current Velocity source.
- Correctness MUST hold on ordinary Solana transaction ordering and public RPC paths. Private relays, ACE/BAM, shred feeds or favored sequencers MAY improve inclusion but MUST NOT be required to prevent theft, insolvency or replay.

## 4. Conceptual account and ownership schema

Names below describe required responsibilities, not deployed structs, exact PDA seeds or upstream account layouts. Final seed encodings MUST be fixed-width/domain-separated, versioned, collision-tested and documented in the SDK. Every market-owned account MUST bind the market identifier and expected program owner.

| Logical account | Authority / identity | Required state and constraints |
| --- | --- | --- |
| Protocol configuration | Governed config authority | Version, supported programs/mints, constrained upgrade/pause roles; no per-trade global writable counters |
| Market configuration | Market identity PDA | Collateral mint, index identity, oracle policy hash, engine instance, lifecycle, versioned risk/fee configuration |
| Pool authority and maker mapping | Market-scoped PDA | Exact native maker account identity and custody mapping; no arbitrary destination |
| Pool equity ledger | Market-scoped accounting | Share supply, recognized equity, encumbered amounts, pending redemption allocation, accounting epoch |
| Trader mapping | Market plus owner/account identity | Native trader account binding; owner/delegate permissions and nonce domain |
| Matcher configuration | Pool maker plus matcher program/context/delegate | Version, quote parameters, oracle band, side capacities, fee policy, sequence/expiry rules; exact wrapper opt-in binding |
| IOC request (transaction data; no persistent account in MVP) | Trader signature plus unique sequence | Side, maximum size, limit, fee cap, expiry slot, expected market/config/generation; all terminal in one transaction |
| Persistent order intent (post-MVP only) | Trader plus unique nonce | Side, maximum size, limit, TTL, observation/time, filled size, reserved risk/fees, status |
| Pending-risk aggregate | Market/side scoped | Conservative aggregate reservations if persistent orders are enabled; no scan of all orders |
| Collateral custody | Verified token-account owner | Exact collateral mint/program, vault authority, conservation to native/application ledgers |
| Backstop vault | Controller PDA, exact collateral mint | Funded undrawn principal, deployed principal, cash, aggregate obligations; never a native backing claim before successful allocation |
| Backstop epoch/share position | Market/domain/epoch and LP owner | Shares, entry loss/recovery snapshots, term, reward terms, pending redemption, loss-adjusted ownership |
| Domain allocation ledger | Market plus native source domain | Native allocation identity; available, liened/consumed/recoverable state from verified engine reads |
| Reward escrow and obligations | Constrained controller PDA | Actual funded tokens, reserved offers, accrued payable rewards, unreserved budget |
| Withdrawal request | LP/epoch/nonce | Share amount, locked shares, loss-adjusted claim rules, reservation, request state |
| Oracle policy/observation cache | Verified data path | Feed identity, sequence/publication time, acceptance slot, confidence/deviation data, policy version |
| Recovery state | Market scoped | Failure reason, last trusted observation, mode entry, authorized transition evidence, unresolved liabilities |

SEC-04: validate account owner, signer, writable designation, PDA derivation, mint, token program, upstream program ID, market binding and lifecycle for every instruction. Reject duplicate/aliased accounts where roles require distinct custody or ledgers. No caller-supplied arbitrary CPI program or destination.

SEC-05: collateral MVP uses an explicitly allowlisted mint and token-program behavior. Unsupported transfer fees, hooks, rebasing, freeze behavior or extensions MUST fail admission, not be treated as ordinary transfers. Actual received/debited balances MUST reconcile with accounting. The index token need not be held in custody, but its supply/admin/transfer characteristics still affect listing risk.

## 5. Capital accounting and numerical rules

### 5.1 Four different capital roles

CAP-02: maintain distinct beneficial ownership and encumbrance for:

1. **Pool maker equity:** capital supporting trading exposure, adjusted for native losses and liabilities. Not simultaneously an independent insurance allocation.
2. **Deployed domain backing/insurance:** capital legally and natively assigned to specific claim support, subject to liens and consumption. Nominal deposited amount is not freely recoverable principal.
3. **Undrawn committed reserve:** funded, locked, deployable capital under an explicit commitment. It is not current domain backing until deployment succeeds. Scenario analysis MAY model future activation and its failure/latency. Actual new-risk admission MUST NOT gain capacity from it unless the required draw and native allocation succeed atomically with admission, or the capital has already been deployed. An offchain promise or expected next-slot keeper action is insufficient.
4. **Funded reward escrow:** funds belonging to reward obligations/budget. Not maker equity, trader collateral or available rescue backing.

Trader capital and claim obligations remain separately identified. One physical vault, if mandated by native design, does not collapse these economic ledgers.

CAP-03: compute capacity from eligible unencumbered capital, current native claims, conservative gap-loss estimates, pending order reservations, and pending redemption policy. Do not subtract the same obligation twice or pledge one dollar to two source sides. H equals 1 is insufficient admission evidence.

CAP-04: maintain incrementally updated per-side/gross exposure and conservative pending-risk aggregates. Trade admission MUST be bounded in compute and account count, not scan all positions. If native freshness requires cranking, use bounded cranks and refuse unsafe execution until their prerequisites are met. Approximate aggregates MUST conservatively bound exact requirements; establish this with differential tests.

CAP-05: risk scenarios MUST include discontinuous upside and downside, oracle/keeper latency and concentrated positions. Numeric leverage, margin, OI, skew, shock and maximum-order settings are BLOCKED pending calibration. Finite stress limits are not a maximum possible loss claim.

CAP-06: funding semantics MUST come from the pinned native engine, with payer/recipient, sign, timing, bounds, realized-versus-accrued treatment and insolvency behavior documented. Pool inventory spread is an execution adjustment, not an independent market price to manufacture circular funding. Any additional capital charge requires a separate funded/collectible ledger and reviewed economics.

### 5.2 Arithmetic

CAP-07: prices, token amounts, base positions, shares, rates and time MUST have explicit integer units/scales. Use checked wide intermediate arithmetic, explicit signed conversions, bounded inputs and deterministic overflow errors. Floating point is forbidden in consensus calculations. Client displays MUST use the same published conversion library or exact equivalent test vectors.

CAP-08: define rounding per operation: charges/reservations conservatively upward, credits/share minting downward unless native semantics require otherwise; account for residual dust explicitly. Never reverse a native rounding rule by assumption. Test repeated split/merge/deposit/withdraw/fill cycles for value extraction. Minimum sizes and dust redemption rules are release blockers until their bounds are specified.

### 5.3 Public units and ledger reconciliation

CAP-12: the public interface MUST use integer strings with an explicit market unit descriptor: collateral in the mint's smallest units; base exposure in signed integer `base_lots`; a positive rational `base_units_per_lot`; price as a positive rational number of collateral atoms per base lot; and shares in integer share units. No JavaScript floating-point amount is accepted as authoritative. Fix maximum numerators, denominators, lot sizes and native conversion rules in the versioned market profile before implementation acceptance; reject unrepresentable prices rather than round them to zero. Internal engine scales remain native and must be mapped exactly or with explicitly conservative rounding.

For an accepted, fully reconciled pool epoch with positive conservative NAV, the reference share operations are `minted = floor(deposit_atoms * total_shares / nav_atoms)` and `redeemed = floor(burned_shares * nav_atoms / total_shares)`, subject to native withdrawability. An exact-assets withdrawal burns shares with ceiling rounding. These formulas do not define NAV, confer liquidity on encumbered assets or solve bootstrap donation attacks. Initial share scaling, locked minimum shares and zero/negative-NAV epochs require the LP-06 tests; ordinary deposits into such an epoch are disabled until an explicit bootstrap/recapitalization path applies.

CAP-13: report two distinct reconciliations: physical token balances versus cumulative transfers, and economic ownership/claims versus the native state. Internal reserve deployment changes allocations, not aggregate tokens. Pending share redemptions remain part of the share-ownership model until burned; do not also deduct them as a fixed cash debt and thereby charge LPs twice. Every subtraction in NAV or capacity must cite the exact obligation and its ownership ledger.

CAP-14: settlement denomination is USDC units, not guaranteed US dollars. If the index is asset/USD, the approved adapter MUST define asset/USDC conversion or explicitly document the parity assumption and depeg halt policy. The collateral issuer's freeze/blacklist powers, depeg and redemption risk are accepted dependencies to assess, not supposedly revoked authorities. A frozen collateral vault cannot guarantee withdrawals even when protocol accounting is solvent. Simulate depeg and freeze/unavailability separately from index-token price shocks.

## 6. Market admission and lifecycle

Onchain-enforceable admission includes approved oracle verifier and feed binding, required observations, configured freshness/confidence checks, deposited creator seed, funded reward budget if offers are enabled, supported collateral behavior, configured risk caps, and availability of required keeper instructions. A transaction cannot prove human due diligence, beneficial ownership diversity or honest trading volume merely from a signed assertion.

SEC-06: distinguish machine-verifiable eligibility from attestations. For the first market, a reviewed allowlist MAY gate activation. If an attestation is used for spot-source independence or token risk, disclose attestor identity, scope, expiry, revocation and trust assumptions. Do not market that as trustless permissionless activation. Later permissionless activation requires demonstrably enforceable criteria or an explicitly accepted attestation model.

Proposed market modes:

- `PROPOSED`: configuration and seed collection only; no trading.
- `ACTIVE` (displayed as `NORMAL`): permitted trading within every oracle/capital/native gate.
- `GUARDED`: new risk remains possible only under tighter configured size/leverage/capacity limits; backstop deployment and liquidation urgency may increase, but rewards remain funded and capped.
- `REDUCE_ONLY`: no new risk. An action labeled “close” is not sufficient; evaluate its portfolio/capacity effect. Native protective paths remain separately authorized.
- `PRICE_UNCERTAIN`: price-dependent trading, withdrawals and liquidations requiring unreliable valuation are blocked; safe collateral additions, order cancellations and non-price-dependent maintenance remain available.
- `RECOVERY`: native loss/recovery handling and explicitly specified resolution procedures; no ordinary new risk or naive share minting into a known deficit.
- `WIND_DOWN`: no new risk; bounded settlement and loss-adjusted exits according to native constraints.
- `CLOSED`: liabilities resolved or explicitly finalized under documented native rules; no account closure that destroys outstanding claims.

ORC-01: `PRICE_UNCERTAIN` is different from insolvency `RECOVERY`. A missing price is not proof a token is worthless; zero price, last-price forever, and administrator-selected convenient settlement price are not fallback defaults.

SEC-07: publish a transition table with caller roles, objective preconditions, events and permitted instruction classes. Risk-tightening may be expedited by a bounded emergency role; loosening risk or changing oracle identity MUST require governed delay and outstanding-order handling. No administrative withdrawal of user collateral. Exact governance/upgrade delays and role assignments are release blockers, not arbitrary placeholder defaults.

Minimum transition contract (additional native restrictions still apply):

| Transition | Initiator and prerequisites | Effect on pending work |
| --- | --- | --- |
| PROPOSED → ACTIVE | Approved activation authority, or permissionless caller presenting every required unexpired certificate and onchain gate | New version recorded; no inherited unvalidated orders |
| ACTIVE → GUARDED | Any caller proving configured headroom, volatility, skew, oracle-confidence or liquidation-latency stress | Apply tighter limits immediately; enable eligible pre-shortfall deployment |
| GUARDED → REDUCE_ONLY | Any caller proving stop threshold, failed deployment, expired certificate or required oracle/capital condition | Reject exposure increase; preserve valid reduction/protective paths |
| GUARDED → ACTIVE | Exit threshold held for configured hysteresis interval plus current certificates | Restore normal limits; do not force backing withdrawal |
| ACTIVE → REDUCE_ONLY | Any caller proving a configured risk/certificate-expiry condition, or bounded emergency authority | Reject new exposure; future persistent intents cancel/expire by their specified rule; preserve valid protective actions |
| Any trading mode → PRICE_UNCERTAIN | Any caller proving oracle failure, or automatic check during an attempted operation | Price-dependent actions reject even before mode change lands; no last-price fills |
| PRICE_UNCERTAIN → REDUCE_ONLY | Configured evidence of valid observations and native freshness | No automatic return to full risk; old incompatible intents cannot execute |
| REDUCE_ONLY → ACTIVE | Current certificates, restored capital/price gates and required governed approval | Configuration-version checks remain mandatory |
| Any open mode → RECOVERY | Native recovery condition or explicitly authorized recovery procedure | Native loss/claim treatment; ordinary new-risk trades disabled |
| REDUCE_ONLY or RECOVERY → WIND_DOWN | Published native-compatible resolution predicates | Bounded settlement, liabilities and ownership preserved |
| WIND_DOWN → CLOSED | Native obligations resolved or finalized under the specified claim rules | No remaining actionable user claims may be destroyed |

A failing trade transaction cannot be relied on to persist a protective-mode event: Solana atomic rollback also removes its attempted state changes. Every affected instruction MUST check the protective predicate directly. A separate successful permissionless transition instruction may persist the mode and event.

## 7. Oracle contract and price-failure recovery

ORC-02: canonical observation validation MUST verify feed identity, verifier owner/program, signature/proof path, exponent/units, positive representable price, publication sequence/time, freshness, confidence and policy-specific deviation constraints. Chain time/slot bounds and oracle publication time are different quantities. Reject future-dated, replayed, regressing and incorrectly normalized observations.

ORC-03: a spot-based feed is not safe solely because it is onchain or averaged. Admission must model manipulation cost versus extractable perp value, venue concentration, removable liquidity, wash volume and delayed averaging under real jumps. A publisher count is not independent spot liquidity. Minimum history/depth/source rules and confidence/deviation thresholds require measured evidence before release.

ORC-07: the risk certificate MUST bind the market/mint, source configuration, parameter version, covered maximum exposure and reachable capital/reward budgets, issuance/expiry, signer policy and evidence digest. Risk-increasing execution must check its current validity onchain, not rely on a keeper having already changed the mode. Model attacker extraction including automatically deployable reserve and rewards, net of recoverable manipulation inventory and any attacker-owned liquidity. Capital inflows, source changes or exposure increases outside the certificate's analyzed envelope require revalidation; reject or quarantine them from activation rather than silently expanding the attack surface. Evidence digests authenticate a report's identity, not its correctness.

ORC-04: IOC execution requires the canonical current eligible publication under the configured adapter and slot/time policy. Record sequence plus publication/acceptance times. The caller MUST NOT choose a favorable observation from several stale-but-technically-valid accounts or omit a newer mandatory update. Define one unambiguous selection rule and replay it in adversarial tests. If the oracle infrastructure cannot enforce that rule without privileged selection, execution is BLOCKED pending redesign.

ORC-05: deviation checks may reject genuine jumps. Halt behavior MUST be observable and testable. Recovery to price-valid operation requires an objective, configured observation sequence or an explicit governed resolution path with delay and disclosure; one keeper cannot bypass a deviation guard. Exact recovery criteria are release blockers.

ORC-06: during price uncertainty, do not execute purportedly risk-reducing trades against stale prices merely to keep a “close” button functional. Permit only operations whose safety is established without the missing value, such as cancellation and supported collateral top-up. Track queue age and alert. If prolonged outage or token failure prevents normal recovery, use a separately reviewed native-compatible resolution mechanism. Mainnet release is blocked without defined residual-claim treatment.

## 8. Order execution contract

The MVP uses direct signed market and limit **IOC** transactions against one isolated maker. There is no persistent order account, offchain matcher, waiting executor or cancellation race in the core path. A transaction either settles its verified `exec_size` atomically or makes no trade. Trigger, GTC and TWAP orders are post-MVP features implemented as permissionless intent/filler flows only after their reservation and optionality rules are specified.

### 8.1 IOC request and matcher contract

EXE-03: every request binds wrapper program, market/config version, `market_id` generation, trader and maker identities, side, maximum base lots, limit price, maximum total fee, expiry slot, oracle policy version and a unique trader sequence. The request MUST expire within a bounded window. It MUST NOT authorize a backend to substitute accounts, extend expiry, loosen limits or change configuration after signing.

EXE-04: the same transaction reads fresh authoritative state, obtains a matcher response, revalidates the returned bindings, checks trader margin and maker/domain capacity, and settles equal-and-opposite native exposure. Pending maker redemption is excluded from new-risk capacity. If preparatory native cranks are required, the IOC may reject until they complete; those cranks cannot reserve a favorable stale quote.

MAT-01: the matcher is LP-scoped and enabled only through the wrapper's exact `(matcher program, context, delegate)` configuration. It MUST be deterministic over versioned inputs and return request identity, maker identity, asset index/generation, oracle price, signed `exec_size`, execution price and fee. The wrapper MUST treat all response bytes as adversarial, reject mismatches and settle `exec_size`, never the requested quantity.

MAT-02: derive the quote from the accepted oracle observation, pre-trade maker inventory, actual execution size and versioned spread/impact parameters. Apply an oracle deviation band, positive-price bound, per-order maximum, per-side inventory/OI capacity and conservative pending-redemption deduction. A formula producing a number is not sufficient capacity.

MAT-03: buy cash debits round up and sell receipts round down. All multiplication/division MUST use checked widened arithmetic with explicit scale conversions. Reject overflow, zero/negative price, invalid sign, out-of-band quote, non-monotonic price or a response whose fill increases risk beyond the request. Golden vectors MUST match Rust, SDK and simulator exactly.

MAT-04: one wrapper/matcher call MUST fit declared compute, account and transaction-size budgets at p99 observed devnet conditions. Maker-local context avoids a protocol-wide writable orderbook lock but does not remove the maker account's own contention. Record conflict/failure rate under load; do not advertise theoretical Solana TPS as venue throughput.

EXE-05: the user's limit and fee cap apply to the actual executed price, size and total fees. Zero fill MAY return success only with an explicit reason/event and no financial mutation; otherwise reject. Partial IOC fills are permitted only if `0 < abs(exec_size) <= requested_size`, the user limit holds for that fill, and all fees/risk are computed from actual fill. A retry uses a new sequence.

EXE-06: no global best-price, FIFO or participant-driven price discovery is claimed for the MVP. When multiple maker adapters are later enabled, the routing and tie-breaking rule MUST be onchain-verifiable or the UI must describe it as an untrusted routing suggestion.

### 8.2 Post-MVP persistent orders

EXE-07: GTC, trigger, stop and TWAP orders require a separate versioned intent account, conservative margin/capacity reservation, exact-once fill/cancel/expiry transitions and permissionless execution. Configuration changes define whether existing intents cancel or remain bound to their original safe version. No such instruction is enabled in the MVP.

EXE-08: future filler designs MUST be tested against oracle lag, selective execution, keeper withholding, cancel-after-information optionality, self-sandwiching and priority-fee competition. Private relays MAY improve inclusion but cannot be a correctness assumption.

### 8.3 Simulator reference quote model

This model makes the initial experiment reproducible; it is not an assertion of sufficient market-making or manipulation protection. Let `x` be net trader exposure in base lots (the pool holds `-x`), `q` a signed requested change (buy positive), `P` the canonical collateral-atoms price per lot, `Q > 0` a fixed inventory scale, `k >= 0` an inventory coefficient and `s >= 0` a half-spread. All quantities use exact rational arithmetic in the reference model.

```text
Phi(x) = P * (x + k*x*x/(2*Q))
signed_cash_debit = Phi(x + q) - Phi(x) + P*s*abs(q)
average_execution_price = signed_cash_debit / q    (q != 0)
                       = P * (1 + k*(x + q/2)/Q + s*sign(q))
```

The debit is the signed trade value, not a requirement to transfer the whole notional out of a margined trader's wallet. The matcher maps the execution price and actual `exec_size` into native matched positions; separate nonnegative fees follow native/application ownership rules. Risk admission is independent of `Q`: this scale is not a claim of real backing.

Require the whole inventory interval from `x` to `x+q` to lie within `[-X, X]`, with `k*X/Q + s < 1` and an approved positive-price margin. Do not clamp an invalid quote into range; reject it. Keep `Q`, `k` and the policy version fixed for the property under test. Changing them, using a different oracle price, giving rebates or allowing other traders' intervening flow changes the analysis.

For a closed inventory loop at the same `P` and configuration, potential differences telescope; spreads and nonnegative fees remain costs. This is a limited reference property, not a proof against market-moving or timing attacks. Integer implementation must round buy debits upward and sell receipts downward, avoid overflow, and show that splitting orders cannot create value through dust. Quote parameter changes invalidate affected pending intents according to their bound configuration version.

Test vector without fees: `P=1`, `Q=1000`, `k=0.1`, `s=0.001`, `x=0`, `q=100` gives debit `100.6` and average price `1.006`; closing with `x=100`, `q=-100` gives debit `-100.4`, leaving a round-trip cost of `0.2`. These dimensionless rational values are test fixtures, not launch settings. Production may use a different monotone quote model only with replacement invariants and vectors.

## 9. Liquidation, reduction and backstop activation

EXE-11: liquidation MUST use verified pinned native eligibility, valuation and loss-allocation rules, not the pool's displayed spread. Specify actual supported position transfer/close path, liquidator margin, pool participation, per-instruction progress bounds and fee recipient. A retail intent's timing window MUST NOT be imposed on protective native liquidation without evidence that it is safe.

EXE-12: do not wait for new LP deposits, reward spikes or optional makers before attempting valid native protective action. Liquidation may still fail for oracle uncertainty, capacity, inclusion or native settlement reasons; each reason needs an explicit event/error and operational alert. A liquidation bounty cannot guarantee a counterparty or block inclusion.

CAP-09: deployment reads current native domain eligibility and fresh stress state, verifies funded undrawn commitment and reward constraints, transfers through the constrained authorized backing path, and updates ownership/encumbrance atomically where supported. Failure MUST not count undrawn money as deployed. Multi-step native paths require a fail-closed intermediate state and recovery procedure.

CAP-10: activation thresholds MUST precede projected shortfall, may trigger while native support ratios equal 1, and use hysteresis to prevent repeated reward farming. Prove trigger manipulation cannot cheaply extract more reward than its total cost under modeled attacks. Existing realized deficits require disclosed recapitalization terms, not ordinary LP entry.

CAP-11: optional voluntary reduction rebates and urgency rewards are post-core experiments, disabled in the MVP unless separately specified/tested. A rebate is payable only after verified net reduction of the specified stress requirement and bounded against funded budget. “Any close earns a rebate” is not acceptable.

BST-01: the controller PDA MUST be the configured Percolator backing authority for only the declared market generation and source domains. It has no arbitrary trader withdrawal, oracle, upgrade or destination authority. Controller instructions bind wrapper program, market account, collateral mint/token program, `market_id`, domain, authority epoch and expected fee policy.

BST-02: a commitment transfers actual USDC into the controller vault and mints loss-aware epoch shares after reconciling cash, deployed principal, upstream cumulative loss/recovery and obligations. Undrawn principal is standby capital, not Percolator backing and not admitted new-risk capacity. An availability label must publish amount, term, domain permissions and activation latency.

BST-03: `activate_backstop` is permissionless and idempotent by controller sequence. It verifies `GUARDED`/eligible stress predicates, hysteresis, requested amount, funded cash, epoch/domain limits, reward budget and current wrapper generation/authority epoch. It then signs one atomic CPI to the pinned domain top-up path. Only CPI success changes deployed principal or support capacity; failure changes neither and pays no reward.

BST-04: for the candidate wrapper, `TopUpBackingBucket` is the preferred primitive where its exact semantics pass P1: it binds generation, authority epoch, monotonic intent, backing-fee/insurance-share policy, live domain and future expiry. `TopUpInsuranceDomain` MAY be used only for a separately specified insurance tranche. The implementation MUST NOT silently substitute one because both transfer USDC.

BST-05: synchronize the upstream `BackingDomainLedgerAccountV16` before share mint, activation accounting, withdrawal and redemption. `cumulative_loss_atoms` and `cumulative_recovery_atoms` remain separate monotonic observations; recovery does not erase the historical loss counter. Controller ownership uses snapshot deltas plus verified cash/deployed state. If the final pin changes these semantics, this design is BLOCKED until remapped and retested.

BST-06: backing expiry is eligibility for native release checks, not a promised LP redemption time. Withdrawal first proves native principal is fresh, unencumbered and authorized, then atomically updates the upstream bucket and controller accounting. Consumed/impaired/liened principal remains unavailable. Reward escrow cannot repay lost principal unless a separately funded, explicit recapitalization rule says so.

## 10. LP accounting, commitments and redemption

LP-01: pool shares and backstop ownership are separate instruments with explicit risk disclosures. Commitments specify funded principal, minimum term, deployment permission, loss allocation, availability/deployment reward rules, fees, expiry and redemption conditions. A minimum term is not a guaranteed principal-return date.

LP-02: NAV/share calculations MUST reconcile to native realizable support and liabilities, including accrued funding/fees, claim impairment, encumbrances, consumed backing, pending rewards owed by the relevant ledger, and withdrawal obligations. Do not value fully redeemable shares from a gross vault balance or optimistic unrealized PnL. The exact conservative valuation function is a release blocker requiring proof/test vectors against the pinned engine.

LP-03: before minting or redeeming shares, settle/recognize applicable losses and verify the accounting epoch. Deposits into impaired or uncertain-value pools MUST be rejected or placed in a separately specified recapitalization epoch with explicit loss acceptance. New entrants must not receive underpriced claims on old capital; old LPs must not transfer undisclosed known losses to entrants.

LP-04: reserving a withdrawal locks the corresponding shares and records its versioned rule for loss participation. A queue is not a fixed senior USDC debt unless fully funded and actually removed from at-risk backing under native rules. Subsequent losses and gains apply according to the published share/epoch policy until redemption; no double entitlement to shares and cash.

LP-05: pending redemption policy MUST reduce new-risk admission conservatively without pretending encumbered native backing is already withdrawable. Execution can pay only loss-adjusted, unencumbered amounts after required term, state freshness and native gates. Insufficient liquidity leaves a visible pending request with defined cancellation/recovery behavior, not a silent revert loop or discretionary indefinite freeze.

LP-06: bootstrap seed/share issuance, minimum liquidity, rounding dust, donation effects and empty-pool transitions MUST resist first-depositor/inflation attacks. These parameters require documented tests. Deposits received but not incorporated into accepted NAV cannot silently change another user's execution economics.

LP-07: reserve promised fixed rewards when accepting commitments; total reserved plus accrued obligations MUST never exceed escrow funding. Variable rewards may accrue only within published funded limits. No repayment from prospective volume, reward-token valuation or unrelated trader collateral. Repeated deployment/withdrawal or identity splitting must not duplicate rewards.

## 11. Instruction requirements and observable outcomes

This table defines application operations, not exact native instruction names. Final instruction schemas MUST document every account, authority, unit and bound in generated IDL plus written invariants.

| Operation | Preconditions | Required postconditions |
| --- | --- | --- |
| Propose/configure market | Authorized creation, unique identity, supported programs/mint, valid bounded configuration | Immutable identity bindings established; no trading enabled implicitly |
| Activate market | Admission checks, seed, oracle readiness, configuration approvals, native initialization complete | ACTIVE only after all checks; event records policy/version |
| Register/deposit trader | Owner/delegate, correct custody/native mapping, supported mode | Actual transferred collateral reconciles; ordinary deposit does not claim to increase domain backing |
| Withdraw trader | Owner/delegate, fresh required state, native withdrawal entitlement, no reserved-order bypass | Native liabilities remain supported; destination bound; no share/backstop ledger affected |
| Execute IOC | Owner/delegate, active policy, canonical eligible oracle, fresh native state, sequence/expiry/limit/fee/capacity | Verified matcher `exec_size`; equal/opposite native positions and actual fees committed atomically, or no trade |
| Create/fill/cancel persistent intent (post-MVP) | Feature enabled under separate reservation/optionality specification | Exactly one terminal outcome and exact reservation release; no effect while feature is disabled |
| Crank/settle native state | Valid bounded account set and native prerequisites | Native progress only; cannot bypass application admission or mint rewards without policy |
| Liquidate/transfer | Native eligibility, valid oracle, receiving-account capacity, allowed path | Native loss/position updates; bounded verified keeper reward; no fabricated full payout |
| Deposit pool equity | Valid NAV epoch, eligible mode, mint safeguards | Shares minted from actual received value; prior losses recognized |
| Commit backstop | Defined term/permission, actual funds, valid epoch, sufficient reward budget | Undrawn principal and obligations tracked; not yet native backing |
| Activate backstop | Trigger/domain eligibility, funded available commitment, constrained authority | Successful native backing allocation and matching loss-aware ownership update |
| Request/cancel redemption | Share/commitment owner, eligible request state | Shares/claims locked or released consistently; pending-risk aggregate adjusted |
| Execute redemption | Term/state/native gates, loss-adjusted NAV, sufficient unencumbered principal | Only owned eligible value paid; share/claim burn and reservations reconciled |
| Claim rewards | Accrued unpaid entitlement and actual funded escrow | Exact once-only payment; no principal withdrawal shortcut |
| Enter protective mode | Objective condition or permitted bounded emergency authority | Mode and cause recorded; allowed safe operations preserved |
| Recover/wind down/close | Published recovery predicates, native lifecycle compatibility, residual liabilities handled | No erased claimant records or silent settlement-price invention |
| Govern configuration | Correct role/timelock, allowed parameter range, pending-order policy | New version published; no retroactive order economics without explicit cancellation/rejection rule |

SEC-08: use stable typed application errors, mapping native errors without concealing their cause. Required categories include invalid authority/account/mint/version/generation/domain; matcher binding or response mismatch; oracle stale/future/replayed/confidence/deviation; market mode; sequence replay or request expiry; limit or fee exceeded; insufficient margin/capacity; native freshness/recovery block; NAV uncertain; term incomplete; backing encumbered; reward budget exhausted; arithmetic overflow. Error names are design requirements, not an asserted upstream enum.

API-01: events MUST include market/config version, generation/domain where applicable, request sequence, relevant account/epoch identifiers, requested and executed amounts with units, state transition and native reference. Record IOC outcome, future intent lifecycle, oracle-mode changes, liquidation progress, backstop deployment/loss/recovery, share issuance/redemption and reward payments. Do not emit an event asserting successful payment before atomic success. Event data must not include secrets or unnecessary personal information.

## 12. SDK, backend, indexer and frontend

API-02: publish a typed SDK for direct wallet IOC submission, account decoding, exact integer conversions, quote simulation, future intent lifecycle, oracle prerequisites, recovery explanations and native error interpretation. Transaction builders MUST bind supported program IDs, expected market/owner, generation and config version. Backend suggestions are untrusted and wallet users must not sign opaque arbitrary instructions.

API-03: backend APIs are read/transaction-construction services, never custody or trading authority. Suggested resources: markets and policy/status; versioned quotes; IOC receipts/history; pool NAV/withdrawal queues; commitments/rewards; health/recovery; and post-MVP intents only when enabled. Quote responses include observation time, configuration/version, size/side, estimated fees, capacity and expiry; they are estimates, not guaranteed execution promises. Numeric values serialize as exact integer strings plus units.

API-04: index transactions with slot, signature, instruction/event position and commitment/finality. Deduplicate by canonical event identity; support rollback/reconciliation of nonfinal observations and replay from a durable checkpoint. Idempotent consumers MUST not double-count fills/rewards after retries. Reconcile indexed balances/exposure to authoritative accounts periodically and surface drift. An indexer cannot infer finality from HTTP success.

API-05: APIs MUST enforce schema validation, bounded pagination/query cost, timeouts and rate limits. Authenticate private preferences or user-specific offchain data with nonce/domain/expiry-bound wallet challenges; reading public onchain positions need not be falsely presented as private. No private keys in backend database, logs, analytics or browser bundles. WebSocket subscriptions need connection/backpressure limits and resynchronization checkpoints.

API-06: frontend MUST distinguish quote time, wallet approval, submission, simulation failure, landed/confirmed/finalized, zero/partial/full IOC fill and request expiry. For future persistent orders, cancellation remains pending until confirmed and a prior fill wins by chain order. Display pool capacity, oracle freshness, market mode, native payout impairment/settlement limits, principal-at-risk and dollar reward budget. No “guaranteed APY,” “guaranteed exit” or H-equals-1 success banner implying withdrawability.

API-07: disable misleading actions when the oracle or indexer is stale, but preserve explained direct collateral/backstop top-up and future persistent-order cancellation paths where onchain rules permit them. LP screens distinguish maker equity, deployed backing, undrawn locked principal, accrued funded rewards and pending loss-adjusted redemption. A displayed order estimate must not be represented as realized PnL or claim support.

### 12.1 Proposed versioned service contract

API-08: use `/v1` routes and a common response envelope containing `schema_version`, `cluster_genesis_hash`, `program_id`, `market_config_version` where applicable, `as_of_slot`, `commitment`, and a stale-data indicator. Financial numbers are integer/rational strings with the unit descriptor from CAP-12. These endpoints expose cached data or construct unsigned transactions; they never create authoritative financial state.

| Endpoint | Minimum request / response contract |
| --- | --- |
| `GET /v1/markets` | Bounded cursor/limit; market identities, eligibility/mode, capacity, certificate expiry and observation status |
| `GET /v1/markets/{market}` | Exact market identity; configuration, native bindings, pool/backstop ledgers, oracle provenance and trust disclosures |
| `POST /v1/quotes` | Market, trader, side, integer lots and optional price/fee limits; returns indicative quote, limits, fees, observation/config versions, expiry and rejection reasons |
| `POST /v1/transactions/build` | Strict enum of supported operation plus typed arguments and owner; returns unsigned transaction, required signers, instruction summary, blockhash and last-valid block height |
| `GET /v1/orders/{signature-or-intent}` | IOC receipt by signature/sequence; post-MVP intent lifecycle and reservations only when that feature is enabled |
| `GET /v1/accounts/{owner}/positions` | Public onchain positions, displayed versus native realizable support and pending settlement; bounded pagination |
| `GET /v1/markets/{market}/liquidity` | Maker NAV/ownership, commitments, accrued funded rewards and loss-adjusted withdrawal queues as distinct fields |
| `GET /v1/health` | Service readiness, indexer/RPC lag and supported versions; no credentials, database strings or privileged internals |
| `WS /v1/stream` | Bounded subscriptions to markets/orders/accounts, event identity/cursor, finality updates and explicit gap/resync messages |

Wallets sign client-side and may submit directly through their chosen RPC. Submission or relay, if later offered, must not imply confirmation. On blockhash expiry, construct a new message and obtain a new user signature; do not silently mutate a signed transaction. Retry processing deduplicates trader sequences, future intent IDs and chain event IDs, not merely HTTP request IDs. A stale API response must never relax an onchain guard.

## 13. Operational and deployment requirements

OPS-01: localnet/devnet deployments MUST use separate keys, RPC credentials, collateral mint, program IDs, config and telemetry from any production environment. Reproducible builds and verified deployed artifacts are required; publish commit/toolchain/build hashes and source. Deployment scripts must validate cluster/genesis identity and refuse unexpected destinations.

OPS-02: production custody/config upgrades require documented multisig or equivalent constrained governance, separation from keeper keys, key rotation and incident procedures. Upgrade authority, emergency powers and delay tradeoffs MUST be visible to users. No unrestricted administrator rescue transfer or arbitrary oracle substitution.

OPS-02A: publish an authority matrix for wrapper upgrade, matcher upgrade, controller upgrade, market activation, risk tightening, risk loosening, oracle-policy change, reward funding and emergency pause. Upgrade authorities MUST be separate from routine keeper/filler keys; mainnet requires multisig custody, delayed risk loosening/upgrades where practicable, tested rotation and a path to revoke or make programs immutable after maturity. Emergency pause preserves collateral top-up, safe maintenance and claim-preserving wind-down where native rules permit.

OPS-03: monitor oracle age/deviation/confidence, keeper inclusion/progress, order age/failure causes, capital/claim ratios, per-side exposure and reservations, reward solvency, withdrawal queue age, indexer lag/reconciliation and native recovery state. Thresholds derive from calibration. Alerts MUST reach named operators through tested escalation paths; an automated alert is not incident resolution.

OPS-04: operate redundant independent executors/oracle delivery paths where supported and test outage/restart behavior. Keepers need only bounded hot-wallet SOL, not custody authority. Document RPC failures, congestion, failed cranks, prolonged oracle outage, detected manipulation, drained rewards, pool loss and compromised admin/keeper response. Never relax price checks automatically merely because transactions stop filling.

OPS-05: mainnet rollout, if authorized, starts with constrained market/capital exposure and documented rollback/protective-mode decisions. A contract upgrade cannot reverse already-finalized transfers. Audit, legal/compliance assessment, disclosures, incident staffing and distribution restrictions require their own approval; this specification provides no legal conclusion.

## 14. Verification and acceptance tests

TEST-01: unit/property tests MUST cover checked arithmetic, all rounding edges, source-domain bookkeeping, conservation, IOC sequence/replay/expiry, matcher request/response bindings, state transition permissions, post-MVP reservation exact-once release, share accounting and reward budget conservation.

TEST-02: differential integration tests against the pinned engine/wrapper MUST establish ordinary deposit versus backing behavior, matched pool/trader exposure, funding settlement, liens/realization, native withdrawal constraints, liquidation, impairment and recovery. Compare application NAV/capacity bounds with authoritative state after arbitrary valid operation sequences. No wrapper-bypass entrypoint may admit prohibited risk.

TEST-03: adversarial stateful fuzzing MUST include aliased accounts, cross-market/generation/domain substitution, malicious matcher return data, counterfeit feeds/programs, future/stale/regressing observations, extreme exponents/prices, overflow, duplicate rewards, first-depositor attacks, donation/NAV manipulation, withdrawal/IOC/deployment races, post-MVP multiple pending intents, dust cycling and invalid CPI account substitution.

TEST-04: execution simulations MUST include ordinary flow, no external makers, one-sided launch demand, abrupt wick/reversal, permanent repricing, pool-threatening rally, spot liquidity removal, manipulated spot/TWAP, stale price, publisher/keeper outage, selective execution, cancellation optionality, block congestion, withdrawal wave, no fresh LP entrants and exhausted rewards. Use independently specified stress scenarios rather than optimize the controller to one replay.

TEST-05: compare static reserve, native-rate-only, reactive JIT and precommitted deployment with equal total loss-bearing capital and equal funded subsidy budgets. Record payout impairment, forced reduction, execution/settlement latency, failed orders, LP net loss/return, reward efficiency and capital utilization. Do not attribute a benefit to the controller if liquidation or admission policy changed between baselines.

TEST-06: acceptance thresholds for LP loss tolerance, manipulation resistance, p95/p99 execution/protective-action latency, maximum supported accounts/compute, failed-order rate and capacity-bound tightness MUST be set before judging the final experiment. Values are release blockers until calibrated and approved; arbitrary numerical defaults would create false assurance.

TEST-07: localnet/devnet end-to-end tests MUST cover wallet transaction inspection, IOC expiry/partial/zero/full fill, sequence replay, malicious matcher responses, future intent/fill/cancel races when enabled, oracle-failure UI, recovery state, withdrawal queue, reward exhaustion, two competing keepers, indexer rollback/replay, restart/idempotency, rate limits and secret exclusion. Contract security review, independent audit and retesting of findings are mandatory mainnet gates, not substitutes for economics testing.

TEST-08: run the pinned wrapper's reproducible SBF build, all-target tests, LiteSVM regression fixtures, stateful fuzz corpus and applicable Kani targets before each wrapper release. Document exclusions and coverage boundaries: local wrapper/ABI proofs do not establish correctness of the engine, CPI callees, oracle economics or the whole deployed composition. Add Trident or equivalent application-level stateful fuzzing and Surfpool/mainnet-fork scenarios for real account layouts without treating fork success as authorization to touch live funds.

## 15. Core invariants requiring evidence

The following are proof/test obligations, not established facts about this unimplemented system:

- No reachable application/native route can spend another market's assets or evade admission policy.
- Every committed trade is native-valid and conserves matched exposure and accounted value, including fees and rounding.
- No matcher response can change the requested market, maker, asset, oracle basis or direction, and `abs(exec_size)` never exceeds the signed request.
- Pending orders and withdrawal requests cannot over-reserve or double-pledge capital.
- Undrawn reserves, deployed backing, maker equity and rewards cannot be counted twice.
- A failed or stale backstop CPI cannot increase deployed principal, support capacity, LP loss, reward accrual or keeper payment.
- Upstream loss and recovery counters reconcile monotonically to controller epochs without treating recovery as erasure of prior loss.
- Share issuance/redemption cannot shift already-known losses without explicit recapitalization acceptance.
- Every fixed reward promise is funded and every payment occurs at most once.
- Oracle publication selection and cancellation mechanics do not create an unbounded, cheaply exercisable option against LPs.
- Bounded aggregates conservatively represent obligations without scanning all accounts per trade.
- Protective and recovery paths preserve native claim/lien semantics and cannot silently manufacture or erase payouts.
- Under calibrated scenarios, the proposed capital policy beats or justifies its cost versus equally funded baselines; outside them no solvency guarantee is claimed.

## 16. Release blocker register

Each blocker needs an owner, dated evidence link and pass/fail decision in the execution plan. Absence of evidence is failure, not implied approval.

| Blocker | Evidence required | Blocks |
| --- | --- | --- |
| Compatible engine/wrapper and pool matching ABI | Candidate pair reproduction, final commit/build pins, `TradeCpi`/domain instruction matrix, native integration tests | Localnet trading onward |
| Matcher safety and transaction budget | Golden vectors, malicious-return tests, compute/account/lock benchmarks | Localnet trading onward |
| Backstop authority and upstream ledger semantics | Atomic CPI traces, generation/domain/epoch checks, loss/recovery and redemption properties | Backstop localnet onward |
| Non-bypassable policy/authority boundary | Call graph and adversarial direct-entrypoint tests | Localnet trading onward |
| Conservative pool NAV and domain ownership | Ledger specification, worked impairment/epoch cases, differential/property tests | LP-valued prototype onward |
| Deterministic quote and funded capacity model | Simulator specification, monotonicity/round-trip tests, scenario calibration | Devnet economic acceptance |
| Oracle identity/source and manipulation model | Source independence/depth analysis, attack-profit bounds, live-data replay | Live-asset devnet acceptance/mainnet |
| Canonical newer-observation/timing rule | Implementable verifier design, censorship/cherry-pick/cancel-option tests | Execution acceptance |
| Native funding, liquidation and recovery semantics | Code-referenced mapping and adverse-state integration tests | Devnet acceptance |
| Numeric caps, precision, dust and delays | Parameter report with predeclared acceptance thresholds | Devnet acceptance/mainnet |
| LP entry/withdrawal/recapitalization policy | Signed-off lifecycle, first-depositor/donation/loss-allocation tests | LP release |
| Price-failure and permanent-token-failure resolution | Objective recovery procedure plus residual-claim tests | Mainnet |
| Underwriter and trader economics | Realistic capital commitments, fee willingness, equal-budget results | Product/mainnet decision |
| Security and production operations | Audit/remediation, verified build, keys/roles, incident drills, indexer reconciliation | Mainnet |
| Deployment and legal/compliance authorization | Explicit approvals for actual scope/jurisdictions | Mainnet |

## 17. Primary design references

- [Pinned Percolator engine overview](https://github.com/aeyakovenko/percolator/blob/8eb7142aada316f6c476f5c4fa815d3a806706d5/README.md) and [source](https://github.com/aeyakovenko/percolator/blob/8eb7142aada316f6c476f5c4fa815d3a806706d5/src/v16.rs): risk accounting reference, not a deployment endorsement or asserted compatible wrapper pair.
- [Candidate Percolator wrapper](https://github.com/aeyakovenko/percolator-prog/tree/2b1d025c004f92d3f89bac00113be90a0cbbcf63) and [`v16_program.rs`](https://github.com/aeyakovenko/percolator-prog/blob/2b1d025c004f92d3f89bac00113be90a0cbbcf63/src/v16_program.rs): `TradeCpi`, domain top-up, authority epoch, intent sequence and backing-ledger integration reference. The repository's educational/unaudited warning remains controlling.
- [Phoenix matching engine](https://docs.phoenix.trade/phoenix/matching-engine/matching-engine), [spline liquidity](https://docs.phoenix.trade/phoenix/matching-engine/spline-liquidity), [margin math](https://docs.phoenix.trade/phoenix/margin-and-risk/margin-math), [mark price](https://docs.phoenix.trade/phoenix/margin-and-risk/mark-price) and [liquidations](https://docs.phoenix.trade/phoenix/margin-and-risk/liquidations): reference for sequence-safe liquidity, capacity, conservative order risk and progressive liquidation. [Rise public SDK snapshot](https://github.com/Ellipsis-Labs/rise-public/tree/38cc64a9cd7b7301639cf2df30b53e8a3747b723) is an interface reference, not Phoenix core source.
- [Percolator Trade](https://percolator.trade/) and [X updates](https://x.com/PercolatorTrade): direct category/product benchmark; its claims are not adopted as independent audit or usage evidence.
- [ORDR architecture](https://ordr.trade/), [X updates](https://x.com/ordrtrade) and [public landing-repository snapshot](https://github.com/CHA0S-LABS/ordr-landing/tree/94c82bda7549411bc1731869b0caa1f0a4e1c801): future maker-state reference. No public ORDR protocol repository was established in this review.
- [Velocity protocol](https://docs.velocity.exchange/protocol), [keepers/orderbook](https://docs.velocity.exchange/protocol/how-it-works/orderbook-and-keepers), [oracles](https://docs.velocity.exchange/protocol/how-it-works/oracles), [order types](https://docs.velocity.exchange/protocol/trading/order-types) and [delisting](https://docs.velocity.exchange/protocol/risk-and-safety/delisting-process): permissionless operations and wind-down reference. The [archived `protocol-v2` snapshot](https://github.com/velocity-exchange/protocol-v2/tree/13e8e9b8d614f3b62e3a65a8c372c819e6529aeb) is Drift-derived historical code, not treated as the current private Velocity program.
- [GMX liquidity documentation](https://docs.gmx.io/docs/providing-liquidity/): isolated backing, capacity and synthetic-market payout-risk precedent; parameters are not imported.
- [Solana transaction pipeline](https://solana.com/docs/core/transactions/transaction-pipeline): transaction/account execution constraints; no throughput guarantee is inferred.
- [Foundation onchain-perps brief](https://solana.com/news/build-onchain-perps): favors participant-driven price formation; our pool-priced MVP is a product-first tradeoff, not a claim to meet that specific criterion.

This document specifies requirements and unresolved research gates. It does not authorize deployment, attest to current protocol safety, or replace an audited implementation and operational readiness review.

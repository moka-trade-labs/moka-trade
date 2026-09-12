# Execution plan: architecture to public devnet MVP

- Date: 2026-09-12.
- Status: proposed implementation plan; not evidence of completed engineering, security review, or commercial validation.
- Product direction: [refined idea](dynamic-jit-liquidity-perp.md).
- Decision evidence: [idea-validation review](idea-validation-review.md).
- Requirements and invariants: [production-oriented specification](spec.md).

## 1. What this plan delivers

The deliverable is a public **devnet MVP** for one admitted token market: an isolated USDC-capital pool acts as maker, a minimal matcher supplies deterministic quotes, a candidate-pinned Percolator wrapper supplies authoritative custody/risk/settlement, and a separate controller owns funded backstop commitments. A funded capital policy and transparent loss accounting can be tested without requiring competing professional makers at launch.

This is not unrestricted day-zero listing. Registration can be permissionless, but activation requires the published oracle, capital, token, and risk checks. If a human or committee attests to offchain market quality, disclose that trust boundary rather than calling activation fully permissionless.

The MVP excludes a CLOB, persistent GTC/trigger/TWAP orders, competing quote auctions, automated spot hedging, cross-market collateral, a governance token, and guaranteed full-profit exits. Its trade primitive is a direct signed market or limit IOC. An isolated pool is still a directional underwriter; neither isolation nor automation eliminates insolvency.

### Frozen implementation direction

| Layer | MVP decision | Release condition |
| --- | --- | --- |
| Risk/custody | Fork and pin Percolator engine `8eb7142aada316f6c476f5c4fa815d3a806706d5` with wrapper candidate `2b1d025c004f92d3f89bac00113be90a0cbbcf63`; keep local changes minimal | Candidate becomes a release pin only after reproducible build, license review, upstream tests and the P1 compatibility matrix pass |
| Execution | One LP-scoped onchain matcher using `TradeCpi`; oracle-anchored, inventory/size-adjusted, capacity-bounded IOC fills | Returned `exec_size`, price and bindings are treated as adversarial and revalidated by the wrapper |
| Backstop | Application controller PDA is the configured source-domain backing authority; controller owns LP epochs and reward escrow | Undrawn funds affect displayed standby capacity only; new-risk capacity changes only after successful atomic native allocation |
| Oracle/admission | One supported adapter and one versioned, expiring risk certificate for the first asset | Freshness, confidence, identity, depth/manipulation assumptions and token controls pass; otherwise no activation |
| Services | Typed SDK, permissionless keeper, rebuildable indexer/read API and web UI | No service is custody, price or risk authority; direct wallet construction remains possible |

Phoenix, Percolator Trade, ORDR and Velocity are architecture references, not imported dependencies. Phoenix motivates progressive risk stages and future maker adapters; Percolator Trade makes permissionless long-tail listing a direct competitive baseline; ORDR motivates maker-local state if professional makers arrive; Velocity motivates permissionless execution/cranks and explicit wind-down. None justifies adding a CLOB or cross-margin lending to the MVP.

“Production-grade smart contracts” is a quality target expressed through requirements, review, and tests—not a label earned by finishing the code. Public devnet acceptance and real-money mainnet approval are separate decisions. This plan does **not** authorize a mainnet deployment or movement of funds.

## 2. Ownership, dependencies, and schedule assumptions

Roles may be held by the same person during prototyping, but security approval of that person's own critical implementation is not independent review.

| Role | Accountable for |
| --- | --- |
| Product/risk owner | Target cohort, economic assumptions, admission policy, user disclosures, go/no-go decisions |
| Protocol engineer | Engine/wrapper compatibility, onchain accounting, execution, access control, program tests |
| SDK/backend engineer | Typed interfaces, transaction construction, indexing, permissionless keeper software |
| Frontend engineer | Accurate quotes, LP loss/exit UX, wallet flows, devnet experience |
| Independent reviewer | Threat-model challenge, implementation review, audit findings and retests |
| Operator | Infrastructure, monitoring, key policies, incident drills, release evidence |

The protocol dependency path is P0 → P1 → P2/P3. P4 (services/SDK) and P5 (frontend) are also required inputs to P6 integration, followed by P7 acceptance. SDK interface design begins in P1, backend development follows stable instruction contracts, and frontend work can use mocks before integration. P3 testing starts alongside P2; it is not postponed until the programs are finished. P8 requires a separate approval after P7.

Planning assumptions for a small, experienced team: P0 1–2 weeks; P1 1–3; P2 3–6; P3 2–4 overlapping with P2; P4 2–3; P5 2–3 overlapping with P4; P6–P7 2–3. A roughly 10–18 week devnet effort is a planning range, not a delivery promise. Upstream incompatibility, economic failure, or serious security findings can invalidate it. Independent audit procurement/remediation and mainnet readiness are not included.

Execute through four reviewable releases rather than one long build:

| Release | Demonstration | Hard gate |
| --- | --- | --- |
| R0 — accounting proof | Simulator plus exact upstream build/tests and transition traces | Reject the architecture if native backing/withdrawal semantics or equal-capital economics fail |
| R1 — localnet vertical slice | Deposit → IOC trade → margin check → reduce/liquidate → withdraw on pinned binaries | No frontend mock may substitute for the authoritative transition |
| R2 — backstop localnet | Commit → atomic domain activation → realized loss/recovery sync → loss-adjusted redemption | No double counting, unfunded reward, authority bypass or cross-domain spend |
| R3 — public devnet | UI/SDK, two keepers, indexer rebuild, oracle/protective modes and incident drills | All MVP requirements traced to evidence; no real-value collateral |

## 3. Repository work products

Proposed application stack: Rust for native program integration and exact risk math; a typed TypeScript SDK shared by services and UI; TypeScript API/keeper services; PostgreSQL for rebuildable indexed history; and a React/Next.js web application with wallet-standard-compatible signing. These are planning choices, not installed dependencies or validated version combinations. P1 pins the actual toolchain and SDK interfaces; do not upgrade the existing Anchor scaffold blindly. Start without microservices or Redis unless measured load/reliability requires them. Host the web app separately from continuously running keepers, which need monitored process supervision and narrowly funded transaction keys.

The ideation repository's Anchor counter scaffold was intentionally not migrated. It was not a partially completed exchange and must not influence the pinned wrapper or instruction ABI decision.

All paths below are **proposed**, unless explicitly labeled existing. P1 may revise them after selecting a compatible upstream pair. Do not install dependencies or generate production code merely to make the directory tree look complete.

| Work product | Proposed location | Owner |
| --- | --- | --- |
| Existing design documents | `finalized/` | Product/risk |
| Architecture records, upstream pins, compatibility matrix and deployment manifest | `perp-dex/docs/architecture/` | Protocol |
| Economic simulator, scenario fixtures, benchmark reports | `perp-dex/sim/` | Risk + protocol |
| Minimally modified pinned wrapper fork | `perp-dex/vendor/percolator-prog/` | Protocol + reviewer |
| LP-scoped pool matcher | `perp-dex/programs/matcher/` | Protocol |
| Backstop vault/controller | `perp-dex/programs/backstop/` | Protocol |
| Market registry/admission policy, if wrapper constraints require it | `perp-dex/programs/registry/` | Protocol |
| Generated interface artifacts and typed transaction builders | `perp-dex/packages/sdk/` | SDK + protocol |
| Property, integration, adversarial, and snapshot tests | `perp-dex/tests/` | Protocol + reviewer |
| Crank/liquidation/oracle keeper service | `perp-dex/services/keeper/` | Backend |
| Non-authoritative indexer/API | `perp-dex/services/indexer/` | Backend |
| Trader and LP application | `perp-dex/apps/web/` | Frontend |
| Runbooks, deployment manifests, incident exercises | `perp-dex/ops/` | Operator |

## 4. P0 — Validate the market and freeze a testable architecture

Owner: product/risk owner, with protocol engineer. Dependency: none.

Work:

- Define the eligible cohort using actual oracle quality, executable spot depth, token controls, liquidity concentration, and underwriting availability—not market cap alone.
- Collect a small, explicitly sampled set of candidate assets and record why each passes or fails. Separate no-existing-perp demand from a demonstrated willingness to trade at feasible fees and caps.
- Interview potential traders and USDC underwriters. Document capital amount, lock/exit conditions, acceptable drawdown, required compensation, and creator-subsidy assumptions. Expressions of interest are not funded commitments.
- Choose isolated pool-as-maker execution; specify price inputs, inventory/size adjustments, collateral, per-side/gross exposure limits, and loss allocation. Keep mark/risk valuation distinct from execution pricing.
- Build the economic model and threat model before optimizing the dynamic curve. Include gaps, rugs, permanent repricing, oracle manipulation, keeper delay, failed exits, and reserve exhaustion.
- Compare static backing, native-rate incentives, and precommitted deployment with equal total capital and funded reward budgets. Separate maker equity, dedicated backing, undrawn commitments, and reward escrow.
- Decide whether a dynamic capital controller earns its complexity. If it loses against an equally funded static reserve, ship the simpler policy or stop; do not rescue the thesis by silently increasing its subsidy.

Outputs: approved architecture record; candidate admission matrix; threat model; simulator assumptions; benchmark protocol; requirement-to-test matrix linked to `spec.md`.

Exit gate: the team can name who pays profitable traders after a gap, explain every capital ledger, identify a plausible admitted first market, and state falsifiable economic acceptance criteria. If there is no defensible reference price or willing loss bearer at viable terms, do not proceed as though software can solve the missing economics.

## 5. P1 — Prove the engine and accounting integration

Owner: protocol engineer. Dependency: P0 architecture and ledger model.

Work:

- Pin a compatible Percolator engine and wrapper revision, toolchain, interface layout, and license dependencies. Record the manifest's actual dependency commit; independently inspected upstream HEADs are not automatically compatible.
- Begin with the candidate pair `percolator@8eb7142aada316f6c476f5c4fa815d3a806706d5` and `percolator-prog@2b1d025c004f92d3f89bac00113be90a0cbbcf63`. Run `cargo build-sbf --tools-version v1.52`, `cargo test --all-targets`, the repository's LiteSVM regression/stateful-fuzz fixtures, and applicable Kani jobs. Record every exclusion: wrapper Kani coverage does not prove the full engine/CPI system.
- Map `TradeCpi`, `SetMatcherConfig`, `TopUpInsuranceDomain`, `TopUpBackingBucket`, `WithdrawBackingBucket`, `SyncBackingDomainLedger`, per-asset `market_id`, authority epochs, lifecycle states and recovery paths. Verify exact accounts, signers, units, return bindings and writable locks against source and tests.
- Build a minimal integration spike showing account margin deposit, direct IOC maker trade, domain backing top-up with future expiry, loss/recovery ledger synchronization, loss realization, and permitted withdrawal against the selected binary.
- Demonstrate that ordinary margin deposits do not magically become dedicated backing, consumed backing does not remain redeemable, and no dollar backs multiple domains.
- Decide whether the pool/controller can be an external authorized program or requires a narrowly scoped wrapper change. Check that direct calls cannot bypass admission, exposure, or authorization controls.
- Define generated IDL or equivalent canonical interface artifacts and typed SDK models now. Do not invent a backend-only interpretation of program state.

Outputs: reproducible integration harness; exact pins and build manifest; instruction/state mapping; interface artifact; acceptance traces; revised architecture record if needed.

Exit gate: accounting transitions reconcile against the selected program, not only a simulator. Any required engine change is explicitly reviewed and its proof/test impact understood. If the selected wrapper cannot safely expose the needed semantics, stop and revise the architecture before building the UI.

## 6. P2 — Implement the onchain programs

Owner: protocol engineer. Dependency: P1 accounting proof and interfaces; P3 runs concurrently.

Implement in small reviewed increments:

1. Wrapper integration and market lifecycle: reproducible fork, registration, admission/activation, bounded parameter updates, `NORMAL`/`GUARDED`/`REDUCE_ONLY`/`RECOVERY`, and deterministic resolution rules.
2. Pool matcher: LP-scoped configuration, reference-price validation, spread/inventory/size quote, signed IOC limits, expiry/sequence checks, hard per-side capacity, adversarial response binding, and atomic `TradeCpi` settlement.
3. Custody and isolated pool ownership: wrapper vault authorities, maker deposits, loss-aware shares, fee ownership, source-domain backing, and pending redemptions.
4. Margin and liquidation integration: supported collateral additions, risk reductions, liquidation/crank paths, and native recovery without bypasses.
5. Funded backstop controller only if P0/P1 justify it: market/domain-bound PDA authority, separate standby vault and reward escrow, share epochs, deployment eligibility, atomic `TopUpBackingBucket`/insurance CPI, upstream loss/recovery counter synchronization, budget reservation, accrual, and loss-adjusted release.
6. Events and versioned state/ABI needed for indexing, reconciliation, and user receipts.

Outputs: program source; versioned interfaces; unit/property tests per increment; documented authorities and account constraints; migration/version strategy.

Exit gate: all applicable `CAP`, `ORC`, `EXE`, `LP`, and `SEC` requirements in `spec.md` have concrete tests or a clearly identified unresolved blocker. Zero unreviewed privileged withdrawal paths, double-pledged backing, or unfunded reward obligations are acceptable.

## 7. P3 — Adversarial integration and security verification

Owner: protocol engineer and independent reviewer. Dependency: begin with P1 harness; expand throughout P2.

Tests must include:

- Conservation and ownership reconciliation after every transition, including realized losses, partial settlement, fees, rewards, and redemptions.
- Zero new LP arrivals; both-sided jumps; near-zero rug; permanent repricing; wrong-way maker inventory; no executable exit; exhausted rewards and reserves.
- Manipulated, stale, divergent, and missing oracle inputs, including manipulation intended to drain the backstop or farm incentives.
- Concurrent withdrawal/fill, quote expiry/fill, deployment/settlement, and deposit-after-known-loss cases; account substitution, unauthorized CPI, replay, arithmetic boundaries, and bypass attempts.
- Deterministic snapshot/fork-style local tests using recorded account states from the exact deployed/upstream versions. Record snapshot slot, configuration, program hash, and account provenance. Fork fixtures are diagnostic environments, not permission to transact with live user funds.
- Compute, account-size, and transaction-inclusion stress; fail-closed handling when required atomic paths exceed limits.
- Equal-capital economic regressions with fixed scenarios/seeds and explicit parameter sweeps. Report trader impairment together with LP losses, not either alone.

Outputs: invariant/test traceability matrix; reproducible failing/passing cases; security review report; resolved findings; residual-risk register; scenario benchmark report.

Exit gate: no unresolved critical/high-severity issues under the agreed severity policy; all accounting and authority invariants pass; severe shocks produce the specified fallback instead of invented collateral or concealed losses. Independent review for a devnet milestone is not a substitute for a real-money audit.

## 8. P4 — SDK, backend, and permissionless keepers

Owner: SDK/backend engineer. Dependency: interface work begins in P1; integration requires reviewed P2 paths.

Work:

- Generate types from the canonical interface; build transaction/quote helpers, account decoders, error mapping, simulation support, and version checks. Submit slippage and expiry constraints onchain, not only in browser state.
- Build an indexer/API for history, discovery, telemetry, and convenience. Treat indexed state as cached views of chain state; the API never approves trades, overrides prices, allocates collateral, or authorizes withdrawals.
- Implement independent, restart-safe keepers for permitted oracle updates, settlement/cranks, liquidations, and eligible deployments. Design idempotency and progress tracking from confirmed onchain state.
- Add a keeper competition harness: two independent instances must race safely, earn at most one bounded reward for one state transition, and recover after either disappears. Baseline correctness must not depend on private relays, Jito BAM or a favored RPC.
- Use fee-payer-only keeper keys by default. A keeper must not possess vault withdrawal, upgrade, or unconstrained parameter authority. Any unavoidable special oracle authorization is separate and explicitly documented.
- Support RPC failures, retries, bounded priority-fee spending, rate limits, health checks, and alternate operators. Confirm finality policy and reconcile skipped/reorganized observations rather than double-counting events.

Outputs: SDK package; service contracts; reconciliation jobs; keeper runbook; key/permission inventory; `API` and applicable `OPS` tests.

Exit gate: a user can construct valid transactions without trusting the hosted API, and another operator can run the keeper. Database outage cannot change financial truth. Recovering a database means rebuilding its view from chain history; it does not roll back the blockchain.

## 9. P5 — Frontend with honest execution and loss UX

Owner: frontend engineer. Dependency: early mocks from P1 interfaces; integrated acceptance against P4 SDK.

Work:

- Build market discovery/eligibility, deposit/withdraw collateral, bounded trade, position reduction, and transaction receipt flows.
- Display execution price, fees, price impact, requested versus fillable size, stale-data state, and enforceable user limits. Do not call an indicative preview a guaranteed fill.
- Separate displayed PnL from supported withdrawable proceeds and pending settlement. Explain native impairment/recovery without implying H alone guarantees a cash exit.
- For LPs, show principal at risk, loss-adjusted share value, funded dollar rewards over the commitment period, pending redemption, and minimum lock versus actual release conditions. Avoid annualized yield as the only risk/reward display.
- Show admission/activation authority and upgrade controls. Clearly label devnet/test collateral, mock feeds, and simulated scenarios.
- Provide recoverable states for rejected trades, expired requests, missing keepers, indexer lag, and failed wallet submissions; verify mobile and keyboard accessibility for core flows.

Outputs: trader/LP application; state/error map; integration tests; user disclosure copy.

Exit gate: users can identify who takes losses, what their quote guarantees, and why a withdrawal may be unavailable. UI balances reconcile with SDK/onchain state; stale API data cannot silently remove transaction protections.

## 10. P6 — End-to-end integration and devnet release candidate

Owner: operator with all engineering owners. Dependency: P2–P5 acceptance.

Work:

- Build and verify the release artifacts; record source commit, upstream pins, toolchain, program hashes, interfaces, configuration, and addresses in a deployment manifest.
- Deploy only to the approved local/test/devnet environment. Use deterministic devnet scenarios or clearly labeled test feeds where a realistic live feed is unavailable; passing synthetic scenarios does not validate real liquidity.
- Exercise the complete trader/LP/keeper lifecycle, including reserve exhaustion and resolution, with the actual deployed binaries and UI.
- Verify alerts for backing headroom, oracle freshness/confidence, maker exposure, unsuccessful liquidations/cranks, reward depletion, transaction failures, and reconciliation discrepancies.
- Drill RPC/indexer/keeper outage, key rotation, no-new-risk activation, service rebuild, and emergency communication. Parameter changes must respect existing obligations; an incident action cannot rewrite settled history.

Outputs: devnet release manifest; end-to-end report; monitoring dashboards; incident drill records; reproducible build instructions; documented known limitations.

Exit gate: an operator other than the original author can reproduce the deployment in an approved test environment, recover services from documented procedures, and explain the safe response to each critical alert.

## 11. P7 — Public devnet MVP acceptance

Owner: product/risk owner signs off with engineering and operator. Dependency: P6.

The MVP is done only when:

- At least one explicitly admitted test market demonstrates the full trader and LP lifecycle using the chosen onchain pool-as-maker architecture.
- A trader can deposit, open, reduce and close through direct signed IOC transactions; a backstop LP can commit, observe activation/loss/recovery and redeem eligible loss-adjusted value; a third party can liquidate, crank and activate without privileged custody.
- The release is publicly usable with test funds and clear disclosures; user feedback distinguishes product demand from incentive farming or testnet activity.
- The published evidence includes economic baselines, zero-arrival and catastrophic-loss tests, supported caps, actual payout/LP outcomes, and known failures.
- Every mandatory MVP requirement in `spec.md` is mapped to a passing test, reviewed evidence, or an explicit scope exclusion approved before launch—not silently marked complete.
- No hosted service has financial authority inconsistent with the specification, and documented independent keeper operation is exercised.
- All financial reconciliation checks pass and remaining non-critical issues are tracked with owners.

This is a **public devnet MVP**, not proof of sustainable LP supply, mainnet security, unrestricted listing feasibility, or zero ADL. The next product decision may be to simplify or stop if testing contradicts the thesis.

## 12. P8 — Optional capped real-money pilot

Owner: separate risk/security/operational approvers. Dependency: P7 plus explicit authorization for any real-money action.

Required before a go/no-go decision:

- Independent audit of the actual integration, custody, matcher, reward logic, and chosen upstream dependencies; remediation and retesting; documented residual risks.
- Economic validation with real underwriting terms and conservative capital/OI limits, plus review of oracle/manipulation assumptions for each proposed asset.
- Relevant legal/compliance assessment of operating and distributing the product; no assumption that “permissionless” removes obligations.
- Verified builds and deployed-program identity checks; approved multisig/upgrade controls, signer separation, timelocks where appropriate, and tested emergency powers limited to their documented purpose.
- Operational coverage, incident communications, secure key custody, monitoring, funded keeper budgets, and a controlled wind-down procedure.
- Explicit approval of the exact network, program artifacts, asset, capital cap, user-access policy, and funds to be moved. No mainnet commands belong in a generic implementation checklist.

Start with one market and capped exposure only if approved. Expansion requires new evidence, not merely higher demand. Never describe a database restore, an upgrade, or a compensating transaction as a blockchain rollback.

## 13. Requirement traceability and handoff

Use the concrete IDs in `spec.md`; the following prefixes identify test families, not substitutes for individual assertions.

| Requirement family | Primary phases | Evidence |
| --- | --- | --- |
| `CAP` — capital conservation, allocation, and capacity | P0–P3 | Simulator ledger checks; integration transition traces; property tests |
| `ORC` — oracle and admission policy | P0–P3, P6 | Asset admission record; invalid-input/manipulation tests; oracle alerts |
| `EXE` — execution and risk enforcement | P1–P3, P6 | Fill/expiry/race tests; direct-entrypoint checks; E2E receipts |
| `MAT` — matcher quote and response safety | P1–P3, P6 | Golden vectors; adversarial CPI responses; compute/account-lock tests |
| `BST` — backstop authority, epochs and deployment | P0–P3, P5–P6 | Atomic allocation traces; loss/recovery synchronization; reward/redemption tests |
| `LP` — ownership, rewards, and release | P0–P3, P5–P6 | Loss-aware share tests; funded-budget reconciliation; redemption scenarios |
| `SEC` — authorities and adversarial safety | P1–P3, P6, P8 | Threat model; permissions review; exploit regressions; independent review |
| `API` — SDK/backend boundary | P1, P4–P6 | Interface compatibility; independent transaction construction; index rebuild |
| `OPS` — deployment and incident readiness | P4, P6–P8 | Manifests; verified artifacts; outage/key/incident drills |
| `TEST` — acceptance and coverage | P0–P8 | Versioned traceability matrix; benchmark reports; milestone sign-offs |

At every gate, record the decision, evidence, owner, unresolved assumptions, and conditions that would invalidate approval. Shipping the next layer must not conceal an unresolved failure in the layer below it.

### Initial concrete requirement-to-test map

These tests are planned, not implemented or passing. Expand the matrix to every mandatory requirement as its interface is frozen.

| Test case | Spec IDs | Phase and required result |
| --- | --- | --- |
| Ordinary deposit versus domain allocation | CAP-01, CAP-02, EXE-01 | P1: distinct native effects verified; no fabricated support increase |
| Bypass wrapper and substitute foreign market accounts | SEC-02, SEC-04 | P1–P3: all invalid paths reject without value movement |
| IOC while LP requests redemption | CAP-03, CAP-04, EXE-04, LP-04 | P2–P3: pending redemption excluded from new-risk capacity; trade or withdrawal cannot double pledge |
| Rational quote fixtures and split/closed-loop trades | CAP-07, CAP-08, EXE-05 | P0/P2: reference vectors match; rounding cannot create value |
| Stale/future/counterfeit prices and expired risk certificate | ORC-02, ORC-04, ORC-07 | P2–P3: risk increase rejects even before a keeper persists protective mode |
| Cheap oracle move unlocks funded rescue/rewards | ORC-03, ORC-07, CAP-10 | P0/P3: rejection or capacity reduction; no practical profitable modeled attack |
| IOC expiry/replay/partial fill and malicious matcher response | EXE-03–EXE-08, MAT-01–MAT-04 | P1–P3/P6: signed bounds hold; actual `exec_size` settles once or no trade occurs |
| Entry after known loss, zero NAV, donation and share rounding | LP-02–LP-06, CAP-12, CAP-13 | P2–P3: no hidden loss transfer or duplicate withdrawal liability |
| Missing price versus valid extreme jump | ORC-01, ORC-05, ORC-06, EXE-11 | P3/P6: distinct protective/recovery behavior, no invented settlement price |
| USDC depeg/freeze or changed collateral behavior | CAP-14, SEC-05 | P3/P6: published denomination and interruption policy respected |
| RPC/indexer restart, nonfinal rollback and repeated delivery | API-04, API-08, OPS-04 | P4/P6: deterministic reconciliation; no duplicate financial history |
| Signed IOC expires or configuration/generation changes | EXE-03, API-02, API-08 | P4–P6: no silent transaction mutation or obsolete fill economics |
| Backstop activation races twice or uses stale generation/domain | BST-01–BST-05, CAP-09 | P2–P3: one atomic allocation at most; failed CPI changes no capacity/reward/ownership |
| Upstream loss then recovery across LP epochs | BST-05–BST-06, LP-02–LP-06 | P2–P3: monotonic counters reconcile; no historical loss erasure or share-value transfer |
| No entrants; static versus dynamic with equal resources | TEST-04, TEST-05 | P0/P3/P7: joint trader/LP results reported, benefit or simplification decision justified |
| Reproduce deployment and rehearse emergency response | OPS-01–OPS-05, TEST-07 | P6/P8: actual environment, hashes, roles and incident evidence approved |

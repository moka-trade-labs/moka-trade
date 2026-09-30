# Agent instructions

## Source of truth

1. `docs/spec.md` defines required behavior.
2. `docs/execution-plan.md` defines delivery order and gates. Upstream pins, route allowlist and oracle design live in `docs/architecture/`.
3. `docs/dynamic-jit-liquidity-perp.md` explains product and architecture rationale.
4. `docs/idea-validation-review.md` records evidence and unresolved market assumptions.

If documents conflict, stop and resolve the specification rather than choosing a silent default.

## Current state

- This repository is a clean implementation workspace; copied research is not implemented code.
- No audit, compatible upstream release pair, deployed program, accepted market, or mainnet permission exists yet.
- MVP pins are Percolator wrapper `5cb331dde354517c6371a8acf92cecb194f3bb73` with its declared engine `4db11a8cb0053815e23a35d3a7d3edc265d8d866` (see `docs/architecture/p1-upstream-reproduction.md`). Upstream is unaudited research code with a public list of open findings; our programs use only the SEC-10 route allowlist.

## Architecture constraints

- Preserve Percolator as the authoritative risk, claim, liquidation, backing, and recovery system; do not create a parallel PnL ledger.
- MVP execution is direct signed market/limit IOC against one isolated real-capital maker per token. Persistent orders, CLOB, cross-margin, lending, and spot hedging are deferred.
- Each listed token is its own Percolator market group with its own USDC vault, insurance, backing and LP vault (spec SEC-09). Nothing is shared between tokens.
- Application-owned programs are `programs/matcher`, `programs/lp-vault` (maker + backing share classes) and `programs/market-registry` (permissionless listing, `marketauth`, DEX-pool oracle adapter). The wrapper and engine are used unmodified at the pinned pair; our programs call only the SEC-10 route allowlist.
- Prices for tokens without a Pyth feed come only from the onchain DEX-pool adapter (spec ORC-08, `docs/architecture/dex-oracle-design.md`). No human key or server pushes marks on any cluster.
- The MVP dynamic LP yield is the engine's native backing utilization fee (spec BST-07). The precommitted backstop controller is deferred research.
- Services and keepers propose transactions; all authority, custody, price, capacity, and reward checks remain onchain.

## Engineering rules

- Use checked integer/fixed-point arithmetic with explicit units and conservative rounding.
- Validate owners, signers, PDAs, mints, program IDs, market generation, domain, authority epoch, oracle identity/freshness/confidence, expiry, sequence, limits, fees, and CPI responses.
- Add a requirement ID from `docs/spec.md` to every protocol test and security-sensitive PR description.
- Reproduce upstream tests before modifying vendored code. Document exact commits, toolchains, features, generated interfaces, and deployed hashes.
- Never place wallet keypairs, RPC secrets, auth tokens, private deployment material, or `.env` values in the repository.
- Do not deploy or move real funds without explicit authorization and the mainnet gates in the specification.

## Branches and PRs

- Name branches after the work: `phase-<N>/<issue>-<short-slug>`, e.g. `phase-2/10-portfolio-lifecycle`; docs-only work uses `docs/<slug>`. Cloud sessions with a pre-assigned branch name use it and say so in the PR description.
- One issue per PR, reviewed by the owner before the next issue starts (owner decision 13).

## Session continuity and learning

- Read `docs/HANDOFF.md` at the start of every session and update it before ending a session that changed decisions or state.
- When the owner asks for a "daily lesson" (or similar), follow `docs/research/learning-curriculum.md` §9 and update `docs/research/learning-progress.md`.

## First implementation task

Phase 1 (repo tooling and CI), then Phase 2 in `docs/execution-plan.md`: the LiteSVM vertical slice against the pinned wrapper binary, proving deposit → IOC trade → domain backing → mark shock → liquidation → loss/recovery → backing earnings → permitted withdrawal, plus two-market-group isolation.


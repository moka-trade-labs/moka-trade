# Agent instructions

## Source of truth

1. `docs/spec.md` defines required behavior.
2. `docs/execution-plan.md` defines delivery order and gates.
3. `docs/dynamic-jit-liquidity-perp.md` explains product and architecture rationale.
4. `docs/idea-validation-review.md` records evidence and unresolved market assumptions.

If documents conflict, stop and resolve the specification rather than choosing a silent default.

## Current state

- This repository is a clean implementation workspace; copied research is not implemented code.
- No audit, compatible upstream release pair, deployed program, accepted market, or mainnet permission exists yet.
- Candidate research pins are Percolator engine `8eb7142aada316f6c476f5c4fa815d3a806706d5` and wrapper `2b1d025c004f92d3f89bac00113be90a0cbbcf63`. They become build pins only after P1 passes.

## Architecture constraints

- Preserve Percolator as the authoritative risk, claim, liquidation, backing, and recovery system; do not create a parallel PnL ledger.
- MVP execution is direct signed market/limit IOC against one isolated real-capital maker. Persistent orders, CLOB, cross-margin, lending, and spot hedging are deferred.
- Application-owned programs are initially the LP-scoped matcher and backstop controller. Keep wrapper changes minimal, pinned, reproducible, and separately reviewable.
- Undrawn commitments do not increase active backing or new-risk capacity until successful atomic allocation to the exact market generation and source domain.
- Services and keepers propose transactions; all authority, custody, price, capacity, and reward checks remain onchain.

## Engineering rules

- Use checked integer/fixed-point arithmetic with explicit units and conservative rounding.
- Validate owners, signers, PDAs, mints, program IDs, market generation, domain, authority epoch, oracle identity/freshness/confidence, expiry, sequence, limits, fees, and CPI responses.
- Add a requirement ID from `docs/spec.md` to every protocol test and security-sensitive PR description.
- Reproduce upstream tests before modifying vendored code. Document exact commits, toolchains, features, generated interfaces, and deployed hashes.
- Never place wallet keypairs, RPC secrets, auth tokens, private deployment material, or `.env` values in the repository.
- Do not deploy or move real funds without explicit authorization and the mainnet gates in the specification.

## First implementation task

Complete R0/P1: reproduce the candidate upstream builds, write the ABI compatibility matrix, and prove deposit → IOC trade → domain backing → loss/recovery → permitted withdrawal on local test infrastructure.


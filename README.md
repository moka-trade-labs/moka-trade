# Moka Trade

Moka Trade is a Solana perpetuals protocol for post-launch tokens that have defensible spot liquidity but no useful perp market. The first release is one isolated USDC market with an onchain pool maker, a pinned Percolator risk wrapper, an LP-scoped IOC matcher, and a separately funded domain backstop.

## Current status

Requirements and execution planning only. No production program, passing protocol test suite, audit, devnet deployment, or mainnet authorization is implied.

## Authoritative documents

Read these in order:

1. [Protocol specification](docs/spec.md) — normative behavior and invariants.
2. [Execution plan](docs/execution-plan.md) — milestones, ownership, tests, and release gates.
3. [Product thesis](docs/dynamic-jit-liquidity-perp.md) — problem, architecture rationale, and competitive benchmark.
4. [Validation review](docs/idea-validation-review.md) — market evidence, assumptions, and rejection tests.

Files under `docs/research/` are historical background. They never override the specification.

## Monorepo layout

| Path | Responsibility |
| --- | --- |
| `programs/matcher/` | LP-scoped IOC pricing and fill response |
| `programs/backstop/` | Funded commitments, domain activation, rewards, and LP loss accounting |
| `vendor/` | Reviewed upstream source snapshots; empty until P1 pins pass |
| `crates/` | Shared Rust math, types, and test utilities |
| `packages/sdk/` | Typed TypeScript account decoders and transaction builders |
| `apps/web/` | Trader and LP web application |
| `apps/api/` | Non-authoritative read API and transaction construction |
| `services/indexer/` | Rebuildable onchain event/account indexer |
| `services/keeper/` | Permissionless crank, liquidation, and backstop automation |
| `sim/` | Economic simulator and equal-capital benchmarks |
| `tests/` | Cross-program, adversarial, fork, and end-to-end tests |
| `ops/` | Deployment manifests, monitoring, and incident runbooks |

## Start here

Begin with P0/R0 and P1 in the execution plan. Do not generate product code until the candidate Percolator engine/wrapper pair reproduces, its ABI is mapped, and the accounting transition harness passes.


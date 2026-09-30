# Moka Trade

[![CI](https://github.com/moka-trade-labs/moka-trade/actions/workflows/ci.yml/badge.svg)](https://github.com/moka-trade-labs/moka-trade/actions/workflows/ci.yml)

Moka Trade is a Solana perpetuals protocol for post-launch tokens that have real spot liquidity but no useful perp market. Any token meeting onchain listing rules can be listed. Each token gets its own isolated Percolator market (own USDC vault, insurance and backing), a mark price read onchain from its deepest DEX pool, and an LP vault with maker shares (the counterparty) and backing shares (earning the engine's stress-responsive backing yield).

## Current status

Phase 1 (repo tooling and CI): scripted, hash-verified build of the pinned upstream plus a Rust workspace skeleton. No production program, passing protocol test suite, audit, devnet deployment, or mainnet authorization is implied.

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
| `programs/lp-vault/` | Per-token LP vault: maker share class (maker portfolio) and backing share class (backing buckets) |
| `programs/market-registry/` | Permissionless listing rules, market-group authority, DEX-pool oracle adapter |
| `vendor/` | Pinned upstream checkouts and verified `.so` files, fetched by `scripts/build-upstream.sh` (not committed) |
| `crates/` | Shared Rust math (`moka-math`) and types/pins (`moka-types`) |
| `scripts/` | Toolchain setup, pinned upstream build, SBF build helper |
| `packages/sdk/` | Typed TypeScript account decoders and transaction builders |
| `apps/web/` | Trader and LP web application |
| `apps/api/` | Non-authoritative read API and transaction construction |
| `services/indexer/` | Rebuildable onchain event/account indexer |
| `services/keeper/` | Permissionless crank, liquidation, and backstop automation |
| `sim/` | Economic simulator and equal-capital benchmarks |
| `tests/` | Cross-program, adversarial, fork, and end-to-end tests |
| `ops/` | Deployment manifests, monitoring, and incident runbooks |

## Start here

```bash
scripts/setup-toolchain.sh && scripts/build-upstream.sh
cargo test --workspace
```

Full local setup, everyday commands and troubleshooting: [docs/development.md](docs/development.md). Session state and next step: [docs/HANDOFF.md](docs/HANDOFF.md). Phases are in the execution plan; the upstream pin, route allowlist and reproduction record are in `docs/architecture/p1-upstream-reproduction.md`. Application programs start only after the Phase 2 LiteSVM vertical slice passes against the pinned wrapper binary.


# Repository layout, program layout and progress tracking

- Date: 2026-09-30
- Answers: "Is a monorepo enough once we add backend, frontend and indexers? One Anchor program or several? How do we track progress?"

## 1. Monorepo: yes, it scales for this project

One repository with a Cargo workspace (programs, crates, tests) and a pnpm workspace (SDK, services, web) is the standard shape for Solana protocols at our size. It keeps the onchain types, the SDK encoders and the tests in one PR, which is what catches ABI drift. Everything planned fits the existing layout:

| Path | Contents | Language |
| --- | --- | --- |
| `programs/matcher` | LP-scoped quote program (Pinocchio) | Rust |
| `programs/lp-vault` | Maker and backing share classes (Anchor) | Rust |
| `programs/market-registry` | Listing, `marketauth`, DEX-pool oracle adapter (Anchor) | Rust |
| `crates/moka-math`, `crates/moka-types` | Shared fixed-point maths, units, account layouts, golden vectors | Rust |
| `tests/` | LiteSVM/Mollusk suites and mainnet fixtures | Rust |
| `vendor/` | Pinned upstream clones (not committed; fetched by script) | — |
| `packages/sdk` | Typed TS SDK | TypeScript |
| `services/keeper`, `services/indexer`, `apps/api` | Off-chain services (non-authoritative) | TypeScript |
| `apps/web` | Trader/LP app | TypeScript/React |
| `sim/` | Economic simulator (Track B) | Rust or Python |
| `ops/` | Surfpool configs, manifests, runbooks | — |

Split a repository out only for a concrete reason: a separate team with its own release cadence (e.g. the web app), or a public SDK that external integrators depend on. Revisit at Phase 9.

## 2. One program or several?

Technically a single program could do everything. Solana forbids re-entrancy (A → wrapper → A), but trades are sent by traders straight to the wrapper, and our own calls into the wrapper (deposits, backing, mark pushes) never trigger the matcher. We still recommend **three programs in one workspace, sharing crates**:

| Reason | Detail |
| --- | --- |
| Blast radius | A bug or bad upgrade in listing code cannot touch LP funds, and vice versa |
| Hot-path size | The matcher runs on every trade. Keeping it tiny and framework-free (Pinocchio) keeps its CU and audit surface minimal |
| Audit scoping | Auditors can price and review each program separately; the matcher can be frozen (made immutable) early |
| Upgrade policy | Different programs can move to immutable/multisig at different times |

Framework choice: **Anchor 1.0** for `lp-vault` and `market-registry` (account validation macros and generated IDL for the SDK) and **Pinocchio** for `matcher` (the upstream fixture is already raw `solana-program`, and the hot path benefits). Shared types live in `crates/`, never duplicated.

## 3. Progress tracking: GitHub Issues + Milestones + one Project board

- **Milestones** = phases in `docs/execution-plan.md` ("Phase 1 — Repo tooling and CI", …).
- **Issues** = one deliverable each, with a template containing: goal, spec requirement IDs, acceptance checklist, curriculum module, links.
- **Labels:** `phase:N`, `area:{programs,sdk,keeper,web,docs,ops,learning}`, `type:{feature,test,security,research}`, `blocked`.
- **Project board** (GitHub Projects): Backlog → Ready → In progress → Review → Done, grouped by milestone.
- **PRs:** small; must reference the issue and spec IDs; CI green; the PR description lists what the reviewer should verify.
- **Session handoff:** `docs/HANDOFF.md` is updated at the end of every working session (human or agent) with state, decisions and next step.

The initial issue backlog is drafted in `docs/roadmap/issue-backlog.md`. Create the issues after the planning PR is merged.

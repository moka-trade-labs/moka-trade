## Issue

Closes #

## Spec IDs

<!-- Requirement IDs from docs/spec.md (e.g. EXE-01, SEC-10). Required for protocol and security-sensitive changes. -->

## What changed

## How to verify

<!-- Commands a reviewer can run; see docs/development.md. -->

- [ ] `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

## Checklist

- [ ] Checked integer arithmetic only; no floats in financial paths (CAP-07)
- [ ] Only SEC-10 allowlisted wrapper routes are called
- [ ] Owners, signers, PDAs, mints and program IDs validated where accounts are read
- [ ] No keypairs, RPC secrets or `.env` values committed
- [ ] `docs/HANDOFF.md` updated if state or decisions changed

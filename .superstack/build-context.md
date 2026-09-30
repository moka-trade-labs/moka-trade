# Build Context

## Stack

| Field | Value |
| --- | --- |
| Updated | 2026-09-13 |
| Status | Requirements and execution plan only |
| Chain | Solana |
| Architecture | Pinned Percolator wrapper/engine + LP-scoped IOC matcher + separate domain-backing controller; isolated pool-as-maker |
| Toolchain / upstream pair | MVP pin: wrapper `5cb331dde354517c6371a8acf92cecb194f3bb73` + declared engine `4db11a8cb0053815e23a35d3a7d3edc265d8d866`; Agave `v3.0.10` `cargo-build-sbf` with platform-tools `v1.52`; see `docs/architecture/p1-upstream-reproduction.md` |
| Client / services | Typed SDK, non-authoritative API/indexer, permissionless keepers, trader/LP web UI; specific versions not selected |

## Build Status

| Field | Value |
| --- | --- |
| Perp MVP complete | No |
| Perp tests passing | Not implemented |
| Devnet perp deployed | No deployment verified |
| Mainnet perp deployed | No deployment verified |
| Program ID | No production program selected; local counter identity is not a deployment claim |
| Security review | None for the proposed implementation |
| Ready for mainnet | No |

```json
{
  "defi": {
    "protocol_type": "custom",
    "program_id": null,
    "security_review": "none",
    "oracle_integration": "unselected; approved adapter and specific market feed required",
    "emergency_pause": false,
    "emergency_pause_requirement": "required; NORMAL/GUARDED/REDUCE_ONLY/PRICE_UNCERTAIN/RECOVERY behavior specified but not implemented"
  }
}
```

## Handoff

Follow [execution-plan.md](../docs/execution-plan.md) and [spec.md](../docs/spec.md). The initial acceptance target is a public devnet MVP with test funds. A real-money pilot requires a new explicit authorization and independent security/economic/operational review. No installed skill, dependency, deployed program or completed test is implied by this handoff.

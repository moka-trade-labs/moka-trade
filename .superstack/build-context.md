# Build Context

## Stack

| Field | Value |
| --- | --- |
| Updated | 2026-09-13 |
| Status | Requirements and execution plan only |
| Chain | Solana |
| Architecture | Pinned Percolator wrapper/engine + LP-scoped IOC matcher + separate domain-backing controller; isolated pool-as-maker |
| Toolchain / upstream pair | P1 candidates: engine `8eb7142aada316f6c476f5c4fa815d3a806706d5`, wrapper `2b1d025c004f92d3f89bac00113be90a0cbbcf63`, wrapper build tools `v1.52`; not release pins until reproduced and integration-tested |
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

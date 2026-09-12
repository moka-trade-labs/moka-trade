# Idea Context

## Chosen Idea

| Field | Value |
| --- | --- |
| Name | Moka Trade |
| Slug | moka-trade |
| One-liner | Solana pool-funded perps for underserved tokens that pass spot/oracle and underwriting gates |
| Updated | 2026-09-13 |
| Why crypto | Public custody, auditable capital and permissionless transaction submission are core requirements |
| Initial architecture | Pinned Percolator wrapper/engine, LP-scoped IOC matcher, isolated pool maker, and separate domain-backing controller; CLOB and competitive makers deferred |

## Validation

```json
{
  "go_no_go": "pivot",
  "verdict_detail": "Go validate a narrower spot-liquid cohort; no production build or launch approval.",
  "confidence": 0.65,
  "demand_signals": [
    {"type": "weak", "evidence": "User-provided ADL anecdote; no verified customer commitment."},
    {"type": "adjacent_product", "evidence": "Primary docs describe derp.trade, Perk, Wasabi and Omnipair overlap; not evidence of demand for this product."}
  ],
  "risks": [
    {"category": "technical", "description": "Thin reference liquidity may permit profitable extraction from pool and backstop.", "severity": "critical"},
    {"category": "economic", "description": "No proven underwriting terms; dynamic activation may not beat static backing.", "severity": "high"},
    {"category": "market", "description": "Direct competitors and leverage substitutes; distribution unvalidated.", "severity": "high"},
    {"category": "integration", "description": "Engine-wrapper pairing and all novel custody/accounting paths unverified.", "severity": "high"},
    {"category": "regulatory", "description": "Operating and distribution obligations require jurisdiction-specific professional review before mainnet.", "severity": "high"}
  ],
  "next_steps": [
    "Screen 10–20 candidate tokens using actual spot venues, depth and oracle provenance.",
    "Interview target traders and obtain non-binding underwriting terms.",
    "Run equal-capital, equal-subsidy static-versus-dynamic stress tests.",
    "Prove native integration and close specification blockers before public devnet acceptance."
  ]
}
```

The pivot is a narrower asset cohort and pool-first execution, not an approved change to spot leverage. No interviews, audited integration, economic simulations or deployed perp MVP are claimed complete. The Foundation pricing preference is not met by calling the pool onchain.

## Source Reports

- [Refined idea](../docs/dynamic-jit-liquidity-perp.md)
- [Full validation review](../docs/idea-validation-review.md)
- [Execution plan](../docs/execution-plan.md)
- [Production requirements](../docs/spec.md)

# Maintaining our own Percolator fork: what, why, when, how

- Date: 2026-09-30
- Status: explainer and decision framework. **No fork exists yet.** The MVP uses upstream at the pinned pair unmodified (spec SEC-10). The fork decision is Phase 14.
- Learning track: curriculum module P2 ("Operating a fork").

## 1. What "maintaining a fork" means

Upstream is `aeyakovenko/percolator` (engine) and `aeyakovenko/percolator-prog` (wrapper). Today we *pin* a commit: we build exactly upstream's code. A **maintained fork** means we keep our own copies of those repositories, deploy **our** build, and take responsibility for three things:

1. **Selecting** which upstream changes to adopt, and when.
2. **Adding** our own patches (e.g. fixing an open finding upstream hasn't merged, or removing routes we never use).
3. **Proving** each release: building it reproducibly, running the full test suite and recording every known failure.

It is not a one-time copy. It is a small ongoing product with its own release process.

## 2. The common pattern, and the common failure

A typical maintained Percolator fork looks like this:

- engine and wrapper forks built together (the wrapper depends on the engine by local path, often with a feature flag for fork-specific APIs);
- upstream changes adopted one at a time, each in a commit that names the upstream SHA it adopts;
- per-release deploy branches or tags, plus a written audit-scope document that states which commits are actually deployed.

The typical failure mode is **drift**: deploy branches accumulate commits that never return to `main`; `main` stops being an ancestor of what is deployed; and older generations keep running on some cluster. Then nobody can say with certainty which code is live. **A fork without discipline drifts until nobody knows what is actually deployed.** The process in §5 exists to prevent this.

## 3. Pros and cons for us

| Pros | Cons |
| --- | --- |
| Fix open upstream findings on our routes without waiting | We own every bug we introduce; upstream proofs and tests may no longer apply to our code |
| Delete unused routes (batch, EWMA, live insurance withdrawal…) to shrink the attack and audit surface | Every upstream sync becomes merge work in a 17k-line wrapper and 21k-line engine that move daily |
| Trim the trade path for compute (performance L5) | Needs strong Rust, Solana and formal-methods skills (curriculum P2, S6) |
| Stable, audited release cadence independent of upstream's research loop | Audit cost is on us; "Percolator" guarantees become "Percolator-plus-our-patches" |

## 4. When to fork (decision criteria for Phase 14)

Fork only if at least one of these is true at audit time:

1. An open upstream finding on an **allowlisted** route has no upstream fix and blocks mainnet.
2. The auditor requires removing unused routes from the deployed binary.
3. Upstream becomes inactive or changes license or direction.
4. Measured compute of the trade path blocks a product goal (performance L5).

Otherwise, keep pinning upstream and bump deliberately.

## 5. How we would run it (if chosen)

- **Repos:** `moka-trade-labs/percolator` and `moka-trade-labs/percolator-prog` as GitHub forks, so upstream history is preserved. This monorepo's `vendor/` script fetches our fork at a pinned tag.
- **Branches:** `upstream/main` (a mirror, never edited); `moka/main` (upstream + our patches, always releasable); `release/vX.Y` tags built by CI; no deploy-only branches.
- **Patches:** each patch is one commit with a `MOKA-PATCH-NNN` ID listed in `PATCHES.md` (reason, spec ID, upstream PR/finding link, test that proves it). The goal is zero patches: upstream every fix we can.
- **Sync ritual (weekly or per release):**
  1. Fetch upstream; read the new `open_findings.tsv` and `invariant_status.tsv` diffs.
  2. Rebase or merge `moka/main` onto the chosen upstream commit.
  3. Run the full upstream suite plus our suite; compare against the previous release's known-failure list.
  4. Tag a release candidate; record hashes in the P1 record and the deployment manifest.
- **Release proof:** reproducible build (verified-build tooling), a byte-hash match between CI artifact and deployed program, and an audit diff limited to our patches plus the upstream delta.

## 6. What you need to learn first

Curriculum modules R1–R4 (Rust), S1–S6 (Solana), P1–P2 (Percolator internals and fork operations) and T1–T3 (testing and formal methods). Practice project: fork a small open-source Solana program, add one patch, and run the sync ritual against its upstream twice before touching Percolator.

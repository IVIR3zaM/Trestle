# PLAN — converge this fork onto the upstream template

**Goal.** Make this repository indistinguishable from a fresh clone of the
upstream template that was fully onboarded and kept current, so that future
upstream releases apply cleanly and improvements born here can flow back.

**Upstream reference:** the sibling checkout, which must read the pinned version
at every iteration (preflight check).

## Hard rules

1. The working tree must stay usable at the end of every iteration. Never leave
   it half-migrated.
2. Anything queued to flow upstream is generalised first — no local specifics.
3. ~~Contributions are batched and sent at the end of the effort.~~
   **SUPERSEDED — see rule 6.** Batching lost the reasoning behind each change by
   the time it was written up. Left here as the audit trail; do not follow it.
4. Never add the upstream as a permanent remote.
5. Do not stall on a decision that is the maintainer's to make — record it as
   `blocked(user)` and take the next item.
6. Contribute one entry at a time, at the moment it lands, while the reasoning is
   still fresh. Replaces rule 3.

## Done when

The version files match upstream, every local-only improvement is either
contributed or explicitly deferred with a reason, and a dry-run of the upstream
update flow completes with no manual intervention.

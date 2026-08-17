# Journal — converge-onto-upstream-template

Append-only. Newest first. One entry per iteration, each addressable by its number so
a unit's `note` can point at it.

---

## Entry 12 — 2026-03-04 · unit 1.2 · in_progress

**Did:** applied six more upstream-ahead changes; the config loader now matches
upstream exactly.

**Verified:** `npm test` → 214 passing. `npm run build` → clean.

**Learned:** three of the remaining six touch the build script, which is unit 1.4 —
they cannot be applied until that question is answered. Reordering: 1.3 can proceed
independently, so the next iteration should take that instead of continuing 1.2.

**Next:** 1.3, not 1.2. (A guess — re-check the repo first.)

**Blocked:** none new; 1.4 still waiting on the maintainer.

**Commit:** `a3f9c21`

---

## Entry 11 — 2026-03-03 · unit 1.4 · blocked

**Did:** attempted to retire the local build script. Stopped before changing
anything.

**Verified:** not applicable — no change made.

**Learned:** the local script does a signing step that upstream's has no equivalent
for. Dropping it silently would break releases; contributing it is a design question
about whether upstream wants signing at all. **This is not mine to decide** — recorded
as blocked and moved on rather than guessing.

**Next:** 1.2, continue applying upstream-ahead changes.

**Blocked:** drop the signing step, or contribute it upstream?

**Commit:** none (no change)

---

## Entry 10 — 2026-03-03 · unit 1.2 · in_progress

**Did:** applied twelve upstream-ahead changes, mechanical.

**Verified:** `npm test` → 214 passing.

**Learned:** nothing.

**Next:** continue 1.2.

**Blocked:** none.

**Commit:** `7c1e4b0`

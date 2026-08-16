# LOG — iteration journal (append-only)

Newest at the top. One entry per iteration.

---

## 2026-03-04 · iteration 12 · 1.2 — in-progress

Did: applied six more upstream-ahead changes; the config loader now matches
upstream exactly.

Verified: `npm test` → 214 passing. `npm run build` → clean.

Learned: three of the remaining six touch the build script, which is item 1.4 —
they cannot be applied until that question is answered. Reordering: 1.3 can
proceed independently, so the next iteration should take that instead of
continuing 1.2.

Next: 1.3, not 1.2. (A guess — re-check the repo first.)

Blocked: none new; 1.4 still waiting on the maintainer.

Commit: `a3f9c21`

---

## 2026-03-03 · iteration 11 · 1.4 — blocked(user)

Did: attempted to retire the local build script. Stopped before changing
anything.

Verified: n/a — no change made.

Learned: the local script does a signing step that upstream's has no equivalent
for. Dropping it silently would break releases; contributing it is a design
question about whether upstream wants signing at all. **This is not mine to
decide** — recorded in STATE and moved on rather than guessing.

Next: 1.2, continue applying upstream-ahead changes.

Blocked: drop the signing step, or contribute it upstream?

Commit: none (no change)

---

## 2026-03-03 · iteration 10 · 1.2 — in-progress

Did: applied twelve upstream-ahead changes, mechanical.

Verified: `npm test` → 214 passing.

Learned: nothing.

Next: continue 1.2.

Blocked: none.

Commit: `7c1e4b0`

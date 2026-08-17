# Journal — extract-and-migrate

Append-only. Newest first.

---

## Entry 2 — 2026-04-11 · unit H02.2 · in_progress

**Did:** pointed the export caller at the extracted module; three call sites left.

**Verified:** `make test` → green. `make test-reporting` → green.

**Learned:** the admin dashboard caller (H02.3) was retired last quarter, so that
queue item does not apply. Marked `n_a` rather than `done`, since nothing was
migrated.

**Next:** finish H02.2's remaining call sites.

**Blocked:** none.

**Commit:** `4d2b8ef`

---

## Entry 1 — 2026-04-10 · unit H02.1 · done

**Did:** migrated the billing caller.

**Verified:** `make test` → green.

**Learned:** nothing.

**Next:** H02.2.

**Blocked:** none.

**Commit:** `1a7c093`

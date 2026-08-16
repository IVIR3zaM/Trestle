# STATE — work queue

**How the loop uses this file:** each iteration takes the **first item whose
status is `todo`**, executes it, updates the row, and appends to `LOG.md`.
Oversized items may stay `in-progress` across iterations with notes.

**Statuses:** `todo` · `in-progress` · `blocked(user): <question>` · `done` · `n/a`

## Phase 0 — scaffold

| ID | Item | Status | Notes |
|---|---|---|---|
| 0.1 | Adopt upstream directory layout | done | |
| 0.2 | Adopt upstream config format | done | two local keys kept; see LOG 2 |

## Phase 1 — reconcile divergence

| ID | Item | Status | Notes |
|---|---|---|---|
| 1.1 | Classify every diff: upstream-ahead / local-ahead / local-only | done | 31 files |
| 1.2 | Apply upstream-ahead changes | in-progress | 18 of 24 applied |
| 1.3 | Generalise local-ahead changes for upstream | todo | |
| 1.4 | Retire the local build script in favour of upstream's | blocked(user): the local script has a signing step upstream has no equivalent for — drop it, or contribute it? | |

## Phase 2 — close out

| ID | Item | Status | Notes |
|---|---|---|---|
| 2.1 | Dry-run the upstream update flow | todo | |
| 2.2 | Retire this effort's tooling | todo | |

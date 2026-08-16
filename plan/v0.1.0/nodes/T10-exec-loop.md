---
id: T10
title: Executor — loop shape
tier: standard
deps: [T09, T12]
---

## Goal

Drive a loop-shaped plan: orient against current reality, pick the next item, do
it, verify, journal, stop.

Blocked on **D5** — if Trestle emits rather than orchestrates, this node ships a
prompt and a journal validator instead of a driver, and shrinks considerably.

## Requirements

Model it on `fixtures/source/loop-shape/`, which is a worked example of a loop
that ran to completion. `docs/PRIOR-SHAPES.md` explains why each artifact exists:

- Queue with `todo` / `in-progress` / `blocked(user): <question>` / `done` / `n/a`.
- **Append-only journal with a fixed entry format** — what was done, what was
  verified and how, what was *learned*, what's next, what's blocked. The
  `Learned:` line is the only channel by which discovery reaches the next
  iteration; the format must make omitting it awkward.
- Re-orient from the repo at the start of each iteration, not from the journal
  alone. **Where journal and repo disagree, the repo wins** and the discrepancy
  is recorded.
- One item per iteration, then stop.
- Revert on unrecoverable failure rather than leaving a half-applied change — the
  next iteration starts cold and cannot tell deliberate from abandoned.

## Acceptance

- `npm run test:exec-loop` with the mock harness: picks the first `todo`;
  `blocked(user)` items are skipped, not attempted; a failed verification reverts
  and journals; a journal entry missing `Learned:` is rejected.
- Replaying the loop fixture's queue produces a sane iteration order, and its
  `blocked(user)` item is skipped rather than attempted.

## Out of scope

Graph shape (T11), dashboard (T13).

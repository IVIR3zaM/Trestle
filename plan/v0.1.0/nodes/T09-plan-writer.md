---
id: T09
title: Plan writer (atomic, validating, non-clobbering, drafts)
tier: standard
deps: [T07, T08, T27]
---

## Goal

`trestle plan write` — take a plan the agent synthesised, prove it valid, and land
it in the repo as plain files.

Under `D5` this is the **write barrier**. It is the only way a plan enters the
repo, so every guarantee about plan quality is enforced here or nowhere.

## Requirements

- **Standard location:** `.trestle/plans/<name>/` — index, one file per unit,
  decisions, and (for loop shapes) an empty journal ready to append to.
- **Plain files, in git, reviewable in a PR.** The plan is a document the team can
  argue with, not an opaque artifact.
- **Validation before writing, always.** Runs the full T07 gauntlet. A plan that
  fails is rejected with errors naming the offending units — and nothing is
  written, not even partially. There is no `--force` and no `--skip-validation`: a
  plan that can't pass the gauntlet is not a plan Trestle should be blamed for.
- **Atomic.** Write to a temp directory, then rename into place. A killed process
  must leave either the old plan or the new one, never a half-written mixture.
- **Never overwrite an existing plan without `--replace`**, and **never** touch a
  plan that has progress recorded against it — not even with `--replace`. Amending
  such a plan is T25's job, and pointing at it in the error message is the whole
  reason T25 exists.
- **Drafts are the default path** (`D13`). `--draft` writes the plan in the `draft`
  state, **starts the dashboard if it isn't running, and returns a deep link to that
  draft** — which the agent hands to the user in chat. The user goes from answering
  questions to looking at a rendered plan with nothing typed in between.
  - A draft is on disk and in git like any plan, so it is reviewable in a PR too.
  - A draft is never executable: `trestle next` returns nothing for it and says why.
  - Approval flips the state and nothing else. **There is no approval endpoint in
    the UI** (`D13`) — the user says the word in their chat, and the plan they
    approve is byte-identical to the one they looked at.
- **Print next steps in terms of the user's own interface**, not Trestle's. After a
  successful write the user should be told the draft URL, what to say in their editor
  to approve, and where status appears. The previous version of this node emitted an
  executor prompt here; that moved to T04, because the prompt belongs to the
  integration rather than to the plan.
- Reads `trestle conventions` (T08) and `trestle standards` (T27) so ingested
  preconditions land on the units they match, **each carrying the rule id and
  citation it came from** (T02), rather than being reapplied at verify time.

## Acceptance

- `cargo test -p trestle-plan --test writer` — round-trips through the T02 parser;
  an invalid plan writes nothing at all (asserted on the filesystem, not just by
  exit code); refuses a plan with recorded progress and names T25 in the message;
  `--replace` works on a plan with no progress and is refused on one with.
- Writing a plan into a dirty working tree is safe and reversible: `git status`
  shows only `.trestle/`.
- Crash test: kill mid-write, assert the previous plan is intact and readable.
- Every ingested precondition from a T08 or T27 fixture appears on the units it
  matches and on no others, carrying its rule id and citation.
- `--draft` produces a plan in the `draft` state with a resolvable deep link;
  `trestle next` refuses to select from a draft and says why; approving it changes
  the state and leaves every unit file byte-identical, so what was reviewed is what
  runs.

## Out of scope

Amending (T25). Running it (T10/T11). Emitting the integration files (T04, T23).

---
id: T12
title: Status + journal store
tier: standard
deps: [T02]
---

## Goal

How progress is recorded and read, for both shapes. The dashboard, `trestle next`,
`trestle verify` and the CLI all depend on this, so it ships before any of them.

## Requirements

- **Separable from the plan definition.** Execution must not rewrite plan bodies —
  otherwise a diff can't distinguish "the plan changed" from "work happened". T25
  depends on this separation being clean: amending a plan should require no status
  rewriting at all.
- Readable without parsing prose. The dashboard polls it; it must be cheap.
- Covers both shapes: unit status + timestamps for graphs, queue position + journal
  offset for loops.
- **The full state set from T02**: `draft`, `verified`, `done`, `done(overridden)`
  (T11) and `superseded` (T25). An overridden unit is never conflated with a plain
  `done` — `trestle status` counts them separately and always shows the override
  count, even when it is zero, so the absence of overrides is itself visible.
- **Only `trestle verify` may write `done`.** This node provides the store; the
  restriction is a property of the API it exposes. There is no public setter for
  the `done` state, and T11's acceptance asserts exactly one writer exists in the
  workspace. Design the store so that assertion is easy to make true.
- Records **who ran what and when** — and under `D14` this stops being a
  future-proofing gesture and becomes load-bearing. Every state transition records
  the **role** (`planner` / `implementer` / `verifier`) and the **harness id** that
  produced it, because that is what the dashboard's multi-agent view renders and
  what makes a review verdict attributable. Where the harness cannot name a session,
  record `unknown` rather than inventing an identity — but the role is always known,
  since it comes from `.trestle/config.toml`.
- **Review verdicts are recorded, pass and fail alike**, with their reason. A unit
  that bounced twice before clearing review is a fact worth keeping; overwriting it
  with a final `done` destroys the only evidence that the reviewer is doing anything.
- **Records nothing it cannot observe.** Token usage is `unknown` in v0.1.0
  (`D11`). The field exists; it is never filled with an estimate.
- Crash-safe: a killed process must not leave the file unreadable. Write-rename,
  not in-place mutation.
- Concurrent readers are always safe. Concurrent writers are out of scope for
  v0.1.0 but must not be designed out — and note that two editor windows on one
  repo means two MCP servers (T24), so concurrent *readers* is the normal case, not
  the exotic one.

## The journal

The loop's journal lives here, because it is state rather than definition.

- **Append-only, fixed entry format**: what was done, what was verified and how,
  what was *learned*, what's next, what's blocked.
- `trestle journal append` validates the entry and **rejects one missing
  `Learned:`**. That line is the only channel by which discovery reaches the next
  iteration (`docs/PRIOR-SHAPES.md`), and the format must make omitting it awkward
  — under `D5` a validator is the only thing that can make it awkward, since
  Trestle is not the one writing the entry.
- Never rewrites or reorders existing entries. A correction is a new entry that
  references the old one.

## Acceptance

- `cargo test -p trestle-status` — round-trips both shapes; a truncated file is
  detected rather than silently misread; a simulated crash mid-write leaves the
  previous state intact.
- Reading status never requires loading the full plan.
- `done(overridden)` never reads back as `done`; the override reason and timestamp
  survive a round trip.
- A journal entry missing `Learned:` is rejected with a message saying why;
  existing entries are byte-identical after any append.
- Ten concurrent readers during a write never observe a partial file.

## Out of scope

Displaying it (T13/T14). Running oracles (T11). Estimating spend (T20).

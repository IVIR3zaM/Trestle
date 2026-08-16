---
id: T12
title: Status format + reader
tier: standard
deps: [T02]
---

## Goal

How progress is recorded and read, for both shapes. The dashboard, both
executors, and the CLI all depend on this, so it ships before any of them.

## Requirements

- **Separable from the plan definition.** Execution must not rewrite plan bodies —
  otherwise a diff can't distinguish "the plan changed" from "work happened".
- Readable without parsing prose. The dashboard polls it; it must be cheap.
- Covers both shapes: node status + timestamps for graphs, queue position +
  journal offset for loops.
- Records **who ran what and when**, so a multi-agent future (and the dashboard's
  parallel-work view) needs no format change.
- Crash-safe: a killed process must not leave the file unreadable. Write-rename,
  not in-place mutation.
- Concurrent readers are always safe; concurrent writers are out of scope for
  v0.1.0 but must not be designed out.

## Acceptance

- `npm run test:status` — round-trips both shapes; a truncated file is detected
  rather than silently misread; a simulated crash mid-write leaves the previous
  state intact.
- Reading status never requires loading the full plan.

## Out of scope

Displaying it (T13/T14).

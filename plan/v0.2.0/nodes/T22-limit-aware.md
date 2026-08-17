---
id: T22
title: Limit-aware backoff and resume
tier: standard
deps: [T20, T21]
---

## Goal

Survive rate limits and usage caps without losing work or hammering a wall.

## The observed behaviour to design against

Unattended overnight runs of graph-shaped plans have been observed to survive
interruption for exactly one reason: **plan state lives on disk and readiness is
recomputed at each firing.** Nothing clever is involved, and nothing clever
should be added. A limit-aware scheduler is that property plus knowing when not
to bother trying — see `docs/PRIOR-SHAPES.md`.

## Requirements

- **Detect** limit conditions from the harness's exit code and output, mapped to
  the T04 failure taxonomy. Distinguish *rate-limited* (retry later) from *quota
  exhausted* (retry much later) from *not authenticated* (never retry — tell the
  user).
- **Parse the reset time when the harness gives one**; otherwise back off
  exponentially with jitter, capped. Never poll a limit tighter than the interval
  the schedule already uses — that spends nothing but produces failures.
- **Never mark a unit done when a firing is cut short.** Leave it `todo`, record
  the interruption in status. A half-finished unit marked done is the single
  worst outcome available.
- **Optional budget ceiling** (from T20): stop arming further firings once
  recorded spend crosses a user-set threshold, and say so plainly rather than
  failing quietly.
- Tier-aware degradation is **off by default**. Silently dropping a `deep` unit to
  a cheap model to fit under a limit trades correctness for cost without asking —
  offer it as an explicit opt-in, never a default.
- Record every limit event in status so the dashboard can show "waiting until
  HH:MM" rather than an unexplained stall.

## Acceptance

- `npm run test:limits` — each limit class maps to the correct wait; a firing
  killed mid-unit leaves it `todo` and the next selects it; the budget ceiling
  halts arming; tier degradation does not occur unless explicitly enabled.
- A simulated night of limited firings completes the same units, in the same
  order, as an unlimited night — just slower.

## Out of scope

Predicting limits before they happen.

---
id: T17
title: CLI + first two harness adapters
tier: standard
deps: [T04, T09]
---

## Goal

The thing a user actually runs, plus real adapters behind the T04 contract.

Blocked on **D1** (which harnesses) and **D6** (language/runtime).

## Requirements

Commands: `trestle plan` (survey → questions → proposal → approve → write),
`trestle status`, `trestle ui`, and — depending on D5 — `trestle run`.

- **The question flow is the product's first impression.** Batched, each question
  showing why it's asked, what it blocks, options with tradeoffs, and a
  recommendation. Answerable non-interactively from a file for CI use.
- The proposal is reviewable before anything is written to disk. Nothing touches
  the repo until the user approves.
- `detect()` finds the installed harness; if several, ask; if none, say exactly
  what to install.
- Errors are actionable: "not authenticated — run `X`", never "request failed".
- Works from `npx` with no install (if D6 lands on Node).

Two adapters, so the abstraction is tested rather than assumed.

## Acceptance

- `npm run test:cli` — full flow against the mock harness; both real adapters pass
  the T04 conformance suite; a cancelled approval leaves the repo untouched
  (`git status` clean).
- A new user can go from install to written plan without reading the docs.

## Out of scope

Dashboard (T13-T15).

---
id: T06
title: Ambiguity detection + question generation
tier: deep
deps: [T04, T05]
---

## Goal

Compare the user's stated goal against the survey and find what is genuinely
undecided — then resolve what can be resolved by reading code, and ask about the
rest.

## The distinction that matters

Two kinds of ambiguity, handled differently:

- **Answerable from the code.** "Is there already a storage abstraction?" — go
  look. Never ask the user something the repo already answers; that is the
  fastest way to make the tool feel dumb.
- **Requires the user.** Product behaviour, tradeoffs with no objective winner,
  anything irreversible, anything touching cost or security posture.

Getting this split wrong in either direction is the failure mode: asking too much
is annoying, asking too little produces confidently wrong plans.

## Requirements

- Questions are **batched**, not drip-fed one at a time.
- Every question carries: why it's being asked, what it blocks, the options with
  tradeoffs, and a recommendation. Copy the shape of `decisions.md` in this repo.
- Questions map to plan units, so an unanswered one blocks exactly the right work
  instead of the whole plan.
- Uses the T04 mock in tests — no real harness calls in CI.

## Acceptance

- `npm run test:ambiguity` — fixtures where a question is answerable from code
  (must not be asked) and where it is not (must be asked, with options and a
  recommendation).
- No question is generated without a stated blast radius.

## Out of scope

Asking them interactively (T17), turning answers into a plan (T07).

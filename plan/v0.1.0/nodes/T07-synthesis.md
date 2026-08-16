---
id: T07
title: Plan synthesis
tier: deep
deps: [T02, T03, T06]
---

## Goal

Turn survey + goal + answers + chosen shape into a valid plan in the T02 format.

## Requirements

- **Every unit gets an oracle**, drawn from the commands the survey actually
  found. A unit with no runnable check must become a human gate — never a unit
  with a hand-waved acceptance line. This rule is the difference between a plan
  that can be executed unattended and one that only looks like it.
- **Units are contracts, not tasks.** "The existing suites pass unmodified against
  the new store" survives contact with reality; "implement the store" does not.
- Dependency edges are derived from the code graph where possible, not invented.
- Unresolved questions become blocking decisions attached to specific units.
- Human gates on: product judgement, irreversible actions, and anything the
  rubric flagged as low-confidence.
- When the rubric says "both" (T03), synthesise **both plans** plus the tradeoff
  comparison.

## Acceptance

- `npm run test:synthesis` — output validates against the T02 schema for all
  three shapes; every unit has an oracle or a gate; no unit depends on one that
  doesn't exist; a cyclic result is impossible (assert it).
- **Regression corpus**: each entry pairs a fixture repo and a goal with the
  plan a human wrote for it. Start with the two in `fixtures/source/` — both were
  written by hand before Trestle existed, so they are known-good answers to
  compare against. Synthesis is judged on whether it finds the same first unit
  and the same load-bearing dependency, not on matching text.

## Out of scope

Writing to disk (T09), rendering (T14).

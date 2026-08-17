---
id: T07
title: Synthesis prompt + plan validation gauntlet
tier: deep
deps: [T02, T03, T06, T19]
---

## Goal

The agent writes the plan. Trestle makes a bad one impossible to land.

Two deliverables that are two halves of one mechanism: the **prompt** that tells
the agent how to synthesise survey + goal + answers + shape into a plan, and the
**gauntlet** — a validator strict enough that the agent iterates against it
instead of shipping something plausible-looking.

## Why the gauntlet is the load-bearing half

Under `D5` Trestle cannot inspect the agent's reasoning. It can only inspect the
artifact. So every rule that used to live in synthesis code becomes a check that
runs on the written plan, and **a rule with no check is not a rule** — it is a
sentence in a prompt that a tired model will skip.

The rules, each as a check:

| Rule | Check |
|---|---|
| **Every unit gets an oracle** | reject any unit with neither `oracle` nor `gate: human` |
| Oracles are real | every oracle command must appear in the T05 survey's discovered-commands set, or be flagged `unverified` and counted in the report |
| No invented dependencies | every `deps` entry must name an existing unit |
| No cycles | topological sort must succeed |
| Units are contracts, not tasks | reject unit titles matching imperative-task patterns (`implement …`, `add …`, `fix …`) with no `done_when` clause |
| Unresolved questions block specific units | every open decision must name at least one unit, and those units must be `blocked` |
| Human gates where required | product judgement, irreversible actions, and anything the rubric flagged low-confidence must carry `gate: human` |
| Tiers are abstract | reject any vendor model name in the plan (T19) |

The imperative-title check is the one worth arguing about, and it is worth having
anyway: *"the existing suites pass unmodified against the new store"* survives
contact with reality; *"implement the store"* does not, and the difference is
detectable in the text.

**A unit with no runnable check must become a human gate** — never a unit with a
hand-waved acceptance line. That rule is the difference between a plan that can be
executed unattended and one that only looks like it, and it is check #1 above.

## The prompt

`templates/synthesize.md`, shipped through T04. It must:

- require `trestle survey --json`, `trestle conventions --json` and
  `trestle shape --json` to be read first, and require the plan to state where it
  **disagrees with the deterministic shape baseline** and why (T03)
- require dependency edges to be derived from the survey's module graph where
  possible, not invented
- require `trestle plan validate` to pass before `trestle plan write` is called,
  and tell the agent that validation errors are the expected way to converge
- when the rubric says *both* (T03), require **both plans** plus the tradeoff
  comparison

## Acceptance

- `cargo test -p trestle-plan --test gauntlet` — each rule above has a fixture
  that violates it and is rejected with a message naming the offending unit and
  path; a valid plan of each of the three shapes passes; a cyclic plan is rejected
  (assert it, don't assume the sort catches it).
- **Recorded-transcript corpus.** Capture real agent output once, for at least
  three goals across the fixture repos, and commit it under
  `fixtures/transcripts/`. Assert that the gauntlet accepts what a good agent
  produced and rejects hand-mutated copies of it. This does not test the agent —
  it tests that the gauntlet is neither too loose to be useful nor so tight that
  real output can't pass, which is the failure mode that would make the tool
  unusable.
- **Regression corpus, as an eval not a unit test.** Each entry pairs a fixture
  repo and a goal with the plan a human wrote for it. Start with the two in
  `fixtures/source/` — both were written by hand before Trestle existed, so they
  are known-good answers. Scored on whether synthesis finds **the same first unit
  and the same load-bearing dependency**, not on matching text. Run by hand; the
  score is reported in T18, not asserted in CI, because it costs tokens and
  requires a live agent.

## Out of scope

Writing to disk (T09). Rendering (T14). The shape decision itself (T03).

---
id: T14
title: Dashboard — work view
tier: standard
deps: [T13]
---

## Goal

Show the plan: units, dependencies, what is running, what is done, what is
blocked and on what.

## Requirements

- **Graph shape**: a DAG rendered so the critical path and the parallelisable
  branches are visible at a glance. Status by colour *and* a second cue — never
  colour alone.
- **Loop shape**: the queue in order, with the journal readable inline. A loop
  view that pretends to be a graph is worse than an honest list.
- **Parallel work in progress** is the headline: when several units are running,
  the user should see that immediately without hunting.
- Blocked units link to the question blocking them, with the decision text.
- A unit's detail shows its oracle, its tier, and its last verification result.
- Where the rubric returned "both", show the two proposals side by side with the
  tradeoff table — this is the screen where the user chooses.
- No horizontal page scroll; wide graphs scroll inside their own container.

## Acceptance

- `npm run test:ui-work` — renders all three shapes from fixtures; blocked states
  reachable; the both-plans comparison renders; status conveyed by two cues.
- Legible at 1280px and on a laptop screen without zooming.

## Out of scope

Code view (T15). Any control.

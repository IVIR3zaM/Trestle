---
id: T18
title: Dogfood — plan a real repo end to end
tier: deep
gate: human
deps: [T10, T11, T14, T15, T16, T17]
---

## Goal

Point Trestle at real repositories and judge whether the plans are any good.
Human-gated: this node is a judgement call, and no oracle can make it.

## The test

Bring your own repositories. Pick three that match these profiles, each of which
has a **known-good human answer** you can check Trestle against — that is the
whole point, so do not pick work you haven't already thought through.

1. **A multi-week change with independent tracks** — for example adding a second
   deployment target to a service that has one. Expect **graph**. Check that it
   finds the shared-core extraction as an early unit, and that it asks about the
   things you know are genuinely undecided rather than guessing.
2. **A convergence or migration effort with unsettled requirements**, where you
   would iterate rather than plan. Expect **loop**. **If Trestle says graph here,
   the rubric is biased toward structure and that is a release blocker, not a
   curiosity.**
3. **A small, well-tested bugfix.** Expect **loop**, and a plan proportionate to
   the work. A tool that ceremonially graphs a two-hour task will be uninstalled
   after one use.

`fixtures/source/` contains worked examples of the first two profiles if you want
a reference for what a good answer looks like — but dogfooding on fixtures is not
dogfooding. Use real repositories.

## Acceptance

- `npm test` green.
- All three shape recommendations match the human answer, **or** the divergence is
  understood and written up.
- The written plans are ones you would actually follow — judged by executing at
  least one of them.
- The code view's blast radius matches what a human reviewer expects.
- No outbound connection during any of it (T16 running throughout).

## Out of scope

Publishing, packaging for a registry, a launch.

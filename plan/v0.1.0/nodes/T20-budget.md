---
id: T20
title: Budget estimation + token accounting
tier: standard
deps: [T02, T19]
---

## Goal

Tell the user what a plan will roughly cost **before** they run it, and what it
actually cost **after**. Tokens are the real currency of this work and running a
plan blind is the most common way people get surprised.

## Requirements

**Before** — a per-plan estimate from unit count, tier mix, and the file surface
each unit touches. Present it as a **range with the assumptions stated**, never a
single confident number: an honest ±50% beats a precise-looking lie. Break it
down by tier so the effect of the mix is visible.

**During/after** — record actual usage per unit in the status file (T12), where
the harness reports it. Not all do; where usage is unavailable, record
`unknown` rather than estimating and presenting it as measured.

**Levers the estimate should make visible:**
- moving units between tiers
- whether the plan is worth graphing at all — if the ceremony costs more than the
  work, that is a shape signal and should feed back into T03
- units whose surface is large enough that they should be split

**Hard rule:** Trestle never bills anything and never sees a bill. All spend is on
the user's own harness account. Estimation is arithmetic over token counts, and
any currency figure must be clearly labelled as the user's own configured rate,
not a Trestle price.

## Acceptance

- `npm run test:budget` — estimates are ranges with assumptions attached;
  unavailable usage records `unknown` and never a silent estimate; per-tier
  breakdown sums to the total.
- Estimating a plan makes no harness call — it is arithmetic, not inference.

## Out of scope

Enforcing a budget ceiling mid-run — that's the scheduler's job (T22).

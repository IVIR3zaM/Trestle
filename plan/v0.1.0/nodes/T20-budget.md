---
id: T20
title: Budget estimation (pre-run, estimate-only)
tier: standard
deps: [T02b, T19]
---

## Goal

Tell the user what a plan will roughly cost **before** they run it. Tokens are the
real currency of this work and running a plan blind is the most common way people
get surprised.

Blocked on **D11** — which also records what this node lost.

## What this node no longer does

The previous version promised actual usage per unit, recorded after the fact. Under
`D5` **Trestle never sees a request, a response, or a bill** — the agent calls
Trestle, not the other way round. Actual usage is therefore `unknown`, permanently,
in v0.1.0.

This is a real reduction and the copy must say so plainly rather than shipping a
column that is always empty and letting users assume it's a bug. `D11` records the
options for getting it back in v0.2.0 (reading harness-local session logs, behind a
`best-effort` label); none of them belong here.

## Requirements

- A per-plan estimate from unit count, tier mix, and the file surface each unit
  touches. Present it as a **range with the assumptions stated**, never a single
  confident number: an honest ±50% beats a precise-looking lie. Break it down by
  tier so the effect of the mix is visible.
- **State where the constants came from.** On a first run there is no history, so
  the per-tier token constants ship with the binary. They must carry a documented
  provenance — measured from what, when, on which harness — or be labelled a
  starting guess. An estimate whose basis is undocumented is a number with a
  costume on.
- **Record `unknown` for actual usage**, in a field that exists and is never filled
  with an estimate. `trestle status` says `unknown (not observable — see docs)`
  rather than showing a blank.
- Estimates are recomputed after an amend (T25), since the unit count changed.

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

- `cargo test -p trestle-budget` — estimates are ranges with assumptions attached;
  usage records `unknown` and never a silent estimate; per-tier breakdown sums to
  the total.
- Estimating a plan makes no call of any kind — it is arithmetic over the plan and
  the survey, and it runs with the network denied (T16).
- The shipped token constants have a provenance string, asserted present.
- On an integration where tiering is inert (T19), the per-tier breakdown still
  renders but carries the note that the mix will not change which model runs — the
  breakdown is then a statement about *work*, not about spend, and must not be
  presented as a lever it isn't.

## Out of scope

Observing actual usage (`D11`, deferred — see
[`../../v0.2.0/README.md`](../../v0.2.0/README.md)). Enforcing a budget ceiling
mid-run, which needs the unattended lane and is deferred with it.

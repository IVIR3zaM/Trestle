---
id: T19
title: Model roster + abstract tier resolution
tier: standard
deps: [T04]
---

## Goal

Let a plan say how much thinking a unit is worth **without naming a vendor's
model**, and have each harness resolve that to something it actually has.

## The mistake to avoid

`fixtures/source/graph-shape/plan.yaml` shows the right instinct — it declares
`tier: deep`, not a model name. The tempting shortcut is to write the vendor's
model directly, which works for exactly one harness: a plan carrying `opus` is
unrunnable on any other, and a portable planner cannot ship vendor names in its
own format.

Plans declare an **abstract tier**; the harness maps it:

| Tier | Means | Typical work |
|---|---|---|
| `cheap` | exact spec, small surface, fast oracle | config, env plumbing, mechanical edits |
| `standard` | real code against a strong oracle | the default for implementation |
| `deep` | contracts others build on; correctness no compiler checks | protocols, concurrency, cross-cutting design |

## Requirements

- `HarnessCapabilities` (T04) gains a roster: which models exist, which is the
  default, and the tier each maps to.
- A harness with one model maps all three tiers to it and **says so**, so the
  user knows tiering is inert rather than silently believing it worked.
- A harness without subagents can't vary model per unit mid-session — report that
  as a capability gap; don't pretend.
- Resolution is a pure function: `(tier, capabilities) → model | unsupported`.
- User override in config, per tier, per harness. Someone who wants everything on
  one model should get that without editing plans.

## Acceptance

- `npm run test:roster` — every tier resolves for a multi-model harness; a
  single-model harness resolves all three and reports the degradation; an
  unknown tier is rejected rather than defaulted.
- No vendor model name appears anywhere in the plan schema (assert by grep over
  `schema/`).

## Out of scope

Cost estimation (T20). Choosing which tier a unit gets — that's synthesis (T07).

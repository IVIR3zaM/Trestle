---
id: T19
title: Abstract tiers + per-integration mapping (advisory)
tier: standard
deps: [T04]
---

## Goal

Let a plan say how much thinking a unit is worth **without naming a vendor's
model**, and be honest about the fact that Trestle can no longer make that
happen.

## What changed

The previous version of this node specified a resolver: `(tier, capabilities) →
model`, with Trestle picking the model per unit. Under `D5` Trestle does not
invoke the agent, so **it cannot choose a model.** Tiers become an *advisory hint
in the plan* that the agent may honour, and the honest job of this node is to
make clear where they are real and where they are decoration.

This is a genuine loss and the documentation must not obscure it.

## The mistake still worth avoiding

`fixtures/source/graph-shape/plan.yaml` shows the right instinct — it declares
`tier: deep`, not a model name. The tempting shortcut is to write the vendor's
model directly, which works for exactly one harness: a plan carrying `opus` is
unrunnable on any other, and a portable planner cannot ship vendor names in its
own format.

| Tier | Means | Typical work |
|---|---|---|
| `cheap` | exact spec, small surface, fast oracle | config, env plumbing, mechanical edits |
| `standard` | real code against a strong oracle | the default for implementation |
| `deep` | contracts others build on; correctness no compiler checks | protocols, concurrency, cross-cutting design |

## Requirements

- **The vocabulary is closed.** Three tiers, validated by the schema (T02). An
  unknown tier is rejected rather than defaulted.
- **Each integration declares whether tiering is real** (`capabilities.subagents`
  in T04). Two cases, both reported plainly by `trestle doctor` and in the emitted
  prompt:
  - **Real** — the harness can dispatch a unit to a differently-configured
    subagent. The integration ships the mapping as *its own* documentation (this
    repo's `.claude/agents/trestle-{cheap,standard,deep}.md` is exactly that layer),
    and the emitted prompt instructs the agent to honour the declared tier.
  - **Inert** — everything runs on whatever model the user has selected. The
    prompt says so, `doctor` says so, and the plan's tiers are labelled advisory.
    **A user must never believe tiering worked when it did not.**
- **No vendor model name may appear under `plan/` or in the schema.** Enforced by
  a test that greps, not by review.
- Tiers still do real work even when inert: they feed the budget estimate (T20)
  and they tell a *human* reader which units deserve attention. Say that, so the
  field doesn't look pointless on single-model setups.
- User override in config, per tier, per integration — for the harnesses where the
  mapping is real and the user disagrees with it.

## Acceptance

- `cargo test -p trestle-integration --test tiers` — an unknown tier is rejected;
  an integration declaring `subagents = false` reports tiering inert in `doctor`
  output and in its rendered prompt; an integration declaring `subagents = true`
  renders the declared mapping.
- Grep assertion: no vendor model name in `schema/` or under `plan/`.
- The rendered prompt for an inert integration contains the degradation statement.
  **Asserted on the rendered output**, not on the template — a warning that a
  templating bug can silently drop is not a warning.

## Out of scope

Cost estimation (T20). Choosing which tier a unit gets — that is synthesis (T07).
Actually dispatching to a subagent, which is the harness's business.

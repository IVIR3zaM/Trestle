---
id: T01
title: Product contract + privacy threat model
tier: deep
gate: human
deps: []
---

## Goal

Write down what Trestle is and — more importantly — what it must never do, in a
form the rest of the graph can be checked against. Everything else descends from
this node, so it goes first.

## Deliverables

**`docs/PRODUCT.md`** — the contract:

- The six-step flow (survey → interrogate → shape → absorb → show → write/run)
  and what each step is allowed to assume about the others.
- **D0 restated as an architectural invariant**: Trestle performs no inference.
  Every model call belongs to the user's harness. State the consequence
  explicitly — Trestle's product surface is *prompts, schemas and validation*,
  not reasoning.
- What v0.1.0 does not do, so nodes don't quietly grow.

**`docs/THREAT-MODEL.md`** — the privacy guarantee, written adversarially:

| Party | Sees | Must never see |
|---|---|---|
| Trestle itself | everything on disk | — (it emits nothing) |
| The user's harness vendor | whatever that harness already sends | not Trestle's concern, but must be stated |
| Anyone on the LAN | nothing — dashboard is loopback-bound | all of it |

Enumerate every channel by which code could leave — HTTP client, DNS, telemetry
SDK, crash reporter, update check, a dependency that phones home at install,
the dashboard binding to `0.0.0.0`, a diagnostic bundle — and state the
countermeasure for each. **A channel with no automated check is a gap; name it
as one.** T16 turns this list into tests.

Be honest about the boundary in the user-facing copy: Trestle adds no new
recipient of your code, and cannot stop the one you already chose.

## Acceptance

- `bash scripts/check-product-doc.sh` — asserts both documents exist, that the
  threat model's channel table has a countermeasure in every row, and that no
  row says "TODO".
- Every claim in `README.md` traces to a statement in one of these documents.
- The no-inference invariant is stated somewhere an agent reading only
  `PRODUCT.md` cannot miss.

## Out of scope

Any code. Adapter design (T04). The egress tests themselves (T16) — this node
produces the list they must cover.

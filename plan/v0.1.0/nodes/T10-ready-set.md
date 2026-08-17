---
id: T10
title: Ready-set computation (trestle next)
tier: standard
deps: [T09, T12]
---

## Goal

Answer one question, for both shapes, deterministically: **what should be worked
on next, and if nothing, why not.**

This is the command the agent calls at the start of every pass. It replaces the
two executor nodes the previous plan had — under `D5` the *driving* is the agent's
job, and what Trestle owes it is a correct answer to this question.

## Requirements

- **Readiness is computed from dependencies and status, never remembered.** This
  is what makes cold resume exact rather than interpretive, and it is the single
  property the deferred scheduler work (v0.2.0) will rest on. It must hold now
  even though nothing unattended runs yet.
- **Graph shape:** ready = `todo` and every dependency `done`. Report the ready
  set, not one node, and let the caller pick — with a suggested pick and the
  reason (prefer whatever unblocks the most downstream work; break ties by
  cheapest tier).
- **Loop shape:** the first `todo` in queue order, plus the journal offset the
  agent must read from. A loop's next-item is trivial; **the non-trivial part is
  telling the agent to re-orient from the repo, not from the journal.** Where they
  disagree, the repo wins (`docs/PRIOR-SHAPES.md`) — so the output includes the
  journal's own claims flagged as claims.
- **Human-gated units are reported, never suggested.** Output them in a separate
  field with the decision they turn on, so an agent following the prompt cannot
  accidentally pick one up.
- **Units blocked by an unresolved decision are reported as `blocked`** with the
  question id, and excluded from the ready set.
- **Nothing ready is a first-class answer, not an error.** Report which of *all
  gated*, *all blocked*, *all done*, *plan is still a draft*, or *dependency
  deadlock* applies. A graph whose contract units are all human-gated legitimately
  reports zero executable work — this repo's own plan is in that state right now —
  and the message must say so plainly rather than looking like a failure.
- **`--role <role>` filters to what that role can act on** (`D14`). With no
  `verifier` configured the flag changes nothing. With one configured,
  `--role verifier` returns units in `verified` awaiting review, and
  `--role implementer` excludes them — so each agent asks one question and gets its
  own queue rather than filtering a shared list and getting it wrong.
- A unit returned to `todo` by a failed review carries the reviewer's reason, so the
  implementer sees why it came back without being told separately.
- **Scope advisory:** where a unit's declared file surface is large enough that it
  is unlikely to fit one pass, say so in the output. The agent is told to split it
  into sub-units with proper edges rather than attempt it. Trestle advises; it
  cannot enforce.
- Stable `--json` shape, versioned, since the agent depends on it.

## Acceptance

- `cargo test -p trestle-exec --test next` — correct ready set on a fixture graph
  including a diamond and a long chain; gated units in the gated field and absent
  from ready; decision-blocked units excluded with their question id; each of the
  four "nothing ready" causes reported distinctly.
- Loop fixture: returns the first `todo`, skips the `blocked(user)` item rather
  than suggesting it, and reports the journal offset.
- A draft plan returns nothing selectable and reports *"still a draft"* as the cause.
- `--role verifier` on a fixture with a configured reviewer returns exactly the
  `verified` units and `--role implementer` returns exactly the `todo` ones; with no
  reviewer configured both return the same set the unflagged call does.
- A unit bounced by a failed review carries the reason in the output.
- **Determinism:** the same plan and status always produce byte-identical output.
  Asserted, because a `next` that varies run to run makes resume unverifiable.
- Reading `next` never requires the full plan bodies — index and status only.

## Out of scope

Doing the work. Verifying it (T11). Firing it on a schedule (v0.2.0).

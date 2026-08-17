---
id: T25
title: Plan amend — additive re-planning of a live plan
tier: standard
deps: [T09, T12]
---

## Goal

Answer the question the previous plan had no answer for: **the plan is wrong at
unit 7 — now what?**

T09 correctly refuses to overwrite a plan with progress recorded against it. That
is the right refusal and it leaves a hole, because a plan being wrong partway
through is the normal case, not the exceptional one.

Blocked on **D12** (amend vs version vs nothing).

## Why this is a node and not a footnote

A loop absorbs discovery through its journal — that is the entire mechanism, and
`docs/PRIOR-SHAPES.md` is explicit that a loop's rules are *superseded in place,
marked, never deleted*. A graph as specified has nowhere to put discovery. Without
this node the user's only options are hand-editing a plan the validator may then
reject, or abandoning the plan — and the second is what people will actually do.

## Requirements

`trestle plan amend` (stdin, validated, atomic — the same guarantees as
`plan write`):

- **Additive operations only:** add units, add dependency edges, mark a unit
  `superseded` with a reason and a pointer to what replaced it, split a unit into
  sub-units with proper edges, attach a new decision to existing units.
- **Never deletes.** A superseded unit stays in the plan, marked, the way the loop
  fixture marks superseded rules. The plan is the record of what was thought, not
  just what is currently believed.
- **Cannot un-`done` a unit that passed its oracle.** Hard invariant. If work must
  be redone, that is a *new* unit that depends on the old one — which is also the
  honest description of what is happening.
- **Cannot silently change a unit that is in progress.** Refuse, and name the unit.
- Every amend records: what changed, why, when, and by whom, in a form that reads
  as a normal git diff. The point is that a teammate can review the amendment the
  same way they reviewed the plan.
- **Re-runs the full T07 gauntlet on the amended plan.** An amend that introduces a
  cycle, an oracle-less unit, or a dangling dependency is rejected — the plan is
  never left in a state the validator would reject.
- Status for unchanged units carries forward untouched. Status is separable from
  definition (T12), so this should require no status rewriting at all — and if it
  does, that is a signal T12 got the separation wrong.

## The prompt half

The agent needs to be told when to reach for this, or it won't. Shipped through
T04 alongside the execution prompt:

- A dependency you discovered is missing → amend, add the edge, continue. Normal.
- A unit's premise turned out to be false → **stop**, mark it superseded with the
  reason, file a decision. Do not build on it.
- The whole shape looks wrong → stop and tell the user. Re-shaping is not an
  amend, and an agent quietly converting a graph into something else is worse than
  an agent that stops.

## Acceptance

- `cargo test -p trestle-plan --test amend` — adding a unit and an edge preserves
  all existing status; marking a unit superseded retains it in the plan; an amend
  attempting to un-`done` a verified unit is rejected; an amend touching an
  in-progress unit is rejected naming it; an amend producing a cycle is rejected.
- After any amend, `trestle plan validate` passes — asserted on every fixture
  amendment, since an amend that leaves the plan invalid is the worst outcome here.
- `trestle next` immediately reflects an amended edge, computed rather than
  remembered (T10). Asserted, because this is the property that makes amending
  safe mid-run.
- The diff of an amend against the pre-amend tree is human-reviewable — asserted
  loosely, by checking that no unrelated unit file is rewritten.

## Out of scope

Re-shaping a plan (loop ↔ graph). Versioning plans (`D12` option (b), not taken).
Deciding *what* to amend — that is the agent's judgement.

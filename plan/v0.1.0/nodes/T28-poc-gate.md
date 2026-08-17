---
id: T28
title: POC gate — is the shape recommendation any good?
tier: deep
gate: human
deps: [T03, T05]
---

## Goal

Stop, run the thing against real repositories, and find out whether the core claim
holds **before** building the fifteen nodes that assume it does.

By this point `trestle survey` and `trestle shape` work. That is the whole product
thesis in two commands: read a repo, and say whether the work wants a loop or a
graph, with reasoning you can argue with. Everything downstream — synthesis, the
writer, the executors, the dashboard, the integrations — is machinery for acting on
an answer this node decides is worth acting on.

Human-gated because no oracle can judge it. **If the recommendation is bad, the
correct outcome is to stop and fix the rubric**, not to proceed and hope the rest of
the product compensates.

## Why the graph pauses here

Without this gate, nothing is runnable until T17 — six layers deep, weeks of
spare-time work — and the first real feedback on the product's central claim arrives
after almost all of the cost has been spent. That is the wrong order for the one
thing that, if wrong, invalidates everything built on it.

The gate is placed on **T07** and nothing else. The format (T02), the integration
contract (T04) and the egress test (T16) do not depend on the shape decision being
good, so they proceed in parallel while this is evaluated. Only work that *builds on
the shape answer* waits.

## The test

Five real repositories, yours, each with a **known-good human answer** you have
already thought through. Do not pick work you haven't.

| Profile | Expect |
|---|---|
| Multi-week change with independent tracks | **graph** |
| Migration with unsettled requirements | **loop** |
| Small, well-tested bugfix | **loop** |
| Repo-wide rename or audit | **graph** |
| Something you genuinely can't call | **low confidence / both** |

Two of the five must be ones you'd answer **loop**, because that is the failure this
node exists to catch. `fixtures/source/` has worked examples of the first two
profiles, but **dogfooding on fixtures is not dogfooding** — the rubric was written
with those in view.

Run `trestle survey --json` and `trestle shape --json` on each. Record the verdict,
the confidence, and the signal values that drove it.

## What counts as passing

- **At least four of five match the human answer**, and any divergence is understood
  and written up rather than explained away.
- **The reasoning is arguable.** For each result, the signal values are plausible on
  inspection — a repo you know has no parallelism must not score high on it. A right
  answer for a wrong reason is a failure here; it will not survive the next repo.
- **The low-confidence case actually returns low confidence** rather than a
  confident wrong answer. Guessing quietly is the failure mode that matters most.
- **The survey is fast enough to be usable** on the largest of the five, and its
  partial-analysis labelling is honest about what it could not parse (`D3`).

## What to do if it fails

Say so, and fix the rubric before proceeding. Concretely:

- A bias toward `graph` means the weights in T03 are wrong, or a signal is measured
  badly in T05. Both are amendable; neither is a reason to continue.
- If the signals themselves turn out not to separate the cases, that is a finding
  about the *product*, not the code — record it in `decisions.md` as a new decision
  and stop. A shaping tool whose shaping signal doesn't discriminate has no core.

**Do not weaken this node's criteria to get past it.** Same rule as an oracle.

## Acceptance

- `cargo test --workspace` green.
- The five evaluations are written up in `docs/POC-FINDINGS.md`: repo profile,
  expected shape, actual shape, confidence, the signal values, and a verdict.
- Four of five match, or the divergences are explained and the rubric amended.
- A human has signed off in that document, by name and date.

## Out of scope

Plan synthesis (T07), writing plans (T09), anything with a UI. This node judges one
question: is the shape answer good enough to build on.

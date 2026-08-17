---
id: T03
title: Shape-decision rubric (loop vs graph vs both)
tier: deep
gate: human
deps: [T00, T05]
---

## Goal

The product's actual differentiator. Given a surveyed repo and a stated goal,
decide **loop**, **graph**, or **present both**, and explain why in terms the
user can argue with.

A rubric that always says "graph" is worthless — graphs are frequently the wrong
answer, and a tool that can't say so will lose people's trust the first time it
costs them a morning.

## Who decides, under inverted control

The final call is the agent's, because weighing a goal against a codebase is
inference (`D0`). But leaving the shape *entirely* to the agent would forfeit the
product's differentiator to whichever model the user happens to be running — and
models have a known bias toward producing structure.

So the work splits:

- **T05 measures the signals** — parallelism, oracle presence and speed, unit
  count, module fan-out. Deterministic, from the code.
- **This node scores them** into a baseline recommendation with confidence, exposed
  as `trestle shape --json`. Pure function, no I/O, no agent.
- **T07's prompt requires the agent to state where it disagrees with the baseline
  and why.** Disagreement is allowed and sometimes right; silent divergence is not.

That is what keeps "the rubric must be willing to say loop" **testable** rather
than a hope about prompt wording — the assertion lives in this node's test suite,
where a regression fails CI.

## This node does not depend on the plan format

Deliberately. The rubric's output is `{shape, confidence, reasoning[], signals[]}` —
a small struct of its own, not a plan. It needs the **survey's signals** (T05) and
nothing from T02.

Decoupling it is what makes the vertical slice possible: `trestle survey` plus
`trestle shape` is a working answer to the product's central question, reachable
without the format, the synthesis prompt, the writer or the executors. **T28 judges
that answer before the rest of the graph is built on it.** Keep this node free of
the plan format so that stays true.

## Signals

Each is derivable from the survey (T05) or from the user's goal. The rubric must
state how each is measured, not just named.

| Signal | Toward loop | Toward graph |
|---|---|---|
| Parallelism available | work is inherently sequential | independent tracks exist |
| Oracle speed | fast tests/compiler already present | slow, missing, or manual verification |
| Interruption | one sitting, user present | spans days, unattended, rate limits |
| Completeness | "make it work" | rename, deprecation, audit, migration |
| Requirements settled | exploratory, likely to change | contracts known up front |
| Size | under ~10 units | more |
| Contracts first | none | several consumers of one interface |
| Human decisions | few, answerable inline | several, must block specific work |

Two of these deserve more weight than the rest, and the rubric should say so:

- **A fast oracle strongly favours the loop.** Iterating against a compiler beats
  any amount of structure. Structure substitutes for a missing signal.
- **Unattended execution strongly favours the graph.** A loop cannot compute
  readiness after an interruption; it re-derives it from prose, and two readers
  can disagree.

## Deliverables

- `docs/SHAPE-RUBRIC.md` — signals, how each is measured, weighting, and worked
  examples in both directions. Include at least one case that comes out **loop**,
  argued as strongly as the graph cases.
- `crates/trestle-shape/` — pure function: survey signals + goal + answers →
  `{shape, confidence, reasoning[], alternative?}`. No I/O, so it is testable
  without an agent.
- `trestle shape --json` — the baseline, with every signal's measured value and its
  contribution shown. **The agent must be able to argue with the score**, which
  means the output carries the arithmetic, not just the verdict.
- **The both-ways path.** Below a confidence threshold the output is *both plans*
  with a tradeoff table, and the user picks. Specify the threshold and justify it.
  Guessing quietly at low confidence is the failure mode that matters here.

## Acceptance

- `cargo test -p trestle-shape` — a fixture table of scenarios with expected
  shapes, including: a two-hour bugfix with good tests (**loop**), a repo-wide
  rename (**graph**), a multi-week release with unsettled requirements (**both**),
  and a case that must return low confidence.
- **At least a third of the fixture corpus returns `loop`**, and the test asserts
  that proportion. A rubric drifting toward structure is the specific regression
  worth a guard, and "no fixture came out loop" must fail rather than pass quietly.
- Every recommendation carries reasoning naming the signals that drove it, and each
  signal's measured value appears in the output.
- The rubric is a pure function — asserted by the crate having no I/O dependency.

## Out of scope

Rendering the comparison (T14). Synthesising the plans themselves (T07) — this
node decides the *shape*, T07 fills it in. Measuring the signals (T05).

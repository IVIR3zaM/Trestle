---
id: T03
title: Shape-decision rubric (loop vs graph vs both)
tier: deep
gate: human
deps: [T02]
---

## Goal

The product's actual differentiator. Given a surveyed repo and a stated goal,
decide **loop**, **graph**, or **present both**, and explain why in terms the
user can argue with.

A rubric that always says "graph" is worthless — graphs are frequently the wrong
answer, and a tool that can't say so will lose people's trust the first time it
costs them a morning.

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
- `src/shape/rubric.ts` — pure function: survey + goal + answers → `{shape,
  confidence, reasoning[], alternative?}`. Pure and deterministic so it is
  testable without a harness.
- **The both-ways path.** Below a confidence threshold the output is *both plans*
  with a tradeoff table, and the user picks. Specify the threshold and justify it.
  Guessing quietly at low confidence is the failure mode that matters here.

## Acceptance

- `npm run test:rubric` — a fixture table of scenarios with expected shapes,
  including: a two-hour bugfix with good tests (**loop**), a repo-wide rename
  (**graph**), a multi-week release with unsettled requirements (**both**), and
  a case that must return low confidence.
- Every recommendation carries reasoning naming the signals that drove it.
- The rubric is a pure function — no I/O, no harness call.

## Out of scope

Rendering the comparison (T14). Synthesising the plans themselves (T07) — this
node decides the *shape*, T07 fills it in.

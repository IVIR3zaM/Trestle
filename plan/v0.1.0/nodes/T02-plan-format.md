---
id: T02
title: Plan format spec (graph, loop, hybrid in one schema)
tier: deep
gate: human
deps: [T01]
---

## Goal

Define the on-disk format every other component reads or writes. **This is the
highest-leverage node in the project.** Get it wrong and every consumer inherits
the mistake.

Blocked on **D2** — resolve it before starting.

## The hard requirement

One format must express three shapes without making any of them second-class:

**Graph** — nodes, dependency edges, per-node oracle, model tier, human gates,
`todo`/`done`/`blocked`. Reference: `fixtures/source/graph-shape/`.

**Loop** — an ordered queue, an append-only journal, and `blocked(user):
<question>` states. Reference: `fixtures/source/loop-shape/` — `PLAN.md` (goal
+ hard rules), `STATE.md` (queue with statuses), `LOG.md` (append-only, fixed
entry format), `DEFERRED.md`. **Read that fixture before designing this.** A
working loop is more structured than the naive picture of one: ordered phases,
explicit human-blocked states, an audit trail, and superseded rules marked in
place rather than deleted. `docs/PRIOR-SHAPES.md` explains why each part is
there.

**Hybrid** — which is what most real work is. A graph whose nodes are executed
loop-style, or a queue with a few genuine dependency edges.

## Design constraints

1. **The journal is not optional for loops.** Collapsing a loop into a chain-shaped
   graph loses it, and the journal is precisely how a loop carries discovery
   forward. If the schema can't hold it, the schema is wrong.
2. **Plain text, git-friendly, reviewable in a PR.** One file per unit, plus an
   index. Diffs should be readable by a human.
3. **Status is separable from definition.** T12 needs to read progress without
   parsing prose; execution must not rewrite plan bodies.
4. **Forward-compatible.** Unknown keys are ignored, never fatal — v0.2.0 will add
   fields and old plans must still load.
5. **Human-editable.** Users will hand-edit these. Prefer a format that survives
   it.

## Deliverables

- `docs/PLAN-FORMAT.md` — normative spec with a worked example of each shape.
- `schema/plan.schema.json` — machine-checkable.
- `fixtures/expressed/` — both source fixtures (`fixtures/source/graph-shape/`
  and `fixtures/source/loop-shape/`) expressed in this format, plus at least one
  hybrid. **If either source cannot be expressed without loss, the format is not
  done.** They are the acceptance bar precisely because neither was written for
  it — a format proved only against examples designed for it has been proved
  against nothing.

## Acceptance

- `npm run test:schema` — every fixture validates; every fixture round-trips
  (parse → serialise → parse) without loss; unknown keys survive a round trip;
  malformed plans fail with a message naming the offending path.
- A reviewer can read `PLAN-FORMAT.md` and hand-write a valid plan without
  looking at the schema.

## Out of scope

Synthesis (T07), execution (T10/T11), rendering (T14). This node ships a format
and its tests, nothing that produces or consumes plans in anger.

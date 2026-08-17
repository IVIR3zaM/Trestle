---
id: T02
title: Plan format spec (graph, loop, hybrid in one schema)
tier: deep
gate: human
deps: [T01]
status: split
---

> **This node was split and is not executable.** Spec-and-fixtures is a design
> decision a human settles; the parser is implementation against a settled spec, and
> ~25 files in one pass on the project's highest-leverage node was the scope risk.
>
> - **[`T02a`](T02a-plan-format-spec.md)** — normative spec, JSON schema, expressed
>   fixtures. Proves the format can *express* both source fixtures without loss.
> - **[`T02b`](T02b-plan-parser.md)** — `crates/trestle-plan`, round-trip fidelity,
>   error-message quality. Proves it can be *implemented* without loss.
>
> Both sub-node files are self-contained; this file is kept as the record of what was
> split and why. Every node that depended on T02 now depends on `T02b`, which
> subsumes `T02a`.

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
6. **Machine-*writable*, and this is new.** Under `D5` the agent writes plans in
   this format, so the schema is not only a reader's contract — it is the surface
   the agent iterates against. Two consequences: validation errors must name the
   offending path and say what was expected (an error an agent can act on), and the
   schema must be strict enough that a plausible-looking bad plan fails. A
   permissive schema pushes all the load onto T07's gauntlet.
7. **States the whole lifecycle needs**, not just `todo`/`done`. All of these must
   exist in the format from the start — retrofitting a status value touches every
   consumer:
   - `draft` — a plan written but not yet approved, so the dashboard can render it
     before the user has committed to it (`D13`, T13/T14)
   - `verified` — the oracle passed but a configured reviewer hasn't cleared it
     (`D14`). With no reviewer configured this collapses into `done`.
   - `done(overridden)` — T11's recorded override
   - `superseded` — T25's additive amendment
8. **Provenance on a unit's oracles.** A precondition attached from a user standard
   carries the rule id and citation it came from (`SEC-04 §14.2`, T08/T27). A
   reviewer looking at a unit must be able to trace *why* an extra command is
   attached back to the clause that caused it — otherwise ingested standards become
   an unexplained pile of commands, which is how people start deleting them.
9. **Roles are not in the plan.** Who does what (`D14`) is a property of the user's
   setup, recorded in `.trestle/config.toml`. Putting roles in the plan would make
   plans non-portable between people — the same mistake as writing a vendor model
   name into a tier.

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

- `cargo test -p trestle-plan` — every fixture validates; every fixture round-trips
  (parse → serialise → parse) without loss; unknown keys survive a round trip;
  malformed plans fail with a message naming the offending path.
- A reviewer can read `PLAN-FORMAT.md` and hand-write a valid plan without
  looking at the schema.
- **Error-message quality is asserted, not hoped for:** for each of a fixture set
  of malformed plans, the error names the offending path and the expectation. This
  is the interface the agent converges against (`D5`), so a bad message is a real
  defect, not a cosmetic one.
- Multi-repo is not implemented and must not be precluded: the schema reserves how
  a unit names its repo, and a v0.2.0 multi-repo plan would not require a breaking
  change. Assert by writing one such plan as a forward-compatibility fixture.

## Out of scope

Synthesis (T07), execution (T10/T11), rendering (T14). This node ships a format
and its tests, nothing that produces or consumes plans in anger.

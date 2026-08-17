---
id: T02a
title: Plan format — normative spec, JSON schema, expressed fixtures
tier: deep
gate: human
deps: [T00, T01]
---

## Goal

Define the on-disk format every other component reads or writes, and prove it can
**express** both shapes people actually use. **This is the highest-leverage node in
the project.** Get it wrong and every consumer inherits the mistake.

Half of the original T02. This half settles the format; [`T02b`](T02b-plan-parser.md)
implements it. The seam is what each half proves: T02a proves the format can express
the source fixtures without loss, T02b proves it can be implemented and round-tripped
without loss. `D2` is resolved — one schema with a `shape:` discriminator — so the
shape of the answer is settled and this node fills it in.

## The hard requirement

One format must express three shapes without making any of them second-class:

**Graph** — units, dependency edges, per-unit oracle, tier, human gates, statuses.
Reference: `fixtures/source/graph-shape/` (`plan.yaml`, `decisions.md`, `units/`).

**Loop** — an ordered queue, an append-only journal, and `blocked(user): <question>`
states. Reference: `fixtures/source/loop-shape/` — `PLAN.md` (goal + hard rules),
`STATE.md` (queue with statuses), `LOG.md` (append-only, fixed entry format),
`DEFERRED.md`. **Read that fixture before designing anything.** A working loop is far
more structured than the naive picture of one: ordered phases, explicit
human-blocked states, an audit trail, and superseded rules marked in place rather
than deleted. `docs/PRIOR-SHAPES.md` explains why each part is there.

**Hybrid** — what most real work is. A graph whose units are executed loop-style, or
a queue with a few genuine dependency edges. Per `D2` it is *a graph whose units may
each carry a queue and journal* — not a third schema. It must not be the case that
renders or validates worst.

## Design constraints

1. **The journal is not optional for loops.** A loop plan without one fails
   validation. Collapsing a loop into a chain-shaped graph loses the journal, which
   is precisely how a loop carries discovery forward. If the schema can't hold it,
   the schema is wrong. This was rejected option (c) of `D2`; do not rediscover it.
2. **Plain text, git-friendly, reviewable in a PR.** One file per unit, plus an
   index. Diffs must be readable by a human.
3. **Status is separable from definition.** T12 reads progress without parsing prose;
   execution must never rewrite plan bodies.
4. **Forward-compatible.** Unknown keys are ignored on read and preserved on
   round-trip, never fatal — v0.2.0 adds fields and old plans must still load.
5. **Human-editable.** Users will hand-edit these. Prefer a format that survives it.
6. **Machine-writable.** Under `D5` the *agent* writes plans in this format, so the
   schema is the surface the agent iterates against. It must be strict enough that a
   plausible-looking bad plan fails — a permissive schema pushes all the load onto
   T07's gauntlet. Specify, for every constraint, what the error must name.
7. **Every status the lifecycle needs, from the start.** Retrofitting a status value
   touches every consumer:
   - `draft` — written but not approved, so the dashboard can render it before the
     user has committed (`D13`)
   - `todo`, `in_progress`, `blocked` (including `blocked(user): <question>`)
   - `verified` — oracle passed, a configured reviewer hasn't cleared it (`D14`);
     collapses into `done` when no reviewer is configured
   - `done`, and `done(overridden)` — T11's recorded override, a distinct permanent
     state (`D9`)
   - `superseded` — T25's additive amendment, marked in place, never deleted
8. **Provenance on a unit's oracles.** An oracle attached from a user standard carries
   the rule id and citation it came from (`SEC-04 §14.2`). A reviewer must be able to
   trace why an extra command is attached back to the clause that caused it —
   otherwise ingested standards become an unexplained pile of commands, which is how
   people start deleting them.
9. **Roles are not in the plan.** Who does what (`D14`) lives in
   `.trestle/config.toml`. Roles are a property of the user's setup, not of the work;
   putting them in the plan makes plans non-portable between people — the same
   mistake as writing a vendor model name into a tier.
10. **Tiers are abstract.** `cheap` / `standard` / `deep`, never a vendor model name.

## Deliverables

- **`docs/PLAN-FORMAT.md`** — the normative spec, with a worked example of each
  shape. A reviewer must be able to read it and hand-write a valid plan **without
  looking at the schema**.
- **`schema/plan.schema.json`** — machine-checkable, and the same contract the spec
  describes in prose. Where they disagree the format has two definitions, which is
  the failure this node's oracle exists to catch.
- **`fixtures/expressed/`** — both source fixtures expressed in this format, plus at
  least one hybrid, plus one forward-compatibility fixture that names a unit's repo
  (multi-repo is not implemented and **must not be precluded**; a v0.2.0 multi-repo
  plan must not require a breaking change).
- **`scripts/check-plan-format.sh`** — this node's own oracle, which does not exist
  until this node writes it. Write it first, watch it fail, then write until it
  passes. It must name which assertion failed, not merely that one did. Follow the
  pattern in `scripts/check-product-doc.sh` and `scripts/check-workspace.sh`, and
  stay **dependency-free** — python3 stdlib and standard shell tools only, for the
  reason given at the top of `scripts/status.py`.

## Acceptance

- `bash scripts/check-plan-format.sh` passes, and each assertion below is separately
  named in its output.
- **Losslessness is asserted mechanically, not eyeballed.** For each source fixture,
  extract its identifiers — every queue item, every journal entry, every hard rule,
  every deferred item, every unit id, every dependency edge, every `blocked(user)`
  question — and assert each one appears in the expressed fixture. **If either source
  cannot be expressed without loss, the format is not done.** They are the acceptance
  bar precisely because neither was written for this format: a format proved only
  against examples designed for it has been proved against nothing.
- **Spec and schema agree**: every property the schema defines appears in
  `PLAN-FORMAT.md`, and every key the spec documents exists in the schema. Two
  definitions of one format is the defect this catches, and it will happen.
- All three shapes appear in `fixtures/expressed/`, and the loop fixture carries a
  journal — asserted, not assumed.
- Every status in constraint 7 appears in the schema. Asserted one per status, so a
  missing `done(overridden)` fails loudly rather than surfacing in T11.
- `schema/plan.schema.json` is valid JSON and declares its JSON Schema dialect.
- No vendor model name anywhere in the deliverables.

## Out of scope

The parser, round-trip fidelity and error-message quality — all `T02b`, which is
where a Rust crate first reads this format. Synthesis (T07), execution (T10/T11),
rendering (T14). This node ships a format, its fixtures and its own oracle; nothing
that produces or consumes plans in anger.

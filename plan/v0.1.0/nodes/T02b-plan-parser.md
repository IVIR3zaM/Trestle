---
id: T02b
title: Plan format — parser, round-trip fidelity, error-message quality
tier: standard
deps: [T02a]
---

## Goal

Implement the plan format in Rust: parse it, serialise it, and reject bad plans with
errors an agent can act on. `crates/trestle-plan` is the crate every later consumer
reads plans through, so its error messages are a product surface, not diagnostics.

Half of the original T02. [`T02a`](T02a-plan-format-spec.md) settled the format and
proved it can **express** both source fixtures; this half proves it can be
**implemented** and round-tripped without loss.

**Read `docs/PLAN-FORMAT.md` and `schema/plan.schema.json` first.** They are the
contract. This node does not get to reinterpret them — see *When the spec is wrong*.

## Requirements

- **`crates/trestle-plan`** — parse, serialise, validate. `publish = false` (`D15`).
  `pub(crate)` by default; a public API is a commitment other nodes will build on,
  so make each item public deliberately.
- **Round-trip fidelity.** parse → serialise → parse loses nothing, for every fixture
  in `fixtures/expressed/`.
- **Unknown keys are preserved, never fatal** — v0.2.0 will add fields and old plans
  must still load. A round trip must not drop a key the parser does not know.
  Asserted with a fixture carrying keys the schema does not define.
- **Errors name the offending path and the expectation.** Under `D5` the agent writes
  plans, so a validation error is the interface it converges against. `"invalid
  plan"` is a defect. `"units[3].oracle: required when gate is absent — a unit needs
  either an oracle or a human gate"` is the bar. Every user-facing failure carries a
  stable code and a sentence naming the fix (`AGENTS.md` §3).
- **Strict enough that a plausible-looking bad plan fails.** A permissive schema
  pushes all the load onto T07's gauntlet. Build a `fixtures/malformed/` set of plans
  that look right and are not: a dependency edge naming a unit that does not exist, a
  cycle, a loop with no journal, a unit with neither oracle nor gate, a vendor model
  name in a tier, a `done` status with no oracle result, a duplicate unit id.
- **Status is read without parsing prose** (T12's requirement, asserted here): the
  status of every unit is reachable from structured fields alone.

## When the spec is wrong

T02a proved the format can express both fixtures by hand. If a fixture turns out not
to **round-trip** losslessly, or a constraint in the spec cannot be implemented as
written, that is a real finding about the format and it amends `T02a` — the whole
reason the split exists.

Do not resolve it silently, do not edit `docs/PLAN-FORMAT.md` or
`schema/plan.schema.json` to match what your code happens to do, and do not weaken a
fixture. Report it, and let a human amend the spec. `AGENTS.md` §6 covers exactly
this case.

## Acceptance

- `cargo test -p trestle-plan` — the node's oracle.
- Every fixture in `fixtures/expressed/` parses and validates, including the hybrid
  and the forward-compatibility fixture that names a unit's repo.
- Every fixture round-trips (parse → serialise → parse) without loss. Asserted by
  comparing parsed structures, not by diffing serialised text — a formatter change
  must not read as data loss.
- Unknown keys survive a round trip.
- **Error-message quality is asserted, not hoped for**: for each fixture in
  `fixtures/malformed/`, the test asserts the error names the offending path *and*
  the expectation. A test that only asserts "parsing failed" does not satisfy this —
  the whole point is that the message is actionable, and a bad message is a real
  defect, not a cosmetic one.
- Each malformed fixture is named after the mistake it makes, so a failure report
  says which class of bad plan stopped being caught.
- The crate has no I/O beyond reading the files it is handed, so it is testable
  without a repo or an agent.

## Out of scope

Writing plans to disk atomically (T09). The validation gauntlet (T07) — this node
validates against the schema; T07 adds the judgement checks a synthesised plan must
survive. Status *storage* (T12), rendering (T14), amendment (T25).

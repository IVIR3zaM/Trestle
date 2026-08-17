---
id: T11
title: Oracle runner (trestle verify) + review veto — sole writer of done
tier: deep
deps: [T09, T12]
---

## Goal

`trestle verify <unit>` runs the unit's oracle itself, in the real worktree, and
records the result.

**This is the node that keeps the whole architecture honest.** Everywhere else,
inverting control (`D5`) cost Trestle a guarantee. Here it gains one back, and
only if this node is built exactly as specified.

Blocked on **D9** (the override path).

## The invariant

> There is no way to write `done` except by `trestle verify` running the oracle
> command and observing it succeed.

No `trestle record --done`. No `--assume-pass`. No status field the agent can set.
The agent's report is not an accepted input, because the agent is the producer and
an oracle is by definition external to the producer.

This converts *"the agent should run the oracle"* from a hope into a mechanism. It
is why agent-driven execution is safe when "emit a plan and trust the executor"
would not have been.

Corollaries the implementation must honour:

- **The oracle command comes from the plan on disk**, re-read at verify time —
  never from an argument. Otherwise the agent chooses the oracle, which is the
  same hole by another route.
- **The plan file's oracle is compared against git HEAD** and a change to an
  oracle since the unit was written is reported in the verify output. *Never edit
  an oracle to make it pass* is a rule Trestle can at least make visible.
- **Exit status is the verdict.** Not stdout matching, not a regex, not the
  absence of the word "error".
- A failing oracle leaves the unit `todo` — never `failed` as a terminal state,
  because the next pass must pick it up. Record the attempt, the exit code, and a
  bounded tail of output in status (T12).
- Human-gated units cannot be verified into `done` by this command; resolving a
  gate is a separate, explicitly human action.

## Requirements

- Runs in the **main worktree**, with the repo's own environment, so a passing
  oracle means what the user thinks it means.
- Timeout, with the timeout recorded distinctly from a failure — a killed 20-minute
  test suite is not a red test.
- Multiple oracle commands per unit run in order; the first failure stops and is
  reported. All must pass for `done`.
- Captures output to a bounded log under `.trestle/runs/` (not the plan, not the
  status file). Never unbounded — a 400MB test log in a git repo is its own bug.
- **Precondition oracles from T08 are indistinguishable from a unit's own.** A
  rule the user's `SKILL.md` attached as a precondition is just another command in
  the list. That is what makes an ingested convention *real* rather than advisory.
- `--dry-run` prints what would run without running it.

## The review step, per D14

When the user has configured a `verifier` role — a second agent that checks work
the first one wrote — a passing oracle produces `verified`, not `done`, and
`trestle review <unit> --pass | --fail --reason <text>` clears it.

**The reviewer's power is deliberately asymmetric: it may withhold `done`, never
confer it.**

```
todo ──oracle passes──▶ verified ──review passes──▶ done
  ▲                                    │
  └────────── review fails ────────────┘   (reason recorded, unit returns to todo)
```

- `--pass` on a unit whose oracle has *not* passed is refused. There is no path from
  `todo` to `done` through review, so a reviewing agent cannot grant what the oracle
  withheld — that would reintroduce the exact hole this node exists to close, one
  level up.
- With no `verifier` configured, `verified` and `done` are the same state and none of
  this is visible. The single-agent case must not pay for the multi-agent one.
- `--fail` records the reason and returns the unit to `todo`, where `trestle next`
  will offer it again with the review comment attached.
- The review verdict records which role and which harness produced it (T12).

## The override, per D9

`trestle verify --override --reason <text>` records `done(overridden)` — a state
distinct from `done`, carrying the reason, the timestamp, and who did it.

- `trestle status` counts overridden units separately and always shows the count.
- T14 renders them with a distinct visual state.
- **Documented honestly:** this is not an enforcement boundary. The agent runs
  commands in the user's own shell and could invoke it. What protects the user is
  that an override is loud and permanent in a file that is in git — not that it is
  hard to perform. Say that in the docs; do not imply a control that doesn't exist.

## Acceptance

- `cargo test -p trestle-exec --test verify` — a unit whose oracle exits non-zero
  stays `todo`; a unit whose oracle exits zero becomes `done`; **a unit cannot be
  marked done by any other code path** (assert by grepping the workspace for
  writers of the `done` state and asserting there is exactly one, in this crate);
  a timeout records distinctly from a failure; a mid-list failure does not run
  later commands.
- An oracle changed since the unit was authored produces a warning in the verify
  output — asserted with a fixture whose oracle differs from HEAD.
- `--override` produces `done(overridden)` and never plain `done`; omitting
  `--reason` is an error.
- With a `verifier` configured: a passing oracle produces `verified` and not `done`;
  `trestle review --pass` on a unit that never passed its oracle is **refused**;
  `--fail` returns the unit to `todo` with the reason retrievable from
  `trestle next`. With no verifier configured, the same fixture produces `done`
  directly and no review command is required — asserted both ways, since the cost
  of the multi-agent path falling on single-agent users would be a real regression.
- Verifying a `gate: human` unit is refused with a message explaining how gates
  are resolved.
- Crash test: kill the process between oracle success and status write; the unit is
  still `todo` and re-verifying succeeds. **Never the reverse** — a unit marked
  done by a write that outran its own verification is the worst outcome available
  here.

## Out of scope

Selecting the unit (T10). Displaying results (T14). Retrying on rate limits
(v0.2.0).

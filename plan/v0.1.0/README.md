# Trestle v0.1.0 — build graph

The plan for Trestle's first version, planned the way Trestle itself will plan
things. `graph.yaml` holds the nodes and edges; `nodes/` holds one file per node;
`decisions.md` holds the questions that block them.

```bash
make status
```

## The architecture this plan builds

`D5` is resolved, and it is the decision that shapes everything here: **control is
inverted.** Trestle performs no inference (`D0`) and does not drive the agent
either — the agent drives Trestle, from inside its own interface.

```
   the user                the agent                      Trestle
   ────────                ─────────                      ───────
   "plan this: …"   ──▶    reads the survey        ──▶    trestle survey --json
   (in Copilot Chat,       asks what it can't            trestle conventions --json
    Claude Code,           answer by reading             trestle shape --json
    Codex — never          synthesises a plan            trestle decisions add
    a terminal)            iterates on errors    ◀──▶    trestle plan validate
                           writes it                     trestle plan write
                           does one unit                 trestle next
                           asks for a verdict     ──▶    trestle verify   ← runs the
                                                                            oracle itself
   trestle status
   trestle ui       ◀────────────────────────────────    plan + status files
```

The human types **one** command that matters: `trestle init`. The dashboard starts
itself when a draft is written and the agent hands over a deep link (`D13`), so
`trestle ui` and `trestle status` are for when you feel like looking, not steps in
the flow.

Where the user runs several assistants (`D14`), each is assigned a **role** at init
— `planner`, `implementer`, `verifier` — and gets only that role's prompts. A
configured verifier makes the pipeline strictly stricter: a passing oracle produces
`verified`, and review can withhold `done` but never confer it.

Three consequences worth holding in mind while reading any node:

- **Trestle's product surface is prompts, schemas, validation and deterministic
  analysis.** Every rule that would have lived in synthesis code is now a check on
  the written artifact, because a rule with no check is a sentence in a prompt that
  a tired model will skip (T07).
- **`trestle verify` is the sole writer of `done`** (T11). It runs the oracle
  itself; the agent's claim of success is not an accepted input. This is the one
  guarantee that got *stronger* by inverting control.
- **Some things got weaker, and the nodes say so.** Trestle can no longer choose a
  model (T19), can no longer observe token spend (T20), and can no longer
  regression-test prompt quality (T18 carries that alone).

## Why a graph and not a loop

Applying Trestle's own rubric (see [`nodes/T03`](nodes/T03-shape-rubric.md)) to
building Trestle:

| Signal | Reading | Points to |
|---|---|---|
| Genuine parallelism | high — analysis, dashboard, and integrations barely touch | graph |
| Fast oracle available | yes, but only *after* the schema exists | mixed |
| Must survive interruption | yes — this is spare-time work across weeks | graph |
| Completeness matters | yes — the privacy guarantee must be provably total | graph |
| Requirements settled | **better than before** — 8 of 14 decisions resolved | graph |
| Task size | ~25 units, multi-week | graph |

The dissent that made this marginal has largely resolved. `D5` and `D6` — the two
that would have changed the plan's own shape — are answered, and the remaining open
questions (`D2`, `D3`, `D9`–`D12`) are scoped to specific nodes rather than
pervasive. A graph is now the clear reading rather than a five-to-one call.

## Scope of v0.1.0

**In:** survey a repo and extract a code graph; a deterministic shape baseline the
agent must argue with; a decision store strict enough that lazy questions bounce,
including the elicitation that asks about standards documents living outside the
repo; ingestion of those documents at any size, distilled into a pinned reviewable
rule set with drift detection; a plan format expressing loop, graph and hybrid;
in-repo convention classification with honest enforceability labelling; an atomic
validating plan writer with **drafts the user reviews in the browser**; additive
amendment; ready-set computation; **an oracle runner that is the only writer of
`done`, plus a reviewer that can veto but never grant**; a **self-starting**
read-only local dashboard with draft, work, code and multi-agent views; a proven
no-egress guarantee; **selectable** integrations for Claude Code, Copilot and Codex
plus a generic fallback, **with roles**; an MCP server over the same command
surface; pre-run budget estimation; **a single static binary with `brew` and
installer distribution.**

**Out:** anything unattended — `trestle run`, scheduling, limit-aware backoff (all
moved to [`../v0.2.0/`](../v0.2.0/README.md)); observed token usage, which is not
observable under this architecture (`D11`); controlling execution from the dashboard
(stated as a v2 bonus); multi-repo, though the format must not preclude it; hosted
anything; a plugin marketplace; auth.

## The critical path

```
T01 → T02 → T03 → T07 → T09 → T10/T11 → T17 → T23/T24/T26 → T18
```

T02 (the plan format) remains the single highest-leverage node: every other
component reads or writes it, and `D5` raised its stakes by making the *agent* a
writer of the format rather than only a reader. Its error messages are now an
interface, not a nicety.

T05 (repo survey), T16 (egress test) and T04 (integration contract) branch off T01
early and can proceed in parallel with the format work. T03 depends on T05, because
the rubric scores signals the survey measures.

T27 (external standards) sits on the critical path through T09, and is human-gated:
a distillation of someone's 300-page policy document governs every unit, so a wrong
extraction is worse than no extraction.

T17 (the CLI) is a wide fan-in — it is the surface over everything else — and
T23/T24/T26 all hang off it. That means the last third of the graph is unusually
parallel once T17 lands.

## Conventions

See [`../../docs/PRIOR-SHAPES.md`](../../docs/PRIOR-SHAPES.md) for why each exists:

- **No oracle, no node.** If you can't name a command that proves it done, merge
  it into a node that has one, or make it a human gate.
- **Never edit an oracle to make it pass.** If one is mis-specified, a human
  changes it and says so.
- **Nodes are contracts, not tasks.** "Round-trips every fixture without loss"
  survives contact with reality; "implement the parser" doesn't.
- **One node per pass, then stop.**
- Status is written only by the executor; edit by hand only to unblock.

The first three are the rules T07's gauntlet enforces on plans Trestle produces.
This repo has to hold to them itself.

## Before any of this

Nothing here is built. The first real step is **answering `D2`** in
`decisions.md` — it blocks most of the graph, and it now has more to hold: `draft`
and `verified` states, oracle provenance, and the rule that roles stay out of the
plan. `D3` blocks T05 and T15; `D9`–`D12` are scoped to single nodes and can be
answered when those nodes come up.

Then execute T01 interactively — it is gated for a reason, and its threat model is
what T16 turns into tests.

See [`../../CONTEXT.md`](../../CONTEXT.md) for a self-contained handoff if you're
picking this up in a fresh session.

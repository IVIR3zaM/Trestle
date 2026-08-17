# Trestle v0.1.0 — build graph

The plan for Trestle's first version, planned the way Trestle itself will plan
things. `graph.yaml` holds the nodes and edges; `nodes/` holds one file per node;
`decisions.md` holds the fifteen decisions behind them — all resolved, with their
reasoning and rejected alternatives, so they can be argued with rather than guessed
at.

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
| Requirements settled | **yes** — all 15 decisions resolved | graph |
| Task size | 27 units, multi-week | graph |

The dissent that made this marginal has resolved. Every decision is answered,
including the two (`D5`, `D6`) that would have changed the plan's own shape. A graph
is now the clear reading rather than a five-to-one call.

One loop-shaped concession remains, deliberately: **T28 is a checkpoint, not a
milestone.** T00 → T05 → T03 → T28 is a vertical slice that builds a working
`trestle survey` + `trestle shape`, then stops and asks a human whether the answer is
any good. T07 waits on it. That is iteration wearing a graph's clothes, and it is
there because the shape rubric being wrong would invalidate everything downstream —
so it should be tested in four nodes rather than twenty-four.

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
T00 ─┬─ T05 ── T03 ── T28 ⛔ ──┐        the slice: prove the shape answer first
     └─ T01 ── T02 ───────────┴─ T07 ── T09 ─┬─ T10/T11 ── T17 ─┬─ T23
                                             └─ T25             ├─ T24 ─┬─ T18
                                                                └─ T26 ─┘
```

T02 (the plan format) remains the single highest-leverage node: every other
component reads or writes it, and `D5` raised its stakes by making the *agent* a
writer of the format rather than only a reader. Its error messages are now an
interface, not a nicety.

**T00** is the floor: the Cargo workspace, the binary shell, the lints that enforce
half of `AGENTS.md`, and the CI workflow. `deps: []` and `tier: cheap` — start here.

**T28 is the checkpoint that matters.** By then `trestle survey` and `trestle shape`
work, which is the product thesis in two commands. A human runs them against five
real repositories and decides whether the recommendation is worth building on. If it
isn't, the correct outcome is to fix the rubric — not to proceed and hope the rest of
the product compensates. Only T07 waits on it; T02, T04 and T16 run throughout.

T03 depends on T05 and **not** on T02 — the rubric's output is its own small struct,
not a plan — which is what makes the slice reachable without the format.

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

Nothing here is built, and **nothing is blocked** — all fifteen decisions are
resolved, with their reasoning and rejected alternatives written down in
`decisions.md` to be argued with.

Start with **T00** (mechanical, no decision needed) and **T01** in parallel. T01 is
gated for a reason: it decides what the privacy guarantee actually promises, and its
threat-model channel table is what T16 turns into tests.

See [`../../CONTEXT.md`](../../CONTEXT.md) for a self-contained handoff if you're
picking this up in a fresh session.

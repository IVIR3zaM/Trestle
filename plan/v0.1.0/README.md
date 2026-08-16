# Trestle v0.1.0 — build graph

The plan for Trestle's first version, planned the way Trestle itself will plan
things. `graph.yaml` holds the nodes and edges; `nodes/` holds one file per node;
`decisions.md` holds the questions that block them.

```bash
make status        # once the Makefile exists — until then, read graph.yaml
```

## Why a graph and not a loop

Applying Trestle's own rubric (see [`nodes/T03`](nodes/T03-shape-rubric.md)) to
building Trestle:

| Signal | Reading | Points to |
|---|---|---|
| Genuine parallelism | high — analysis, dashboard, and adapters barely touch | graph |
| Fast oracle available | yes, but only *after* the schema exists | mixed |
| Must survive interruption | yes — this is spare-time work across weeks | graph |
| Completeness matters | yes — the privacy guarantee must be provably total | graph |
| Requirements settled | **no** — seven open decisions | loop |
| Task size | ~22 units, multi-week | graph |

Five of six point to a graph, and the dissent is real: the requirements aren't
settled. That's handled by making the contract nodes **human-gated** and putting
the unresolved questions in `decisions.md` rather than pretending the graph knows
things it doesn't. Once T01–T04 land, the rest is unusually parallel.

Honest caveat: if the answer to **D5** is "Trestle emits rather than
orchestrates", v0.1.0 shrinks by roughly a third and a loop becomes competitive.
Revisit the shape after D5 is answered — that's not a flaw in the plan, it's the
plan telling you which decision matters most.

## Scope of v0.1.0

**In:** survey a repo and extract a code graph; detect ambiguities and generate
questions; recommend a shape with reasoning; synthesise a plan in the standard
format; absorb user conventions; write the plan and execution instructions; a
read-only local dashboard with work and code views; a proven no-egress guarantee;
a CLI with two harness adapters; **abstract model tiers with budget estimation**;
**a limit-aware scheduler** with a local and a cloud-proxy backend.

**Out:** controlling execution from the dashboard (stated as a v2 bonus);
multi-repo (v0.1.0 handles one repo — the format should not preclude more);
hosted anything; a plugin marketplace; auth.

## The critical path

```
T01 → T02 → T03 → T07 → T09 → T10/T11 → T18
```

T02 (the plan format) is the single highest-leverage node: every other component
reads or writes it. If one thing is done carefully, make it that one.

T05 (repo survey), T16 (egress test) and T04 (adapter contract) branch off T01
early and can proceed in parallel with the format work.

The token-awareness and scheduling track (T19 → T20 → T22, and T21) hangs off
T04 and T12, so it runs alongside the dashboard rather than after it. T19 is
worth doing early despite its position: it decides that plans carry abstract
tiers rather than vendor model names, and retrofitting that into the format
later would touch every component.

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

## Before any of this

Nothing here is built and nothing is decided beyond D0. The first real step is
**answering D1, D2 and D5** in `decisions.md` — they block most of the graph, and
D5 may change its shape.

See [`../../CONTEXT.md`](../../CONTEXT.md) for a self-contained handoff if you're
picking this up in a fresh session.

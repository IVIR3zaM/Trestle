# Trestle

**Shape the work before the agent starts.**

Trestle is a local, harness-agnostic planner for AI-assisted engineering work.
Point it at one or more repositories, say what you want to accomplish, and it
will read the code, surface the ambiguities it can't resolve on its own, ask you
about the ones that matter, and hand back a **plan with a defensible shape** —
a loop, a dependency graph, or a hybrid — along with the reasoning for why that
shape fits this particular job.

Approve the plan and Trestle writes it to a standard folder in your repo, tells
you how to run it, and gives you a **local dashboard** to watch it happen.

> A trestle is the frame you build first so the thing you're actually making has
> something to rest on. That's the job.

---

## The problem

Agent work at any real scale falls apart in a predictable way. You describe a
task, the agent starts, and for a while it goes well. Then context fills, the
original requirements blur, verification gets fuzzy, and when something breaks
the cheapest fix is to start over. Nothing carries between sessions except prose.

Two coping strategies have emerged, and most people only know one of them:

- **Loop** — one agent, iterating: look at the state, do the next useful thing,
  check it, journal it, repeat. Cheap, adaptive, and very strong when there's a
  fast test suite to iterate against.
- **Graph** — precompute the dependency structure, give every unit a command
  that proves it done, and let a scheduler pick anything whose prerequisites are
  met. Costly up front, but parallelisable, resumable, and safe to leave running.

Neither is universally right, and **choosing badly is expensive in both
directions**: graphing a two-hour task wastes a morning on ceremony, and looping
a two-week release produces work nobody can resume or verify.

Trestle's core value is not that it runs plans. It's that it tells you which
shape this job actually wants, and why.

## What it does

**1. Survey.** Reads the repositories: languages, module boundaries, real import
and call edges, test commands, CI config, existing conventions. Builds a picture
of the code as a graph, not a pile of files.

**2. Interrogate.** Compares your stated goal against what it found and surfaces
what's genuinely ambiguous. Some questions it answers itself by reading code
(*"is there already a storage abstraction?"* — go look). The rest it asks you,
batched, with the tradeoffs spelled out and a recommendation attached. It never
silently guesses on anything that would change the work.

**3. Shape.** Recommends **loop**, **graph**, or **hybrid**, with the reasoning
made explicit — how much genuine parallelism exists, whether there's a fast
oracle to iterate against, whether the work must survive interruption, whether
completeness matters. **When there is no clear winner, it says so and presents
both plans side by side with the tradeoffs**, and you choose.

**4. Absorb your conventions.** If you have coding standards, a security review
checklist, house agents, or existing skills, Trestle folds them into the plan as
real steps with real verification — not as advice in a preamble that the agent
forgets by step four.

**5. Show.** Renders the proposal from two perspectives before you commit to it:

- **Work view** — the plan itself: units, dependencies, what can run in
  parallel, where a human decision is required, what proves each unit done.
- **Code view** — your actual codebase as a structural graph, with the plan's
  blast radius highlighted. *These are the files this plan will touch, and these
  are the ones that depend on them.*

The second view is the one that catches bad plans early, because scope creep is
much easier to see than to read.

**6. Write and run.** On approval, the plan is written to a standard folder in
your repo — plain files, in git, reviewable in a PR. Trestle tells you how to
execute it with the agent you already use, and the dashboard shows live status
as the work proceeds.

## Token awareness

Not every unit of work deserves your most expensive model, and a planner that
ignores this quietly wastes money.

Plans declare an **abstract tier** — `cheap`, `standard`, `deep` — never a vendor
model name, so the same plan runs on any harness. Each harness maps tiers to
models it actually has, and a single-model harness says plainly that tiering is
inert rather than pretending it worked.

Before you run anything, Trestle estimates the cost as a **range with its
assumptions stated**, broken down by tier — enough to see whether moving units
between tiers is worth it, or whether the plan's ceremony costs more than the
work it organises. Afterwards it records what was actually spent, per unit, where
the harness reports it, and records `unknown` where it doesn't.

Trestle never bills anything and never sees a bill. All spend is on your own
harness account.

## Scheduling

Unattended runs, one unit per firing, with the backend matched to your setup:

- **`local`** — cron, launchd or a systemd timer firing `trestle run`. Zero
  outbound connections. Fires only while the machine is awake.
- **`cloud-proxy`** — registers a routine with your harness vendor's own
  scheduled-agent service, which runs against a clone of your repo. Survives a
  closed laptop. It involves the vendor you already chose, and Trestle says so
  and asks before arming it.
- **`daemon`** — a foreground process you can watch and stop.

The scheduler is **limit-aware**: it distinguishes rate-limited from quota-
exhausted from not-authenticated, waits until the reset when the harness reports
one, and backs off with jitter when it doesn't. A firing cut short leaves its
unit `todo` — never `done` — so the next firing picks it up cleanly. Resumability
comes from state on disk, not from the scheduler being clever.

It also **refuses to arm a schedule over a plan with no executable work**. If
every unit is gated, blocked or finished, you're told why instead of losing a
night to it.

## What it is not

Trestle **does not talk to any model.** It has no API key, no inference, no
account. Every LLM call is made by *your* agent, on *your* machine, under *your*
existing configuration.

That's the architectural decision the privacy guarantee rests on — see below —
and it's also why Trestle works with whatever you already use instead of asking
you to switch.

## Privacy guarantee

**Trestle makes no outbound network connections. None.**

- No telemetry, no analytics, no crash reporting, no update checks.
- Your source code, your plans, your questions and your answers never leave the
  machine.
- The dashboard binds to `127.0.0.1` only. It is not reachable from your LAN.
- Enforced by an automated egress test in CI, not by a promise in a README.

**What this does not cover, stated plainly:** your coding agent is a separate
program with its own network behaviour. Claude Code, Copilot and Codex all send
code to their respective vendors — that's what they are. Trestle neither adds to
that nor can prevent it. What Trestle guarantees is that *it* adds no new
recipient of your code.

## Harness-agnostic

Trestle drives the agent you already have, by shelling out to it the way you
would from a terminal:

| Harness | Status |
|---|---|
| Claude Code | first target |
| OpenAI Codex CLI | planned |
| GitHub Copilot | planned |
| generic (any CLI that takes a prompt and returns text) | planned |

Adapters are small and contributed as plugins. If your setup can be scripted, it
can be a Trestle harness.

## Status

**Pre-implementation.** The v0.1.0 plan lives in [`plan/v0.1.0/`](plan/v0.1.0/README.md)
and is — appropriately — planned as a dependency graph, with the contract nodes
gated on human decisions. Nothing is built yet.

To work on it, start with [`DEVELOPING.md`](DEVELOPING.md). For the plan itself,
start at [`plan/v0.1.0/README.md`](plan/v0.1.0/README.md), then
[`plan/v0.1.0/decisions.md`](plan/v0.1.0/decisions.md) — seven open questions block
most of the graph, and they're the right place for a second pair of eyes.

## Prior art this is built on

Trestle is an assembly of established ideas, not a new invention:

- **DAG build systems** (Make, Bazel) — targets, prerequisites, staleness.
- **Workflow orchestrators** (Airflow, Dagster, Temporal) — durable state,
  retries, resumability.
- **Test oracles** — the decades-old testing concept that a check external to
  the producer decides correctness.
- **Code property graphs / LSP / tree-sitter** — code as a traversable structure.
- **Agent orchestration graphs** (LangGraph, and the orchestrator-worker and
  evaluator-optimizer patterns) — for what happens *inside* a unit of work.

The contribution is the shaping decision and the local, harness-agnostic
packaging — not the graph theory.

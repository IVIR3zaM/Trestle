# Trestle

**Shape the work before the agent starts.**

Trestle is a local, harness-agnostic planner for AI-assisted engineering work.
Point it at a repository, say what you want to accomplish, and it will read the
code, surface the ambiguities that can't be resolved by reading, ask you about the
ones that matter, and hand back a **plan with a defensible shape** — a loop, a
dependency graph, or a hybrid — along with the reasoning for why that shape fits
this particular job.

You do all of that **inside the agent you already use.** Copilot Chat in VS Code,
Claude Code, Codex — Trestle installs into it once and then gets out of the way.
The plan lands in your repo as plain files, and a local dashboard shows you the
work and its blast radius.

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

## How you use it

Install once:

```bash
brew install trestle
```

Set up a repo once:

```bash
trestle init
```

That suggests the agents it found, **lets you pick the ones you actually want**, and
writes their native integration files — a skill for Claude Code, a chat mode for
Copilot, an `AGENTS.md` block for Codex — plus `.trestle/` for the plans. It shows
you every path it will touch before it touches anything, and `trestle uninstall`
reverses it exactly.

If you run more than one assistant, this is also where you say **who does what**:

```text
  planner      survey, questions, shape, synthesis   [Claude Code]
  implementer  writes the code, runs verify          [Claude Code]
  verifier     independent review of finished work   [Codex]
```

Then **stay in your editor.** In Copilot Chat, or Claude Code, or Codex:

> `/trestle plan: add tenant isolation so every query is scoped to an organisation`

Your agent surveys the repo and asks you the questions it genuinely can't answer by
reading — batched, with tradeoffs and a recommendation, rendered in the chat UI
you're already in. Then it hands you a link:

> Draft plan ready — 15 units, graph shape.
> **http://127.0.0.1:7391/drafts/tenant-isolation**
> Say `approve` when you've looked at it.

The dashboard started itself. You look at the actual graph, the blast radius over
your real modules, the estimate, and every standard it attached. Then:

> `/trestle approve` … `/trestle continue`

one unit at a time, verified against a real command, until it's done. To look in
later:

```bash
trestle status
```

## What it does

**1. Survey.** Reads the repository: languages, module boundaries, real import
and call edges, test commands, CI config, existing conventions. Builds a picture
of the code as a graph, not a pile of files.

**2. Interrogate.** Compares your stated goal against what it found and surfaces
what's genuinely ambiguous. Some questions get answered by reading code (*"is
there already a storage abstraction?"* — go look). The rest come to you, batched,
with the tradeoffs spelled out and a recommendation attached. A question that
doesn't name the work it blocks is rejected before you ever see it.

One question gets asked on every new repo, because scanning can't answer it:
**are there standards this work must follow that don't live in the codebase?** The
security team's review checklist, legal's data-handling policy, the accessibility
bar, a compliance framework. Those documents are owned by other people and versioned
somewhere else, and nobody thinks to mention them unless asked.

**3. Shape.** Recommends **loop**, **graph**, or **hybrid**. Trestle scores the
measurable signals itself — how much genuine parallelism exists, whether there's a
fast oracle to iterate against, whether the work must survive interruption, whether
completeness matters — and the agent has to argue with that baseline rather than
around it. **When there is no clear winner, you get both plans side by side with the
tradeoffs**, and you choose.

**4. Absorb your standards.** Coding standards, a security review checklist, house
agents, existing skills — folded into the plan as real steps with real verification,
not as advice in a preamble that the agent forgets by step four. A rule like
*"anything touching data access must pass `npm run sec:authz`"* becomes an extra
oracle on every unit it matches, indistinguishable at verification time from the
unit's own, and carrying a citation back to the clause that caused it.

Size is not a problem. A 300-page company standard is read in chunks and distilled
into a short, pinned rule set that lands **in your repo**, reviewable in a PR. The
source is fingerprinted, so when the security team ships v5 while you're mid-plan,
you find out from a status line rather than from an audit.

Three kinds of honesty come out of that, and they're the point:

- **What can't be enforced.** A rule with no command behind it is a wish, and it's
  labelled one rather than implying a check.
- **What doesn't apply.** *"§44 accessibility — no UI surface in this work."*
  Silently omitting those looks identical to having missed them.
- **What your standard asks for and your repo doesn't have.** *"§23.3 requires
  `npm run secrets:scan` — not in package.json."* Often the standard is right and
  the repo is behind, and you'd rather know now than at 2am.

**5. Show.** The dashboard starts itself and your agent hands you a link straight to
the draft. Three views:

- **Work view** — the plan itself: units, dependencies, what can run in
  parallel, where a human decision is required, what proves each unit done. Graphs
  render as graphs, loops as ordered queues with their journal, hybrids as neither
  pretending to be the other.
- **Code view** — your actual codebase as a structural graph, with the plan's
  blast radius highlighted. *These are the files this plan will touch, and these
  are the ones that depend on them.*
- **Workflow view** — if you run several assistants, which one holds which unit and
  what's queued for each role. Shown only if you configured more than one.

The code view is the one that catches bad plans early, because scope creep is
much easier to see than to read.

The dashboard is deliberately **read-only** — you look here and decide in your chat.
Trestle doesn't drive your agent, so an Approve button could flip a flag but couldn't
start the work, and splitting one action across two surfaces is worse than the single
word it would save.

**6. Write and verify.** The plan is written to `.trestle/plans/` — plain files, in
git, reviewable in a PR. As the work proceeds, **`trestle verify` runs each unit's
oracle itself** and is the only thing that can mark a unit done.

## The architecture, in one paragraph

**Trestle performs no inference, and it does not drive your agent — your agent
drives Trestle.** Trestle ships prompts that teach your agent the workflow, and a
set of deterministic commands it calls as tools: survey the code, score the shape,
validate a plan, compute what's ready, run an oracle. Every model call is your
agent's, on your machine, under your existing configuration and your existing bill.

Two things fall out of that, and they're the reason it's built this way:

**Editor-only agents work.** The VS Code Copilot extension isn't a program you can
shell out to. A planner that drives agents from a terminal simply cannot support it.
One that agents call *into* supports it natively.

**Progress can't be faked.** There is no `trestle record --done`. The only way a
unit becomes `done` is `trestle verify` running the unit's oracle command and
watching it succeed — so *"the agent should verify its work"* is a mechanism rather
than a hope. An oracle is by definition external to the thing it's checking, and
this keeps it that way.

Honest limits of that second claim: an override exists, because a mis-specified
oracle has to be fixable by a human who says so out loud. It records a distinct
state, permanently, in a file that's in git. It is *loud*, not *prevented* — your
agent runs commands in your own shell, and no tool can honestly claim otherwise.

**Multiple agents make it stricter, never looser.** If you configure a second
assistant as a `verifier`, a passing oracle produces `verified` rather than `done`,
and the reviewer clears it. A reviewer can **withhold** `done` and never confer it —
there's no path from `todo` to `done` through review, so a second agent's opinion
can't grant what the oracle refused. With no verifier configured, none of this
exists and nothing changes.

## Token awareness

Not every unit of work deserves your most expensive model, and a planner that
ignores this quietly wastes money.

Plans declare an **abstract tier** — `cheap`, `standard`, `deep` — never a vendor
model name, so the same plan runs on any harness. Where your agent supports
subagents, the tier maps to a real model choice. Where it doesn't, **Trestle says
plainly that tiering is inert** rather than letting you believe it worked.

Before you run anything, `trestle plan estimate` gives a cost **range with its
assumptions stated**, broken down by tier — enough to see whether moving units
between tiers is worth it, or whether the plan's ceremony costs more than the work
it organises.

**What Trestle cannot tell you: what you actually spent.** It never sees a request,
a response, or a bill — that's the same property that makes the privacy guarantee
real. Actual usage records as `unknown`, and that's a stated limitation, not an
empty column waiting to fill in.

Trestle never bills anything. All spend is on your own harness account.

## What it is not

Trestle **does not talk to any model.** It has no API key, no inference, no
account.

It is also not an orchestrator. It doesn't launch your agent, manage sessions, or
run anything overnight — see below.

## Privacy guarantee

**Trestle makes no outbound network connections. None.**

- No telemetry, no analytics, no crash reporting, **no update checks** — not
  opt-in, not weekly. `trestle --version` prints a version; it doesn't ask anyone
  whether that version is current.
- No HTTP client in the dependency tree at all. The strongest guarantee is the
  absence of the capability, not the discipline to leave it alone.
- Your source code, your plans, your questions and your answers never leave the
  machine.
- The dashboard binds to `127.0.0.1` only, and it's the only listener in the
  product — the MCP server is stdio, no socket. It is not reachable from your LAN.
  It starts itself when a draft is written; that's announced every time, the port is
  recorded, it exits when idle, `trestle ui --stop` kills it, and you can turn the
  behaviour off entirely.
- Dashboard assets are compiled into the binary, so there is no CDN to reach for
  and no font to fetch.
- Enforced by an automated egress test in CI, with a planted violation to prove
  the guard can fail. Not by a promise in a README.

Because Trestle never spawns your agent, the egress test needs **no subprocess
exemption at all** — the sandbox around it can be total.

`trestle init` does write files outside `.trestle/`; that's how it installs into
your agent. Every path is declared, shown to you before it's written, wrapped in
markers that leave your own content untouched, and reversible with
`trestle uninstall`. That's tested the same way the network guarantee is.

**What this does not cover, stated plainly:** your coding agent is a separate
program with its own network behaviour. Claude Code, Copilot and Codex all send
code to their respective vendors — that's what they are. Trestle neither adds to
that nor can prevent it. What Trestle guarantees is that *it* adds no new
recipient of your code.

## Harness-agnostic

Trestle installs into the agent you already have:

| Harness | How | Status |
|---|---|---|
| Claude Code | skill + MCP server | first target |
| GitHub Copilot (VS Code) | chat mode + instructions + MCP server | first target |
| OpenAI Codex | `AGENTS.md` block + MCP server | first target |
| generic (any agent that reads `AGENTS.md`) | instructions only | fallback |

Detection is a suggestion, never a verdict: install any subset, including ones it
didn't find and excluding ones it did. Use one, or use several in different roles.

An integration is a manifest plus templates — which files to write where, which
roles it can serve, and what that harness can do. **Adding one requires no Rust and
no knowledge of Trestle's internals.**

## Not in v0.1.0

**Unattended runs.** Nothing pokes your agent at 3am, so scheduling, limit-aware
backoff and `trestle run` are [deferred to v0.2.0](plan/v0.2.0/README.md) along
with their full specifications. The property they depend on — that resumability
comes from state on disk, and readiness is recomputed rather than remembered —
holds in v0.1.0 regardless, so they drop in rather than requiring a rework.

Also deferred: observed token usage, dashboard control, multi-repo plans.

## Status

**Pre-implementation.** The v0.1.0 plan lives in [`plan/v0.1.0/`](plan/v0.1.0/README.md)
and is — appropriately — planned as a dependency graph, with the contract nodes
gated on human decisions. Nothing is built yet.

To work on it, start with [`DEVELOPING.md`](DEVELOPING.md). For the plan itself,
start at [`plan/v0.1.0/README.md`](plan/v0.1.0/README.md), then
[`plan/v0.1.0/decisions.md`](plan/v0.1.0/decisions.md) — eight of fourteen decisions
are resolved, and `D2` (the plan format) blocks most of what remains.

Written in Rust and shipped as a single static binary, because Trestle plans
*other people's* repositories and shouldn't make you install a runtime you don't
otherwise want.

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

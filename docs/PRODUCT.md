# The product contract

What Trestle is, what each step of it is allowed to assume, and what it must never
do. Every node in [`../plan/v0.1.0/`](../plan/v0.1.0/README.md) is checked against
this document; where a node and this file disagree, one of them is wrong and that is
worth a line in `decisions.md` rather than a silent choice.

The privacy half of the contract is [`THREAT-MODEL.md`](THREAT-MODEL.md), which
enumerates every channel by which code could leave the machine. This file is the
behavioural half.

---

## In one paragraph

Trestle is a local planner for AI-assisted engineering work. It reads a repository,
surfaces the ambiguities that reading cannot resolve, recommends whether the work
wants to be a **loop**, a **graph**, or a **hybrid**, folds the user's own standards
in as real verification steps, shows the result from a work perspective and a code
perspective, and writes a plan into the repo as plain files. The user does all of it
from inside the coding agent they already have.

## The three invariants

These three hold everywhere, in every command, at every tier. A change that breaks
one of them is not a feature with a tradeoff; it is a different product.

1. **No inference** (`D0`) — Trestle never calls a model. It has no API key and no
   account. Its product surface is prompts, schemas, validation and deterministic
   analysis; the reasoning is the user's agent's.
2. **Inverted control** (`D5`) — Trestle does not drive the agent. The agent drives
   Trestle, from inside its own interface, calling a deterministic command surface
   as tools.
3. **Unforgeable progress** (`D9`) — `trestle verify` runs the oracle command itself
   and is the only writer of `done`. The agent's claim of success is not an accepted
   input.

Each is stated in full below, with its limits. The limits are part of the invariant:
an overclaimed guarantee is worse than a stated one, because only the second gets
checked.

## The six steps

Under `D5` the agent and Trestle alternate, and a reader who gets that backwards
will design the next component backwards — so ownership is a column, not a footnote.
"Trestle" means deterministic code in this binary. "The agent" means the user's
harness, reasoning against a prompt Trestle shipped.

| Step | Whose step it is | What it is allowed to assume |
|---|---|---|
| 1. Survey | **Trestle** (`trestle survey`) | Only the repository on disk. No goal, no user answers, no plan. Output is labelled partial wherever extraction was heuristic. |
| 2. Interrogate | **The agent**, asking **the user**; Trestle stores and lints the questions | A completed survey. That the repo has already been consulted — a question the code could answer is rejected before the user sees it. |
| 3. Shape | **Trestle scores** (`trestle shape`), the **agent argues with the score** | A survey and the answered decisions. That its own baseline is deterministic and reproducible, so the agent's disagreement is visible as a disagreement. |
| 4. Absorb | **The agent** proposes, **Trestle validates** (`trestle conventions`, `trestle standards`) | A chosen shape. That every rule it attaches is either backed by a command or labelled unenforceable. |
| 5. Show | **Trestle** (`trestle ui`, auto-started on the first draft) | A written draft. That it is read-only: the user looks here and decides in the chat. |
| 6. Write and run | **The agent** calls `trestle plan write`, `trestle next`, `trestle verify`; **`verify` is Trestle's alone** | A validated plan. Never that a unit is done because the agent says so. |

What no step may assume: that a later step will catch its mistakes. Each step
validates its own output, because under `D5` there is no orchestrator above them to
notice.

The human's own surface is three commands:
`trestle init` once, then `trestle status` and `trestle ui` whenever they feel like
looking. Everything else in the command surface exists for the agent to call.

---

## Invariant 1 — No inference (`D0`)

Trestle performs no inference. **No API key, no model call, no account, no vendor
SDK.** Every "intelligent" step in the six above is reasoning the user's agent
performs, on the user's machine, under the user's existing configuration and against
the user's existing bill.

This is not a modesty claim about capability. It is what makes the privacy guarantee
true: a Trestle that held an API key would be a second recipient of the user's
source, and [`THREAT-MODEL.md`](THREAT-MODEL.md) would be fiction.

Three consequences that are requirements, not observations:

- **Trestle's product surface is prompts, schemas, validation and deterministic
  analysis.** Anything that would have been a judgement call inside Trestle's code
  is instead a check on the artifact the agent produced. A rule with no check behind
  it is a sentence in a prompt that a tired model will skip.
- **Validate, and reject rather than coerce.** Output quality varies by harness and
  by model. A malformed plan produces errors the agent can iterate against; it never
  produces a plan Trestle quietly repaired.
- **Trestle cannot know what the work cost.** It never sees a request, a response or
  a bill. Estimates before, `unknown` after — see [Not in v0.1.0](#not-in-v010).

## Invariant 2 — Inverted control (`D5`)

Trestle does not drive the agent; **the agent drives Trestle.** Trestle ships
integration files that teach a harness the workflow, plus a deterministic command
surface the agent calls as tools. Trestle spawns no agent, opens no session, and
holds no conversation.

Two things this buys, and they are the reason it is built this way:

- **Editor-only harnesses are first-class.** A VS Code chat extension is not a
  shellable program. Any design where Trestle reaches down to a harness CLI excludes
  it outright; one the harness calls into supports it natively.
- **Interrogation happens in a real chat surface** — rendered markdown, history, the
  ability to push back on a question — rather than in a terminal prompt.

What it costs, stated where it cannot be mistaken for a feature:

- **Trestle cannot choose which model runs a unit.** Plans declare an abstract tier
  (`cheap`, `standard`, `deep`) and **never a vendor model name**, so a plan stays
  portable. Where the harness has subagents, the tier maps to a real choice; where it
  does not, **tiering is inert and Trestle says so** rather than letting the user
  believe it worked.
- **Trestle cannot observe token usage at all.**
- **Prompt quality is not unit-testable.** There is no mock harness, because there
  is no harness call to mock. Prompt quality is measured by dogfooding and by the
  strictness of the validators. That is a real reduction in automated coverage and
  it is not papered over.
- **The code view carries more weight than it used to.** When a planner drives the
  agent it can refuse to dispatch an over-broad unit. Trestle can only show it. The
  code view is where a human catches what the tool can no longer prevent.

**No capability may exist only over MCP.** The CLI is the substrate; `trestle mcp`
is a stdio wrapper over the same commands. Otherwise harnesses that speak no MCP
become second-class, and the egress test gains a surface it cannot reach.

## Invariant 3 — Unforgeable progress (`D9`)

**There is no way to write `done` except `trestle verify` running the unit's oracle
command and observing it succeed.**

No `trestle record --done`. No `--assume-pass`. The agent's report is not an accepted
input, because the agent is the producer, and an oracle is by definition external to
the producer. This is the mechanism that keeps *no oracle, no node* true when Trestle
is not the one driving.

**Where multiple agents are configured, this gets stricter, never looser** (`D14`). A
configured `verifier` means a passing oracle produces `verified`, and review clears
it to `done`. Review can only ever **withhold** `done`, never confer it:

```
todo ──oracle passes──▶ verified ──review passes──▶ done
  ▲                                    │
  └────────── review fails ────────────┘   (reason recorded)
```

There is no path from `todo` to `done` through review, so a second agent cannot
grant what the oracle refused. With no verifier configured, `verified` and `done` are
the same state and none of this is visible.

### The limit of this claim

An **override** exists: `trestle verify --override --reason <text>` records a
distinct, permanent `done(overridden)` state that the dashboard renders differently
and `trestle status` counts separately — always showing the count, even when it is
zero, so the absence of overrides is visible too.

It exists because a mis-specified oracle has to be fixable by a human who says so out
loud, and a tool with no escape hatch gets worked around in ways that leave no trace.

**It is loud, not prevented.** The agent runs commands in the user's own shell; no
tool in that position can honestly claim to prevent anything. What protects the user
is that the override is recorded in a file that is in git, not that it is hard to
perform. Documentation that implies otherwise is a bug in the documentation.

---

## What Trestle is not

- **Not an orchestrator.** It does not launch the agent, manage sessions, or run
  anything overnight.
- **Not a runtime.** One static binary; no service, no database, no daemon beyond the
  loopback dashboard, which announces itself and exits when idle.
- **Not a second recipient of the user's code.** See
  [`THREAT-MODEL.md`](THREAT-MODEL.md).

## The boundary, stated plainly

The user's coding agent is a separate program with its own network behaviour, and
harnesses send code to their vendors — that is what they are. **Trestle adds no new
recipient of the user's code, and cannot remove the one the user already chose.**

Under `D5` the agent is now the thing *calling* Trestle rather than the thing Trestle
calls. That changes nothing about what the agent sends to its vendor, and **the
user-facing copy must not imply otherwise.** One consequence deserves saying out
loud: Trestle's own output — the survey, the code graph, the questions — enters the
agent's context when the agent calls it, and therefore goes wherever that agent's
context goes. Trestle emits nothing itself; it can still hand the agent something the
agent transmits.

## Not in v0.1.0

Listed here so nodes do not quietly grow into them. Each is deferred with its
specification intact — see [`../plan/v0.2.0/README.md`](../plan/v0.2.0/README.md).

- **Nothing unattended.** Nothing pokes the agent at 3am.
- **No scheduling** of any kind, and no `trestle run`.
- **No observed token usage.** Actual usage records as `unknown`, permanently, in
  v0.1.0. It is a stated limitation, not an empty column waiting to be filled in.
- **No dashboard control.** The dashboard is read-only (`D13`): an Approve button
  could flip a flag but could not start the work, and splitting one action across two
  surfaces is worse than the word it would save.
- **No multi-repo plans.**

The property the deferred work depends on — **resumability comes from state on disk,
and readiness is recomputed rather than remembered** — holds in v0.1.0 regardless, so
that work drops in rather than requiring a rework.

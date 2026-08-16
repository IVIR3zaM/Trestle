# Handoff context

Self-contained briefing for picking Trestle up in a fresh session. Read this,
then `README.md`, then `plan/v0.1.0/README.md`. You should not need the
conversation this came from.

## What Trestle is, in one paragraph

A local, harness-agnostic planner for AI-assisted engineering work. Point it at a
repo, state a goal; it surveys the code, surfaces genuine ambiguities (answering
what it can by reading, asking about the rest), recommends whether the work
should be shaped as a **loop**, a **graph**, or **both with tradeoffs**, folds in
the user's own standards and agents, shows the proposal from a work perspective
and a code perspective, and on approval writes a plan into the repo and tells the
user how to run it with the agent they already have. A local dashboard shows
status. Nothing leaves the machine.

## Where this came from

Trestle generalises two ways of running long agent work that both turned out to
work in practice. Both are written up in full, with worked examples, inside this
repository — **read these before designing anything:**

- **[`docs/PRIOR-SHAPES.md`](docs/PRIOR-SHAPES.md)** — what the graph shape and
  the loop shape are, the artifacts each needs, the properties each has, the
  costs of each, and the rules that make them work. Includes the failure modes
  observed in real use.
- **[`fixtures/source/`](fixtures/source/)** — a worked example of each shape,
  written in the native style people use today rather than in Trestle's format.
  These are the input corpus the plan format must be able to express without
  loss.

Two observations drove the product, and neither is obvious until you have run
both:

- **A working loop is far more structured than the naive picture of one** —
  ordered phases, explicit `blocked(user)` states, a fixed-format append-only
  journal, and superseded rules marked in place rather than deleted. Any format
  that treats a loop as "a graph without edges" will lose the journal, which is
  the loop's entire mechanism for carrying discovery forward.
- **Neither shape is universally right, and choosing badly is expensive in both
  directions.** Nothing tells you which to use. That gap is the product.

## The one architectural decision already made

**Trestle performs no inference.** It has no API key and makes no model calls.
Every "intelligent" step is a prompt Trestle constructs, handed to the *user's*
already-configured agent, plus a parser and schema validator for what comes back.

This falls directly out of the hard privacy requirement — a Trestle that held an
API key would be a second recipient of the user's source. It also means Trestle's
real product surface is **prompts, schemas and validation**, not reasoning. Recorded
as `D0` in `plan/v0.1.0/decisions.md`.

## Vocabulary

| Term | Meaning |
|---|---|
| **Node** | one unit of work, one file, self-contained |
| **Edge** (`deps`) | "this can't start until that is done" |
| **Oracle** | the command that decides done — a test, a compiler, a validator. Never the agent's opinion. |
| **Gate** (`gate: human`) | a node an unattended agent must not attempt |
| **Tier** | how much thinking a node is worth — `cheap`/`standard`/`deep`, resolved per harness, never a vendor model name |
| **Decision** | a question only a human can answer, naming the nodes it blocks |

Rules carried over, and worth keeping: *no oracle, no node*; *never edit an
oracle to make it pass*; *nodes are contracts, not tasks*; *one node per pass,
then stop*.

## Current state

Nothing is built. `plan/v0.1.0/` has 22 nodes; `make status` works and reports
`0/22 done · 1 ready (0 unattended, 1 gated)`.

That reading is correct and expected — T01 is a human gate and everything
descends from it. **A graph whose contract units are all gated reports zero
executable work, and this is normal early in a plan's life.** It is also exactly
why you check readiness before arming any schedule: arming over an unrunnable
plan wastes a night. T21 makes that check mandatory.

## What to do first

Answer the decisions in `plan/v0.1.0/decisions.md`. Seven are open; three block
most of the graph:

- **D2 — one plan format or two?** The most consequential decision in the
  project; every component reads or writes this format. Recommendation: one
  schema with a `shape:` discriminator. Do **not** collapse a loop into a
  chain-shaped graph — that loses the journal, which is how a loop carries
  discovery forward.
- **D5 — does Trestle orchestrate execution, or emit instructions and read
  status?** Defines what v0.1.0 actually is. Recommendation: emit for v0.1.0,
  design the status format so orchestration can be added later. If this lands on
  "emit", v0.1.0 shrinks by about a third and the plan's own shape should be
  reconsidered.
- **D1 — which harnesses ship?** Recommendation: two, so the abstraction is
  tested rather than assumed.

Then execute T01 (product contract + threat model) interactively — it's gated for
a reason. After that, T04, T05 and T16 branch off and can run in parallel.

## Things not to get wrong

- **The rubric must be willing to say "loop".** A tool that always recommends a
  graph is worthless, and will be uninstalled the first time it costs someone a
  morning on a two-hour task. T18 tests exactly this against a known-good loop.
- **The privacy guarantee needs a test, not a promise.** T16 should land early;
  it's cheap and everything after inherits it. Include a planted-violation test —
  a guard never seen to fail is not known to work.
- **The dashboard is a viewer, not a workflow engine.** The requirement is
  "simple dashboard showing the plans nicely from different perspectives".
  Embedding n8n brings a service, a data model, and a licence for a feature
  that's a v2 bonus. (`D4`.)
- **The code view is the differentiator.** Showing blast radius over a real
  module graph is what catches over-broad plans before they run. Most planning
  tools show only the work.
- **Don't ask what the repo can answer.** Reading the code to resolve an
  ambiguity instead of asking is the difference between the tool feeling sharp
  and feeling like a form.

## Not yet done

- `git init` — this folder is not a repository yet.
- Licence, `CONTRIBUTING.md`, code of conduct.
- **Verify the name is free** on npm and GitHub. "Trestle" was chosen for meaning
  (the frame you build first so the real thing has something to rest on) and
  checked only against sibling folders on this machine.

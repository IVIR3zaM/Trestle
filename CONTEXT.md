# Handoff context

Self-contained briefing for picking Trestle up in a fresh session. Read this,
then `README.md`, then `plan/v0.1.0/README.md`. You should not need the
conversation this came from.

## What Trestle is, in one paragraph

A local, harness-agnostic planner for AI-assisted engineering work. Point it at a
repo, state a goal; it surveys the code, surfaces genuine ambiguities (answering
what it can by reading, asking about the rest), recommends whether the work should
be shaped as a **loop**, a **graph**, or **both with tradeoffs**, folds in the
user's own standards and agents, shows the proposal from a work perspective and a
code perspective, and writes a plan into the repo. A local dashboard shows status.
Nothing leaves the machine.

**The user does all of this from inside the agent they already have** — Copilot Chat
in VS Code, Claude Code, Codex. Trestle is installed once with `trestle init` and
then called *by* the agent, not the other way round.

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

## The three architectural decisions already made

### 1. Trestle performs no inference (`D0`)

No API key, no model calls. Every "intelligent" step is reasoning the *user's*
agent performs, against prompts Trestle ships and schemas Trestle validates.

This falls directly out of the hard privacy requirement — a Trestle that held an
API key would be a second recipient of the user's source.

### 2. Control is inverted (`D5`)

Trestle doesn't drive the agent either. **The agent drives Trestle**, calling a
deterministic command surface as tools:

```
trestle survey --json          code graph, discovered oracles, measured shape signals
trestle conventions --json     in-repo rules, classified by enforceability
trestle standards ingest|check external policy documents, chunked and pinned
trestle shape --json           deterministic baseline shape recommendation
trestle decisions add|answer   the question store
trestle plan validate|write    the gauntlet, then an atomic non-clobbering write
trestle plan amend             additive re-planning of a live plan
trestle next --json [--role R] computed ready set / queue position, per role
trestle verify <unit>          runs the oracle, records the result
trestle review <unit>          reviewer veto; can withhold done, never grant it
trestle journal append         validated loop-journal entry
```

The human runs `trestle init`, and after that `trestle status` / `trestle ui` only
when they feel like looking — the dashboard starts itself on the first draft write
and the agent hands over a deep link (`D13`). Plus `trestle mcp`, a stdio wrapper
over the same commands, which the harness spawns.

**Why it matters more than it sounds:** the previous design had Trestle shelling out
to a harness CLI. That excluded the VS Code Copilot extension entirely — it isn't a
shellable program, and *every* planning step needed a live call. It also made the
terminal the interface for interrogation, which is the product's first impression
and the step that benefits most from a rich chat surface.

**Trestle's real product surface is therefore prompts, schemas, validation, and
deterministic analysis.** Every rule that would have lived in synthesis code is now
a check on the written artifact — because a rule with no check is a sentence in a
prompt that a tired model will skip.

### 3. Rust, single static binary (`D6`)

Trestle plans *other people's* repos. Requiring a Node runtime in a Python or Go
shop is backwards for a tool whose pitch is that it fits what you already have.
`brew install trestle`, a `curl | sh` installer, `cargo binstall` — install once,
reuse the binary. Rust specifically for first-class tree-sitter bindings, a truly
static binary, and `cargo-dist` generating the release plumbing.

## The invariant that holds the whole thing together

> There is no way to write `done` except `trestle verify` running the unit's
> oracle command and observing it succeed.

No `trestle record --done`. No `--assume-pass`. The agent's report is not an
accepted input, because the agent is the producer and an oracle is by definition
external to the producer. This is what makes agent-driven execution safe where
"emit a plan and trust the executor" would not have been. See T11 — it's the reason
that node is `tier: deep`.

The override (`--override --reason`) exists because a mis-specified oracle must be
fixable by a human who says so out loud. It records a distinct permanent state. It
is **loud, not prevented** — say that in the docs; the agent runs commands in the
user's own shell and no tool can honestly claim otherwise.

**Multi-agent setups make this stricter, never looser** (`D14`). A configured
`verifier` means a passing oracle produces `verified`, and review clears it to
`done`. Review can only ever *withhold*: there is no path from `todo` to `done`
through review, so a second agent cannot grant what the oracle refused. Getting this
backwards would reintroduce the `D9` hole one level up, which is why T11 owns both
commands rather than splitting them across nodes.

## What inverting control cost

Be honest about these; three nodes exist mainly to state them accurately:

- **T19** — Trestle can't choose a model. Tiers are advisory hints; where the
  harness has no subagents, tiering is *inert* and must be reported as such.
- **T20** — Trestle can't observe token usage at all. Estimates before, `unknown`
  after. `D11` records the options for getting it back in v0.2.0.
- **T18** — the `MockHarness` is gone with the architecture that needed it, and with
  it the only way prompt quality could have been regression-tested. Dogfooding is
  now the sole measurement of the product's most important behaviour. Budget for it.

## Vocabulary

| Term | Meaning |
|---|---|
| **Node** / **unit** | one unit of work, one file, self-contained |
| **Edge** (`deps`) | "this can't start until that is done" |
| **Oracle** | the command that decides done — a test, a compiler, a validator. Never the agent's opinion. |
| **Gate** (`gate: human`) | a node an unattended agent must not attempt |
| **Tier** | how much thinking a node is worth — `cheap`/`standard`/`deep`, advisory under `D5` |
| **Decision** | a question only a human can answer, naming the nodes it blocks |
| **Integration** | the files that teach one harness the Trestle workflow — a manifest plus templates, not code |
| **Role** | `planner` / `implementer` / `verifier`; a property of the user's setup, never of the plan |
| **Gauntlet** | T07's validator set; the checks a synthesised plan must survive to be written |
| **Distillation** | the small pinned rule set T27 extracts from a large external standards document |

Rules carried over, and worth keeping: *no oracle, no node*; *never edit an
oracle to make it pass*; *nodes are contracts, not tasks*; *one node per pass,
then stop*.

## Current state

Nothing is built. `plan/v0.1.0/` has 25 nodes; `make status` reports
`0/25 done · 1 ready (0 unattended, 1 gated)`.

That reading is correct and expected — T01 is a human gate and everything
descends from it. **A graph whose contract units are all gated reports zero
executable work, and this is normal early in a plan's life.** T10 must report it as
a first-class answer rather than as a failure.

`plan/v0.2.0/` holds the deferred unattended lane (T21, T22) with its
specifications intact.

## What to do first

**Answer `D2`** in `plan/v0.1.0/decisions.md` — one plan format or two. It blocks
most of the graph, and `D5` raised its stakes: the *agent* now writes this format,
so the schema must be strict enough that a plausible-looking bad plan fails, and its
error messages are an interface the agent converges against rather than a nicety.

Recommendation stands: one schema with a `shape:` discriminator. Do **not** collapse
a loop into a chain-shaped graph — that loses the journal.

`D3` (tree-sitter vs LSP) blocks T05 and T15. `D9`–`D12` are scoped to single nodes
and can wait until those nodes come up.

Then execute T01 (product contract + threat model) interactively — it's gated for a
reason, and its channel table is what T16 turns into tests. It now needs four
channels the original didn't have: the MCP server, `trestle init` writing outside
`.trestle/`, embedded assets, and the absence of update checks.

After T01, three tracks open in parallel — T04, T05 and T16.

## Things not to get wrong

- **The rubric must be willing to say "loop".** A tool that always recommends a
  graph is worthless, and will be uninstalled the first time it costs someone a
  morning. T03 asserts that at least a third of its fixture corpus comes out
  `loop`, so the guard is in CI rather than in a prompt.
- **The privacy guarantee needs a test, not a promise.** T16 should land early;
  it's cheap and everything after inherits it. Include planted violations — a guard
  never seen to fail is not known to work. There are now two guarantees to plant
  against: outbound connections, and writes outside declared paths.
- **`trestle init` must be idempotent and reversible.** It writes into files humans
  own. Running it twice must produce the same tree; uninstalling must restore the
  original. This is the hard part of T23 and the reason T04 limits emit modes to
  three.
- **The dashboard is a viewer, not a workflow engine.** (`D4`, resolved.)
- **The code view is the differentiator, and it matters more now.** When Trestle
  drove the agent it could refuse to dispatch an over-broad unit; now it can only
  advise. The code view is where the human catches what the tool can no longer
  prevent.
- **Don't ask what the repo can answer.** Reading the code to resolve an ambiguity
  instead of asking is the difference between the tool feeling sharp and feeling
  like a form. T06 lints the crudest cases; the rest is prompt work measured in T18.
- **Do ask what the repo cannot answer.** The standards elicitation (T06) is the one
  question the agent must ask unprompted, because a policy document owned by another
  team is invisible to any scan and users don't think to mention it. Its answer is
  remembered per repo and *confirmed* thereafter — asking it every time is how a
  tool teaches people to ignore it.
- **The multi-agent feature must cost single-agent users nothing.** One harness
  holding all three roles must produce a tree, a state machine and a dashboard
  indistinguishable from one with no role model at all. T04, T10, T11, T14 and T23
  each assert this from their own side.
- **No capability may exist only over MCP.** The CLI is the substrate; the MCP
  server is a wrapper. Otherwise the no-MCP harnesses become second-class and T16
  gains a surface it can't test.

## Not yet done

- `CONTRIBUTING.md` and a code of conduct. **Licensed Apache-2.0** — see `LICENSE`.
- The remote is `github.com/IVIR3zaM/Trestle`. Per `DEVELOPING.md` the executor's
  commit policy moves to one branch per node once there are contributors; while it
  is solo, `main` is fine.
- **Verify the name is free** on crates.io, Homebrew and GitHub. "Trestle" was
  chosen for meaning (the frame you build first so the real thing has something to
  rest on) and checked only against sibling folders on this machine. Note the
  registry to check changed with `D6` — crates.io, not npm.

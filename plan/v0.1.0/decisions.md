# Open decisions — Trestle v0.1.0

Questions that must not be answered by an unattended agent. Each names the nodes
it blocks. Resolve by appending the answer and marking `RESOLVED <date>`, then
unblock the node in `graph.yaml`.

**All seventeen are resolved** (`D0`–`D6`, `D8`–`D17`) and one is deferred to v0.2.0
(`D7`). Nothing blocks the graph.

The two that shaped everything else are `D5` (control is inverted — the agent drives
Trestle) and `D2` (one plan format with a `shape:` discriminator). Read those first
if you are picking this up cold.

---

## D0 — Does Trestle call a model itself, or drive the user's agent?

**Blocks:** T01, T04 — **RESOLVED: neither. Trestle performs no inference, and
(per `D5`) does not drive the agent either — the agent drives Trestle.**

Forced by the hard privacy requirement. If Trestle held an API key it would
become a second recipient of the user's source, and the no-egress guarantee would
be a lie.

Three consequences that shape everything downstream:

- Trestle's own binary can be verified as network-silent, and that verification
  is meaningful (T16).
- Every "intelligent" step — ambiguity detection, plan synthesis, shape
  recommendation — is **reasoning the user's agent performs**, against prompts
  Trestle ships and schemas Trestle validates. Trestle's real product surface is
  **prompts, schemas, validation, and deterministic analysis** — not reasoning.
- Output quality varies by harness and model. Trestle must degrade honestly:
  validate everything the agent produces against a schema, and reject rather than
  coerce.

---

## D1 — Which harness integrations ship in v0.1.0?

**Blocks:** T04, T23 — **RESOLVED 2026-08-17: three, plus a `generic` fallback.
Detection is advisory; the user selects, and may assign roles (`D14`).**

The original framing of this question assumed an adapter was *code that invokes a
model*. Under `D5` an integration is a **manifest plus templates** — which files
to write where, and what that harness's own conventions are. That makes the
marginal cost of the third integration small, and the argument for shipping more
than one (that a single implementation cannot validate an abstraction) still
holds and now costs less to satisfy.

Shipping:

| Integration | Emits | Notes |
|---|---|---|
| `claude-code` | `.claude/skills/trestle/SKILL.md`, `.mcp.json` entry | subagents exist, so tier mapping is real here |
| `copilot` | `.github/chatmodes/trestle.chatmode.md`, marked block in `.github/copilot-instructions.md`, `.vscode/mcp.json` | the VS Code case; **no CLI required** |
| `codex` | marked block in `AGENTS.md`, MCP entry in the Codex config | |
| `generic` | marked block in `AGENTS.md` only | documented fallback, no MCP assumed |

`copilot` is deliberately the *editor* integration, not a CLI one. The previous
design could not support it at all — see `D5`.

**Detection does not decide.** `trestle init` shows what it found and lets the user
install any subset, including harnesses it did not detect and excluding ones it
did. Someone with Copilot installed who wants only Claude Code configured must get
exactly that, and someone running two harnesses in different roles must get both
(`D14`). A detector that guesses and can't be overridden is worse than no detector.

---

## D2 — One plan format, or one per shape?

**Blocks:** T02, and through it most of the graph — **RESOLVED 2026-08-17: (a) one
schema with a `shape:` discriminator and optional per-shape sections.**

The format has to express three things that look different on the surface:

- a **graph** (nodes, dependency edges, per-node oracle, human gates)
- a **loop** (an ordered queue, an append-only journal, `blocked(user)` states) —
  see `fixtures/source/loop-shape/` for a worked example
- a **hybrid**, which is what most real work turns out to be

Rejected: **(b)** two schemas and a converter — doubles the surface every consumer
must handle, and the converter becomes a second place for the format to be wrong.
**(c)** one schema where a loop is a graph whose nodes form a chain — elegant and
wrong. It loses the journal, which is the loop's actual mechanism for carrying
discovery forward, and it would push users toward graphs by making loops feel
second-class. A tool whose format makes one of its two answers awkward will stop
giving that answer.

**The shape of the resolution:**

- A required top-level `shape: graph | loop | hybrid`.
- **Shared spine, always present**: units with ids, titles, `done_when` contracts,
  oracles or gates, tiers, and oracle provenance (T02 §8).
- **`graph` adds** dependency edges. Readiness is computed from them.
- **`loop` adds** an ordered queue and a journal reference. **The journal is not
  optional** — a loop plan without one fails validation, because a loop with no
  journal is just a list someone will lose track of.
- **`hybrid` is a graph whose units may each carry a queue and journal.** Not a
  third schema; a graph unit that is itself iterated. This is what most real work
  turns out to be, so it must not be the case that renders or validates worst.
- Unknown keys are ignored on read and preserved on round-trip, so v0.2.0 can add
  fields without invalidating existing plans.

**This remains the most consequential node in the project.** Every other component
reads or writes this format, and under `D5` the *agent* writes it too — so the
schema must be strict enough that a plausible-looking bad plan fails, and its error
messages are an interface the agent converges against, not a nicety.

---

## D3 — How is the code graph extracted?

**Blocks:** T05, T15 — **RESOLVED 2026-08-17: (a) tree-sitter, with (c) heuristics
as the fallback for unsupported languages. The bar is *useful*, not *accurate*.**

Rejected: **(b)** LSP — semantically accurate, but needs a running language server
per language, and is slow and fragile to set up. Buying a semantically perfect call
graph costs more than the feature is worth in v0.1.0, and it would make the survey's
reliability depend on the user's toolchain being healthy.

`D6` landing on Rust strengthens (a): tree-sitter's Rust bindings are first-class
and link the C library without the static-linking friction cgo would have
introduced.

**The sub-question, answered: useful, not accurate.** Trestle needs *"which modules
depend on which"* for the code view's blast radius and for the parallelism signal
T03 scores. It does not need a sound call graph, and pretending otherwise would set
a bar (a) cannot clear.

Two consequences that are requirements on T05, not caveats:

- **Every partial result is labelled partial**, and the code view says so on the
  view itself (T15). An authoritative-looking incomplete graph is worse than an
  obviously incomplete one, because only the second gets double-checked.
- **The heuristic fallback is import-regex per language plus `git log` co-change
  frequency**, and it is marked as heuristic in the output. Co-change is genuinely
  informative about coupling and genuinely not a dependency edge; conflating the two
  would make the blast radius quietly wrong.

---

## D4 — Dashboard: build small, or embed n8n?

**Blocks:** T13, T14, T15 — **RESOLVED 2026-08-17: (a) build small, and embed the
assets in the binary.**

n8n would bring a mature graph canvas — and a Postgres/SQLite service, a plugin
model, an auth layer, a licence to comply with, and a data model that is about
*executing* workflows rather than *displaying* someone else's plan.

The stated requirement is *"a simple dashboard showing the plans in a nice way and
their nodes from different perspectives"* — a viewer, not a workflow engine.
Adopting n8n means inheriting a service to run and a model to fight.

`D6` adds a second reason: the dashboard's assets are compiled **into** the single
binary, so `trestle ui` has nothing to install and no CDN to reach for. A
dashboard that fetches a webfont makes the no-egress claim false, and the easiest
way to never do that is to have no fetch path at all.

Revisit only if bidirectional control (the stated v2 bonus) turns out to need a
real engine.

---

## D5 — Does Trestle execute, or emit instructions?

**Blocks:** T09, T10, T11, T12, T13, T17 — **RESOLVED 2026-08-17: neither.
Control is inverted — the agent drives Trestle, from inside its own interface.**

The original options were **(a)** Trestle orchestrates the agent, **(b)** Trestle
emits a plan and reads the resulting files, **(c)** both. All three assumed
Trestle sits *above* the agent and reaches down to it through a CLI.

That assumption had two consequences that only became visible when the flow was
written out end to end:

1. **It excluded editor-only harnesses entirely.** The VS Code Copilot extension
   is not a shellable program. Under (a) or (c) Trestle cannot survey, interrogate
   or synthesise for that user at all, because every one of those steps needs a
   live `ask()`. Under (b) execution works but *planning* still doesn't.
2. **It made the terminal the interface for interrogation**, which is the
   product's first impression and the step that most benefits from a rich chat
   surface — rendered markdown, history, the ability to push back on a question.

Inverting it fixes both. Trestle ships **prompts and integration files** that
teach the agent the workflow, plus a **deterministic command surface** the agent
calls as tools:

```
trestle survey --json          code graph, discovered oracles, measured shape signals
trestle conventions --json     the user's own rules, classified by enforceability
trestle shape --json           deterministic baseline shape recommendation
trestle plan validate|write    schema + cycle + oracle checks; atomic write
trestle plan estimate          pre-run token range
trestle next --json            computed ready set / queue position
trestle verify <unit>          runs the oracle and records the result
trestle status --json          progress, without parsing the plan
trestle journal append         validated loop-journal entry
```

The human's interface is their editor. The only Trestle commands a human runs are
`trestle init`, `trestle status` and `trestle ui`.

**What this buys:**

- Editor-only harnesses become first-class (`D1`).
- The adapter contract stops being "invoke an LLM and parse wrapped JSON" and
  becomes "emit these files" — much smaller, and contributable as data (`D10`).
- Structured-output risk largely evaporates. The agent writes files and Trestle
  validates them, so a malformed plan produces errors the agent can iterate
  against instead of a one-shot parse failure.
- Progress becomes unforgeable, because the oracle runner is the only writer of
  `done` (`D9`).

**What this costs, stated plainly:**

- **Trestle can no longer control which model runs a unit.** Tiers become
  advisory hints (see T19). This was already true on single-model harnesses; now
  it is true everywhere.
- **Trestle cannot observe token usage at all** (`D11`).
- **Most of the "intelligence" is no longer unit-testable.** The `MockHarness`
  that was going to let every downstream node be tested without spending tokens
  is no longer needed — but it is also no longer *available* as a way to test
  prompt quality. Prompt quality is now validated by T18 dogfooding and by the
  strictness of the validators, not by unit tests. This is a real reduction in
  automated coverage and should not be papered over.
- **Unattended execution needs something to poke the agent.** That is a separate
  lane, deferred with `D7`.

---

## D6 — What is Trestle written in?

**Blocks:** T05, T13, T17, T26 — **RESOLVED 2026-08-17: Rust, distributed as a
single static binary.**

The original recommendation was TypeScript/Node, defended by `npx trestle`
needing no install. Two things overturn it:

- **Trestle plans other people's repositories.** Requiring a Node runtime in a
  Python, Go or Rust shop is backwards for a tool whose entire pitch is that it
  fits the setup you already have. The binary must be language-agnostic in the
  same sense the plan format is harness-agnostic.
- **Install once, reuse the binary.** `npx` re-resolves per invocation and ties
  the tool to one ecosystem's registry. `brew install trestle`, a `curl | sh`
  installer, and plain release tarballs cover the "try it once" case that `npx`
  was defending, without the runtime dependency.

Rust over Go, on three counts: tree-sitter's Rust bindings are first-class and
`D3` makes code-graph extraction a core feature; a genuinely static binary needs
no cgo dance; and `cargo-dist` produces the Homebrew tap, the shell installer and
the release artifacts from one config (T26).

Go remains the reasonable alternative if contributor breadth becomes the binding
constraint. Note that `D10` weakens that argument — if integrations are data
rather than code, contributing one requires no Rust at all.

An npm wrapper package that downloads the right binary may be added later purely
for discoverability. It must never become the primary path.

---

## D7 — Which scheduler backends ship in v0.1.0?

**Blocks:** nothing in v0.1.0 — **DEFERRED 2026-08-17 to v0.2.0.** See
[`../v0.2.0/README.md`](../v0.2.0/README.md).

Unattended scheduling requires the headless-CLI lane that `D5` made optional, so
it is no longer foundational. Deferring it removes T21 and T22 from v0.1.0 and
lets the shaping decision — which is the actual product — reach users sooner.

The reasoning that was here (local vs cloud-proxy vs daemon, and their differing
privacy postures) is preserved with the deferred nodes, along with the
sub-question worth settling at the same time: **an armed schedule must require
the plan to have executable work**, because arming over a fully-gated plan wastes
a night and the check is nearly free.

One thing must not be lost while this is deferred: **resumability comes from
state on disk, not from a scheduler.** T10 and T11 have to hold that property in
v0.1.0 regardless, or the deferred work will not simply drop in later.

---

## D8 — How does the agent call Trestle?

**Blocks:** T17, T23, T24 — **RESOLVED 2026-08-17: CLI subcommands as the
substrate, with a thin MCP server over the same surface.**

Options were **(a)** CLI only, invoked as shell commands per the installed
instructions, **(b)** MCP only, **(c)** both.

**(c)**, in that order of dependency. The CLI is the real implementation: it works
in a plain shell, in CI, in a `Makefile`, and for harnesses that speak no MCP.
`trestle mcp` is a stdio server that wraps the same commands and adds typed tool
schemas, which measurably reduces the agent calling them wrongly. Anything the
MCP server can do, the CLI can do — no capability may exist only over MCP, or the
no-MCP harnesses become second-class and the egress test (T16) gains a surface it
cannot reach.

Stdio only. No socket, no port. `trestle ui` is the sole listener in the product
and it is loopback-bound (T13).

---

## D9 — How is progress made unforgeable?

**Blocks:** T11, T12 — **RESOLVED 2026-08-17: (b) `trestle verify --override
--reason <text>` recording a distinct `done(overridden)` state, with the limit of
that claim stated in the docs.**

Under `D5` the agent could simply claim a unit passed. The design answer is that
**`trestle verify` runs the oracle command itself and is the only writer of
`done`** — there is no `trestle record --done`, so the agent's assertion is not
an accepted input. This is the mechanism that keeps *no oracle, no node* true when
Trestle is not the one driving.

What is still open is the **override path**, which cannot simply be forbidden:
`docs/PRIOR-SHAPES.md` requires that a mis-specified oracle be fixed by a human
who says so out loud, and a tool with no escape hatch gets worked around in ways
that leave no trace.

Options: **(a)** no override at all — a wrong oracle must be edited in the plan,
which is itself a recorded diff. **(b)** `trestle verify --override --reason
<text>`, which records a distinct `done(overridden)` state that the dashboard
shows differently and `trestle status` counts separately. **(c)** override
allowed only when the process is attached to a TTY.

Rejected: **(a)** no override at all — a tool with no escape hatch gets worked
around in ways that leave no trace, which is strictly worse than a recorded one.
**(c)** TTY-gated — looks like a control and is not one, since the agent runs
commands in the user's own shell.

What actually protects the user is that an override is **loud and permanent** in a
file that is in git, not that it is hard to perform. The docs must say exactly that
rather than implying an enforcement Trestle cannot deliver. T11 carries the
mechanism; T12 counts overridden units separately and always shows the count, even
when it is zero, so the *absence* of overrides is visible too.

---

## D10 — Are integrations data or code?

**Blocks:** T04, T23 — **RESOLVED 2026-08-17: (a) a declarative TOML manifest plus
template files, embedded in the binary and overridable from
`~/.config/trestle/integrations/`.**

An integration now consists of: a detection rule, a set of files to write with
their target paths, that harness's convention-file locations, and a capability
declaration (does it support MCP, does it support subagents).

Options: **(a)** a declarative manifest (TOML) plus template files, loaded from an
embedded directory and overridable from `~/.config/trestle/integrations/`.
**(b)** a Rust trait with one implementation per harness. **(c)** manifest for the
common case, trait for anything that needs logic.

Nothing an integration currently needs to do is computation — it is file placement
and a capability table. Making it data means a contributor adding support for a new
editor writes TOML and Markdown, not Rust, which matters a great deal for a tool
whose value grows with harness coverage. It also makes the integration set testable
by fixture rather than by mock, and it is the contribution `CONTRIBUTING.md` should
be inviting.

Rejected: **(b)** a Rust trait per harness — imposes the project's language on
everyone who wants their editor supported. **(c)** manifest-plus-trait — carries
both mechanisms from day one to serve a case that has not appeared yet, which
`AGENTS.md` §2 rules out directly.

Risk to watch: the first integration that genuinely needs logic will tempt a
templating language into the manifest. Prefer adding a narrow capability flag to
inventing a DSL.

---

## D11 — What happens to token accounting?

**Blocks:** T20 — **RESOLVED 2026-08-17: (a) pre-run estimates only; actual usage
records `unknown`, permanently, in v0.1.0.**

`D5` removed Trestle's ability to observe usage. It never sees a request, a
response, or a bill.

Options: **(a)** pre-run estimates only; record actual usage as `unknown`
permanently. **(b)** ask the agent to self-report usage into the status file via
`trestle record-usage`. **(c)** parse harness-local session logs where they exist
on disk (Claude Code writes them, for instance).

Rejected: **(b)** agent self-reporting — unverified data wearing the costume of a
measurement, which is exactly what T20 already forbids; *"never a silent estimate"*
cuts both ways. **(c)** parsing harness-local session logs — plausible and
privacy-safe, since the files are already on disk, but it is per-harness reverse
engineering of formats with no stability contract. It belongs in v0.2.0 behind a
clearly-labelled `best-effort` flag if anyone wants it.

So the honest v0.1.0 story: a **range with its assumptions stated** before you
run, and `unknown` afterwards. That is less than was planned, and it is what
inverting control actually costs.

---

## D12 — What happens when the plan turns out to be wrong?

**Blocks:** T25 — **RESOLVED 2026-08-17: (a) `trestle plan amend`, additive only.**

T09 correctly refuses to overwrite a plan that has progress recorded against it.
That leaves no answer for the common case: **the plan is wrong at unit 7.** A
loop absorbs discovery through its journal; a graph, as specified, has nowhere to
put it.

Options: **(a)** `trestle plan amend` — an additive operation that can add units
and edges, mark units `superseded` (never delete them), and record why, as a
normal reviewable diff. **(b)** version the plan: freeze `v1`, generate `v2`,
carry status forward for unchanged units. **(c)** nothing in v0.1.0 — tell users
to hand-edit and re-validate.

It matches how the loop fixture handles superseded rules — *marked in place rather
than deleted* — and it keeps the plan a single document the team can argue with
rather than a series of snapshots.

Rejected: **(b)** versioned plans — more correct and much heavier; the moment two
versions exist, every consumer needs to know which one it is reading, and the
dashboard, the status store and `trestle next` all grow a question they don't
currently have. **(c)** nothing — leaves users hand-editing a plan the validator may
then reject, which in practice means abandoning the plan.

Constraint either way: **an amend must never be able to un-`done` a unit that
passed its oracle**, and it must never silently change a unit that is currently
in progress.

---

## D13 — Does the dashboard start itself, and may it write?

**Blocks:** T13, T14, T17 — **RESOLVED 2026-08-17: it auto-starts on the first
draft write, and it stays read-only.**

Two separable questions that arrived together.

**Auto-start: yes.** Requiring `trestle ui` before you can look at a plan is a
command the user shouldn't have to know about. `trestle plan write --draft` starts
the server if it isn't running and returns a **deep link to that specific draft**,
which the agent hands to the user in chat. The user goes from "answer some
questions" to "look at your plan" with nothing typed in between.

Constraints, because a tool that silently opens a listener while its README
promises no network needs to be loud about it:

- Loopback only, same as `trestle ui` — this changes nothing about the binding, only
  about who started it.
- **Announced in the output that started it.** Never silent.
- Port written to `.trestle/ui.port`; `trestle ui --stop` kills it; it exits on an
  idle timeout so a forgotten daemon doesn't outlive the session.
- Disable-able in `.trestle/config.toml`. Some people genuinely do not want a
  background process, and they are not wrong.
- Covered by T16 like any other listener.

**Read-only: yes, still.** An "Approve" button in the draft view is the obvious
next thought and it is a trap. Trestle does not drive the agent (`D5`), so a button
can flip a flag but **cannot start the work** — the user would click Approve and
then still have to return to chat and say `continue`. That splits one action across
two surfaces, which is worse than the single word it was meant to save.

So: **look in the UI, decide in the chat.** The UI is where seeing happens; the
chat is where the conversation's turn-taking already lives. Revisit if and when
bidirectional control (the stated v2 bonus) makes the button able to do the useful
half.

---

## D14 — How are multiple harnesses combined?

**Blocks:** T04, T11, T12, T23 — **RESOLVED 2026-08-17: three roles, assigned at
init; a reviewer may veto but never grant.**

Real setups use more than one agent — writing code in one and checking it in
another is a deliberate configuration, not an accident. Since Trestle drives none
of them (`D5`), "multi-agent" means **each agent knows its role and can ask what is
waiting for it**, and the user moves between them.

Three roles, and no more in v0.1.0:

| Role | Does | Gets |
|---|---|---|
| `planner` | survey, interrogate, shape, synthesise, amend | the planning prompts |
| `implementer` | does units, calls `trestle verify` | the execution prompts |
| `verifier` | independent review of completed work | a read-oriented prompt and `trestle next --role verifier` |

One harness may hold several roles; the common single-agent case is one harness
holding all three, and it must not feel like a configuration exercise.

**The reviewer's power is deliberately asymmetric.** The invariant from `D9` is that
nothing becomes `done` without `trestle verify` running the oracle. A review step
must therefore only ever be able to *withhold* `done`, never confer it:

```
todo ──oracle passes──▶ verified ──review passes──▶ done
  ▲                                    │
  └────────── review fails ────────────┘   (reason recorded)
```

With no `verifier` configured, `verified` and `done` are the same state and nothing
changes. With one configured, the requirement is strictly stronger: oracle **and**
review. A reviewing agent that could mark work done would reintroduce exactly the
hole `D9` closed, one level up.

Rejected: letting roles be per-unit in the plan. Roles are a property of the user's
*setup*, not of the work, and putting them in the plan would make plans
non-portable between people — which is the same mistake as writing a vendor model
name into a tier (T19).

---

## D15 — `trestle` is taken on crates.io. Now what?

**Blocks:** T26 — **RESOLVED 2026-08-17: keep the product name; publish the binary
crate as `trestle-cli`; every library crate is `publish = false`.**

Checked 2026-08-17: `crates.io/crates/trestle` is a real crate — a Rust web-app
scaffolding CLI, v0.1.0, published October 2025, ~195 downloads, with a live repo.
Small and quiet, but published and not abandoned. crates.io does not reassign names
in that situation and asking would be a bad look, so this is a naming problem to
route around rather than contest.

**The blast radius is one name, not twenty-five.** Every oracle in `graph.yaml`
names a crate (`cargo test -p trestle-plan`), and those are *workspace-local*
package names. A crate with `publish = false` never touches the registry, so the
whole internal `trestle-*` namespace is unaffected and **no oracle changes**. Only
the one published artifact needs a globally unique name.

So:

| Thing | Name | Namespace |
|---|---|---|
| Product, repo, docs | **Trestle** | ours |
| Installed binary | **`trestle`** | the user's `$PATH` — crates.io doesn't own binary names |
| Homebrew formula | **`trestle`** | our tap |
| Published crate | **`trestle-cli`** | crates.io — verified free 2026-08-17 |
| Library crates | `trestle-plan`, `trestle-survey`, … | `publish = false`, never published |

The only visible cost is `cargo install trestle-cli` instead of `cargo install
trestle`, and per `D6`/T26 that was never the primary channel — `brew` and the
shell installer are.

**The one thing worth a second's thought before this hardens:** the existing crate
is *also* a Rust CLI, so the two will share search results. That is a mild,
permanent annoyance rather than a blocker. If it is unacceptable, renaming the
product is cheap now and expensive after a launch — but the recommendation is to
keep it. "Trestle" was chosen for a reason that still holds, and 195 downloads of an
unrelated scaffolder is not a real collision.

Still to check before publishing: the Homebrew tap name, and that no trademark
conflict exists in the developer-tools space.

---

## D16 — Who may change an oracle, and what happens when a later node makes one impossible to pass?

**Blocks:** nothing directly, but governs every node that adds a crate — T02, T05,
T09 and nine others — **RESOLVED 2026-08-17: derive the assertion from the graph
instead of freezing a snapshot of it. Repairing an assertion a later node made
counterfactual is legitimate and recorded here; editing an oracle so that your own
node goes green is not, and never becomes so.**

Surfaced by T16. T00's oracle asserted that the workspace contained `trestle-cli`
**and nothing else**, with sound reasoning: T00 creates no library crates, so a crate
present at that point would be a lie about progress. Then T16 added `trestle-egress`
— exactly as `graph.yaml` says it should — and T00's oracle could never pass again.

Options: **(a)** leave it, and label the script a point-in-time gate rather than a
standing check. **(b)** derive the legitimate crate set from `graph.yaml`, which
already names the crate each node owns in that node's oracle (`cargo test -p
trestle-plan`). **(c)** keep the fixed list and extend it each time a crate lands.

Rejected: **(a)** — a `done` node whose oracle fails when re-run is precisely what
this product sells against. Under `D9` the oracle *is* the record of done, so an
oracle that cannot run leaves that node with no record, only a memory of one.
**(c)** — a hand-maintained list drifts, and it would require twelve further nodes to
each edit T00's oracle, which is the act the hard rules forbid. Making the forbidden
thing routine is how a rule stops meaning anything.

**(b)** is also strictly stronger than what it replaced: the fixed list would have
accepted any unowned crate the moment a second member legitimately existed, whereas
the derived form rejects a crate no node claims however many crates exist. It fails
on a typo'd crate name and on an `AGENTS.md`-banned name like `trestle-helpers`
alike, because neither appears in the graph.

**The general rule, since this recurs:** two acts that look similar and are not.

| Act | Verdict |
|---|---|
| Editing an oracle so the node you are executing passes | **Forbidden.** No exceptions, no escalation path. |
| Weakening or deleting a test to go green | **Forbidden.** |
| Repairing an assertion that a *later* node made counterfactual | **Allowed, by a human, recorded here.** |

The test that separates them: *does this change make the node I am currently working
on pass?* If yes, stop. Here the answer was no — T16's oracle is
`cargo test -p trestle-egress`, which is unaffected by anything in
`scripts/check-workspace.sh`.

One consequence worth acting on separately: that check is now a **standing** check
rather than a point-in-time one, so it belongs in CI. Deliberately not done in the
same change, because T16 is editing the workflow file.

---

## D17 — Which dependency licences does Trestle accept?

**Blocks:** nothing directly; taxes every node that wants `serde` derive —
**RESOLVED 2026-08-18: `MIT`, `Apache-2.0`, `Unicode-3.0`. The list loosens one
entry at a time, each with a reason in `deny.toml`.**

Surfaced by T02b. `deny.toml` allowed only `MIT` and `Apache-2.0`, which excludes
`Unicode-3.0` — carried by `unicode-ident`, which arrives through `proc-macro2`/`syn`,
which is to say through `serde`'s `derive` feature. T02b therefore hand-wrote its
decode layer rather than derive it.

That worked out well *there*: hand-written decoding is what made path-qualified error
messages like `units[3].oracle: required when neither gate, order, nor a non-empty
queue is present` possible, and those messages are a product surface under `D5`. But
as a standing policy it is a tax on every remaining crate for no benefit —
`Unicode-3.0` is permissive and imposes no obligation `Apache-2.0` does not already
accept. Three more nodes would each have discovered it and each invented a different
workaround.

Rejected: **leaving it strict** — the strictness was buying nothing and costing real
work, which is the definition of ceremony this project claims to be against.
**Allowing a broad permissive set up front** (`BSD-3-Clause`, `ISC`, `Zlib`, …) —
`deny.toml`'s own opening comment says loosening is a deliberate decision, and adding
licences nothing in the tree needs would make the next addition unremarkable.

Distinguish this from what Trestle *grants*: the product is Apache-2.0 (`LICENSE`,
and `Cargo.toml` since it wrongly claimed `MIT OR Apache-2.0`). What it *accepts*
from dependencies is a separate question, and this is that one.

---

## Open

**Open: none.** D0–D6 and D8–D17 are resolved; D7 is deferred to v0.2.0.

That is not an invitation to stop thinking. `D3`'s "useful, not accurate" bar and
`D9`'s "loud, not prevented" limit are the two most likely to be quietly violated
by an implementation that means well.

Add new decisions here as they surface. Any agent that hits an ambiguity it
cannot resolve from a node file must append it, mark the affected node `blocked`
in `graph.yaml`, and stop.

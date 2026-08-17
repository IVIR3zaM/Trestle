# Open decisions — Trestle v0.1.0

Questions that must not be answered by an unattended agent. Each names the nodes
it blocks. Resolve by appending the answer and marking `RESOLVED <date>`, then
unblock the node in `graph.yaml`.

**Eight are resolved** (`D0`, `D1`, `D4`, `D5`, `D6`, `D8`, `D13`, `D14`) and one is
deferred to v0.2.0 (`D7`). `D2` remains the highest-leverage open question;
`D9`–`D12` fell out of the architecture change recorded in `D5`.

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

**Blocks:** T02, and through it most of the graph

The format has to express three things that look different on the surface:

- a **graph** (nodes, dependency edges, per-node oracle, human gates)
- a **loop** (an ordered queue, an append-only journal, `blocked(user)` states) —
  see `fixtures/source/loop-shape/` for a worked example
- a **hybrid**, which is what most real work turns out to be

Options: **(a)** one schema with a `shape:` discriminator and optional
per-shape sections. **(b)** two schemas and a converter. **(c)** one schema where
a loop is simply a graph whose nodes form a chain.

Recommendation: **(a)**. (c) is elegant and wrong — it loses the journal, which
is the loop's actual mechanism for carrying discovery forward, and it would push
users toward graphs by making loops feel second-class. (b) doubles the surface
that every consumer must handle.

**This is now the most consequential open decision in the project.** Every other
component reads or writes this format, and under `D5` the *agent* writes it too,
which raises the bar on the schema being strict and its error messages being good
enough to iterate against.

---

## D3 — How is the code graph extracted?

**Blocks:** T05, T15

Options: **(a)** tree-sitter — many languages, one dependency, syntactic only
(imports yes, call graphs weakly). **(b)** LSP — semantically accurate, but
requires a running language server per language and is slow and fragile to set
up. **(c)** heuristic — import-statement regexes per language, plus `git log`
co-change frequency.

Recommendation: **(a)** with **(c)** as a fallback for unsupported languages.
Trestle needs "which modules depend on which" for the code view and blast-radius
overlay. It does not need a semantically perfect call graph, and buying one with
(b) would cost more than the feature is worth in v0.1.0.

`D6` landing on Rust strengthens this: tree-sitter's Rust bindings are
first-class and link the C library without the static-linking friction cgo would
have introduced.

Open sub-question: does the code view need to be *accurate*, or *useful*? They
are different bars and (a) only clears the second.

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

**Blocks:** T11, T12

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

Recommendation: **(b)** plus the honest caveat. (c) looks like a control and is
not one — the agent runs commands in the user's own shell, so any TTY check is
advisory at best. What actually protects the user is that an override is
*loud and permanent* in the status file, not that it is hard to perform. Say that
in the docs rather than implying an enforcement Trestle cannot deliver.

---

## D10 — Are integrations data or code?

**Blocks:** T04, T23

An integration now consists of: a detection rule, a set of files to write with
their target paths, that harness's convention-file locations, and a capability
declaration (does it support MCP, does it support subagents).

Options: **(a)** a declarative manifest (TOML) plus template files, loaded from an
embedded directory and overridable from `~/.config/trestle/integrations/`.
**(b)** a Rust trait with one implementation per harness. **(c)** manifest for the
common case, trait for anything that needs logic.

Recommendation: **(a)**. Nothing an integration currently needs to do is
computation — it is file placement and a capability table. Making it data means a
contributor adding support for a new editor writes TOML and Markdown, not Rust,
which matters a great deal for a tool whose value grows with harness coverage. It
also makes the integration set testable by fixture rather than by mock.

Risk to watch: the first integration that genuinely needs logic will tempt a
templating language into the manifest. Prefer adding a narrow capability flag to
inventing a DSL.

---

## D11 — What happens to token accounting?

**Blocks:** T20

`D5` removed Trestle's ability to observe usage. It never sees a request, a
response, or a bill.

Options: **(a)** pre-run estimates only; record actual usage as `unknown`
permanently. **(b)** ask the agent to self-report usage into the status file via
`trestle record-usage`. **(c)** parse harness-local session logs where they exist
on disk (Claude Code writes them, for instance).

Recommendation: **(a)** for v0.1.0. (b) is unverified data wearing the costume of
a measurement, which is exactly what T20 already forbids — *"never a silent
estimate"* cuts both ways. (c) is plausible and privacy-safe (the files are
already local), but it is per-harness reverse engineering of formats with no
stability contract, and it belongs in v0.2.0 behind a clearly-labelled
`best-effort` flag if anyone wants it.

So the honest v0.1.0 story: a **range with its assumptions stated** before you
run, and `unknown` afterwards. That is less than was planned, and it is what
inverting control actually costs.

---

## D12 — What happens when the plan turns out to be wrong?

**Blocks:** T25

T09 correctly refuses to overwrite a plan that has progress recorded against it.
That leaves no answer for the common case: **the plan is wrong at unit 7.** A
loop absorbs discovery through its journal; a graph, as specified, has nowhere to
put it.

Options: **(a)** `trestle plan amend` — an additive operation that can add units
and edges, mark units `superseded` (never delete them), and record why, as a
normal reviewable diff. **(b)** version the plan: freeze `v1`, generate `v2`,
carry status forward for unchanged units. **(c)** nothing in v0.1.0 — tell users
to hand-edit and re-validate.

Recommendation: **(a)**. It matches how the loop fixture handles superseded rules
— *marked in place rather than deleted* — and it keeps the plan a single document
the team can argue with rather than a series of snapshots. (b) is more correct and
much heavier; the moment two versions exist, every consumer needs to know which
one it is reading.

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

## Open

Open: **D2, D3, D9, D10, D11, D12.**
Resolved: D0, D1, D4, D5, D6, D8, D13, D14. Deferred: D7.

`D2` is the one to think about first — it blocks most of the graph, and `D5`
raised its stakes by making the agent a writer of the format rather than only a
reader.

Add new decisions here as they surface. Any agent that hits an ambiguity it
cannot resolve from a node file must append it, mark the affected node `blocked`
in `graph.yaml`, and stop.

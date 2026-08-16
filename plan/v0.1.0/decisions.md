# Open decisions — Trestle v0.1.0

Questions that must not be answered by an unattended agent. Each names the nodes
it blocks. Resolve by appending the answer and marking `RESOLVED <date>`, then
unblock the node in `graph.yaml`.

`D0` is resolved already because it follows directly from a stated product
requirement. The rest are genuinely open, and **D1, D2 and D5 block most of the
graph** — they are the ones worth thinking about first.

---

## D0 — Does Trestle call a model itself, or drive the user's agent?

**Blocks:** T01, T04, T10, T11 — **RESOLVED: drive the user's agent. No inference in Trestle.**

Forced by the hard privacy requirement. If Trestle held an API key it would
become a second recipient of the user's source, and the no-egress guarantee would
be a lie. Instead it shells out to whatever harness the user has already
configured, exactly as they would from a terminal.

Three consequences that shape everything downstream:

- Trestle's own binary can be verified as network-silent, and that verification
  is meaningful (T16).
- Every "intelligent" step — ambiguity detection, plan synthesis, shape
  recommendation — is a **prompt Trestle constructs and hands to the user's
  agent**, plus a **parser for what comes back**. Trestle's real product surface
  is prompts, schemas and validation, not reasoning.
- Output quality varies by harness and model. Trestle must degrade honestly
  rather than pretend uniformity: validate what comes back against a schema, and
  say plainly when a harness returned something unusable.

---

## D1 — Which harnesses ship in v0.1.0?

**Blocks:** T04, T17

Options: **(a)** Claude Code only, with the adapter interface designed for more.
**(b)** Claude Code + Codex CLI, to prove the abstraction is real.
**(c)** All three plus a generic CLI adapter.

Recommendation: **(b)**. One adapter cannot validate an abstraction — the second
is what exposes the assumptions baked into the first, and it's much cheaper to
find them now than after the interface has users. Three is scope creep for v0.1.0.

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

**This is the most consequential decision in the project.** Every other component
reads or writes this format.

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

Open sub-question: does the code view need to be *accurate*, or *useful*? They
are different bars and (a) only clears the second.

---

## D4 — Dashboard: build small, or embed n8n?

**Blocks:** T13, T14, T15

n8n is a general workflow automation platform with a node editor. It would bring
a mature graph canvas — and a Postgres/SQLite service, a plugin model, an auth
layer, a licence to comply with, and a data model that is about *executing*
workflows rather than *displaying* someone else's plan.

Options: **(a)** small purpose-built app — static files plus a read-only local
server, rendering from the plan files. **(b)** embed n8n. **(c)** emit a format
n8n can import, and ship no UI in v0.1.0.

Recommendation: **(a)**, strongly. The stated requirement is *"a simple dashboard
showing the plans in a nice way and their nodes from different perspectives"* —
that is a viewer, not a workflow engine. Adopting n8n means inheriting a service
to run and a model to fight. Revisit only if bidirectional control (the stated
v2 bonus) turns out to need a real engine.

---

## D5 — Does Trestle execute, or emit instructions?

**Blocks:** T10, T11, T12, T13

Sharpest architectural fork in the project.

- **(a) Trestle orchestrates.** It selects the next unit, invokes the harness,
  parses the result, updates status, loops. The dashboard shows genuinely live
  state. Trestle becomes a long-running process with all the failure modes that
  implies.
- **(b) Trestle emits.** It writes the plan plus an executor skill/prompt; the
  user's agent does the driving; Trestle reads the resulting files to display
  status. Far simpler, and matches how graph-shaped plans are already run in
  practice — but "parallel work in progress" in the dashboard becomes a reading
  of files rather than a live view, and multi-agent orchestration is the user's
  problem.
- **(c) Both**: emit by default, orchestrate opt-in.

Recommendation: **(b)** for v0.1.0, with the status format (T12) designed so that
(a) can be added later without changing it. The stated goal includes
"multi-agent, orchestrator or not" — but proving the *planning* is good matters
more in a first version than proving the *running* is, and (b) is a fraction of
the work.

**This decision defines what v0.1.0 actually is**, so it needs an explicit answer
rather than a default.

---

## D6 — What is Trestle written in?

**Blocks:** T05, T13, T17

Options: **(a)** TypeScript/Node — same runtime as most agent tooling, easiest
dashboard story, tree-sitter bindings available, `npx trestle` needs no install.
**(b)** Go — single static binary, trivial distribution, no runtime dependency,
weaker tree-sitter ergonomics. **(c)** Python — best AST/analysis ecosystem,
worst distribution story.

Recommendation: **(a)**. Distribution via `npx` matters for an OSS tool people try
once, and the dashboard is already web. Revisit if the code-graph work turns out
to be the bottleneck.

---

## D7 — Which scheduler backends ship in v0.1.0?

**Blocks:** T21, T22

Trestle can schedule unattended runs three ways, and they are not equivalent in
cost or in privacy posture:

- **`local`** — cron / launchd / systemd timer firing `trestle run`. Zero
  outbound connections. Only fires while the machine is awake, which on a laptop
  means it mostly doesn't.
- **`cloud-proxy`** — register a routine with the harness vendor's own scheduled-
  agent service, which clones the repo and runs there. Survives a closed laptop.
  Requires a git remote, and **necessarily involves the vendor** — acceptable
  since it's the vendor the user already chose, but it must be stated and
  confirmed at arming time, never assumed.
- **`daemon`** — a foreground process. Simplest, most visible, dies with the
  terminal.

Options: **(a)** `local` only. **(b)** `local` + `cloud-proxy`. **(c)** all three.

Recommendation: **(b)**. `local` keeps the strict no-egress story intact for
users who want it, and `cloud-proxy` is what actually makes overnight runs work
on a laptop, which is the environment most of this work happens in. Two backends
also test the contract, the same argument as D1. `daemon` is
a thin variant of `local` and can come later.

Sub-question worth settling at the same time: **does an armed schedule require
the plan to have executable work?** Recommendation: yes, refuse to arm
otherwise. A graph whose contract units are all human-gated reports zero
executable work — arming over that wastes a night, the condition is common early
in a plan's life, and the check is nearly free.

---

## Open

D1–D7 are open. D0 is resolved.

Add new decisions here as they surface. Any agent that hits an ambiguity it
cannot resolve from a node file must append it, mark the affected node `blocked`
in `graph.yaml`, and stop.

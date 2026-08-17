# The plan format

Normative. One format expresses all three shapes — **graph**, **loop**, **hybrid** —
and none of them is second-class. Every Trestle component reads or writes this, and
under `D5` the *agent* writes it too, so the schema is the surface an agent iterates
against rather than only a reader's contract.

`schema/plan.schema.json` is the machine-checkable form. This document is the one you
hand-write a plan from; if the two disagree, that is a bug, and
`scripts/check-plan-format.sh` fails on it.

---

## Files

A plan is a directory. Everything is plain text, in git, reviewable in a PR.

```
.trestle/plans/<name>/
  plan.yaml        definition — what the work is
  units/<id>.md    optional prose body for one unit
  status.yaml      state — where the work has got to
  journal.md       append-only iteration log (loop and hybrid: required)
  decisions.md     questions only a human can answer, naming what they block
```

**Definition and state are separate files, and this is load-bearenough to state
plainly:** `plan.yaml` and `units/*.md` do not change while work proceeds. Only
`status.yaml` does. That makes `D9` structural rather than disciplinary — if
execution never writes a plan body, then any diff to one is by definition a human
amending the plan, and `trestle verify` is visibly the only writer of `done`.

**Unit bodies are optional.** A queue item that reads *"Adopt upstream config
format"* does not need a file, and forcing one is ceremony. A unit that needs
requirements and acceptance criteria gets `units/<id>.md`. The index is authoritative
for structure either way.

## plan.yaml

| Key | Required | Meaning |
|---|---|---|
| `trestle_plan` | yes | Format version, currently `"1"`. Present so a format change is detectable rather than inferred. |
| `shape` | yes | `graph`, `loop` or `hybrid`. |
| `name` | yes | Lowercase, hyphenated. Also the directory name. |
| `goal` | yes | What the work is for, in prose. |
| `done_when` | — | The plan-level completion contract. Distinct from every unit being done: a loop can empty its queue and still not be finished. |
| `oracle` | loop, hybrid | The per-iteration check. See [Oracles](#oracles). |
| `journal` | loop, hybrid | Path to the append-only journal. **Not optional** — see [The journal](#the-journal). |
| `rules` | — | Plan-level invariants. See [Rules](#rules). |
| `phases` | — | Ordered groups over the queue. Each has an `id` and a `title`. A label plus an order, not a second kind of unit. |
| `units` | graph | The work itself. See [Units](#units). |
| `deferred` | — | Consciously postponed. See [Deferred](#deferred). |

Unknown keys are **ignored on read and preserved on round-trip**. v0.2.0 will add
fields, and a plan written by a newer Trestle must still load in an older one rather
than failing at the first unrecognised key.

### Units

| Key | Required | Meaning |
|---|---|---|
| `id` | yes | Unique within the plan. Any string; `G05` and `1.2` are both fine. |
| `title` | yes | One line. |
| `body` | — | Path to a prose file, relative to the plan directory. |
| `done_when` | — | The unit's contract, in one sentence. |
| `deps` | graph, hybrid | *"This cannot start until that is done."* A loop orders by `order` instead. |
| `order` | loop, hybrid | Queue position. |
| `phase` | — | The `id` of a phase this unit belongs to. |
| `tier` | — | `cheap`, `standard` or `deep`. **Never a vendor model name** — a plan that names one stops being portable between harnesses. Advisory under `D5`. |
| `gate` | — | `human`: an unattended agent must not attempt this unit. |
| `oracle` | see below | The command that decides done. |
| `extra_oracles` | — | Oracles attached from the user's own standards, each carrying its `provenance`. Indistinguishable from the unit's own at verification time. |
| `repo` | — | **Reserved.** Multi-repo is not implemented in v0.1.0; the key exists so adding it is not a breaking change. |
| `queue` | hybrid | A nested list of units — a graph unit that is itself iterated. |
| `journal` | hybrid | The journal for that unit's own iteration. |

**Every unit must have an `oracle`, a `gate`, or an `order`.** The first two are the
old rule — *no oracle, no node* — and the third is how a loop satisfies it: a loop's
queue items are verified per iteration against the plan-level `oracle`, not
individually. A unit with none of the three is unverifiable work with no human
holding it, which is the one thing the format refuses to express.

### Oracles

An oracle is the command that decides done. **Never the agent's opinion** — an oracle
is by definition external to the thing it checks.

| Key | Required | Meaning |
|---|---|---|
| `command` | yes | The command to run. |
| `provenance` | — | Where this oracle came from. |
| `source` | with provenance | `unit`, `discovered`, `standard` or `convention`. |
| `rule_id` | source=standard | The rule that attached it, e.g. `SEC-04`. |
| `citation` | source=standard | The clause, e.g. `§14.2`. |

Provenance is required for standards-derived oracles because a reviewer looking at a
unit must be able to trace *why* an extra command is attached back to the clause that
caused it. Without that, ingested standards become an unexplained pile of commands,
which is how people start deleting them.

### Rules

Plan-level invariants that hold for every unit. **Superseded in place, never
deleted** — the audit trail is the point, and the same discipline `D12` settled for
units applies here.

| Key | Required | Meaning |
|---|---|---|
| `id` | yes | e.g. `R3`. |
| `text` | yes | The rule. |
| `status` | yes | `active` or `superseded`. |
| `superseded_by` | when superseded | The rule that replaces it. |
| `reason` | when superseded | Why it was replaced. |
| `replaces` | — | The inverse pointer, on the replacement. |

A superseded rule keeps both `superseded_by` and `reason` because striking a rule
through tells a reader to stop following it but not what to follow instead, and the
reasoning is the part that stops the same mistake being made twice.

### Deferred

Consciously postponed, and **distinct from forgotten**.

| Key | Required | Meaning |
|---|---|---|
| `id` | yes | e.g. `X1`. |
| `item` | yes | What was postponed. |
| `why` | yes | Why. |
| `revisit_when` | yes | What would change the answer. |

Deferred entries are **not units**: they have no oracle and no status, so unit counts,
readiness and estimates are not polluted by work nobody intends to do this time. An
entry without `revisit_when` is indistinguishable from something that was quietly
dropped, so it is required.

## status.yaml

The only file execution writes.

```yaml
plan: converge-onto-upstream-template
units:
  - id: "1.2"
    status: in_progress
    note: 18 of 24 applied
    iteration: 12
  - id: "1.4"
    status: blocked
    blocked_question: >
      the local build script has a signing step upstream has no equivalent
      for — drop it, or contribute it?
```

| Key | Required | Meaning |
|---|---|---|
| `plan` | yes | The plan's `name`. |
| `units` | yes | One record per unit. |
| `id` | yes | The unit's `id`. |
| `status` | yes | See below. |
| `blocked_question` | when blocked | The question. |
| `note` | — | Free text, e.g. *18 of 24 applied*. |
| `iteration` | — | Which iteration last touched it. |
| `oracle_result` | — | `command`, `exit`, and `at`. |
| `override` | — | `reason`, `by`, `at`. See below. |

### Statuses

| Status | Meaning |
|---|---|
| `draft` | Written but not approved, so the dashboard can render it before the user has committed to it (`D13`). |
| `todo` | Not started. |
| `in_progress` | Started. A loop unit may stay here across iterations, with a `note`. |
| `blocked` | Cannot proceed. Carries `blocked_question` — a human owes an answer. |
| `verified` | The oracle passed, and a configured reviewer has not cleared it yet (`D14`). With no reviewer configured this is the same state as `done`. |
| `done` | Finished. |
| `superseded` | Replaced by an amendment (`D25`), marked in place, never deleted. |
| `n_a` | Determined not to apply. Distinct from `deferred`, which is postponed, and from `done`, which happened. |

Statuses that carry data carry it in **sibling fields, not inside the value**. On disk
it is `status: blocked` plus `blocked_question:`, never `blocked(user): <question>`.
The display form `blocked(user)` survives in the dashboard and in prose; it is not the
storage form, because every consumer would then parse an enum and a schema could only
check it with a regex.

The same applies to the override. `done(overridden)` is `status: done` plus an
`override` record with a `reason`, a `by` and an `at`. It is a distinct permanent
state: the dashboard renders it differently, and `trestle status` counts overrides
separately and **always shows the count, even when it is zero**, so the absence of
overrides is visible too. It is loud, not prevented — see
[`PRODUCT.md`](PRODUCT.md#the-limit-of-this-claim).

## The journal

Required for `loop` and `hybrid`. **A loop without a journal is a list someone will
lose track of** — the journal is precisely how a loop carries discovery forward, and
collapsing a loop into a chain-shaped graph loses it. That was `D2`'s rejected option
(c); do not rediscover it.

Markdown, append-only, newest first. One entry per iteration, each addressable by a
stable id so a `note` can point at it.

```markdown
## Entry 12 — 2026-03-04 · unit 1.2 · in_progress

**Did:** applied six more upstream-ahead changes; the config loader now matches
upstream exactly.

**Verified:** `npm test` → 214 passing. `npm run build` → clean.

**Learned:** three of the remaining six touch the build script, which is unit 1.4 —
they cannot be applied until that question is answered.

**Next:** 1.3, not 1.2. (A guess — re-check the repo first.)

**Blocked:** none new; 1.4 still waiting on the maintainer.

**Commit:** `a3f9c21`
```

All six sections are required. **Learned** is the one that matters: it is where an
iteration records something that changes the plan — a reordering, a discovery, a rule
that turned out wrong. An entry with nothing learned says so.

Markdown rather than structured data because the entries are mostly prose, and
"(A guess — re-check the repo first.)" is a sentence, not a field value. `trestle
journal append` validates that the sections are present; it does not attempt to
understand them.

## Worked example — graph

```yaml
trestle_plan: "1"
shape: graph
name: self-hostable-runtime
goal: >
  An internal service is deployable only to one managed cloud. Add a self-hostable
  container option sharing one implementation, without breaking existing deployments.
units:
  - id: G01
    title: Architecture contract for two runtimes
    tier: deep
    gate: human
    oracle:
      command: bash scripts/check-arch-doc.sh
  - id: G05
    title: Standalone HTTP server
    deps: [G02, G03, G04]
    tier: standard
    oracle:
      command: cd server && npm test
```

## Worked example — loop

```yaml
trestle_plan: "1"
shape: loop
name: converge-onto-upstream-template
goal: >
  Make this repository indistinguishable from a fresh clone of the upstream template
  that was fully onboarded and kept current.
done_when: >
  The version files match upstream, every local-only improvement is either
  contributed or explicitly deferred with a reason, and a dry-run of the upstream
  update flow completes with no manual intervention.
oracle:
  command: npm test && npm run build
journal: journal.md
rules:
  - id: R3
    text: Contributions are batched and sent at the end of the effort.
    status: superseded
    superseded_by: R6
    reason: >
      Batching lost the reasoning behind each change by the time it was written up.
  - id: R6
    text: Contribute one entry at a time, at the moment it lands.
    status: active
    replaces: R3
phases:
  - id: P1
    title: reconcile divergence
units:
  - id: "1.2"
    title: Apply upstream-ahead changes
    phase: P1
    order: 4
deferred:
  - id: X1
    item: Adopt upstream's test runner
    why: >
      Migration touches every test file; unrelated to convergence and would dominate
      the diff.
    revisit_when: After the convergence effort closes.
```

## Worked example — hybrid

A graph whose units may each carry a `queue` and a `journal`. **Not a third schema** —
a graph unit that is itself iterated. This is what most real work turns out to be, so
it must not be the case that renders or validates worst.

```yaml
trestle_plan: "1"
shape: hybrid
name: extract-and-migrate
goal: Extract the reporting module, then migrate its callers one at a time.
oracle:
  command: make test
journal: journal.md
units:
  - id: H01
    title: Extract the reporting module
    tier: standard
    oracle:
      command: make test-reporting
  - id: H02
    title: Migrate callers
    deps: [H01]
    journal: journal.md
    queue:
      - id: H02.1
        title: Migrate the billing caller
        order: 1
      - id: H02.2
        title: Migrate the export caller
        order: 2
```

## What this format does not do

- **Roles are not here.** Who does what (`D14`) lives in `.trestle/config.toml`. Roles
  are a property of the user's setup, not of the work; putting them in the plan makes
  plans non-portable between people — the same mistake as writing a vendor model name
  into a tier.
- **Multi-repo is reserved, not implemented.** A unit may carry `repo`; nothing in
  v0.1.0 acts on it.
- **No observed cost.** Estimates live outside the plan and actual usage is `unknown`
  (`D11`).

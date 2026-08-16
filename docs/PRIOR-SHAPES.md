# Prior shapes

Trestle exists because two ways of structuring long-running agent work have
emerged in practice, both of them work, and nothing tells you which to use. This
document describes both in full, so the rest of the repo can refer to them
without depending on any external example.

Reference artifacts in the shapes described here live in
[`fixtures/source/`](../fixtures/source/). They are Trestle's own test corpus.

---

## The graph shape

**Premise:** derive the dependency structure up front, give every unit a command
that proves it done, and let a scheduler pick anything whose prerequisites are
met.

### Artifacts

- **An index** (typically one YAML file) listing every unit with:
  - a stable `id`
  - `deps` — the units that must finish first
  - an **oracle** — a command that returns pass/fail
  - a **tier** — how much thinking the unit is worth
  - optionally `gate: human` — a unit an unattended agent must not attempt
  - a mutable `status`: `todo` / `done` / `blocked`
- **One file per unit**, self-contained: goal, deliverables, acceptance criteria,
  explicit out-of-scope.
- **A decisions file** holding questions only a human can answer, each naming the
  units it blocks.

### How it runs

Compute the ready set (`status: todo` and every dependency `done`), pick one,
dispatch it, run its oracle, record the result, stop. One unit per pass.

### Properties observed in practice

- **Cold resume is exact.** Readiness is computed from files on disk, not
  re-interpreted from prose, so an interrupted run picks up precisely where it
  stopped. This is what makes unattended overnight execution viable.
- **Parallelism is visible.** Units with no edge between them can run at once.
- **Completeness is a fact, not a judgement.** The work ends when the frontier is
  empty rather than when an agent believes it is finished.
- **The plan can be wrong, and that is survivable.** A missing edge is a normal
  discovery; an executor can add it and continue.

### Costs

Real upfront effort before any code moves. Ceremony that is pure overhead on
small tasks. A set of documents that can drift from the code they describe.

### Rules that make it work

1. **No oracle, no unit.** If you cannot name a command that proves it done,
   merge it into a unit that has one, or make it a human gate.
2. **Never edit an oracle to make it pass.** If one is mis-specified, a human
   changes it and says so out loud.
3. **Never mark done without the oracle passing** in the real working tree. The
   agent's own report is not evidence.
4. **Units are contracts, not tasks.** "The existing suites pass unmodified
   against the new store" survives contact with reality. "Implement the store"
   does not.
5. **One unit per pass, then stop.** The value comes from re-orienting against
   reality at each boundary.

### A failure mode worth knowing

A unit's oracle only guards what it names. In one observed case a contract unit
required a specific document be free of stale content; its oracle checked that
document and passed, while the *most-read* file in the repo still carried the
same stale content because nothing named it. Broad claims need broad checks.

---

## The loop shape

**Premise:** don't precompute structure. State the goal and the constraints, and
let the agent discover the next useful step each iteration.

A well-run loop is **more structured than the naive picture of one**. The version
that works in practice is not "an agent improvising"; it has explicit state, and
it looks like this:

### Artifacts

- **A goal file** — the target, the hard constraints (things that must never be
  violated), and the done-conditions. No decomposition, no ordering.
- **A queue** — items in rough priority or phase order, each with a status:
  `todo` / `in-progress` / `blocked(user): <question>` / `done` / `n/a`. The
  `blocked(user)` state carries the question inline, so the reason work stopped
  is never lost.
- **An append-only journal** — one entry per iteration, in a fixed format:

  ```
  ## <date> — <what was done>
  Did:      <one or two sentences>
  Verified: <command> → <result>
  Learned:  <anything that changes what should happen next>
  Next:     <best guess, explicitly a guess>
  Blocked:  <a question only the user can answer, or "none">
  ```

- **A deferred file** — things consciously postponed, so "not now" is
  distinguishable from "forgotten".

### How it runs

Orient against current reality (read the queue tail and the journal tail, then
check the repo — **where they disagree, the repo wins**), pick the next item, do
it, verify against a real signal, journal it, commit, stop.

### Properties observed in practice

- **Zero setup.** Work starts immediately.
- **It cannot be wrong about the plan**, because there is no plan to be wrong.
- **It adapts to discovery.** A problem found mid-iteration folds into the next
  one instead of requiring the structure to be revised.
- **Unbeatable with a strong oracle.** Given a fast compiler or test suite,
  iterate-until-green outperforms any amount of structure.

### Costs

- **No readiness computation, so no safe parallelism.** The loop knows what it
  just did, not what is independent of what.
- **No completeness guarantee.** It stops when it believes it is done.
- **Cold resume is lossy.** State lives in prose; two readers can reach different
  conclusions about what comes next.
- **Unbounded variance.** An iteration might take four tool calls or forty.
- **No natural place to force a human decision** — the agent has to notice it
  should stop and ask, which is exactly the judgement agents are worst at.

### The line that carries the whole thing

`Learned:` in the journal is the only channel by which discovery reaches the next
iteration. A journal format that makes it easy to omit will produce loops that
forget. Trestle's loop executor rejects entries missing it.

### A practice worth copying

When a rule is superseded, **mark it superseded in place** rather than deleting
it, and say what replaced it. Deleting loses the audit trail; leaving it live
produces two rules for one situation with no way to tell which is authoritative.

---

## The hybrid

Most real work is neither. Common combinations:

- A graph whose units are each executed loop-style — **this is the recommended
  default for substantial work.** Structure decides *which* loop to run and when
  it is done; the loop does the converging.
- An ordered queue with a few genuine dependency edges — a loop that needed just
  enough structure to stop one item starting before another.

A format that cannot express a hybrid will push users toward whichever shape it
represents better. Trestle's must express all three as equals.

---

## Choosing

| Signal | Toward loop | Toward graph |
|---|---|---|
| Parallelism | inherently sequential | independent tracks exist |
| Oracle speed | fast tests already present | slow, missing, or manual |
| Interruption | one sitting, user present | spans days, unattended |
| Completeness | "make it work" | rename, deprecation, audit |
| Requirements | exploratory, likely to change | contracts known up front |
| Size | under ~10 units | more |
| Human decisions | few, answerable inline | several, blocking specific work |

Two signals outweigh the rest:

- **A fast oracle strongly favours the loop.** Structure substitutes for a
  missing verification signal; when the signal is already there, structure is
  mostly cost.
- **Unattended execution strongly favours the graph.** A loop cannot compute
  readiness after an interruption — it re-derives it from prose.

The honest default for anything under a day, with a working test suite and a
human at the keyboard, is **loop**. Trestle must be willing to say so.

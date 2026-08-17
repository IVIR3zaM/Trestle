# Working on this codebase

Rules for anyone writing code here, human or agent. [`DEVELOPING.md`](DEVELOPING.md)
covers how to work the build graph; this file covers how to write the code.

Read it before your first edit in a session. It is short on purpose.

---

## 1. Write code a human can follow

The goal is that an engineer who has never seen this repo can read a file top to
bottom and understand what it does. That is a higher bar than "it works", and it is
the bar.

- **Prefer a longer function to a new abstraction.** A 60-line function that reads
  straight through beats four 15-line functions you have to jump between.
- **No indirection without a second case.** A trait with one implementor and no test
  double is not an abstraction, it is a redirect. Write the concrete type. The
  integration contract (T04) is the deliberate exception — there, the second
  implementation *is* the point.
- **No generic parameter until a second concrete type exists.** Not when you can
  imagine one.
- **Two hops maximum.** If finding the code that actually does the work takes more
  than two jumps from the entry point, flatten it.
- **No `utils`, `helpers`, `common`, `misc`, `base`, `manager`.** These are names for
  code nobody wanted to place. Put the function next to the thing it serves, or give
  the module a name that says what it is.
- **Comments explain why, never what.** The code says what. If it doesn't, fix the
  code instead of narrating it.
- **Delete rather than keep.** No commented-out code, no `_v2` alongside `_old`, no
  dead branches "for later". Git remembers.

The failure mode to watch for: producing a layered, interface-heavy design because it
looks professional. Here it reads as noise, and noise is what makes a codebase
unmaintainable.

## 2. SOLID and patterns, when they pay for themselves

Use them. Don't reach for them by default.

- **Apply a pattern to pressure you have actually felt**, not pressure you anticipate.
  The second real caller, the test that needs a seam, the third `match` arm that
  keeps changing together.
- **If you introduce a named pattern, name it in a doc comment and say what forced
  it.** One sentence. If you can't name what forced it, that's the answer.
- **Dependency inversion when there are two implementations or a test needs a seam.**
  Otherwise call the concrete type.
- **No factory, builder, strategy or visitor for a single case.** A struct literal is
  a fine constructor.
- **Single responsibility is about reasons to change, not line count.** Splitting a
  cohesive function into three that are always called together makes it worse.

Rule of thumb: an abstraction should remove more code than it adds, or make a
specific future change obviously cheap. If it does neither, it is decoration.

## 3. Structure and naming

- **The domain vocabulary is fixed** — see the table in [`CONTEXT.md`](CONTEXT.md):
  *unit, edge, oracle, gate, tier, decision, integration, role, shape, gauntlet,
  distillation.* Use exactly those words in type names, module names, CLI flags and
  JSON fields. A synonym for a term already in that table is a bug; add a new term
  to the table before using it.
- **Crate boundaries come from the build graph.** Every oracle in
  [`plan/v0.1.0/graph.yaml`](plan/v0.1.0/graph.yaml) names the crate its node owns
  (`trestle-plan`, `trestle-survey`, `trestle-exec`, …). Don't invent crates the
  graph doesn't name; if you need one, that's a plan amendment, not a judgement call.
- **One concept per file, and the filename is the concept.** `oracle.rs` holds the
  oracle. If a file needs "and" to describe it, split it.
- **`pub(crate)` by default.** Public API is a commitment; make it deliberately.
- **Errors say what to do.** Every user-facing failure carries a stable code and a
  sentence naming the fix. `"request failed"` is not an error message.

## 4. Test first, always

TDD is the habit here, and it fits this project's own rules exactly: a node is
defined by its **Acceptance** section, which is a test specification someone already
wrote for you.

The loop:

1. **Read the node's Acceptance criteria.** Turn each bullet into a test — named
   after the criterion, not after the function it will call.
2. **Run it. Watch it fail.** A test that has never failed proves nothing; you don't
   know it's connected to anything.
3. Write the smallest code that makes it pass.
4. Refactor with the test green.
5. Repeat until every acceptance bullet has a test and the node's oracle passes.

Non-negotiable:

- **Never write a test after the fact to describe what the code already does.** That
  is a snapshot, not a specification, and it will pass for the wrong reasons.
- **Never modify a test to make it pass.** Same rule the product sells about oracles.
  If a test is wrong, a human changes it and says so out loud in the commit.
- **Never delete a failing test to go green.**
- The node's oracle is the outer loop; unit tests are the inner loop. Both run
  before you claim anything.

Where a criterion says *"asserted, not assumed"* — and several do — write the
assertion, even when the property is obviously true today. Those lines exist because
the property is one someone will break later without noticing.

## 5. What is machine-checked, and what isn't

Honesty about this is the same discipline this project sells to its users: a rule
with no command behind it is a wish, and pretending otherwise is worse than
admitting it.

| Rule | Enforced by |
|---|---|
| Formatting | `cargo fmt --check` in CI |
| Lint clean, no warnings | `cargo clippy -- -D warnings` |
| Function length, cognitive complexity | clippy (`too_many_lines`, `cognitive_complexity`) |
| No dead code, no unused deps | clippy, `cargo machete` |
| Dependency policy, no telemetry | `cargo deny` (T16) |
| Tests pass | each node's oracle |
| No vendor model name under `plan/` | grep assertion (T19) |
| **Test-written-first** | **nothing — review only** |
| **"Abstraction pays for itself"** | **nothing — review only** |
| **Naming matches the vocabulary** | **partially; mostly review** |

The bottom three are the ones that matter most and the ones no tool will catch. They
hold because people hold them. If you are an agent and you skipped step 2 of the TDD
loop, say so in your report rather than letting it pass silently — a quiet violation
of an unenforceable rule is exactly the failure this table exists to name.

## 6. When a rule and the work conflict

Say so, and stop. Don't quietly pick one.

If a node's specification demands something this file forbids — an abstraction that
looks unnecessary, a structure that fights the vocabulary — that is a real signal:
either the node is wrong or the rule needs an exception. Both are worth a sentence in
`decisions.md`. Neither is worth a silent workaround.

---

*This file is also a worked example of what Trestle ingests from users (T08). When
Trestle can plan its own remaining work, it will read this file and classify these
rules by enforceability — and the table in §5 is the answer it should arrive at. If
it doesn't, that's a bug in T08, and a useful one.*

---
name: trestle-build
description: Execute the next ready node of the Trestle build graph in plan/v0.1.0/. Selects nodes whose dependencies are satisfied, dispatches each to the agent matching its declared tier, verifies against the node's oracle, commits, and updates status. Use when asked to "build Trestle", "run the next node", or "continue the graph".
---

# Trestle graph executor (bootstrap)

**This is scaffolding, and it is meant to be temporary.** Trestle's whole purpose
is to install executors like this one. Until it can, its own development runs on a
hand-written version. When T09, T10, T11 and T23 land, run `trestle init` on this
repo and let it replace this file — that swap is the real dogfooding milestone.

Do exactly one pass of the loop below, then stop and report.

## 1. Load state

```bash
make status
```

Ready = `status: todo` and every dependency `done`. If nothing is ready, report
why and stop.

## 2. Select

- **Skip any node with `gate: human`.** Report that it needs the user and name
  the decision it turns on. If the only ready nodes are gated, stop.
- Check `plan/v0.1.0/decisions.md`. If an unresolved decision lists this node
  under **Blocks**, set it `blocked`, report, and move on.
- Otherwise prefer the node that unblocks the most downstream work; break ties by
  cheapest tier.

One node per pass.

## 3. Scope check

If the node touches more than ~10 files, spans unrelated packages, or has
separable halves, **split it first**: add sub-nodes to `graph.yaml` with proper
edges, write a node file for each, set the original `status: split`, and execute
only the first. Splitting is cheap; running out of context mid-node is not.

## 4. Dispatch by tier

The node's `tier` is abstract — never a vendor model name. This table is the
Claude Code mapping, and it is exactly the layer T19 specifies for every harness:

| `tier` | Agent | Model |
|---|---|---|
| `cheap` | `trestle-cheap` | haiku |
| `standard` | `trestle-standard` | sonnet |
| `deep` | `trestle-deep` | opus |

Spawn with the Agent tool. Give the subagent the node file path, the oracle
command, and `AGENTS.md` — **not** the whole plan. The node file is self-contained,
and keeping it that way is what holds per-node cost flat.

`AGENTS.md` is the exception worth passing every time: it carries the code rules
(simplicity over abstraction, patterns only when they pay, **test first**) and a
subagent that hasn't read it will produce layered code that passes its oracle and
still has to be rewritten.

**Before dispatching, push.** An isolated worktree branches from `origin/main`,
**not** from local `HEAD`. Dispatching while local commits are unpushed hands the
subagent a repo that does not contain the work its node depends on. This has
already happened once: a node was given a tree with no plan format in it, and the
only reason nothing was built on sand is that both subagents checked their premise
and stopped. Run `git status` for "ahead of origin/main by N commits" and push
first, or run the agent in the main worktree.

**Tell the subagent to verify its own base**, whichever you choose — `make status`
showing an already-`done` dependency as `waits on` is the cheapest possible
detection, and it is what caught this.

**Running two nodes at once** is fine when their crates are disjoint, and worth it
— but use one worktree per agent. In a shared tree, one agent's half-written crate
appears as compile errors in the other's `cargo clippy --all-targets`, and an agent
that cannot tell whose failure it is will chase a phantom. If a session or rate
limit terminates an agent mid-node, **resume it rather than restarting** (its files
and context survive), and drop to one agent at a time.

**Escalation:** if the oracle fails twice at the declared tier, retry once one
tier up and note it in the commit body. Never escalate past `deep`. Never
silently downgrade.

## 5. Verify

Run the node's oracle **yourself**, in the main worktree, after the subagent
returns. The subagent's own report is not evidence.

**A node whose oracle script does not exist yet must write it** — that is part of the
node, not a missing prerequisite. If the oracle command is absent, do not treat it as
a blocker and do not invent a substitute command: the node's Deliverables say to
create it. Never mark a node done against an oracle you had to improvise.

**Run the standing checks too, not only this node's oracle.** A node that adds a
crate changes the workspace, and the checks that guard the whole repo are the ones
that catch it: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo deny check`, `cargo test --workspace`, and every already-`done` node's
oracle. This is not ceremony — T05's first contact with the workspace tripped
T16's egress suite on a doc comment, and T00's crate-ownership check exists
precisely to fire when a new crate appears.

**If the node's acceptance names CI, the node is not done until CI is green.**
Push and check the run. A local oracle pass is necessary and not sufficient when
the node itself promised the check runs in CI — T16 was marked `done` while that
bullet was still unverified, and CI then found two real defects a local run could
not have. `gh run list` after pushing, and read the conclusion rather than the
annotations.

If it fails after escalation: set the node `blocked`, append a diagnosis to
`decisions.md`, leave the work uncommitted, and stop. Do not start another node
after a hard failure.

## 6. Commit

```
<type>: <node title>

Node <id> of the Trestle v0.1.0 build graph. Oracle: <command> ✓
[Escalated cheap→standard: <reason>]
```

Then set `status: done` in `graph.yaml` in a separate commit, so graph state is
recoverable if the node commit is reverted.

Then **push**. The remote is live and CI runs on every push, so an unpushed node
is a node whose CI-facing acceptance is unverified — and it is what makes the next
isolated dispatch stale (step 4).

Work on `main` while this repo is solo. Once there are contributors, switch to one
branch per node.

## 7. Report

Node executed, tier used, oracle result, what is ready next, anything needing the
user. Brief.

## Hard rules

- Never modify a node file to make it easier to pass.
- Never edit a node's oracle to make it green.
- Never delete or weaken a test to go green — the rule in `AGENTS.md` §4 applies to
  every test, not only to oracles.
- Never mark `done` without the oracle passing in the main worktree.
- Never resolve a `decisions.md` question on the user's behalf.
- Never delete or rewrite a test to make a suite pass.
- Never put a vendor model name in `plan/` — tiers are abstract, and this repo
  has to hold to the rule it sells.
- Never dispatch to an isolated worktree with local commits unpushed.
- Never mark `done` on a report alone. The oracle runs in the main worktree, and
  if the node's acceptance names CI, CI runs too.

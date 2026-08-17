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

Spawn with the Agent tool. Give the subagent the node file path and the oracle
command — **not** the whole plan. The node file is self-contained, and keeping it
that way is what holds per-node cost flat.

**Escalation:** if the oracle fails twice at the declared tier, retry once one
tier up and note it in the commit body. Never escalate past `deep`. Never
silently downgrade.

## 5. Verify

Run the node's oracle **yourself**, in the main worktree, after the subagent
returns. The subagent's own report is not evidence.

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

Work on `main` while this repo is solo and unpublished. Once it has a remote and
contributors, switch to one branch per node.

## 7. Report

Node executed, tier used, oracle result, what is ready next, anything needing the
user. Brief.

## Hard rules

- Never modify a node file to make it easier to pass.
- Never edit a node's oracle to make it green.
- Never mark `done` without the oracle passing in the main worktree.
- Never resolve a `decisions.md` question on the user's behalf.
- Never delete or rewrite a test to make a suite pass.
- Never put a vendor model name in `plan/` — tiers are abstract, and this repo
  has to hold to the rule it sells.

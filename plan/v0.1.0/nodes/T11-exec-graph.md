---
id: T11
title: Executor — graph shape
tier: standard
deps: [T09, T12]
---

## Goal

Drive a graph-shaped plan: compute the ready set, pick a node, dispatch, verify
against its oracle, record, stop.

Blocked on **D5**, same as T10.

## Requirements

- Readiness is **computed** from dependencies and status — never remembered. This
  is what makes cold resume exact rather than interpretive.
- Human-gated nodes are surfaced, never attempted.
- Nodes blocked by an unresolved decision are marked `blocked` and skipped.
- **The oracle runs in the main worktree, after the harness returns.** The
  harness's own report is not evidence.
- Never mark done without the oracle passing. Never edit an oracle or a test to
  make one pass — if an oracle is wrong, stop and surface it.
- Model tier honoured where the harness supports subagents; where it doesn't,
  say so rather than silently running everything at one model.
- Scope check before dispatch: a node too large for one pass is split into
  sub-nodes with proper edges, not attempted.

## Acceptance

- `npm run test:exec-graph` with the mock: correct ready set on a fixture graph;
  gated nodes skipped with a report; oracle failure leaves the node `todo` and
  stops; a node cannot be marked done with a failing oracle even if the harness
  claims success.
- Resume test: kill mid-node, restart, the same node is selected again.

## Out of scope

Parallel execution across worktrees — v0.1.0 runs one node per pass. The status
format (T12) must not preclude it.

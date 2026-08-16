---
id: T05
title: Repo survey + code-graph extraction
tier: standard
deps: [T01]
---

## Goal

Read a repository and produce a structured picture of it: languages, module
boundaries, real import edges, test and build commands, CI config, existing
agent conventions (`AGENTS.md`, `CLAUDE.md`, `.cursorrules`, `.claude/`).

Blocked on **D3** (tree-sitter vs LSP vs heuristics).

## Requirements

- **Import/dependency edges between modules.** This is the load-bearing output —
  it feeds the blast-radius overlay (T15) and the parallelism signal (T03).
- **Test and build commands, discovered not assumed.** Read `package.json`
  scripts, `Makefile` targets, `*.xcodeproj` schemes, `pyproject.toml`. These
  become candidate oracles during synthesis, so a wrong guess is expensive.
- **Existing conventions**, so T08 can fold them in.
- **Degrade honestly.** An unsupported language yields a partial graph clearly
  labelled partial — never a confident-looking wrong one.
- Fast enough to run on a large repo without feeling broken; cache by file hash.
- Read-only. The survey never writes to the target repo.

## Acceptance

- `npm run test:survey` against fixture repos in at least three languages,
  including one deliberately unsupported (asserting the partial-result path).
- At least one fixture repo is multi-module with a shared internal package, and
  the survey reports the edge from each consumer to that package. This is the
  relationship the blast-radius overlay depends on, so it is asserted directly
  rather than inferred from a passing run.
- No writes to the surveyed repo; no network access.

## Out of scope

Interpreting the survey (T06), rendering it (T15).

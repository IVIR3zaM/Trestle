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

Under `D5` this is also **the agent's eyes**. `trestle survey --json` is what lets
the agent resolve an ambiguity by reading instead of asking (T06), and what its
dependency edges are derived from (T07). That raises the bar on the JSON contract
being stable and on partial results being labelled as partial.

## Requirements

- **A versioned, stable `--json` contract.** The agent parses it and shipped prompts
  reference its field names, so it carries `schema_version` and breaking a field is
  a product-breaking change (T17). Design it as an interface, not a dump.
- **Import/dependency edges between modules.** This is the load-bearing output —
  it feeds the blast-radius overlay (T15) and the parallelism signal (T03).
- **The measured shape signals T03 scores.** Parallelism (independent module
  clusters), oracle presence and measured runtime, module fan-out, repo size, and
  test-to-source ratio. T03 weighs them; this node is where each gets a number, and
  a signal with no defined measurement is not a signal.
- **Test and build commands, discovered not assumed.** Read `package.json`
  scripts, `Makefile` targets, `*.xcodeproj` schemes, `pyproject.toml`. These
  become candidate oracles during synthesis, so a wrong guess is expensive.
- **Existing conventions**, so T08 can fold them in.
- **Degrade honestly.** An unsupported language yields a partial graph clearly
  labelled partial — never a confident-looking wrong one.
- Fast enough to run on a large repo without feeling broken; cache by file hash.
- Read-only. The survey never writes to the target repo.

## Acceptance

- `cargo test -p trestle-survey` against fixture repos in at least three languages,
  including one deliberately unsupported (asserting the partial-result path).
- Golden-file test of the `--json` output, so a field rename is a visible diff
  rather than a silent break in every shipped prompt.
- Every shape signal T03 consumes is present and has a defined measurement —
  asserted by iterating T03's signal list rather than by hand-written checks, so
  adding a signal there fails here until it is measured.
- At least one fixture repo is multi-module with a shared internal package, and
  the survey reports the edge from each consumer to that package. This is the
  relationship the blast-radius overlay depends on, so it is asserted directly
  rather than inferred from a passing run.
- No writes to the surveyed repo; no network access.

## Out of scope

Interpreting the survey (T06), rendering it (T15).

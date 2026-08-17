---
id: T17
title: CLI command surface
tier: standard
deps: [T05, T08, T09, T10, T11, T20]
---

## Goal

The command surface, split by who calls it.

Under `D5` this is no longer a wizard the user walks through — it is a **tool API
that happens to be a CLI**, plus three commands a human runs directly. Getting the
split right is the design work here.

## The two audiences

**Human-facing.** Three commands, and they should be the only ones a user ever
needs to type:

| Command | Does |
|---|---|
| `trestle init` | select harnesses, assign roles, write integration files (T23) |
| `trestle status` | human-readable progress, overridden count, standards drift |
| `trestle ui` | open the dashboard; `--stop` kills the auto-started one (T13) |
| `trestle doctor` | what was detected, what is degraded, and why |

`trestle ui` is increasingly optional: under `D13` the dashboard starts itself when
a draft is written and the agent hands over a deep link. A user who never types it
is the intended case.

**Agent-facing.** Called from inside the harness, per the emitted prompts. Every
one is deterministic, every one takes `--json`, and **none of them performs
inference or touches the network:**

```
trestle survey --json              code graph, discovered commands, shape signals
trestle conventions --json         in-repo rules, classified by enforceability (T08)
trestle standards ingest|check     external policy documents, chunked (T27)
trestle shape --json               deterministic baseline shape recommendation
trestle decisions add|list|answer  the question store (T06)
trestle plan validate              the gauntlet (T07)
trestle plan write --draft         atomic, validating, non-clobbering (T09)
trestle plan amend                 additive re-planning (T25)
trestle plan estimate              pre-run token range (T20)
trestle next --json [--role R]     ready set / queue position, per role (T10, D14)
trestle verify <unit>              runs the oracle, records the result (T11)
trestle review <unit> --pass|--fail  reviewer veto; never grants done (T11, D14)
trestle status --json              progress without parsing the plan
trestle journal append             validated loop-journal entry (T12)
```

## Requirements

- **`--json` output is a versioned contract.** The agent parses it, and prompts
  shipped in integrations reference its fields. Every JSON-emitting command
  includes a `schema_version`, and breaking a field is a breaking change to the
  product, not an internal refactor. Say so in the docs.
- **Errors are actionable and machine-readable.** Every failure carries a stable
  `code` plus a human sentence naming the fix: `NO_PLAN` → "run `trestle init`
  first"; `PLAN_INVALID` → the offending path; `ORACLE_MISSING` → the unit id.
  Never "request failed" — and note the failure taxonomy is now much smaller than
  the old adapter design needed, because Trestle makes no calls that can fail.
- **Read-only commands must be provably read-only.** `survey`, `conventions`,
  `shape`, `next`, `status`, `plan validate`, `plan estimate`, `standards check`
  write nothing, not even a cache, into the target repo. Cache in a temp dir keyed by
  file hash.
- **`--role` is accepted wherever it is meaningful and ignored gracefully where it
  isn't.** A single-agent user never passes it, and a command rejecting it as an
  unknown flag when no roles are configured would make the multi-agent feature leak
  into setups that don't use it.
- **Exit codes are meaningful.** `0` success, `1` operational failure, `2` invalid
  usage, `3` "nothing to do" for `next` — an agent scripting against this needs to
  tell those apart without parsing prose.
- Every command works with no MCP server present (`D8`).
- No interactive prompt in any agent-facing command. They must be usable from a
  non-TTY context, because that is where they will be called from.

## Acceptance

- `cargo test -p trestle-cli` — golden-file tests for every `--json` output
  against a fixture repo; every error code reachable and tested; read-only
  commands asserted to leave `git status` clean on the fixture repo.
- `schema_version` present in every JSON output, asserted by iterating the command
  list rather than by hand-written per-command tests.
- `trestle doctor` on a fixture repo with a subagent-less harness reports tier
  mapping as inert (T19) and usage reporting as unavailable (T20).
- `--help` for every command is accurate — asserted by a test that every command
  in the dispatch table has help text and at least one example.

## Out of scope

`trestle init` itself (T23). The MCP wrapper (T24). Packaging (T26). Anything
unattended (v0.2.0).

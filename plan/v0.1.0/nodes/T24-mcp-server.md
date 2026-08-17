---
id: T24
title: MCP server over the CLI surface
tier: standard
deps: [T17]
---

## Goal

`trestle mcp` — a stdio MCP server exposing the agent-facing commands as typed
tools, so Claude Code, Copilot and Codex call them correctly instead of
approximately.

Per `D8` this is a **wrapper, not a second implementation.**

## The rule that keeps this cheap

> Every MCP tool is a thin call into the same code path the CLI subcommand uses.
> No capability exists only over MCP.

Reasons, all of which matter:

- Harnesses that speak no MCP must remain fully capable (`D1` ships a `generic`
  integration that assumes none).
- The CLI is what CI, `Makefile`s and humans use.
- T16 tests egress by exercising the CLI. A capability reachable only through the
  MCP path would be a surface the egress test cannot see.

Practically: the tool list is **generated from the CLI dispatch table**, not
hand-maintained. A new subcommand appears as a tool without anyone remembering to
add it, and a test asserts the two lists are equal.

## Requirements

- **Stdio only.** No socket, no port, no HTTP. `trestle ui` is the only listener in
  the product (T13) and it is loopback-bound. This is a threat-model line (T01) and
  a test in T16.
- Typed input schemas per tool, derived from the same argument definitions the CLI
  parses, so the two cannot drift.
- **Read-only tools declared as such**, using whatever annotation the protocol
  offers, so a harness can surface the difference between `survey` and
  `plan write` to the user.
- Output is the command's `--json` payload verbatim, including `schema_version`.
  No reformatting — the prompts reference those field names.
- Errors carry the CLI's stable error `code` plus the human sentence, so an agent
  can act on `NO_PLAN` without parsing English.
- Starts fast and exits when stdin closes. It is spawned per session by the harness
  and must not linger.
- **No state in the server.** Every call reads from disk. Two concurrently running
  servers (two editor windows on one repo) must not be able to disagree, and
  concurrent readers are already required to be safe (T12).

## Acceptance

- `cargo test -p trestle-mcp` — the server completes an MCP handshake; the tool
  list is asserted **equal** to the CLI's agent-facing dispatch table (both
  directions, so neither adding nor removing a command can silently desync); each
  tool round-trips a call against a fixture repo; an invalid argument returns a
  protocol error carrying the CLI error code.
- Read-only tools are annotated read-only, asserted per tool.
- Closing stdin exits within a bounded time.
- No listening socket is opened during a full session — shares the T16 harness.
- The emitted MCP config from T23 actually launches this server: an integration
  test that reads the fixture repo's written config, spawns what it names, and
  handshakes. **This is the one test that proves init and the server agree**, and
  without it a path typo ships silently.

## Out of scope

Adding commands (T17 owns the surface). Any mutation not already available in the
CLI.

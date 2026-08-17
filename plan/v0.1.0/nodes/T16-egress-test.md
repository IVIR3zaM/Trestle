---
id: T16
title: Egress test — prove no outbound connections
tier: standard
deps: [T01]
---

## Goal

Turn the privacy guarantee into an automated check that fails the build. A README
promise is not a guarantee.

This node can and should land **early** — it is cheap, it has no dependencies
beyond the threat model, and every later node inherits its protection.

## Requirements

- Run the full CLI, the MCP server and the dashboard under a **sandbox that denies
  network access**, and assert nothing fails and nothing attempted a connection.
  Denying is stronger than observing: an observer can be raced, a denial cannot.
- Cover every channel enumerated in `docs/THREAT-MODEL.md` (T01). **A channel
  without a corresponding test is a failing condition for this node.**
- Assert the dashboard listener is bound to loopback, and that it is the **only**
  listener — `trestle mcp` must open no socket at all (T24), and no other command
  may listen. Assert by enumerating the process's sockets, not by inspecting code.
- **Dependency audit**: fail on any dependency with a build script that performs
  network access, and on any known-telemetry crate. This is the channel people
  forget — the code is clean and a transitive dependency phones home. `cargo deny`
  in CI, with the policy in the repo.
- **No HTTP client in the dependency tree at all**, and no update-check code path
  (T26). Grep-asserted, because the strongest guarantee is the absence of the
  capability rather than the discipline not to use it.
- **The filesystem blast radius of `trestle init`** (T23) is tested here alongside
  egress, because it is the same class of promise: Trestle writes only to paths it
  declared. Run `init` on a fixture repo under a filesystem sandbox and assert no
  write outside the declared set, including the `$HOME` case.
- Runs in CI on every PR, not just at release.
- **There is no harness-subprocess exemption in v0.1.0.** The previous design needed
  one because Trestle invoked the agent; under `D5` Trestle spawns no agent, so the
  sandbox can be total. If a future version reintroduces the exemption (v0.2.0's
  unattended lane will), it must be narrow, documented, and asserted to be the only
  one. **Removing this exemption is a strictly stronger guarantee than the previous
  plan could offer, and it is worth saying so in the README.**

## Acceptance

- `cargo test -p trestle-egress -- --include-ignored` passes with the network denied
  and **fails** on each of two planted violations: an outbound HTTP request, and a
  write outside the declared path set. A guard that has never been seen to fail is
  not known to work, and there are now two guarantees to plant against.
- Every threat-model channel maps to a named test, asserted by comparing the test
  list against the channel table in `docs/THREAT-MODEL.md` — so adding a channel to
  the document fails this node until it has a test.
- The full agent-facing command surface runs to success with the network denied.
  Not a sample of it — all of it, enumerated from the T17 dispatch table.

## Out of scope

The user's harness behaviour, which is theirs and must be documented as such.


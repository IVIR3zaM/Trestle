---
id: T13
title: Dashboard server (loopback-bound, read-only)
tier: standard
deps: [T12]
---

## Goal

Serve the dashboard locally. Read-only in v0.1.0 — control from the UI is the
stated v2 bonus.

Blocked on **D4** (build small vs embed n8n; recommendation is build small).

## Requirements

- **Binds to `127.0.0.1` only.** Not configurable to `0.0.0.0` in v0.1.0 — an
  accidental LAN exposure would break the product's central promise, and a flag
  is exactly how that accident happens.
- No external assets. Everything is served from disk: no CDN fonts, no remote
  scripts. A dashboard that fetches a webfont makes the no-egress claim false.
- Reads plan + status files; watches for changes and pushes updates over SSE or a
  local WebSocket.
- Never writes to the repo. Not even a cache — use a temp dir.
- Starts fast, exits cleanly, survives a plan file being edited mid-read.

## Acceptance

- `npm run test:server` — asserts the listener is bound to loopback and that a
  connection from a non-loopback interface is refused; asserts zero outbound
  connections during a full session (shares the T16 harness); asserts no writes
  to the target repo.
- Editing a plan file updates the UI without a restart.

## Out of scope

The views themselves (T14, T15). Any mutation endpoint.

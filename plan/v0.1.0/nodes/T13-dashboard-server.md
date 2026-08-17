---
id: T13
title: Dashboard server (loopback, read-only, embedded assets, auto-start)
tier: standard
deps: [T12]
---

## Goal

Serve the dashboard locally. Read-only in v0.1.0 — control from the UI is the
stated v2 bonus.

`D4` is resolved: build small, and **embed the assets in the binary**.

**This is the only listener in the product.** The MCP server is stdio-only (T24),
and nothing else opens a socket. That makes this node the single place the LAN-
exposure risk lives, which is worth knowing while building it.

## Requirements

- **Binds to `127.0.0.1` only.** Not configurable to `0.0.0.0` in v0.1.0 — an
  accidental LAN exposure would break the product's central promise, and a flag
  is exactly how that accident happens.
- **No external assets, and no fetch path at all.** HTML, CSS, JS and fonts are
  compiled into the binary (`include_dir` or equivalent). A dashboard that fetches a
  webfont makes the no-egress claim false; the way to never do that is to have
  nothing to fetch from and no client with which to fetch it.
- Since assets are embedded, `trestle ui` works on a machine with no network, no
  data directory, and no install step beyond the binary itself — which is the T26
  promise applied here.

## Auto-start, per D13

The user should not have to know `trestle ui` exists. `trestle plan write --draft`
starts the server if it isn't running and returns a **deep link to that draft**,
which the agent hands over in chat: answer some questions, then look at your plan,
with nothing typed in between.

A tool whose README promises no network connections must be scrupulous about
opening a listener, so:

- **Announced by whatever started it.** Never silent, never in a log file nobody
  reads.
- Port written to `.trestle/ui.port`; reuse it rather than starting a second server.
- **Idle timeout** — exits after a configurable period with no requests, so a
  forgotten daemon doesn't outlive the session. `trestle ui --stop` kills it now.
- **Disable-able** in `.trestle/config.toml`. Some people do not want background
  processes and they are not wrong; with it off, `trestle ui` still works manually.
- Starting it twice from two shells must not produce two servers.

## Read-only, still — and deliberately

**No mutation endpoint. Not even Approve.** The obvious next thought is an Approve
button on a draft, and it is a trap: Trestle does not drive the agent (`D5`), so the
button could flip a flag but **could not start the work**. The user would click it
and then still have to return to chat and say `continue` — one action split across
two surfaces, which is worse than the single word it was meant to save.

Look in the UI, decide in the chat. Revisit when bidirectional control (the stated
v2 bonus) can make a button do the useful half.
- Reads plan + status files; watches for changes and pushes updates over SSE or a
  local WebSocket.
- Never writes to the repo. Not even a cache — use a temp dir.
- Starts fast, exits cleanly, survives a plan file being edited mid-read.

## Acceptance

- `cargo test -p trestle-ui --test server` — asserts the listener is bound to
  loopback and that a connection from a non-loopback interface is refused; asserts
  zero outbound connections during a full session (shares the T16 harness); asserts
  no writes to the target repo.
- Editing a plan file updates the UI without a restart.
- **No request for any asset leaves the process** — asserted by serving every page
  with the network denied and checking the browser-side console for zero failed
  requests. An embedded-asset claim that has never been tested with the network off
  is not known to be true.
- Runs correctly on a repo with a plan mid-amend (T25) — a plan directory being
  renamed into place must not produce a 500 or a stale render.
- Auto-start: writing a draft starts exactly one server and prints its URL; doing it
  again reuses that server rather than starting a second; `--stop` terminates it;
  the idle timeout fires; disabling it in config means no listener is ever opened.
  **Asserted by enumerating the process's sockets**, not by reading the code path.
- **No route accepts POST, PUT, PATCH or DELETE** — asserted by iterating the router
  rather than by testing a handful of paths, so an added mutation fails this node.

## Out of scope

The views themselves (T14, T15). Any mutation endpoint.

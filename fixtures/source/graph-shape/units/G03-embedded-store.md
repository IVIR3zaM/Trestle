---
id: G03
title: Embedded database store
tier: standard
deps: [G02]
---

## Goal

A persistent store for the self-hosted runtime, so a container survives restarts
without the managed cloud database.

## Why this is small

The store contract already exists as a five-method interface behind a setter for
test injection, and the in-memory test double is a complete working
implementation of it. This is a port, not a design job — the semantics are
already pinned by the existing suites.

## Requirements

- **Per-account monotonic sequence numbers**, allocated inside the same
  transaction as the write. The in-memory double uses a process-local counter
  and hides this entirely; a naive port collides under concurrency.
- A rejected stale write still consumes a sequence number, matching the double.
- Driver selected by an environment variable.

## Acceptance

- `cd core && STORE=embedded npm test` — the same suites that pass against the
  in-memory double pass against this, **unmodified**.
- 50 parallel writes to one account yield 50 distinct sequential numbers.
- Write, close, reopen, read — data survives.

## Out of scope

The HTTP server (G05), packaging (G06), any change to the wire format.

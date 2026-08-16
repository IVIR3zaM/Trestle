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

- Run the full CLI and the dashboard under a **sandbox that denies network
  access**, and assert nothing fails and nothing attempted a connection. Denying
  is stronger than observing: an observer can be raced, a denial cannot.
- Cover every channel enumerated in `docs/THREAT-MODEL.md` (T01). **A channel
  without a corresponding test is a failing condition for this node.**
- Assert the dashboard listener is bound to loopback.
- **Dependency audit**: fail on any dependency with install-time scripts or known
  telemetry. This is the channel people forget — the code is clean and a
  transitive package phones home.
- Runs in CI on every PR, not just at release.
- The harness subprocess is explicitly exempt and that exemption is narrow,
  documented, and asserted to be the only one.

## Acceptance

- `npm run test:egress` passes with the network denied and **fails** if a
  deliberate `fetch()` is introduced into the source (assert the test catches a
  planted violation — a guard that has never been seen to fail is not known to
  work).
- Every threat-model channel maps to a named test.

## Out of scope

The user's harness behaviour, which is theirs and must be documented as such.

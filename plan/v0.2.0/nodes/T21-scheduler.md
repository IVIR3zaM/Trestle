---
id: T21
title: Scheduler contract + local backend
tier: deep
deps: [T12, T17]
---

## Goal

Run a plan unattended on a schedule — one unit per firing — with the backend
chosen to fit the user's environment.

Blocked on **D7** (which backends ship in v0.1.0).

## Why a contract rather than a cron line

The right scheduling mechanism differs by environment, and the difference is not
cosmetic:

| Backend | Fits | Mechanism |
|---|---|---|
| `local` | any harness with a CLI | cron / launchd / systemd timer firing `trestle run` |
| `cloud-proxy` | harnesses with hosted scheduled agents | register a routine with that service; it clones the repo and runs there |
| `daemon` | long sessions, laptop that sleeps | a foreground process the user can watch and stop |

`cloud-proxy` is a **proxy, not an implementation** — Trestle registers the
schedule with the vendor's own service and then gets out of the way. This keeps
the no-inference invariant (D0) intact: Trestle still never calls a model.

## Requirements

- One unit per firing, then exit. Never chain units inside one run.
- **Resumability comes from state on disk, not from the scheduler.** A firing that
  dies leaves the unit `todo`; the next one recomputes readiness and picks it up.
  This is the whole reason unattended execution is safe, and it must hold for
  every backend.
- Refuse to arm a schedule when **nothing is executable** — all units gated,
  blocked, or done. Report why. Arming a schedule over an unrunnable plan wastes
  a night and is a mistake this tool exists to prevent.
- Every backend pushes or commits before exiting; a firing whose work is only in
  a disposable sandbox has accomplished nothing.
- `trestle schedule status` / `pause` / `resume` / `cancel`, uniform across
  backends.
- **Privacy:** `local` and `daemon` make no outbound connection at all.
  `cloud-proxy` necessarily talks to the vendor the user already uses — it must
  say so explicitly at arming time and require confirmation.

## Acceptance

- `npm run test:scheduler` — the local backend fires, runs one unit, exits;
  arming over a fully-gated plan is refused with a clear reason; a killed firing
  leaves the unit `todo` and the next firing selects it again.
- The scheduler contract is satisfiable by a backend written without reading
  Trestle's source.

## Out of scope

Limit handling (T22). Dashboard control (v2).

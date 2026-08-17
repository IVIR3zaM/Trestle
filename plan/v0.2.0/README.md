# Trestle v0.2.0 — deferred work

Not a plan yet. This folder holds work that was specified for v0.1.0 and then
deliberately deferred, so the specification isn't lost and the reason is on the
record.

## What's here, and why it moved

### The unattended lane — T21, T22

v0.1.0 ships **one lane**: the user works inside their own agent's interface
(Copilot Chat, Claude Code, Codex), and the agent calls Trestle's deterministic
commands. That is `D5` in [`../v0.1.0/decisions.md`](../v0.1.0/decisions.md).

Nothing in that lane runs while nobody is watching, because nothing pokes the
agent at 3am. Unattended execution needs a **second lane**:

```
trestle run            # fire one unit against a headless CLI, then exit
trestle schedule ...   # arm/pause/cancel a recurring firing
```

That lane needs a headless harness CLI (`claude -p`, `codex exec`, `copilot -p`),
which the v0.1.0 architecture deliberately made *optional* rather than
foundational. Building both lanes at once roughly doubles v0.1.0 and delays the
part that is actually the product — the shaping decision.

- [`nodes/T21-scheduler.md`](nodes/T21-scheduler.md) — scheduler contract,
  `local` / `cloud-proxy` / `daemon` backends, and the rule that arming over a
  plan with no executable work must be refused.
- [`nodes/T22-limit-aware.md`](nodes/T22-limit-aware.md) — rate-limit vs
  quota-exhausted vs not-authenticated, reset-time parsing, backoff with jitter,
  and never marking a cut-short unit `done`.

`D7` in the v0.1.0 decisions (which backends ship) travels with these and is
marked deferred there.

**The one thing v0.1.0 must not get wrong on their behalf:** resumability comes
from state on disk, not from a scheduler being clever. T10 (`trestle next`) and
T11 (`trestle verify`) have to hold that property in v0.1.0 anyway — readiness
computed from dependencies and status, never remembered — or these two nodes will
not simply drop in later.

### Also deferred, without node files yet

- **Actual token accounting.** Trestle cannot observe usage under the v0.1.0
  architecture. `D11` records the options; the plausible one is reading
  harness-local session logs, per harness, behind a `best-effort` label.
- **Bidirectional dashboard control** — approving a gate or triggering a unit from
  the UI. Stated as a v2 bonus from the start; T13 keeps the server read-only and
  ships no mutation endpoint so this stays an addition rather than a rework.
- **Multi-repo plans.** v0.1.0 handles one repo. The plan format (T02) must not
  preclude more, which is a constraint on T02 rather than work here.

## Before planning this

Two things should be true first, and neither is yet:

1. **v0.1.0 has been dogfooded** (T18). The unattended lane is worth building only
   if the plans are good enough to be worth running unattended.
2. **The shape of this version has been decided on its own merits.** Do not assume
   v0.2.0 is a graph because v0.1.0 was. Three or four nodes with unsettled
   requirements and a fast oracle is exactly the profile the rubric should call a
   **loop** — and if Trestle exists by then, it should be the one to say so.

---
id: T01
title: Product contract + privacy threat model
tier: deep
gate: human
deps: []
---

## Goal

Write down what Trestle is and — more importantly — what it must never do, in a
form the rest of the graph can be checked against. Everything else descends from
this node, so it goes first.

## Deliverables

**`docs/PRODUCT.md`** — the contract:

- The six-step flow (survey → interrogate → shape → absorb → show → write/run)
  and what each step is allowed to assume about the others. **Say which steps are
  the agent's and which are Trestle's** — under `D5` they alternate, and a reader
  who gets that wrong will design the next component backwards.
- **Three architectural invariants**, each stated where an agent reading only
  `PRODUCT.md` cannot miss them:
  1. **No inference** (`D0`). Every model call belongs to the user's harness.
     Trestle's product surface is *prompts, schemas, validation and deterministic
     analysis* — not reasoning.
  2. **Inverted control** (`D5`). Trestle does not drive the agent; the agent
     drives Trestle, from inside its own interface. Trestle's human-facing surface
     is three commands (`init`, `status`, `ui`); everything else is a tool the
     agent calls.
  3. **Unforgeable progress** (`D9`). `trestle verify` runs the oracle itself and
     is the only writer of `done`. The agent's claim of success is not an accepted
     input. State the limit of this too — the override exists, and it is loud
     rather than prevented.
- What v0.1.0 does not do, so nodes don't quietly grow. Explicitly: nothing
  unattended, no scheduling, no observed token usage (all deferred — see
  [`../../v0.2.0/README.md`](../../v0.2.0/README.md)).

**`docs/THREAT-MODEL.md`** — the privacy guarantee, written adversarially:

| Party | Sees | Must never see |
|---|---|---|
| Trestle itself | everything on disk | — (it emits nothing) |
| The user's harness vendor | whatever that harness already sends | not Trestle's concern, but must be stated |
| Anyone on the LAN | nothing — dashboard is loopback-bound | all of it |

Enumerate every channel by which code could leave — HTTP client, DNS, telemetry
SDK, crash reporter, update check, a dependency that phones home at install,
the dashboard binding to `0.0.0.0`, a diagnostic bundle — and state the
countermeasure for each. **A channel with no automated check is a gap; name it
as one.** T16 turns this list into tests.

Four channels the inverted-control architecture adds, which the earlier version of
this node predates and which must appear in the table:

- **The MCP server** (T24). Stdio only — no socket, no port. The countermeasure is
  that no listening socket may be opened by any code path except `trestle ui`.
- **`trestle init` writing outside `.trestle/`** (T23). This is the only place
  Trestle mutates files a human owns, and one integration writes to `$HOME`. Every
  written path must be declared in advance, shown before writing, and reversible.
  A write to an undeclared path is a violation of the same class as a network call.
- **Embedded assets** (`D4`). The dashboard and the integration templates are
  compiled in, so there is no fetch path and no data directory. State this as the
  countermeasure it is.
- **Update checks** (T26). None, ever — not opt-in, not weekly. `--version` prints
  a version; it does not ask anyone whether that version is current.

Be honest about the boundary in the user-facing copy: Trestle adds no new
recipient of your code, and cannot stop the one you already chose. Under `D5` the
user's agent is now the thing *calling* Trestle rather than the thing Trestle
calls — which does not change what that agent sends to its vendor, and the copy
must not imply otherwise.

## Acceptance

- `bash scripts/check-product-doc.sh` — asserts both documents exist, that the
  threat model's channel table has a countermeasure in every row, and that no
  row says "TODO".
- Every claim in `README.md` traces to a statement in one of these documents.
- All three invariants are stated somewhere an agent reading only `PRODUCT.md`
  cannot miss — asserted by the script, one grep per invariant.
- The channel table includes the four inverted-control channels above.

## Out of scope

Any code. Integration design (T04). The egress tests themselves (T16) — this node
produces the list they must cover.

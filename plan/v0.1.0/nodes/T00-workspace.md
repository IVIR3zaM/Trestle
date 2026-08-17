---
id: T00
title: Cargo workspace, binary shell, lints and CI
tier: cheap
deps: []
---

## Goal

The scaffolding every other node's oracle assumes exists.

Every oracle in `graph.yaml` is `cargo test -p <crate>`, and nothing creates the
workspace those crates live in. There is also no `.github/`, so T16's *"runs in CI
on every PR"* has nowhere to run. This node is that floor.

`tier: cheap` and `deps: []` — it is mechanical, it needs no decision, and it can be
done while T01 is still being thought about.

## Deliverables

**The workspace.** A root `Cargo.toml` with a `[workspace]` members list, plus the
`trestle` binary crate as an empty shell that parses no subcommands yet and exits
with a usage message. Library crates are added by the nodes that own them — this
node creates none of them, because a crate with no code in it is a lie about
progress.

**Crate naming, per `D15`.** Library crates are `trestle-plan`, `trestle-survey`,
`trestle-exec` and so on, all with **`publish = false`** — those names are
workspace-local and never reach crates.io. Only the binary crate is published, as
**`trestle-cli`**.

**The binary it produces must be named `trestle`**, via an explicit `[[bin]] name =
"trestle"` in that crate's `Cargo.toml`. Cargo would otherwise name the executable
after the package and ship a `trestle-cli` command, which would leak a crates.io
namespace collision into every command a user ever types. Assert the binary's name,
not just that it builds.

**The lints that enforce half of `AGENTS.md`.**

- `rustfmt.toml` and `cargo fmt --check` in CI
- `cargo clippy -- -D warnings`, with `too_many_lines` and `cognitive_complexity`
  enabled — those two are what §1's "write code a human can follow" cashes out to
- `deny.toml` for `cargo deny`: the dependency policy T16 and T26 both rely on.
  **Start it strict** — no HTTP client, no telemetry crate, no build script with
  network access — because loosening a policy later is a decision someone makes
  deliberately, while tightening one later means auditing everything already in.

**CI.** One workflow, on every PR: `fmt --check`, `clippy -D warnings`, `test
--workspace`, `deny check`. Nothing else yet; T16 and T26 add their own steps to it.

**`trestle --version`** reporting version, target triple and git SHA (T26 needs it;
it is three lines here and a retrofit later).

## Acceptance

- `bash scripts/check-workspace.sh` — `cargo fmt --check`, `cargo clippy -- -D
  warnings`, `cargo test --workspace` and `cargo deny check` all succeed on the empty
  workspace; `cargo build --release` produces a binary named `trestle`;
  `trestle --version` prints a version, a triple and a SHA.
- The CI workflow runs those same four commands — asserted by the script comparing
  its own command list against the workflow file, so the two cannot drift.
- **A planted violation of each lint fails**: an unformatted file, a `#[allow]`-free
  clippy warning, and a dependency the deny policy forbids. A guard never seen to
  fail is not known to work — the same rule T16 applies to egress.
- No library crate exists yet, and the binary does nothing but print usage. **This
  node must not implement anything**, or its cheapness is a lie and the nodes that
  own those crates inherit code they didn't write.

## Out of scope

Any subcommand (T05 and T03 add the first two). Release artifacts and the Homebrew
tap (T26). The egress test itself (T16) — this node only provides the CI it runs in.

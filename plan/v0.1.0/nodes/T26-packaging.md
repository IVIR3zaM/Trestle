---
id: T26
title: Packaging — static binary, Homebrew tap, installer, release CI
tier: standard
deps: [T17]
---

## Goal

**Install once, reuse the binary, on any machine, regardless of what language the
target repo is written in.** That is a product requirement, not a distribution
detail — see `D6`.

## Why this is a v0.1.0 node

Trestle plans *other people's* repositories. A Python shop, a Go shop and an iOS
shop must all be able to install it without acquiring a runtime they don't want.
The previous plan's `npx trestle` inverted that: it made the tool's own ecosystem
the user's problem.

It is also the second half of the T23 promise. `trestle init` claims the user never
needs the terminal again — that only holds if getting the binary in the first place
was one line.

## Requirements

**The binary**

- One statically-linked executable per target, no runtime dependency. Dashboard
  assets and the integration manifests and templates are **embedded** (`D4`, T04),
  so there is no data directory to install and no path to get wrong.
- Targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`,
  `aarch64-unknown-linux-gnu`, plus `*-unknown-linux-musl` for the fully-static
  case. Windows is best-effort in v0.1.0 — state it rather than implying support.
- `trestle --version` reports the version, the target triple, and the git SHA.
  Bug reports are useless without it.
- **Reproducible enough to verify:** publish SHA256 checksums for every artifact.

**The channels**

| Channel | Command |
|---|---|
| Homebrew | `brew install <tap>/trestle` |
| Shell installer | `curl -fsSL <url>/install.sh \| sh` |
| Cargo | `cargo install trestle-cli` / `cargo binstall trestle-cli` |
| Manual | release tarball plus checksum |

**The crate is `trestle-cli`, the binary is `trestle`** (`D15`). `trestle` is taken
on crates.io by an unrelated project; the binary name, the Homebrew formula and the
repo are all in different namespaces and stay `trestle`. Every library crate is
`publish = false`, so only one artifact is ever published and no oracle in
`graph.yaml` is affected.

The Homebrew formula and the installer script are **generated from one config**
(`cargo-dist` or equivalent), not hand-maintained. Hand-maintained release
plumbing drifts and then lies about which version is current.

**The privacy line that packaging must not cross**

- **No update check. Ever.** Not opt-out, not opt-in, not "check once a week". It
  is an outbound connection, and the README says there are none. `trestle
  --version` prints a version; it does not ask anyone whether that version is
  current.
- No install-time telemetry, no post-install script that phones home.
- The installer script fetches from the release host and does nothing else — and it
  is short enough that a cautious user can read it before piping it to a shell.
  Anyone piping a script to `sh` deserves one they can actually audit.
- **Dependency audit in CI**: fail on any dependency with a build script that
  performs network access, and on any known-telemetry crate. This is the channel
  people forget — the code is clean and a transitive dependency phones home. Shares
  the T16 policy.

## Acceptance

- `bash scripts/check-release-artifacts.sh` — for a tagged build: every declared
  target produced an artifact; every artifact has a checksum; the Homebrew formula
  and installer reference the tag actually being released; `trestle --version`
  inside each artifact matches the tag.
- **Exactly one crate is publishable.** Asserted by walking the workspace and
  checking every member except `trestle-cli` carries `publish = false` — publishing a
  library by accident is a name-squat on someone else's behalf and is not undoable.
- The `musl` artifact runs on a distro image with no toolchain installed, asserted
  in CI — the whole point of static linking, and easy to break silently.
- **Grep assertion: no HTTP client, no update-check code path, and no telemetry
  crate anywhere in the dependency tree.** Shares the T16 harness.
- A fresh container installs via the shell installer and runs `trestle init` on a
  fixture repo successfully, with no other software present.
- The installer script is under 100 lines and has no dependency beyond `curl`,
  `tar` and `sh`.

## Out of scope

An npm wrapper (`D6` permits one later, purely for discoverability, and it must
never become the primary path). Distro packages (apt/AUR/nix) — community
contributions, not v0.1.0. Signing and notarisation, which matter for macOS
Gatekeeper and should be their own node once there is a release to sign.

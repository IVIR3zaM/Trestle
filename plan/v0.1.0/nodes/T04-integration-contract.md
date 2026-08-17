---
id: T04
title: Harness integration contract
tier: deep
gate: human
deps: [T01]
---

## Goal

Define what it means to support a harness, now that **Trestle does not drive the
agent — the agent drives Trestle** (`D5`).

An integration is therefore not a client. It is the set of files that teach one
harness the Trestle workflow, plus an honest declaration of what that harness can
and cannot do.

Blocked on **D1** (which integrations ship) and **D10** (data or code).

## What changed, and why this node is not the old one

The previous version of this node specified `Harness { detect, ask, capabilities }`
— Trestle sends a prompt, the adapter runs the agent, Trestle parses the reply.
That design could not support the VS Code Copilot extension at all, because an
editor extension is not a shellable program, and *every* planning step needed a
live `ask()`.

Inverting control removes the entire hard part. There is no prompt transport, no
wrapped-JSON extraction, no failure taxonomy for a call Trestle never makes. What
is left is file placement and a capability table — which is why `D10` proposes
that integrations be **data**.

The `MockHarness` this node was going to ship is gone with it. Nothing downstream
needs one, because nothing downstream calls a model. **Note the cost honestly:
the mock was also the only mechanism by which prompt quality could have been
regression-tested, and there is now no substitute for T18.**

## The contract

```toml
# integrations/copilot.toml
id          = "copilot"
name        = "GitHub Copilot (VS Code)"

[detect]                       # ALL must hold for the integration to be offered
any_path    = [".github/copilot-instructions.md", ".vscode/"]
any_command = ["code"]         # optional; absence is not disqualifying

[capabilities]
mcp         = true             # can consume trestle mcp (D8)
subagents   = false            # → tier mapping is INERT here (T19)
usage_report= false            # → recorded spend is `unknown` (T20, D11)
roles       = ["planner", "implementer", "verifier"]   # which it can serve (D14)

[[emit]]
template    = "copilot/chatmode.md.tmpl"
path        = ".github/chatmodes/trestle.chatmode.md"
mode        = "create"         # create | append_marked | merge_json
roles       = ["implementer"]  # emitted only if this harness holds the role

[[emit]]
template    = "copilot/instructions.md.tmpl"
path        = ".github/copilot-instructions.md"
mode        = "append_marked"

[[emit]]
template    = "copilot/mcp.json.tmpl"
path        = ".vscode/mcp.json"
mode        = "merge_json"

[conventions]                  # where THIS harness keeps user rules, for T08
reads = [".github/copilot-instructions.md", "AGENTS.md"]
```

Three `mode`s and no more, because each is a different reversibility story:

- `create` — Trestle owns the whole file. Refuse if it exists and differs.
- `append_marked` — wrap in `<!-- trestle:begin -->` / `<!-- trestle:end -->`.
  Re-running replaces only what is between the markers. **The user's own text
  outside the markers is never touched**, and removing the block is a complete
  uninstall.
- `merge_json` — key-wise merge into an existing JSON document, touching only
  Trestle's own keys, preserving formatting where the input allows.

## Design points that are easy to get wrong

- **Idempotence is the whole game.** `trestle init` will be run repeatedly, on
  repos where a human has since edited the emitted files. Running it twice must
  produce the same tree as running it once, and must never destroy adjacent
  content. T23 tests this; this node must make it *specifiable*.
- **Capability differences must be explicit, not papered over.** A harness without
  subagents cannot vary model per unit — that is reported as a gap, never
  silently ignored (T19).
- **No capability may exist only over MCP** (`D8`). Every integration must be
  usable with the CLI alone, or the no-MCP harnesses become second-class and T16
  gains a surface it cannot test.
- **`detect()` is advisory, never exclusive.** Offer what was found, let the user
  add any integration by name and deselect any it found (`D1`, T23). A detector that
  guesses wrong and can't be overridden is worse than no detector.
- **Emission is role-filtered** (`D14`). A harness holding only `verifier` gets the
  review prompt and not the implementation one. A harness holding all three gets
  everything, which is the single-agent default and must produce a tree
  indistinguishable from one with no role model at all — the multi-agent feature must
  cost single-agent users nothing.
- **`roles` in `[capabilities]` is a declaration of what the integration can serve**,
  not a claim about what the harness is good at. Trestle has no basis for the second
  and should not pretend to.
- **Trestle writes outside `.trestle/`** — this is new, and it is the only place
  Trestle mutates files a human owns. It is a threat-model entry (T01) and needs
  an egress-adjacent test (T16): every written path must be declared, and nothing
  outside the declared set may be touched.

## Deliverables

- `docs/INTEGRATION-CONTRACT.md` — the manifest schema, the three emit modes and
  their reversibility guarantees, and how to contribute an integration **without
  writing Rust**.
- `crates/trestle-integration/` — manifest parsing, detection, template
  rendering, and the emit-mode implementations.
- The prompt templates themselves are owned by the nodes whose workflow they
  describe (T06 interrogation, T07 synthesis, T10/T11 execution). This node ships
  the mechanism and one worked integration end to end.

## Acceptance

- `cargo test -p trestle-integration` — the manifest schema round-trips; each
  emit mode is tested against (a) a missing file, (b) a file Trestle wrote, (c) a
  file a human has edited around Trestle's markers; `append_marked` provably
  preserves surrounding content; `merge_json` provably preserves foreign keys.
- Emitting into a fixture repo twice produces a byte-identical tree.
- Removing every marked block and Trestle-owned file returns the fixture repo to
  its original state — asserted, not assumed.
- A capability declared `false` appears in `trestle doctor` output as a stated
  degradation.
- Role-filtered emission: a manifest whose templates declare roles emits only the
  templates for the roles that harness holds, and a harness holding every role emits
  a tree byte-identical to one produced with no roles configured.
- A new integration can be added by writing a manifest and templates only, with
  no change to any Rust file. **Assert this** by having the test suite load a
  fixture integration from a directory rather than from the embedded set.

## Out of scope

The installer UX (T23). The MCP server (T24). Prompt content (T06, T07, T10, T11).

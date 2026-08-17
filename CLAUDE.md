# Claude Code — project instructions

The engineering rules for this repo live in [`AGENTS.md`](AGENTS.md), so that every
harness reads the same file rather than each getting its own drifting copy. Trestle
is harness-agnostic; its own instructions should be too.

@AGENTS.md

Two things specific to working here:

- **How to work the build graph** — one node per pass, tiers, oracles, commit
  policy — is in [`DEVELOPING.md`](DEVELOPING.md) and the `/trestle-build` skill.
- **Picking this up cold?** Read [`CONTEXT.md`](CONTEXT.md) first. It is a
  self-contained briefing and you should not need any prior conversation.

# Contributing to Trestle

Thanks for looking. Trestle is pre-implementation — the plan exists, the code does
not — so what's useful right now is different from what will be useful in a month.

Read [`AGENTS.md`](AGENTS.md) before writing any code. It is short, and it is the
part of this document people actually need.

## The contribution we most want

**A harness integration.** Trestle's value grows directly with the number of coding
assistants it fits into, and integrations are deliberately **data, not code**
([`D10`](plan/v0.1.0/decisions.md)): a TOML manifest saying which files to write
where, plus Markdown templates.

You do not need to know Rust, and you do not need to understand Trestle's internals.
If your assistant reads instructions from a file, it can probably be an integration.
See [`plan/v0.1.0/nodes/T04-integration-contract.md`](plan/v0.1.0/nodes/T04-integration-contract.md)
for the manifest schema — and if that document isn't enough to write one without
reading the source, **that is a bug in the document** and we'd like the report.

## Right now, before there is code

The most valuable thing is disagreement with the plan.

- **[`plan/v0.1.0/decisions.md`](plan/v0.1.0/decisions.md)** records fifteen
  decisions with their reasoning and what was rejected. If one is wrong, saying so
  now is cheap and saying so later is not.
- **[`plan/v0.1.0/nodes/`](plan/v0.1.0/nodes/)** is one file per unit of work, each
  with acceptance criteria. A node whose criteria can't actually be met as written is
  worth an issue.
- **The shape rubric must be willing to say "loop."** A planner that always
  recommends a dependency graph is worthless. If you have a real project where you
  know the right answer, that's a test case we want.

## How the work is organised

This repo is built as a dependency graph of its own units — the same structure
Trestle produces. [`DEVELOPING.md`](DEVELOPING.md) explains how to work it; the short
version:

```bash
make status
```

One node per pass. Every node names an **oracle** — the command that decides it is
done. Three rules are not negotiable:

- **No oracle, no node.** If you can't name a command that proves it done, it's a
  human gate, not a unit.
- **Never edit an oracle, or a test, to make it pass.** If one is wrong, change it
  as a human and say so out loud in the commit.
- **Test first.** The node's acceptance criteria are already a test specification.
  Write the test, watch it fail, then implement.

## Pull requests

- Branch from `main`. One node, or one coherent change, per PR.
- The commit message should say *why*, not *what* — the diff covers what.
- CI runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test
  --workspace` and `cargo deny check`. All four must be green.
- **If you skipped test-first, say so in the PR.** Nothing enforces it
  ([`AGENTS.md` §5](AGENTS.md)), which is exactly why declaring it is the only thing
  that keeps it real.

## Things that will be turned down

Not to be discouraging — these are decisions already made with reasons written down,
so a PR that reverses one without engaging the reasoning is a wasted afternoon:

- **Anything that makes an outbound network connection.** No telemetry, no analytics,
  no update check, no crash reporting, no CDN asset. This is the product's central
  promise and it's enforced by a test, not a policy.
- **An API key, or any inference inside Trestle** (`D0`). Every model call belongs to
  the user's own agent.
- **A vendor model name in the plan schema** (T19). Tiers are abstract, so the same
  plan runs on any assistant.
- **A mutation endpoint in the dashboard** (`D13`). It is a viewer.
- **Speculative abstraction** — an interface with one implementation, a generic with
  one concrete type, a pattern applied to pressure nobody has felt yet
  ([`AGENTS.md` §2](AGENTS.md)).

If you think one of these is wrong, open an issue arguing the decision rather than a
PR changing the code. That's a real conversation and we'll have it.

## Licence

By contributing you agree your work is licensed under the
[Apache License 2.0](LICENSE), the same as the rest of the project.

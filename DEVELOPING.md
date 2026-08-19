# Developing Trestle

## The bootstrap problem

Trestle's job is to plan work and install an executor into your agent. It cannot
do that for itself yet, so its own development runs on hand-written scaffolding:

| Once Trestle exists | Today, by hand |
|---|---|
| the agent asks Trestle to plan | `plan/v0.1.0/` was written by hand |
| `trestle init` installs the executor | `.claude/skills/trestle-build/SKILL.md` |
| the harness maps tiers to models | `.claude/agents/trestle-{cheap,standard,deep}.md` |
| `trestle next` | `make status` |
| `trestle verify` | the executor runs the oracle itself |
| `trestle status` / the dashboard | `make status`, `make graph`, and reading files |

**Replacing each row with the real thing is the milestone that matters.** When T09,
T10, T11 and T23 land, run `trestle init` on this repo and delete the bootstrap
skill. If Trestle can't plan its own remaining work, that is the most useful bug
report the project will ever get.

## Toolchain

Rust (`D6`), stable. `make status` needs `python3`, which macOS and every mainstream
Linux ship — deliberately not Node, since requiring one language's runtime to develop
a tool that plans repos in any language is the thing `D6` rejected.

**T00 creates the workspace**, the `trestle` binary shell, the lints that enforce
half of `AGENTS.md` (`fmt`, `clippy -D warnings`, `cargo deny`) and the CI workflow
they run in. After that, each node's oracle names the crate it owns, and a node
creates the crate its oracle names.

Library crates are `publish = false` and workspace-local; only the binary is
published, as `trestle-cli` (`D15` — `trestle` is taken on crates.io by an unrelated
project). The binary, the Homebrew formula and the repo all stay `trestle`.

## Start here

### 1. Take T00 — nothing blocks it

```bash
make status
```

**All fifteen decisions are resolved**, so nothing in the graph is waiting on a
judgement call. T00 is `tier: cheap` with `deps: []`: the workspace, the binary
shell, the lint config, the CI workflow. Every other oracle assumes it exists, and it
can be done while T01 is still being thought about.

The decisions and their reasoning — including what was rejected and why — are in
`plan/v0.1.0/decisions.md`. Read `D5` (control is inverted) and `D2` (one format,
`shape:` discriminator) before designing anything.

### 2. Do T01 with a human in the loop

```bash
make status
```

T01 (product contract + threat model) is gated deliberately: it decides what the
privacy guarantee actually promises, and every later node is checked against it.
Work through it in a normal interactive session — not via the executor.

Its output is what T16 turns into tests, so the threat model's channel list needs to
be exhaustive rather than representative. It now covers four channels the original
node predated: the MCP server, `trestle init` writing outside `.trestle/`, embedded
assets, and the absence of update checks.

### 3. Then run the graph

```
/trestle-build
```

One node per invocation: it picks a ready node, dispatches it to the agent
matching its tier, runs the oracle itself, commits, and stops.

After T00 and T01, four tracks open in parallel — **T02** (plan format), **T04**
(integration contract), **T05** (repo survey) and **T16** (egress test).

**Take T05 first, then T03, then stop at T28.** That is the vertical slice, and the
whole reason the graph is ordered this way: four nodes gets you `trestle survey` and
`trestle shape` running against real repositories, and T28 is a human gate that asks
whether the shape answer is any good *before* the other twenty units are built on it.
T16 is worth slotting in early alongside — it is cheap, and every node after it
inherits the protection.

## Order worth following

`make status` is authoritative. `make graph` draws it by layer — everything on a
row can run in parallel, and each node shows what it needs and what it unlocks —
and `make graph-mermaid` emits the same graph as a mermaid flowchart if you want
boxes and arrows. The listing below is the same information, kept here so the
shape of the plan survives without running anything:

```
0.  T00  T01                     workspace (cheap, unblocked) + the product gate
1.  T02  T04  T05  T16           four parallel tracks
2.  T03                          ── the slice ──  needs T05 only, not T02
3.  T28                          ⛔ HUMAN GATE — is the shape answer any good?
    ├─ if no: fix the rubric. Do not proceed.
    └─ if yes: everything below unlocks
4.  T06  T08  T12  T19           (these run alongside 2–3; none needs T28)
5.  T07  T13  T20  T27           T07 waits on T28; T27 is human-gated
6.  T09  T14  T15                T09 needs T07+T08+T27
7.  T10  T11  T25                all three need T09+T12
8.  T17                          fan-in over T05,T08,T09,T10,T11,T20
9.  T23  T24  T26                all hang off T17 only — parallel
10. T18                          human-gated dogfood
```

**T00 → T05 → T03 → T28 is a vertical slice**, and it is the point of this ordering.
It gets `trestle survey` and `trestle shape` working against real repositories in
four nodes, so the product's central claim — *does it say "loop" when it should?* —
is tested before the other twenty units are built on top of it. Without the gate,
nothing runs until T17 and the first real feedback arrives after almost all the cost
is spent.

**T02 is the highest-leverage node in the project.** If one thing gets done
carefully, make it that one. Its acceptance bar is expressing both fixtures in
`fixtures/source/` without loss — and neither was written for the format, which
is the point.

**T11 is the one that has to be exactly right.** It is the sole writer of `done`,
and that single restriction is what makes agent-driven execution trustworthy. It
owns `trestle review` too, deliberately: splitting the writers of `done` across two
nodes is how you get a hole. It is `tier: deep` for that reason and not because the
code is hard.

**T27 is human-gated because its output governs everything downstream.** A
distillation of someone's 300-page policy document attaches oracles to units across
the whole plan; a wrong extraction is worse than no extraction, and no oracle can
tell you it read §26 correctly.

**T17 is a wide fan-in.** Nothing after it is blocked by anything before it except
through T17, so the last third of the graph is unusually parallel once it lands.

## Working rules

**How to write the code** — simplicity over abstraction, patterns only when they pay
for themselves, and test-first — is in [`AGENTS.md`](AGENTS.md). Read it before your
first edit. The rules below are about working the *graph*, which is a different
thing.

Carried from `docs/PRIOR-SHAPES.md`, and non-negotiable here:

- **No oracle, no node.** If you can't name a command that proves it done, merge
  it into a node that has one, or make it a human gate.
- **Never edit an oracle to make it pass.** If one is mis-specified, change it as
  a human and say so out loud in the commit.
- **Nodes are contracts, not tasks.**
- **One node per pass, then stop.**
- **Tiers are abstract.** No vendor model name ever appears under `plan/`. The
  mapping lives in `.claude/agents/`, which is the harness-specific layer. This
  repo has to hold to the rule it sells.

The first three are exactly what T07's gauntlet enforces on plans Trestle produces.
A rule this repo won't hold itself to is a rule the product shouldn't ship.

Four more, learned by breaking them rather than by foresight:

- **Push before dispatching an isolated agent.** A worktree branches from
  `origin/main`, not local `HEAD`, so unpushed work simply is not there. Two agents
  were once handed a tree missing the node they depended on.
- **A local oracle pass is not the whole acceptance.** If a node's Acceptance names
  CI, it is not done until CI is green on the pushed commit.
- **Re-run the standing checks after any node that adds a crate** —
  `fmt`, `clippy --all-targets`, `deny`, `test --workspace`, and the already-`done`
  oracles. New crates are exactly what those guards exist to meet.
- **A node file can be wrong, and saying so is the job.** T02b found the plan format
  rejected its own worked example; T05 found its own base commit was stale. Both
  reported instead of working around it, which is why both were cheap to fix.

## When the plan is wrong

Expected, not exceptional. A missing edge is a normal discovery — add it, note it
in the commit, continue. If a node's premise turns out to be false, stop and say
so rather than building on it.

That is also the behaviour T25 (`trestle plan amend`) exists to make first-class for
users, so how it feels to do here is direct evidence for how that node should work.

If an ambiguity appears that you can't resolve from the node file, append it to
`decisions.md`, mark the affected node `blocked`, and stop. Do not guess.

## Not yet set up

- Remote is `github.com/IVIR3zaM/Trestle`, **public**, with CI green on every push.
  While this is solo work, commit to `main` and push each node; switch to one branch
  per node once there are contributors.
- Licensed **Apache-2.0** (`LICENSE`), with `CONTRIBUTING.md` and
  `CODE_OF_CONDUCT.md` in place. The contribution to invite is a harness
  integration, which `D10` made data rather than code.
- **The name is partly verified.** `trestle` is taken on crates.io (`D15`, routed
  around); GitHub is ours. Still to check: the Homebrew tap name, and any trademark
  conflict in developer tools.

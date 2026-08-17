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
| `trestle status` / the dashboard | `make status`, and reading files |

**Replacing each row with the real thing is the milestone that matters.** When T09,
T10, T11 and T23 land, run `trestle init` on this repo and delete the bootstrap
skill. If Trestle can't plan its own remaining work, that is the most useful bug
report the project will ever get.

## Toolchain

Rust (`D6`), stable. `make status` needs `python3`, which macOS and every mainstream
Linux ship — deliberately not Node, since requiring one language's runtime to develop
a tool that plans repos in any language is the thing `D6` rejected.

The workspace doesn't exist yet. T02 creates the first crate; each node's oracle
names the crate it owns, and a node creates the crate its oracle names.

## Start here

### 1. Answer the blocking decision

```bash
$EDITOR plan/v0.1.0/decisions.md
```

Eight of fourteen are resolved. **`D2` — one plan format or two — is the one that
blocks most of the graph**, and `D5` raised its stakes: under inverted control the
*agent* writes this format, so the schema must be strict enough that a
plausible-looking bad plan fails, and its error messages are the interface the agent
converges against. It also has more to hold now — `draft` and `verified` states,
oracle provenance, and the rule that roles stay out of the plan (`D14`).

`D3` (how the code graph is extracted) blocks T05 and T15 and is worth answering at
the same time. `D9`–`D12` are scoped to single nodes and can wait.

Resolve by appending your answer and marking `RESOLVED <date>`.

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

After T01, three tracks open in parallel — **T04** (integration contract), **T05**
(repo survey) and **T16** (egress test). T16 is worth taking early: it is cheap,
and every node after it inherits the protection.

## Order worth following

`make status` is authoritative; this is the same information as layers, so you can
see which tracks are genuinely independent:

```
0.  T01                          the gate everything descends from
1.  T02  T04  T05  T16           four parallel tracks open here
2.  T03  T06  T08  T12  T19      T03 and T06 need T05 as well as T02
3.  T07  T13  T20  T27           T07 is a fan-in; T27 is human-gated
4.  T09  T14  T15                T09 needs T07+T08+T27
5.  T10  T11  T25                all three need T09+T12
6.  T17                          fan-in over T05,T08,T09,T10,T11,T20
7.  T23  T24  T26                all hang off T17 only — parallel
8.  T18                          human-gated dogfood
```

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

## When the plan is wrong

Expected, not exceptional. A missing edge is a normal discovery — add it, note it
in the commit, continue. If a node's premise turns out to be false, stop and say
so rather than building on it.

That is also the behaviour T25 (`trestle plan amend`) exists to make first-class for
users, so how it feels to do here is direct evidence for how that node should work.

If an ambiguity appears that you can't resolve from the node file, append it to
`decisions.md`, mark the affected node `blocked`, and stop. Do not guess.

## Not yet set up

- Remote is `github.com/IVIR3zaM/Trestle`. While this is solo work, commit to
  `main`; switch to one branch per node once there are contributors.
- Licensed **Apache-2.0** (`LICENSE`). No `CONTRIBUTING.md` or code of conduct yet —
  worth having before the first outside PR, since T04 is designed so integrations
  can be contributed as data and that is the contribution to invite.
- **The name is unverified.** "Trestle" was checked only against sibling
  directories on one machine. Check **crates.io**, Homebrew and GitHub before
  publishing — `D6` changed which registry matters.

# Developing Trestle

## The bootstrap problem

Trestle's job is to plan work and emit an executor for it. It cannot do that for
itself yet, so its own development runs on hand-written scaffolding:

| Once Trestle exists | Today, by hand |
|---|---|
| `trestle plan` produces the graph | `plan/v0.1.0/` was written by hand |
| Trestle emits an executor | `.claude/skills/trestle-build/SKILL.md` |
| Harness maps tiers to models | `.claude/agents/trestle-{cheap,standard,deep}.md` |
| `trestle status` | `make status` |
| The dashboard | reading files |

**Replacing each row with the real thing is the milestone that matters.** When
T09 and T11 land, regenerate this repo's own executor with Trestle and delete the
bootstrap version. If Trestle can't plan its own remaining work, that is the most
useful bug report the project will ever get.

## Start here

### 1. Answer the blocking decisions

```bash
$EDITOR plan/v0.1.0/decisions.md
```

Seven are open. Three block most of the graph and should be settled before any
code is written:

- **D2 — one plan format or two?** Every component reads or writes this format.
- **D5 — does Trestle orchestrate execution, or emit instructions?** This defines
  what v0.1.0 actually is. If it lands on "emit", v0.1.0 shrinks by about a third
  and the plan's own shape is worth reconsidering.
- **D1 — which harnesses ship?**

Each has a recommendation and reasoning. Resolve by appending your answer and
marking `RESOLVED <date>`.

### 2. Do T01 with a human in the loop

```bash
make status     # → T01 ready, human-gated
```

T01 (product contract + threat model) is gated deliberately: it decides what the
privacy guarantee actually promises, and every later node is checked against it.
Work through it in a normal interactive session — not via the executor.

Its output is also what T16 turns into tests, so the threat model's channel list
needs to be exhaustive rather than representative.

### 3. Then run the graph

```
/trestle-build
```

One node per invocation: it picks a ready node, dispatches it to the agent
matching its tier, runs the oracle itself, commits, and stops.

After T01, three tracks open in parallel — **T04** (adapter contract), **T05**
(repo survey) and **T16** (egress test). T16 is worth taking early: it is cheap,
and every node after it inherits the protection.

## Order worth following

```
T01 ──┬─→ T02 → T03 ──→ T07 → T09 ──→ T10/T11 ──→ T18
      ├─→ T04 → T19 ──→ T20 ─────────→ T22
      ├─→ T05
      └─→ T16
```

**T02 is the highest-leverage node in the project.** If one thing gets done
carefully, make it that one. Its acceptance bar is expressing both fixtures in
`fixtures/source/` without loss — and neither was written for the format, which
is the point.

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

## When the plan is wrong

Expected, not exceptional. A missing edge is a normal discovery — add it, note it
in the commit, continue. If a node's premise turns out to be false, stop and say
so rather than building on it.

If an ambiguity appears that you can't resolve from the node file, append it to
`decisions.md`, mark the affected node `blocked`, and stop. Do not guess.

## Not yet set up

- No remote. Add one when you're ready to publish; the executor's commit policy
  changes to one branch per node at that point.
- No licence, `CONTRIBUTING.md`, or code of conduct.
- **The name is unverified.** "Trestle" was checked only against sibling
  directories on one machine. Check npm and GitHub before publishing.

---
id: T07
title: Synthesis prompt + plan validation gauntlet
tier: deep
deps: [T02b, T03, T06, T19, T28]
---

## Goal

The agent writes the plan. Trestle makes a bad one impossible to land.

Two deliverables that are two halves of one mechanism: the **prompt** that tells
the agent how to synthesise survey + goal + answers + shape into a plan, and the
**gauntlet** — a validator strict enough that the agent iterates against it
instead of shipping something plausible-looking.

## Why the gauntlet is the load-bearing half

Under `D5` Trestle cannot inspect the agent's reasoning. It can only inspect the
artifact. So every rule that used to live in synthesis code becomes a check that
runs on the written plan, and **a rule with no check is not a rule** — it is a
sentence in a prompt that a tired model will skip.

The rules, each as a check:

| Rule | Check |
|---|---|
| **Every unit gets an oracle** | reject any unit with neither `oracle` nor `gate: human` |
| Oracles are real | every oracle command must appear in the T05 survey's discovered-commands set, or be flagged `unverified` and counted in the report |
| No invented dependencies | every `deps` entry must name an existing unit |
| No cycles | topological sort must succeed |
| Units are contracts, not tasks | reject unit titles matching imperative-task patterns (`implement …`, `add …`, `fix …`) with no `done_when` clause |
| Unresolved questions block specific units | every open decision must name at least one unit, and those units must be `blocked` |
| Human gates where required | product judgement, irreversible actions, and anything the rubric flagged low-confidence must carry `gate: human` |
| Tiers are abstract | reject any vendor model name in the plan (T19) |

### Checks earned by this repo's own build, not imagined

Each of these came from a real failure while building Trestle. They are here
because **a learning that lives only in `DEVELOPING.md` or the bootstrap executor
never reaches a user** — nothing in either ships. The only routes to a user's plan
are this gauntlet, the schema, a shipped prompt, and CLI behaviour. So a lesson
worth keeping has to become one of those, and the gauntlet is the strongest of the
four because it is a check rather than a sentence.

| Rule | Check | What it came from |
|---|---|---|
| **A unit body must not restate structural fields** | reject a `units/<id>.md` whose frontmatter carries `deps`, `tier` or `gate` — the index is authoritative | Ten node files in this repo's own plan drifted from `graph.yaml`, six of them for months. Nothing reads a body file's frontmatter, so a stale `deps:` there misleads exactly the reader who opened it to understand the unit |
| **The oracle must reach what the acceptance claims** | for each `done_when`/acceptance clause naming a command or check, that command must appear in the unit's `oracle` or `extra_oracles`, or the unit must carry an explicit gap note | `fixtures/source/graph-shape/decisions.md` says it outright — *"G05's oracle does not run the build, so a future pass could mark it done over a still-broken build — the oracle is narrower than the problem."* T16 hit the same shape: its acceptance named CI, its oracle did not run CI, and it was marked done while that half was unverified |
| **A plan's own examples obey the plan's own rules** | any example or fixture the plan embeds is validated by the same gauntlet as the plan | T02a's spec stated a rule its own worked example violated, and only implementing the rule in a parser surfaced it |
| **One definition per fact** | *unenforceable — prompt only, and labelled as such* | T05 and T03 each hold a canonical list of the shape signals because T03 did not exist yet. No check can tell that two lists are meant to be the same list; the prompt must ask, and `AGENTS.md` §5 says to name what cannot be checked rather than imply it can |

The last row is the honest one. Three of these are commands; one is a wish, and it
is marked as a wish. A gauntlet that pretended otherwise would be making exactly
the claim this project exists to stop people making.

The imperative-title check is the one worth arguing about, and it is worth having
anyway: *"the existing suites pass unmodified against the new store"* survives
contact with reality; *"implement the store"* does not, and the difference is
detectable in the text.

**A unit with no runnable check must become a human gate** — never a unit with a
hand-waved acceptance line. That rule is the difference between a plan that can be
executed unattended and one that only looks like it, and it is check #1 above.

### The pre-mortem check (`D18`)

`trestle plan write --draft` **refuses a plan with no `premortem` block.** That is
the only enforcement available: under `D0`/`D5` Trestle performs no inference, so it
cannot make an agent think and cannot observe whether it did — the artifact is all
there is. Say that limit out loud wherever the step is described; a doc implying the
thinking is guaranteed would overclaim exactly as one implying the `D9` override is
prevented would.

| Rule | Check |
|---|---|
| The pre-mortem ran | reject a draft with no `premortem` block. Presence is the signal — an absent or empty `risks` list is ambiguous between *found nothing* and *never ran* |
| Findings changed the plan | every `findings` entry needs `hardened_by`; a finding with no change belongs in `risks` instead |
| Accepted risks are reasoned | every `risks` entry needs `why_not_hardened`, exactly as `deferred` needs `revisit_when` |
| Named units exist | every unit id in `findings[].units` or `risks[].units` must name a real unit |

**The purpose is a hardened plan, not a risk register.** A pre-mortem that produced a
tidy list and an unchanged plan has done nothing — it documented the danger instead
of removing it. The prompt must say so in those terms.

The block is required by *presence*, never by volume. A small loop's pre-mortem
legitimately finds nothing and costs two sentences, which is what keeps this from
becoming the ceremony the product exists to prevent.

## The prompt

`templates/synthesize.md`, shipped through T04. It must:

- require `trestle survey --json`, `trestle conventions --json` and
  `trestle shape --json` to be read first, and require the plan to state where it
  **disagrees with the deterministic shape baseline** and why (T03)
- require dependency edges to be derived from the survey's module graph where
  possible, not invented
- require `trestle plan validate` to pass before `trestle plan write` is called,
  and tell the agent that validation errors are the expected way to converge
- when the rubric says *both* (T03), require **both plans** plus the tradeoff
  comparison

`templates/premortem.md`, also shipped through T04. Run by the `verifier` where one
is configured and the `planner` otherwise (`D14`'s asymmetry: an author is the
worst-placed party to imagine their own plan failing), and it must cost single-agent
users nothing — with no verifier configured the planner runs it and the flow is
indistinguishable. It must ask for concrete failure modes tied to named units, and
require each to be **fixed in the plan** unless it genuinely cannot be, in which case
it becomes a `risks` entry carrying why.

## Acceptance

- `cargo test -p trestle-plan --test gauntlet` — each rule above has a fixture
  that violates it and is rejected with a message naming the offending unit and
  path; a valid plan of each of the three shapes passes; a cyclic plan is rejected
  (assert it, don't assume the sort catches it).
- The pre-mortem rules each get a fixture too: a draft with no `premortem`, a
  finding with no `hardened_by`, a risk with no `why_not_hardened`, and a
  `findings[].units` naming a unit that does not exist.
- **Recorded-transcript corpus.** Capture real agent output once, for at least
  three goals across the fixture repos, and commit it under
  `fixtures/transcripts/`. Assert that the gauntlet accepts what a good agent
  produced and rejects hand-mutated copies of it. This does not test the agent —
  it tests that the gauntlet is neither too loose to be useful nor so tight that
  real output can't pass, which is the failure mode that would make the tool
  unusable.
- **Regression corpus, as an eval not a unit test.** Each entry pairs a fixture
  repo and a goal with the plan a human wrote for it. Start with the two in
  `fixtures/source/` — both were written by hand before Trestle existed, so they
  are known-good answers. Scored on whether synthesis finds **the same first unit
  and the same load-bearing dependency**, not on matching text. Run by hand; the
  score is reported in T18, not asserted in CI, because it costs tokens and
  requires a live agent.

## Out of scope

Writing to disk (T09). Rendering (T14). The shape decision itself (T03).

---
id: T08
title: In-repo convention discovery + classification
tier: standard
deps: [T02, T05]
---

## Goal

Fold the user's own coding standards, house agents and existing skills into the
plan as **real units with real verification** — not as advice in a preamble the
agent forgets by step four.

This node owns the **in-repo** case and the **classification vocabulary** that T27
also uses. External policy documents — the 300-page company standard that lives in
someone's Documents folder — are T27's job, because they need chunked ingestion,
distillation, pinning and drift detection, none of which a repo scan requires.

## Requirements

Discover and ingest:

- `AGENTS.md`, `CLAUDE.md`, `.cursorrules`, `.github/copilot-instructions.md`
- `.claude/skills/*/SKILL.md`, `.claude/agents/*.md`
- `.github/prompts/*.prompt.md`, `.github/chatmodes/*.chatmode.md`
- Anything the user explicitly points at (`--standards ./docs/security.md`)

**Ingestion is harness-independent, and deliberately so.** A user running Copilot
may well keep their security pipeline in a `.claude/skills/*/SKILL.md` — the rules
in it are still their rules. Trestle reads every known convention location
regardless of which integration is installed, and the harness only determines
what T04 *emits*, never what T08 *reads*. Each integration declares its own
convention locations (`[conventions].reads` in the T04 manifest) so the set grows
with the integration set instead of being hardcoded here.

Trestle's own **marked blocks** from `trestle init` (T23) must be excluded — ingesting
Trestle's instructions to the agent as if they were the user's engineering
standards would be a comic failure mode, and it is one line of code to prevent.

## The classification vocabulary

Owned here, used by T27 too, so there is one vocabulary and not two:

- **`precondition`** — a gate on matching units. *"All UI changes go through the
  design schema first"* becomes an extra oracle attached to the units it matches.
- **`unit`** — its own node with its own oracle. *"Security review before release."*
- **`human_gate`** — a requirement no command can check. *"PII migrations need a
  reviewed rollback plan."* Real, blocking, and unautomatable — which is different
  from unenforceable.
- **`tier_hint`** — e.g. a house agent declared for a class of work.
- **`context`** — style preferences with no verification. Labelled honestly as
  unenforceable rather than implying they'll be checked.
- **`not_applicable`** — the rule is real but this plan doesn't touch what it
  governs. Reporting *"no UI surface in this work"* is a statement; omitting it
  looks identical to having missed the rule.

The last two matter most: a standard with no oracle is a wish, and the plan should
say which of the user's rules it can actually enforce and which it simply doesn't
reach.

**A precondition must resolve to a command, or it is not a precondition.** The rule
*"must pass `npm run sec:authz`"* becomes an extra oracle command on every matching
unit — indistinguishable, at verify time, from the unit's own (T11). That
indistinguishability is the whole feature: it is what makes an ingested convention
*real* rather than advisory. A rule naming a command the survey cannot find is
classified unenforceable, and says which command it couldn't find.

## Acceptance

- `cargo test -p trestle-conventions` — fixtures for each source format; each
  classified correctly; unenforceable rules flagged as such in the output.
- A fixture repo whose `AGENTS.md` carries a rule of the form "changes of kind X
  must pass check Y first" yields that rule attached as a precondition on
  matching units — not merely quoted in a preamble.
- A rule naming a command absent from the survey is classified unenforceable and
  names the missing command. **A precondition pointing at a command that does not
  exist would fail at verify time on every unit it touched**, which is worse than
  admitting up front that the rule can't be checked.
- Trestle's own marked blocks in `AGENTS.md` / `copilot-instructions.md` are not
  ingested — asserted on a post-`init` fixture repo.
- **This repo's own `AGENTS.md` is a fixture.** Its §5 table states which of its
  rules are machine-checked and which are review-only; classification must arrive at
  the same answer. It is the one input where a hand-written ground truth already
  exists, which makes it the cheapest real test available here.
- Convention locations come from the T04 integration manifests; adding a location
  to a manifest is picked up with no change here.

## Out of scope

External policy documents — discovery of them (T06's elicitation) and ingestion of
them (T27). Running the checks (T11). Emitting instruction files (T04, T23).

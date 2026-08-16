---
id: T08
title: User convention ingestion (skills, agents, standards)
tier: standard
deps: [T02, T05]
---

## Goal

Fold the user's own coding standards, security review checklists, house agents
and existing skills into the plan as **real units with real verification** —
not as advice in a preamble the agent forgets by step four.

## Requirements

Discover and ingest:

- `AGENTS.md`, `CLAUDE.md`, `.cursorrules`, `.github/copilot-instructions.md`
- `.claude/skills/*/SKILL.md`, `.claude/agents/*.md`
- Anything the user explicitly points at (`--standards ./docs/security.md`)

Then classify each into how it should appear in the plan:

- **A gate on every unit** — e.g. "all UI changes go through the design schema
  first" becomes a precondition attached to matching units.
- **A dedicated unit** — e.g. "security review before release" becomes its own
  node with its own oracle.
- **A tier hint** — e.g. a house agent declared for a class of work.
- **Context only** — style preferences with no verification. Label these honestly
  as unenforceable rather than implying they'll be checked.

That last category matters: a standard with no oracle is a wish, and the plan
should say which of the user's rules it can actually enforce.

## Acceptance

- `npm run test:conventions` — fixtures for each source format; each classified
  correctly; unenforceable rules flagged as such in the output.
- A fixture repo whose `AGENTS.md` carries a rule of the form "changes of kind X
  must pass check Y first" yields that rule attached as a precondition on
  matching units — not merely quoted in a preamble.

## Out of scope

Executing them (T10/T11).

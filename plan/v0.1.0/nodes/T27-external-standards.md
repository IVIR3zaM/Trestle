---
id: T27
title: External standards ingestion + distillation store
tier: deep
gate: human
deps: [T08]
---

## Goal

Absorb a policy document that lives **outside the repo** and is **far too large to
fit in a prompt** — a company engineering standard, a security review checklist, a
legal data-handling policy — and turn it into a small, pinned, reviewable rule set
that governs the plan.

Human-gated because **the distillation silently governs every unit**. A wrong
extraction is worse than no extraction: it attaches the wrong oracle to eleven
units, or quietly drops the one rule that mattered.

## Why this is separate from T08

T08 scans the repo and classifies what it finds. That works because the inputs are
small, structured, and already in the working tree.

The real case is none of those things. A 340-page standard owned by a security team
and versioned separately is:

- **not in the repo**, so no scan finds it
- **too large for one pass**, so it cannot be read the way `AGENTS.md` is
- **mostly unenforceable prose**, so most of it must be honestly discarded
- **changing without you**, so a plan built against v4 must notice when v5 lands

Each of those is a mechanism, not a paragraph, which is why this is its own node
with its own oracle.

## Requirements

**Discovery is a question, not a scan.** T06 owns the elicitation question that asks
whether such documents exist — engineering, security, legal, accessibility,
compliance, marketing. This node owns what happens once the user names one. The
answer is persisted per repo so the question is *confirmed* on later plans, never
re-asked from scratch.

**Chunked ingestion.**

```
trestle standards ingest --source <path> [--chunk N]
```

Returns the document in ordered chunks with stable section offsets, so the agent
processes it section by section and appends rules incrementally. The store
deduplicates by rule id. A document that cannot be read in one pass must not
require it to be, and **coverage is tracked**: `sections_read` versus
`sections_total`, so a partial ingestion is visible rather than looking complete.

**Distillation, stored in the repo.**

`.trestle/standards/<name>.yaml` — small, in git, reviewable in a PR. Per rule:

- `id` and `cite` — the section the rule came from. **A rule with no citation is
  rejected**; an unattributable rule cannot be checked against the source by a
  human, which makes the approval gate meaningless.
- `class` — one of `precondition`, `unit`, `human_gate`, `tier_hint`, `context`,
  `not_applicable`
- `oracle` — the command, for `precondition` and `unit` only
- `applies_to` — path globs, so a precondition lands on the units it matches

**`not_applicable` is a first-class classification and new here.** A standard covers
an entire company; a given plan touches a fraction of it. Reporting *"§44
accessibility — no UI surface in this work"* is a useful statement, and silently
omitting those rules looks identical to having missed them.

**Source pinning and drift detection.** Record the path, a SHA-256 of the content,
and the read date. `trestle standards check` compares and reports drift. **A plan
built against a superseded standard is exactly the failure this feature exists to
prevent**, so drift surfaces in `trestle status` rather than only on request.

**The missing-command report.** A rule naming a command the T05 survey cannot find
is reported as a problem, not silently downgraded:

```
⚠ your standard names checks this repo does not have
  SEC-27  §23.3 requires `npm run secrets:scan` — not in package.json
```

Left alone, that rule would attach as a precondition and fail at verify time on
every unit it touched. Surfacing it up front is the whole value — it often means
the standard is right and the repo is behind.

**Approval.** The coverage report and the full rule list are presented for approval
before anything is attached to a plan. Approving records who approved and when.

## Acceptance

- `cargo test -p trestle-standards` — a fixture document of at least 10,000 lines
  ingests across multiple chunks with correct offsets and no duplicated or dropped
  rules; `sections_read` is accurate and a deliberately partial ingestion reports as
  partial.
- A rule with no `cite` is rejected. A `precondition` with no `oracle` is rejected.
- A rule naming a command absent from the survey appears in the problems report and
  is **not** attached as a precondition.
- Changing one byte of the source is detected as drift by `trestle standards check`
  and appears in `trestle status`.
- The distillation round-trips, and every `precondition` resolves to a command the
  survey found — asserted, since this is what T11 will execute.
- **Coverage arithmetic is asserted**: enforceable + context + not_applicable +
  problems equals the total rules extracted. A category quietly swallowing rules is
  the failure mode that would make the report a lie.
- Ingesting the same source twice produces an identical store.

## Out of scope

Reading the document (the agent's job). Attaching rules to units (T09). Running the
commands (T11). Fetching a document from a URL — v0.1.0 takes a local path only,
because fetching one would be an outbound connection and there is no exception to
that (T16). Say so plainly if a user pastes a URL: export it and point at the file.

---
id: T06
title: Decision store + question schema + interrogation prompt
tier: deep
deps: [T02, T05]
---

## Goal

Make interrogation work when the questions are asked **in the user's own chat
interface** rather than by a Trestle terminal wizard.

Trestle no longer generates the questions — the agent does, because generating
them is inference (`D0`, `D5`). What Trestle owns is the part that makes them
good: a **schema no lazy question can satisfy**, a **store** that ties each
question to the work it blocks, and the **prompt** that sets the standard.

## The distinction that still matters

Two kinds of ambiguity, handled differently:

- **Answerable from the code.** *"Is there already a storage abstraction?"* — go
  look. Never ask the user something the repo already answers; that is the fastest
  way to make the tool feel dumb.
- **Requires the user.** Product behaviour, tradeoffs with no objective winner,
  anything irreversible, anything touching cost or security posture.

Getting this split wrong in either direction is the failure mode: asking too much
is annoying, asking too little produces confidently wrong plans.

**Trestle cannot enforce this split — it is a judgement.** What it can do is make
the lazy version fail validation, and give the agent the survey it needs to
resolve the first kind by reading. Both matter, and the second is why this node
depends on T05.

## Requirements

**The question schema.** Two `kind`s, because they need different fields and
forcing one shape on both produces a bad version of each.

`kind: decision` — a tradeoff with no objective winner. Must carry, or be rejected:

- `why` — what is undecided and what made it undecided
- `blocks` — the unit ids it gates. **A question with no blast radius is
  rejected.** An unanswered question must block exactly the right work, never the
  whole plan.
- `options` — two or more, each with its tradeoff stated
- `recommend` — one option, with reasoning. A question with no recommendation is
  rejected: "here are three choices, you pick" is the tool declining to think.

`kind: elicitation` — asking for information Trestle cannot derive at all. Has no
options and no recommendation, because there is nothing to weigh. Must carry:

- `why` and `changes_what` — what the answer will alter about the plan. An
  elicitation that changes nothing is a survey question and is rejected.
- `scope: repo | plan` — whether the answer is remembered for the repository or
  applies to this plan only

Both kinds carry `resolved_by` / `resolved_at`, appended on answer, never
overwriting the question.

Copy the shape of `decisions.md` in this repo. It is the worked example, and it is
the format the store must be able to hold without loss.

**The standards elicitation is built in**, and it is the reason `elicitation`
exists. On the first plan in a repository, the agent must ask whether standards,
checklists or policy documents apply that do **not** live in the codebase — from
engineering, security, legal or privacy, accessibility, compliance, platform,
marketing. Scanning cannot find these; they are owned by other teams and versioned
separately, and a user who is never asked will not think to mention them.

- `scope: repo` — the answer is persisted in `.trestle/config.toml` and
  **confirmed** on later plans, never re-asked from scratch. Being asked the same
  question every time is how a tool teaches people to ignore it.
- The question must state the consequence, because it is not obvious: a rule naming
  a command becomes an extra oracle on matching units, and a rule without one
  becomes a human gate. Answering "none" and producing the document a week later
  means the plan was built around its absence.
- A named document is handed to T27; nothing about reading it belongs here.

**The store** — `trestle decisions` subcommands:

- `trestle decisions add` (stdin, validated) — the agent files a question
- `trestle decisions list --json` — open questions and what they block
- `trestle decisions answer <id> <option>` — records the answer with a timestamp
- Answering a question **unblocks its units and nothing else**, computed rather
  than remembered
- Human-readable on disk, in git, reviewable in a PR — the same file a human can
  edit by hand

**The prompt** — `templates/interrogate.md`, shipped through T04 into each
harness. It must:

- require the agent to run `trestle survey --json` **before** asking anything, and
  to state which candidate questions it resolved by reading
- require questions to be **batched**, not drip-fed one at a time
- tell the agent that `trestle decisions add` will reject an under-specified
  question, so it iterates against the validator rather than against the user

## Acceptance

- `cargo test -p trestle-decisions` — a `decision` missing `blocks`, `options` or
  `recommend` is rejected with a message naming the missing field; an `elicitation`
  carrying `options` is rejected as the wrong kind; an `elicitation` with no
  `changes_what` is rejected; answering a question unblocks exactly its listed units
  and no others; the repo's own `decisions.md` round-trips through the store without
  loss.
- A `scope: repo` answer persists to `.trestle/config.toml` and is offered for
  confirmation rather than re-asked — asserted across two successive plans in one
  fixture repo.
- The standards elicitation is present on a first plan in a repo with no recorded
  answer, and absent-but-confirmable on the second. **Asserted, because it is the
  only question the agent must ask without being prompted by something it found**,
  and a prompt-only guarantee would silently rot.
- A `RESOLVED` question is never mutated by a later write; the answer is appended.
- Fixture test for the "already answered by the repo" case: given a survey that
  reports a storage abstraction exists, a question asking whether one exists is
  flagged by `trestle decisions add --lint` as likely answerable from the survey.
  **This is a lint, not a rejection** — it can produce false positives, and
  blocking on it would be worse than the problem.

## Out of scope

Asking them (the agent, in its own UI). Turning answers into a plan (T07).
Rendering blocked units (T14).

## Known limitation to record, not hide

Whether the agent actually reads the survey before asking is not verifiable by
Trestle. The lint above catches the crudest cases. Everything else is T18's job,
and if dogfooding shows agents asking questions the repo answers, the fix is
prompt work here — not a new validator.

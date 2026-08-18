---
id: T14
title: Dashboard — drafts, work view, multi-agent workflow
tier: standard
deps: [T13]
---

## Goal

Show the plan: units, dependencies, what is running, what is done, what is
blocked and on what — and **who is doing it**, when more than one agent is involved.

Under `D13` this is where a plan is *reviewed*, not just monitored. The agent hands
the user a deep link to a draft and the user decides from what they see here, so a
view that renders a draft badly costs a bad plan.

## The drafts view

- **Drafts are the landing surface**, listed separately from active plans and
  visually distinct from them. A draft is a proposal, and it must never be mistakable
  for work in progress.
- Deep-linkable: `/drafts/<name>` renders that specific draft directly, because the
  link comes from a chat message and dumping the user on an index is friction.
- Everything the decision needs on one screen: the shape and why, the unit graph,
  the estimate, the questions that were asked and how they were answered, the
  standards attached and where each came from (T08/T27), and the units that are
  human-gated.
- **Says how to accept it**, in the user's own terms — "say `approve` in your chat"
  — because there is deliberately no button (`D13`) and a dead end here would be
  the single most confusing moment in the product.
- Rendering a draft must not require it to be complete: a plan mid-synthesis renders
  what exists and says what's missing.

## The work view

- **Graph shape**: a DAG rendered so the critical path and the parallelisable
  branches are visible at a glance. Status by colour *and* a second cue — never
  colour alone.
- **Loop shape**: the queue in order, with the journal readable inline. A loop
  view that pretends to be a graph is worse than an honest list.
- **Hybrid**: neither pretends to be the other. A graph whose nodes run loop-style
  shows the graph, and opening a node shows its queue and journal. This is the shape
  most real work turns out to be, so it cannot be the one that renders worst.
- **Parallel work in progress** is the headline: when several units are running,
  the user should see that immediately without hunting.
- Blocked units link to the question blocking them, with the decision text.
- A unit's detail shows its oracle, its tier, and its last verification result.
- **Oracles carry their provenance** (T02): a precondition attached from a user
  standard shows the rule id and citation it came from, so an unexplained extra
  command never appears.
- **Overridden units are visually distinct from `done`** (T11), with the reason
  shown. An override that looks like a pass defeats the point of recording it. The
  header carries the count.
- **Superseded units are shown, not hidden** (T25). Collapsing them behind a toggle
  is fine; dropping them is not.
- **Where tiering is inert** (T19), the tier badge says so rather than implying the
  unit ran on a different model than the rest.
- Where the rubric returned "both", show the two proposals side by side with the
  tradeoff table — this is the screen where the user chooses.
- No horizontal page scroll; wide graphs scroll inside their own container.

## The multi-agent workflow view

Shown **only when the user has configured more than one role** (`D14`). A
single-agent user must never see an empty pipeline diagram explaining a feature they
don't use.

- The role pipeline as it is actually configured — which harness holds `planner`,
  `implementer`, `verifier` — read from `.trestle/config.toml`, not invented.
- **Which agent currently holds which unit**, and what is queued for each role. The
  useful question a two-agent user has is *"what's waiting on the verifier?"*, and
  it should be answerable at a glance.
- Units in `verified` (oracle passed, review pending) are their own visible column
  or band. This is the state that exists only in multi-agent setups and the one
  people will be confused by first.
- **Review history per unit** — pass and fail verdicts with reasons and roles, from
  T12. A unit that bounced twice tells you something about either the work or the
  reviewer, and both are worth seeing.
- Honest about what it is: this shows a **workflow the user operates**, not one
  Trestle orchestrates. Trestle drives nothing (`D5`), and a view implying otherwise
  would mislead about what happens if the user walks away.

## The pre-mortem block (`D18`)

The draft view shows `premortem` — what was found, what changed because of it, and
which risks were accepted with their reasons. **A risk nobody reads is a risk nobody
accepted**, and this view is the moment a human is deciding whether to approve.

Show findings and risks distinctly: a finding is a danger that was removed, a risk is
one being carried. Collapsing them into one list loses the difference the format
exists to record.

## Acceptance

- `cargo test -p trestle-ui --test work_view` — renders all three shapes from
  fixtures; blocked states reachable; the both-plans comparison renders; status
  conveyed by two cues.
- A draft renders at its deep link, is visually distinct from an active plan, and
  states how to accept it.
- A fixture plan containing an overridden unit and a superseded unit renders both
  distinctly from `done` — asserted, since these are the two states a viewer could
  most easily mislead someone about.
- A precondition sourced from a standard renders its rule id and citation.
- With one role configured, **the multi-agent view is absent** — asserted, because
  showing it empty is the failure mode. With three roles configured across two
  harnesses, the pipeline, the per-role queues, the `verified` band and the review
  history all render.
- Legible at 1280px and on a laptop screen without zooming.

## Out of scope

Code view (T15). Any control — there is no mutation endpoint (`D13`, T13).

---
id: T15
title: Dashboard — code view with blast radius
tier: standard
deps: [T05, T13]
---

## Goal

Show the **codebase** as a graph, with the plan's blast radius overlaid: these
are the modules this plan touches, and these are the ones that depend on them.

This is the distinctive view. The work view shows what the agent will do; this
one shows what it will do it *to*, and it is where over-broad plans become
obvious. Scope creep is far easier to see than to read.

It matters more under `D5`, not less. When Trestle drove the agent it could refuse
to dispatch an over-broad unit; now it can only advise (T10). **This view is where
the human catches what the tool can no longer prevent** — which makes it the
strongest remaining check on a bad plan before it runs.

## Requirements

- Module-level nodes with real import edges from T05 — not a file tree.
- Overlay: directly touched, transitively affected, untouched. Three states, two
  visual cues each.
- Clicking a module lists the plan units that touch it, and vice versa from T14.
- **Honest about partial data.** Where T05 could not analyse a language, say so on
  the view rather than rendering an authoritative-looking incomplete graph.
- Large repos must stay usable — collapse by directory, don't render 4,000 nodes.

## Acceptance

- `cargo test -p trestle-ui --test code_view` — renders from a survey fixture;
  blast radius matches an independently computed expected set; partial-analysis
  banner appears for a repo with an unsupported language.
- On a multi-module fixture with a plan scoped to one module, that module shows
  as touched, its dependents as transitively affected, and everything else as
  untouched. An overlay that marks the whole repo affected is a failure, not a
  conservative default.

## Out of scope

Editing. Cross-repo view (v0.1.0 is single-repo).

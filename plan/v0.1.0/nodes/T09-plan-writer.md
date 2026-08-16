---
id: T09
title: Plan writer + execution instructions
tier: standard
deps: [T07, T08]
---

## Goal

Write an approved plan to a standard folder in the user's repo, and tell them
exactly how to run it with the harness they have.

## Requirements

- Standard location: `.trestle/plans/<name>/` — index, one file per unit,
  decisions, and (for loop shapes) an empty journal ready to append to.
- **Plain files, in git, reviewable in a PR.** The plan is a document the team
  can argue with, not an opaque artifact.
- Emit an executor the user's harness can actually run: a skill for Claude Code,
  the equivalent for other harnesses, with the shape's rules baked in.
- **Never overwrite an existing plan without an explicit flag**, and never touch
  a plan that already has progress recorded against it.
- Print next steps: the exact command, where status appears, how to open the
  dashboard.

## Acceptance

- `npm run test:writer` — round-trips through the T02 parser; refuses to clobber
  a plan with recorded progress; generated executor references only files it
  actually created.
- Writing a plan into a dirty working tree is safe and reversible (`git status`
  shows only `.trestle/`).

## Out of scope

Running it (T10/T11).

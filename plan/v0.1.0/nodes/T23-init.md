---
id: T23
title: trestle init — integration selection + role assignment
tier: standard
deps: [T04, T17]
---

## Goal

One command, run once per repo, after which the user never has to touch a terminal
to use Trestle again.

**This is the product's first impression** — the role the interrogation wizard held
in the previous design. It has about thirty seconds to establish that Trestle fits
the setup the user already has, and it does that by writing files into that setup
rather than asking the user to come to Trestle.

Blocked on **D1** (which integrations), **D10** (data or code), **D14** (roles).

## Detection proposes; the user decides

Detection is a **starting point, not a verdict** (`D1`). Three cases must all work,
and the second and third are why a pure auto-detect installer is wrong:

1. **Detected and wanted** — the common case, one keystroke.
2. **Detected and not wanted.** Copilot is installed on the machine but the user
   wants only Claude Code configured for this repo. Deselecting must be trivial and
   must not feel like fighting the tool.
3. **Wanted and not detected.** The harness is installed somewhere the detector
   doesn't look, or is about to be. `--with <id>` adds it, with a note about what
   was assumed.

```
$ trestle init

Which assistants should this repo be set up for?
Detection is a suggestion — select whatever you actually use.

  [x] GitHub Copilot (VS Code)   detected   .github/copilot-instructions.md
  [x] Claude Code                detected   .claude/
  [ ] OpenAI Codex               not found  (selectable anyway)
  [ ] generic (AGENTS.md only)

Roles — who does what? One assistant may hold several.

  planner      survey, questions, shape, synthesis   [Claude Code       ▾]
  implementer  writes the code, runs verify          [Claude Code       ▾]
  verifier     independent review of finished work   [Codex             ▾]
                                                     [none — skip review]
```

`--yes` takes detection plus all-roles-to-one-harness for scripted setup, and
`--role implementer=claude-code --role verifier=codex` sets them non-interactively.

## Roles, per D14

- **The single-agent case must stay trivial.** One harness holding all three roles
  is the default and the whole role UI collapses to a single line. Nobody
  configuring one assistant should have to learn a role model.
- Each integration declares which roles it can serve (T04). A harness with no
  read-only mode may still serve `verifier`; the role is about *what it is asked to
  do*, not about a capability it must prove.
- The role assignment is written to `.trestle/config.toml` and drives three things:
  which prompt template each harness receives, what `trestle next --role` returns,
  and whether the dashboard shows the multi-agent view at all (T14).
- **A `verifier` changes the state machine** — units land in `verified` rather than
  `done` until reviewed (T11). Say this at init time, once, plainly. A user who
  configures a reviewer and then finds nothing reaching `done` will assume a bug.
- Selecting no verifier is a first-class choice, not a degraded one.

## What it writes

```
Will write:
  .trestle/config.toml                          create   (integrations + roles)
  .trestle/plans/                               create
  .github/chatmodes/trestle.chatmode.md         create   (implementer prompt)
  .github/copilot-instructions.md               append marked block (14 lines)
  .vscode/mcp.json                              merge 1 key
  .claude/skills/trestle/SKILL.md               create   (planner prompt)
  .mcp.json                                     merge 1 key
  .gitignore                                    append marked block (1 line)

Nothing else in this repo will be touched. Proceed? [y/N]
```

Then it prints where to go next — and *next* is the editor, not another command.

## Requirements

- **Show the full write plan before writing anything**, with the mode per path
  (create / append marked / merge). `--dry-run` prints it and exits.
- **Idempotent.** Running it twice produces the same tree as running it once. This
  is the hard part and the reason the emit modes in T04 are constrained to three.
- **Re-runnable to change your mind.** Adding a harness or reassigning a role later
  is the same command; it must add and rewrite only what changed, and **remove the
  files of a harness that was deselected**. An init that can only add is one users
  will work around by hand-deleting files, which breaks uninstall.
- **Reversible.** `trestle uninstall` removes marked blocks and Trestle-owned files,
  restoring the repo to its prior state. Test it against a repo whose emitted files
  have since been hand-edited.
- **Never touch content outside the markers.** A user with 200 lines of their own
  `copilot-instructions.md` must find all 200 intact.
- **Writes only inside the repo**, and only to declared paths, with one deliberate
  exception below.
- Refuses to run outside a git repository, with an actionable message. The plan is
  meant to be reviewable in a PR, and that only works in git.
- Records the Trestle version that wrote the files, so upgrades can migrate them
  rather than guessing.
- Appends `.trestle/runs/` and `.trestle/ui.port` to `.gitignore` (marked block) —
  plans, status and standards are committed; oracle logs and runtime state are not.
- **States each selected harness's degradations once** (T19, T20): no subagents
  means advisory tiers, no usage reporting means `unknown` spend. Once, here — not
  repeated on every later command.

## The `$HOME` exception

Codex reads MCP server config from a user-level file, not a repo-level one. So one
integration genuinely needs to write outside the repo.

Handle it explicitly rather than quietly: any write outside the repo is listed
separately in the write plan, under its own heading, and requires a distinct
confirmation. If declined, install the rest and report exactly what was skipped and
what the user must add by hand.

## Acceptance

- `cargo test -p trestle-cli --test init` — against fixture repos for each shipped
  integration: the write plan matches expectation; running twice is byte-identical;
  `--dry-run` writes nothing; a hand-edited emitted file is preserved outside its
  markers.
- **Selecting a subset of what was detected installs exactly that subset** — the
  detected-but-deselected harness gets no files at all.
- **Selecting an undetected harness works** and notes what was assumed.
- Re-running with a harness removed deletes that harness's files and marked blocks
  and leaves the others untouched.
- Role assignment round-trips through `.trestle/config.toml`; each harness receives
  the template for the roles it holds and no others; a single harness holding all
  three roles produces a config with no multi-agent surface anywhere.
- Configuring a `verifier` produces the state-machine notice in the output.
- `trestle uninstall` returns each fixture repo to its pre-init tree, asserted by
  comparing hashes — including the case where the user edited around the markers.
- A repo with none of the detectable harnesses offers the `generic` integration with
  a message saying what that means, not an error.
- Running outside git fails with an actionable message and writes nothing.
- A declined `$HOME` write leaves the repo-level install complete and reports the
  manual step.

## Out of scope

The templates themselves (T06, T07, T10, T11 own their prompts). The MCP server
being *runnable* (T24) — this node only writes the config that points at it.

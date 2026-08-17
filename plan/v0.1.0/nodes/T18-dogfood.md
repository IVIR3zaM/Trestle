---
id: T18
title: Dogfood — plan a real repo end to end, from the editor
tier: deep
gate: human
deps: [T10, T11, T14, T15, T16, T20, T23, T24, T25, T26]
---

## Goal

Point Trestle at real repositories, from inside a real editor, and judge whether
the plans are any good. Human-gated: this node is a judgement call, and no oracle
can make it.

**It carries more weight than it used to.** Under `D5` most of the intelligence is
prompt-shaped, and prompts are not unit-testable — the `MockHarness` that would
have made them so is gone with the architecture that needed it (T04). This node is
the only place prompt quality is measured at all, so budget for it accordingly and
do not treat it as a formality at the end of the graph.

## The test

Bring your own repositories. Pick three that match these profiles, each of which
has a **known-good human answer** you can check Trestle against — that is the whole
point, so do not pick work you haven't already thought through.

1. **A multi-week change with independent tracks** — for example adding a second
   deployment target to a service that has one. Expect **graph**. Check that it
   finds the shared-core extraction as an early unit, and that it asks about the
   things you know are genuinely undecided rather than guessing.
2. **A convergence or migration effort with unsettled requirements**, where you
   would iterate rather than plan. Expect **loop**. **If Trestle says graph here,
   the rubric is biased toward structure and that is a release blocker, not a
   curiosity.**
3. **A small, well-tested bugfix.** Expect **loop**, and a plan proportionate to
   the work. A tool that ceremonially graphs a two-hour task will be uninstalled
   after one use.

`fixtures/source/` contains worked examples of the first two profiles if you want a
reference for what a good answer looks like — but dogfooding on fixtures is not
dogfooding. Use real repositories.

## The UX test, which is new and separate

Run the whole of profile 1 in **VS Code with Copilot**, and again in **Claude
Code**, and check the claim the architecture was chosen for:

- After `trestle init`, **is any terminal command needed?** `trestle status` and
  `trestle ui` are permitted (they are for looking, not doing). Anything else the
  user had to type by hand is a defect in the emitted prompts, not a user error.
- Does the agent actually call `trestle survey` before asking questions, or does it
  ask things the repo answers? T06's lint catches the crudest cases; this is where
  the rest surface.
- Does it call `trestle verify` rather than declaring success? If not, the prompt
  is at fault — the invariant holds (nothing got marked done), but the user is left
  doing verification by hand, which is worse UX than the old design.
- Do the validators earn their keep? Count how many times `trestle plan validate`
  rejected something and whether the agent converged. **Zero rejections across
  three repos means the gauntlet is too loose**, not that the agent is good.
- Install the binary the way a user would (`brew` or the installer, T26), on a
  machine that has never had it. Not `cargo run`.

## Acceptance

- `cargo test --workspace` green.
- All three shape recommendations match the human answer, **or** the divergence is
  understood and written up.
- The written plans are ones you would actually follow — judged by executing at
  least one of them to completion, including at least one amend (T25), because a
  plan that survives no discovery has not been tested.
- The code view's blast radius matches what a human reviewer expects.
- The editor-only claim holds for both harnesses, or the failures are listed as
  known limitations in the README rather than quietly omitted.
- No outbound connection during any of it (T16 running throughout).
- The synthesis eval from T07 is scored and the number is recorded here, whatever
  it is.

## Out of scope

Publishing, a launch announcement, the v0.2.0 unattended lane.

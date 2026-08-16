---
id: T04
title: Harness adapter contract
tier: deep
gate: human
deps: [T01]
---

## Goal

One interface through which Trestle drives any coding agent, so the rest of the
codebase never knows which harness is in use.

Blocked on **D1** (which harnesses ship in v0.1.0).

## The contract

Trestle sends a prompt and a response schema; the adapter runs the user's agent
and returns parsed, validated output.

```ts
interface Harness {
  readonly id: string;
  detect(): Promise<boolean>;          // is this harness installed and configured?
  ask<T>(req: AskRequest<T>): Promise<AskResult<T>>;
  capabilities(): HarnessCapabilities; // structured output? subagents? tool use?
}
```

Design points that are easy to get wrong:

- **Structured output is not guaranteed.** Some harnesses reliably return JSON;
  some wrap it in prose; some ignore the instruction. The adapter owns extraction
  and validation, and **must fail loudly rather than return a half-parsed
  object** — a silently mangled plan is worse than an error.
- **Capability differences must be explicit**, not papered over. If a harness
  can't spawn subagents, plans that assume model tiering must degrade visibly,
  with the user told what was lost.
- **No inference in Trestle** (D0). The adapter shells out; it never calls an API.
- **Cost and time are the user's.** Every `ask` is billed to their account, so the
  contract should make call count visible and encourage batching.
- **Failure taxonomy**: not installed, not authenticated, rate-limited, timed out,
  unparseable output, user cancelled. Each needs a distinct error the CLI can act
  on — "something went wrong" is useless when the fix is `gh auth login`.

## Deliverables

- `docs/HARNESS-CONTRACT.md` — the interface and its guarantees.
- `src/harness/types.ts`, plus a `MockHarness` returning canned responses. The
  mock is what lets every downstream node be tested without spending tokens, so
  it ships here rather than later.
- A conformance suite any adapter must pass.

## Acceptance

- `npm run test:adapter-contract` — the mock passes the conformance suite; each
  failure mode maps to its distinct error type; malformed output is rejected
  rather than coerced.
- A new adapter can be written against the docs without reading Trestle's source.

## Out of scope

Real adapters (T17). Prompt content — each consuming node owns its own prompts.

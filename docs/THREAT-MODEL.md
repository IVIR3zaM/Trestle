# Privacy threat model

The privacy half of the product contract; the behavioural half is
[`PRODUCT.md`](PRODUCT.md). Written adversarially: the question throughout is not
*"does Trestle behave well?"* but *"by what mechanism could a line of the user's code
leave this machine, and what stops it?"*

**This document is an input to code.** T16 turns the channel table below into a test
suite, and a channel added here fails that node until it has a test. The table is
therefore a machine-read interface, and its format is fixed — see
[Reading the table](#reading-the-table).

The guarantee being defended, in one line:

> **Trestle makes no outbound network connections. None.**

And its honest boundary, which is equally part of the guarantee:

> Trestle adds no new recipient of the user's code, and cannot remove the one the
> user already chose.

---

## Who sees what

| Party | Sees | Must never see |
|---|---|---|
| **Trestle itself** | everything on disk in the repository it is pointed at, plus its own config | nothing is withheld from it — the point is that it emits nothing |
| **The user's harness vendor** | whatever that harness already sends, plus Trestle's own output once the agent reads it into context | not Trestle's concern to control, but entirely Trestle's job to state |
| **Anyone on the LAN** | nothing — the dashboard is bound to `127.0.0.1` and is the only listener in the product | all of it |
| **Any other network peer** | nothing — no code path opens an outbound connection | all of it |

The second row is the one that gets misread. Trestle cannot make a harness stop
sending code to its vendor, and must never imply that it does. What it can promise is
that it introduces no *additional* recipient — and under `D0` that promise is
structural rather than behavioural: there is no API key, so there is no second
recipient to configure.

## Reading the table

Each row is one mechanism by which data could leave, or by which Trestle could write
where it was not invited. Columns:

- **ID** — stable, of the form `CH-01`. Never renumbered; a retired channel keeps its
  id and says so.
- **Channel** — the mechanism, stated concretely enough to test.
- **Countermeasure** — what actually stops it. Structural absence beats discipline
  wherever it is available: the strongest countermeasure is that the capability is
  not in the binary.
- **Check** — the name of the automated test that holds it, or the literal `GAP`.

**`GAP` is not a free pass.** Every `GAP` row must have a matching entry under
[Gaps](#gaps) saying why no test is possible and what would close it. A channel with
no automated check is a gap and is named as one; a `GAP` with no entry is a
documentation bug, and this document's own oracle rejects it.

## Channels

| ID | Channel | Countermeasure | Check |
|---|---|---|---|
| CH-01 | Outbound HTTP or HTTPS from Trestle's own code | **No HTTP client in the dependency tree at all.** The absence of the capability, not the discipline to leave it alone. Enforced by the `cargo deny` ban list, so a transitive reintroduction fails the build | `egress::no_http_client_in_dependency_tree` |
| CH-02 | DNS resolution, as a side channel that leaks names even when no payload follows | No resolver dependency, and no name resolved from repository content. The whole command surface runs with the network denied | `egress::full_surface_under_network_denial` |
| CH-03 | A raw TCP or UDP socket opened by any command | Denied at the sandbox rather than observed at runtime: an observer can be raced, a denial cannot. Asserted by enumerating the process's own sockets | `egress::no_outbound_socket_opened` |
| CH-04 | A telemetry or analytics SDK, added deliberately or inherited | None in the tree. The `cargo deny` policy names the known offenders explicitly, and the policy lives in the repo where a change to it shows up in review | `egress::no_telemetry_crate_in_tree` |
| CH-05 | A crash reporter uploading a panic, backtrace or repository path | No reporter dependency and no panic hook that writes anywhere but stderr. A panic is a local message, not an event | `egress::no_crash_reporter_in_tree` |
| CH-06 | An update check — the channel that feels harmless and is not, since it reports version plus platform on a schedule | **None, ever.** Not opt-in, not weekly, not first-run. `trestle --version` prints a version and exits; it asks nobody whether that version is current | `egress::version_command_makes_no_request` |
| CH-07 | A dependency that phones home from its build script at build or install time | `cargo deny` fails the build on a dependency with a network-touching build script, and the build itself runs with the network denied | `egress::no_build_script_network_access` |
| CH-08 | The dashboard binding to `0.0.0.0` or any routable address, exposing plans to the LAN | Bound to `127.0.0.1` only, whether started by the user or auto-started on the first draft write. Asserted by enumerating the listening socket's address, never by reading the source | `egress::dashboard_binds_loopback_only` |
| CH-09 | A second listener opened by some other command, so that the audited one is not the only one | `trestle ui` is the **only listener** in the product. `trestle mcp` speaks **stdio** only — no socket, no port. Asserted across the full command surface, not just the MCP server | `egress::ui_is_the_only_listener` |
| CH-10 | The dashboard fetching a remote asset — a CDN script, a webfont, a map tile — which would make the no-egress claim false from inside the browser | Every asset is **compiled into the binary** (`D4`), so there is no fetch path and no data directory to populate. A page that cannot reach out cannot be made to | `egress::dashboard_assets_are_embedded` |
| CH-11 | `trestle init` writing outside `.trestle/` — the only place Trestle mutates files a human owns, and one integration writes into `$HOME` | Every written path is declared in the integration manifest, shown to the user before anything is written, wrapped in markers that leave surrounding content untouched, and reversed exactly by `trestle uninstall`. A write to an undeclared path is a violation of the same class as a network call | `egress::init_writes_only_declared_paths` |
| CH-12 | The integration override directory `~/.config/trestle/integrations/` becoming a write target or an execution path | Read-only, and read as data — a manifest and templates, never code that runs (`D10`). Nothing writes there except `trestle init` writing paths it declared | `egress::integration_override_dir_is_read_only` |
| CH-13 | A diagnostic or support bundle — the classic exfiltration path with a helpful name | No such command exists, and none may be added while this document stands. There is nothing to attach and nowhere to send it | `egress::no_diagnostic_bundle_command` |
| CH-14 | Git subprocesses used by the survey reaching a remote, since `git log` and `git fetch` are the same binary | Only read-only local subcommands are invoked, from an allowlist in the source. No remote-contacting subcommand appears anywhere: no `fetch`, `pull`, `push`, `clone`, or `ls-remote`. Asserted against the allowlist, so adding one fails the suite | `egress::git_invocations_are_local_read_only` |
| CH-15 | The oracle command that `trestle verify` runs, which is arbitrary and can do anything a shell can | The command is the user's own, written into a plan file that is in git and reviewable before it ever runs. Trestle adds no capability the user's own test suite did not already have, and grants no privilege — but it cannot sandbox it, and does not pretend to | GAP |
| CH-16 | Trestle's own output entering the agent's context — the survey, the code graph, the questions — and travelling wherever that context travels | Structural: Trestle emits nothing itself, and the output is plain text the user can read. This is the boundary in row two of the party table, not a leak Trestle can close | GAP |
| CH-17 | Data Trestle writes into the repository — plans, distilled standards, the journal — travelling with the repository when the user pushes it | Everything written is plain text in git, reviewable in a PR before it goes anywhere, and Trestle never runs a remote git command (CH-14). The user's push is the user's decision, made with the diff in front of them | GAP |

## Gaps

Three channels have no automated check. Each is here because the honest answer is
that no test can hold it, and a countermeasure with no test behind it is a wish
(`AGENTS.md` §5 makes the same point about rules).

### CH-15 — The user's own oracle command

`trestle verify` exists to run a command Trestle did not write. Sandboxing it would
break the product: real oracles pull dependencies, start databases, and sometimes
legitimately reach a network the user chose to give them.

What holds instead is provenance and visibility — the command lives in a plan file in
git, and plan synthesis records where each oracle came from, so an oracle nobody
recognises is visible as a diff rather than as behaviour. This is the same shape as
the `D9` override: **loud, not prevented.**

Closing it would require a per-oracle sandbox policy the user opts into, which is a
v0.2.0 conversation and not obviously worth its complexity.

### CH-16 — Trestle's output inside the agent's context

The agent calls Trestle, reads the result, and sends whatever it sends to its vendor.
No test Trestle can write observes that, because it happens in another program the
user installed on purpose.

The countermeasure is honesty in the copy: the guarantee is that Trestle adds no new
recipient, never that the user's existing harness stopped being one. Anything stronger
would be a claim about software Trestle does not ship.

### CH-17 — The user pushing the repository

Plans are meant to live in git — that is how a team argues with them in a PR. The
consequence is that a plan containing module names and file paths goes wherever the
repository goes. This is not a leak; it is the feature, and the user performs it
deliberately with the diff in front of them.

What Trestle owes here is that nothing is written outside the paths it declared
(CH-11) and that it never pushes anything itself (CH-14). Both of those are tested.

## How this is enforced

- **Planted violations, or the guard is not known to work.** T16 asserts that the
  suite **fails** on each of two deliberately planted violations: an outbound HTTP
  request, and a write outside the declared path set. A guard that has never been
  seen to fail is not evidence.
- **Every channel maps to a named test**, compared against this table, so adding a
  row here fails T16 until the test exists.
- **Denial, not observation.** The suite runs with the network denied.
- **No subprocess exemption.** The previous architecture needed one, because Trestle
  invoked the agent. Under `D5` Trestle spawns no agent, so the sandbox around the
  egress suite can be total. This is a strictly stronger guarantee than the earlier
  design could offer.
- **In CI on every pull request**, not only at release.

## Out of scope

The user's harness and its network behaviour, which belongs to the user and to that
vendor. It is documented as the boundary above rather than left for someone to
discover.

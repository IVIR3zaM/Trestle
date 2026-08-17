//! T16: turns `docs/THREAT-MODEL.md`'s channel table into a running test
//! suite. Every `#[test]` below whose name matches a `Check` value in that
//! table *is* the automated proof for that row; `channel_table_check_column_matches_test_list`
//! is what keeps the two from drifting apart.
//!
//! Several channels name a capability that has not been built yet — the
//! survey's git invocations (T05), the dashboard (T13/T14/T15), `trestle
//! init` (T23), the MCP server (T24). Two different honest shapes show up
//! below for that situation, and each is called out where it appears:
//!
//! - **Real but currently vacuous** (CH-10, CH-12): a grep over source that
//!   does not exist yet finds nothing, which is what a real absence would
//!   also produce. The mechanism is genuine and starts holding real weight
//!   the moment the code lands.
//! - **Pending, and self-destructing** (CH-08, CH-11): the command itself
//!   (`trestle ui`, `trestle init`) does not exist, so there is nothing to
//!   check. These assert that the command is *still* unrecognized, with a
//!   comment on why — so the day it is recognized, this very test starts
//!   failing and says exactly what to replace it with, instead of quietly
//!   continuing to pass while checking nothing.
//!
//! `cargo test -p trestle-egress -- --include-ignored` is the node's
//! oracle; nothing here is actually `#[ignore]`d (there is no harness
//! subprocess exemption in v0.1.0 — see the node file), but the flag is
//! honoured all the same in case a future addition needs it.

mod channel_table;
mod command_surface;
mod dependency_lock;
mod filesystem_denial;
mod network_denial;
mod repo_paths;
mod sockets;
mod source_scan;

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

fn product_rust_files() -> Vec<PathBuf> {
    repo_paths::product_crate_dirs()
        .iter()
        .flat_map(|dir| repo_paths::rust_files_under(dir))
        .collect()
}

/// Fails loudly, with the reason, rather than letting a test that depends
/// on network denial pass without the denial ever having run — the
/// requirement is "fail or skip loudly and say why", and a panic is the
/// loudest thing a test can do: it shows up in the summary regardless of
/// output capturing.
fn assert_network_denial_mechanism_works() {
    match network_denial::probe() {
        network_denial::Denial::Denied => {}
        network_denial::Denial::NotDenied(detail) => panic!(
            "the network-denial mechanism ran but did not produce its expected denial signature: \
             {detail}. A test that passed here would be passing for the wrong reason — this platform's \
             guard is not known to work."
        ),
        network_denial::Denial::Unavailable(reason) => panic!(
            "no working network-denial mechanism is available on this machine (target_os = \"{}\"): \
             {reason}. CI (ubuntu-latest) is expected to have one; this test fails loudly on purpose \
             instead of silently passing on a platform where the denial was never enforced.",
            std::env::consts::OS
        ),
    }
}

// ---------------------------------------------------------------------
// CH-01 — Outbound HTTP or HTTPS from Trestle's own code
// ---------------------------------------------------------------------

#[test]
fn no_http_client_in_dependency_tree() {
    let lock_text = dependency_lock::read(&repo_paths::cargo_lock_path());
    let names = dependency_lock::package_names(&lock_text);
    let offenders = dependency_lock::present(&names, dependency_lock::HTTP_CLIENTS);
    assert!(
        offenders.is_empty(),
        "HTTP client crate(s) found in Cargo.lock: {offenders:?} — the guarantee is that this \
         capability is absent from the dependency tree, not that nothing calls it"
    );
}

// ---------------------------------------------------------------------
// CH-02 — DNS resolution as a side channel / the whole surface runs denied
// ---------------------------------------------------------------------

#[test]
fn full_surface_under_network_denial() {
    assert_network_denial_mechanism_works();
    let binary = command_surface::trestle_binary();
    let surface = command_surface::enumerate(&binary);
    assert!(
        !surface.is_empty(),
        "parsed zero invocable commands out of `trestle --help` — the parser or the help text's shape \
         changed, and this must cover the *actual* surface, not a hardcoded sample of it"
    );
    for argv in &surface {
        let output = network_denial::run_denied(&binary.to_string_lossy(), argv, None)
            .unwrap_or_else(|reason| panic!("{reason}"));
        assert!(
            output.status.success(),
            "trestle {} failed with the network denied: stdout={:?} stderr={:?}",
            argv.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

// ---------------------------------------------------------------------
// CH-03 / CH-09 — sockets: none opened, and at most one listener
// ---------------------------------------------------------------------

/// Every listening socket seen on any command in the enumerated surface,
/// polled for the lifetime of each child process. Shared by CH-03 and
/// CH-09, which differ only in what they conclude from the same
/// observation (nothing should listen at all vs. at most one thing should).
fn listening_sockets_across_surface(binary: &PathBuf) -> Vec<(Vec<String>, sockets::Listener)> {
    let mut found = Vec::new();
    for argv in command_surface::enumerate(binary) {
        let mut child = Command::new(binary)
            .args(&argv)
            .spawn()
            .expect("spawn trestle");
        let pid = child.id();
        for _ in 0..100 {
            found.extend(
                sockets::listeners_of(pid)
                    .into_iter()
                    .map(|listener| (argv.clone(), listener)),
            );
            if child.try_wait().ok().flatten().is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        let _ = child.wait();
    }
    found
}

#[test]
fn no_outbound_socket_opened() {
    // Validate the sensor before trusting a negative result from it: spawn
    // a process that deliberately opens a real listening socket, and
    // confirm the enumeration mechanism finds it. Without this half, "no
    // listener found" below would be indistinguishable from "the sensor
    // never looked" (the same trap the node's instructions call out
    // explicitly: do not stub a fake listener and assert against your own
    // stub — this instead points the real sensor at a real, independent
    // process).
    let mut planted = Command::new("python3")
        .args(["-c", "import socket, time\ns = socket.socket()\ns.bind((\"127.0.0.1\", 0))\ns.listen(1)\ntime.sleep(2)\n"])
        .spawn()
        .expect("spawn a python3 listener to validate the socket-enumeration sensor");
    std::thread::sleep(Duration::from_millis(300));
    let observed = sockets::listeners_of(planted.id());
    let _ = planted.kill();
    let _ = planted.wait();
    assert!(
        !observed.is_empty(),
        "the socket-enumeration sensor did not detect a real listening socket opened by an independent \
         process — a 'no listener' result elsewhere would not be evidence of anything"
    );

    let binary = command_surface::trestle_binary();
    let found = listening_sockets_across_surface(&binary);
    assert!(
        found.is_empty(),
        "a command in the surface opened a socket: {found:?}"
    );
}

#[test]
fn ui_is_the_only_listener() {
    // The sensor is validated in `no_outbound_socket_opened` (CH-03)
    // against a real, independently-opened listening socket; this test
    // reuses it to say the count below is zero, not "found nothing because
    // it looked at nothing".
    let binary = command_surface::trestle_binary();
    let found = listening_sockets_across_surface(&binary);
    assert!(
        found.is_empty(),
        "a command in the surface opened a listening socket: {found:?}. `trestle ui` (T13) and \
         `trestle mcp` (T24) do not exist yet — once they do, their argv is enumerated the same as any \
         other command (`command_surface::enumerate` reads `--help` at test time), so this assertion \
         starts distinguishing 'exactly one' from 'more than one' automatically, with no edit needed \
         here."
    );
}

// ---------------------------------------------------------------------
// CH-04 — telemetry / analytics
// ---------------------------------------------------------------------

#[test]
fn no_telemetry_crate_in_tree() {
    let lock_text = dependency_lock::read(&repo_paths::cargo_lock_path());
    let names = dependency_lock::package_names(&lock_text);
    let offenders = dependency_lock::present(&names, dependency_lock::TELEMETRY);
    assert!(
        offenders.is_empty(),
        "telemetry/analytics crate(s) found in Cargo.lock: {offenders:?}"
    );
}

// ---------------------------------------------------------------------
// CH-05 — crash reporter
// ---------------------------------------------------------------------

#[test]
fn no_crash_reporter_in_tree() {
    let lock_text = dependency_lock::read(&repo_paths::cargo_lock_path());
    let names = dependency_lock::package_names(&lock_text);
    let offenders = dependency_lock::present(&names, dependency_lock::CRASH_REPORTERS);
    assert!(
        offenders.is_empty(),
        "crash-reporter crate(s) found in Cargo.lock: {offenders:?}"
    );

    let hooks = source_scan::find_mentions(&product_rust_files(), "panic::set_hook");
    assert!(
        hooks.is_empty(),
        "a custom panic hook was found: {hooks:?} — the default hook already writes only to stderr; a \
         custom one needs review to keep that property true rather than assumed"
    );
}

// ---------------------------------------------------------------------
// CH-06 — update checks
// ---------------------------------------------------------------------

#[test]
fn version_command_makes_no_request() {
    assert_network_denial_mechanism_works();
    let binary = command_surface::trestle_binary();
    let output =
        network_denial::run_denied(&binary.to_string_lossy(), &["--version".to_string()], None)
            .unwrap_or_else(|reason| panic!("{reason}"));
    assert!(
        output.status.success(),
        "trestle --version failed with the network denied"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim_start().starts_with("trestle "),
        "unexpected --version output: {stdout:?}"
    );

    let files = product_rust_files();
    for keyword in [
        "update_check",
        "check_for_update",
        "latest_version",
        "UpdateCheck",
    ] {
        let mentions = source_scan::find_mentions(&files, keyword);
        assert!(
            mentions.is_empty(),
            "found a possible update-check code path ({keyword}): {mentions:?}"
        );
    }
}

// ---------------------------------------------------------------------
// CH-07 — a dependency's build script reaching the network
// ---------------------------------------------------------------------

#[test]
fn no_build_script_network_access() {
    assert_network_denial_mechanism_works();
    let output = network_denial::run_denied(
        "cargo",
        &[
            "build".to_string(),
            "-p".to_string(),
            "trestle-cli".to_string(),
        ],
        Some(&repo_paths::repo_root()),
    )
    .unwrap_or_else(|reason| panic!("{reason}"));
    assert!(
        output.status.success(),
        "cargo build -p trestle-cli failed with the network denied: stderr={:?} — a build script that \
         needs the network is exactly what this must catch",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------
// CH-08 — the dashboard's bind address
// ---------------------------------------------------------------------

#[test]
fn dashboard_binds_loopback_only() {
    let binary = command_surface::trestle_binary();
    let output = Command::new(&binary)
        .arg("ui")
        .output()
        .expect("run trestle ui");
    assert!(
        !output.status.success(),
        "`trestle ui` succeeded — T13 (the dashboard server) has landed. This test is a pending \
         placeholder (docs/THREAT-MODEL.md, CH-08) and must be replaced with the real check: start \
         `trestle ui`, enumerate its listening socket with `sockets::listeners_of`, and assert the \
         address is 127.0.0.1 and nothing else."
    );
}

// ---------------------------------------------------------------------
// CH-10 — dashboard assets fetched instead of embedded
// ---------------------------------------------------------------------

#[test]
fn dashboard_assets_are_embedded() {
    // Real but currently vacuous: T13/T14/T15 have not landed, so there are
    // no `.html`/`.js`/`.css` files under `crates/` yet, and this finds
    // none — the same result a real absence of remote fetches would
    // produce. The scan itself does not change when the dashboard lands.
    let offenders = source_scan::asset_files_with_external_urls(&repo_paths::product_crate_dirs());
    assert!(
        offenders.is_empty(),
        "asset file(s) reference an external http(s) URL: {offenders:?} — dashboard assets must be \
         compiled into the binary (D4), not fetched"
    );
}

// ---------------------------------------------------------------------
// CH-11 — `trestle init`'s filesystem blast radius
// ---------------------------------------------------------------------

#[test]
fn init_writes_only_declared_paths() {
    let binary = command_surface::trestle_binary();
    let output = Command::new(&binary)
        .arg("init")
        .output()
        .expect("run trestle init");
    assert!(
        !output.status.success(),
        "`trestle init` succeeded — T23 has landed. This test is a pending placeholder \
         (docs/THREAT-MODEL.md, CH-11) and must be replaced with the real check: run `trestle init` on \
         a fixture repo, including a $HOME fixture, wrap every path outside the declared set in \
         `filesystem_denial::DeniedDir`, and assert init completes with no write denied — i.e. every \
         write it performs lands inside what it declared. `planted_write_outside_declared_paths_is_denied` \
         below already proves the guard itself works; this test wires it to a real command."
    );
}

// ---------------------------------------------------------------------
// CH-12 — the integration override directory
// ---------------------------------------------------------------------

#[test]
fn integration_override_dir_is_read_only() {
    // Real but currently vacuous: T04/T23 have not landed, so nothing in
    // product source mentions this path yet, and the loop below finds no
    // lines to check — again, what a real absence produces.
    let write_markers = [
        "fs::write",
        "File::create",
        "OpenOptions",
        "remove_file",
        "create_dir",
        ".write(true)",
    ];
    for (file, line_no, line) in
        source_scan::find_mentions(&product_rust_files(), "trestle/integrations")
    {
        for marker in write_markers {
            assert!(
                !line.contains(marker),
                "{}:{line_no} looks like a write into the integration override directory: {line}",
                file.display()
            );
        }
    }
}

// ---------------------------------------------------------------------
// CH-13 — a diagnostic/support bundle command
// ---------------------------------------------------------------------

#[test]
fn no_diagnostic_bundle_command() {
    let files = product_rust_files();
    for keyword in ["diagnostic", "bundle"] {
        let mentions = source_scan::find_mentions(&files, keyword);
        assert!(
            mentions.is_empty(),
            "found a possible diagnostic/support-bundle code path ({keyword:?}): {mentions:?}"
        );
    }
}

// ---------------------------------------------------------------------
// CH-14 — git subprocess allowlist
// ---------------------------------------------------------------------

#[test]
fn git_invocations_are_local_read_only() {
    let invocations = source_scan::git_invocations(&product_rust_files());
    assert!(
        !invocations.is_empty(),
        "no `git` invocation found in product source — this test currently certifies build.rs's \
         `git rev-parse --short HEAD`; if that call ever moves or is removed, this assertion needs a \
         real invocation to exercise the allowlist against, or it passes vacuously"
    );
    for (file, subcommand) in &invocations {
        assert!(
            source_scan::GIT_ALLOWLIST.contains(&subcommand.as_str()),
            "{} invokes `git {subcommand}`, which is not on the read-only allowlist {:?}",
            file.display(),
            source_scan::GIT_ALLOWLIST
        );
    }
}

// ---------------------------------------------------------------------
// The channel table itself: every row maps to a named test or a gap entry
// ---------------------------------------------------------------------

fn own_test_names() -> HashSet<String> {
    let output = Command::new("cargo")
        .args(["test", "-p", "trestle-egress", "--", "--list"])
        .current_dir(repo_paths::repo_root())
        .output()
        .expect("list this crate's own tests via `cargo test -p trestle-egress -- --list`");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .map(str::to_string)
        .collect()
}

fn missing_tests(rows: &[channel_table::ChannelRow], test_names: &HashSet<String>) -> Vec<String> {
    rows.iter()
        .filter_map(|row| row.check.clone())
        .filter(|check| !test_names.contains(check))
        .collect()
}

#[test]
fn channel_table_check_column_matches_test_list() {
    let text = channel_table::read(&repo_paths::threat_model_path());
    let rows = channel_table::parse(&text);
    assert!(!rows.is_empty(), "parsed zero rows out of the channel table — the parser or the table's shape in docs/THREAT-MODEL.md changed");
    let gap_ids = channel_table::gap_ids(&text);
    let test_names = own_test_names();

    let missing = missing_tests(&rows, &test_names);
    assert!(
        missing.is_empty(),
        "channel(s) with a named Check that has no matching test in this crate: {missing:?}"
    );

    for row in &rows {
        if row.check.is_none() {
            assert!(
                gap_ids.contains(&row.id),
                "{} is marked GAP in the channel table but has no `### {}` entry under `## Gaps`",
                row.id,
                row.id
            );
        }
    }

    // Asserted directly, not merely assumed: prove the comparison above
    // would actually catch a channel with no matching test, by running it
    // again against the real table plus one synthetic row naming a test
    // that provably does not exist. This is what makes "adding a row fails
    // this suite until it has a test" true rather than aspirational.
    let mut augmented = rows;
    augmented.push(channel_table::ChannelRow {
        id: "CH-PLANTED".to_string(),
        check: Some("egress::this_test_name_does_not_exist_and_never_will".to_string()),
    });
    let still_missing = missing_tests(&augmented, &test_names);
    assert_eq!(
        still_missing,
        vec!["egress::this_test_name_does_not_exist_and_never_will".to_string()],
        "a channel row naming a nonexistent test was not flagged — the comparison this suite relies on \
         is not actually checking what it claims to"
    );
}

// ---------------------------------------------------------------------
// Planted violations — a guard never seen to fail is not known to work
// ---------------------------------------------------------------------

#[test]
fn planted_outbound_http_request_is_denied() {
    // Unwrapped first: the planted attempt must NOT show the guard's
    // signature here, or the guard below would pass even doing nothing —
    // this machine having no route to the internet at all would otherwise
    // look identical to a real denial.
    if let network_denial::Denial::Denied = network_denial::probe_unwrapped() {
        panic!(
            "the unwrapped connection attempt already showed the network-denial mechanism's signature \
             — the guard is not known to work, because nothing distinguishes it from doing nothing"
        );
    }

    match network_denial::probe() {
        network_denial::Denial::Denied => {}
        network_denial::Denial::NotDenied(detail) => {
            panic!("planted outbound network request was not denied: {detail}")
        }
        network_denial::Denial::Unavailable(reason) => {
            panic!("network-denial mechanism unavailable: {reason}")
        }
    }
}

#[test]
fn planted_write_outside_declared_paths_is_denied() {
    if let filesystem_denial::Availability::Unavailable(reason) = filesystem_denial::is_available()
    {
        panic!("filesystem-denial mechanism unavailable: {reason}");
    }
    let scratch =
        std::env::temp_dir().join(format!("trestle-egress-fs-planted-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).expect("create scratch dir");

    // Unguarded baseline: the identical write, with no guard at all, must
    // succeed — otherwise the denial below would prove nothing.
    let baseline = scratch.join("baseline");
    std::fs::create_dir_all(&baseline).expect("create baseline dir");
    filesystem_denial::attempt_write(&baseline, "unwrapped.txt").expect(
        "the unwrapped write must succeed — it establishes what the guard is judged against",
    );

    // Guarded: the planted violation, denied.
    {
        let denied = filesystem_denial::DeniedDir::create_under(&scratch, "undeclared");
        let result = filesystem_denial::attempt_write(&denied.path, "planted-violation.txt");
        assert!(
            result.is_err(),
            "a write outside the declared path set was not denied"
        );
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied,
            "the write failed, but not with a permission error — that is not evidence the guard denied it"
        );
    } // `denied`'s Drop restores the write bit so cleanup below can run.

    // The declared side still accepts writes: the guard is scoped to
    // "outside the declared set", not "no writes anywhere".
    let declared = scratch.join("declared");
    std::fs::create_dir_all(&declared).expect("create declared dir");
    filesystem_denial::attempt_write(&declared, "ok.txt")
        .expect("a write inside the declared set must still succeed");

    std::fs::remove_dir_all(&scratch).ok();
}

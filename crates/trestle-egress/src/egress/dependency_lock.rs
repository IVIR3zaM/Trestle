//! Grep-based checks over `Cargo.lock` — the strongest guarantee is that a
//! capability is absent from the dependency tree, not that nothing in the
//! tree happens to use it today (`docs/THREAT-MODEL.md`, CH-01/CH-04/CH-05).
//!
//! These lists mirror `deny.toml`'s ban categories by hand, the same way
//! `deny.toml` itself is hand-maintained. They are a second, independent
//! guarantee alongside `cargo deny check` in CI — a name added to only one
//! of the two is a hole, so both need updating together.

use std::collections::HashSet;
use std::path::Path;

/// Every `name = "..."` inside a `[[package]]` block in `Cargo.lock`. Read
/// as plain text rather than parsed as TOML, on purpose — `Cargo.lock`'s
/// format is stable and grep-simple, and adding a TOML-parsing dependency
/// to prove the *absence* of dependencies would be an odd trade.
pub(super) fn package_names(cargo_lock_text: &str) -> HashSet<String> {
    cargo_lock_text
        .lines()
        .filter_map(|line| line.strip_prefix("name = \"")?.strip_suffix('"'))
        .map(str::to_string)
        .collect()
}

pub(super) const HTTP_CLIENTS: &[&str] = &[
    "reqwest",
    "curl",
    "hyper",
    "hyper-util",
    "ureq",
    "isahc",
    "surf",
    "attohttpc",
];

pub(super) const TELEMETRY: &[&str] = &[
    "sentry",
    "opentelemetry",
    "tracing-opentelemetry",
    "posthog",
    "analytics",
];

pub(super) const CRASH_REPORTERS: &[&str] = &[
    "sentry",
    "sentry-backtrace",
    "sentry-panic",
    "bugsnag",
    "rollbar",
    "airbrake",
    "crashpad",
];

/// Which names from `banned` are actually present in `found` — the empty
/// list is the passing case.
pub(super) fn present(found: &HashSet<String>, banned: &[&str]) -> Vec<String> {
    banned
        .iter()
        .filter(|name| found.contains(**name))
        .map(|s| (*s).to_string())
        .collect()
}

pub(super) fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

//! The agent-facing command surface, enumerated from the binary itself —
//! never hardcoded here. T17 (the CLI command surface) has not landed yet,
//! so today's surface is small (`--version`, `--help`), but the parsing
//! below reads whatever `trestle --help` prints at test time. When T17 adds
//! subcommands, this list grows with it automatically; nobody has to
//! remember to update a list in this crate.
//!
//! `--help`'s exact format is not yet a `clap`-generated one (T17 has not
//! introduced `clap`), so this parses today's ad hoc two-section shape:
//! `Usage:` lines and — for forward compatibility with `clap`'s default
//! output once T17 lands — a `Commands:` section. Whichever section
//! actually exists in the output is the one that contributes entries.

use std::path::PathBuf;
use std::process::Command;

use super::repo_paths;

/// Builds `trestle-cli` if its binary is not already present, then returns
/// the binary's path. `cargo test -p trestle-egress` alone does not build
/// `trestle-cli` — that is a different package — so every test that runs
/// the CLI needs this first.
pub(super) fn trestle_binary() -> PathBuf {
    let path = repo_paths::trestle_binary_path();
    if !path.exists() {
        let status = Command::new("cargo")
            .args(["build", "-p", "trestle-cli"])
            .current_dir(repo_paths::repo_root())
            .status()
            .expect("spawn cargo build -p trestle-cli");
        assert!(
            status.success(),
            "cargo build -p trestle-cli failed while preparing the command surface"
        );
    }
    path
}

/// One invocable command: the argv Trestle is run with (not including the
/// binary name itself).
pub(super) type Invocation = Vec<String>;

/// Parses `trestle --help`'s output into the concrete commands an agent can
/// run today. A line is a concrete invocation only if it names no
/// placeholder (`[...]`/`<...>`) — `trestle [OPTIONS]` is a pattern, not a
/// command; `trestle --version` is a command.
pub(super) fn enumerate(binary: &PathBuf) -> Vec<Invocation> {
    let output = Command::new(binary)
        .arg("--help")
        .output()
        .expect("run trestle --help");
    // main.rs currently writes usage to stderr; a future clap-based --help
    // would write it to stdout. Read both, in that order, so this survives
    // T17 without edits here.
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut invocations = from_usage_lines(&text);
    invocations.extend(from_commands_section(&text));
    invocations.sort();
    invocations.dedup();
    invocations
}

fn from_usage_lines(text: &str) -> Vec<Invocation> {
    text.lines()
        .filter_map(|line| {
            let rest = line
                .trim_start()
                .strip_prefix("Usage: trestle ")
                .or_else(|| line.trim_start().strip_prefix("trestle "))?;
            if rest.contains('[') || rest.contains('<') {
                return None;
            }
            Some(
                rest.split_whitespace()
                    .map(str::to_string)
                    .collect::<Invocation>(),
            )
        })
        .filter(|argv: &Invocation| !argv.is_empty())
        .collect()
}

fn from_commands_section(text: &str) -> Vec<Invocation> {
    let Some(section_start) = text.find("Commands:") else {
        return Vec::new();
    };
    text[section_start..]
        .lines()
        .skip(1)
        .take_while(|line| !line.trim().is_empty())
        .filter_map(|line| line.split_whitespace().next())
        .map(|name| vec![name.to_string()])
        .collect()
}

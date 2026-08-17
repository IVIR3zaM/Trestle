//! `D3`'s other required fallback signal: how often two files change
//! together, from `git log`. This is coupling evidence, never a dependency
//! edge — the caller (`survey.rs`) is required to mark every edge this
//! produces `heuristic: true` for exactly that reason (see `docs/THREAT-MODEL.md`
//! CH-14 for why the git invocation itself is restricted to `log`, a
//! read-only local subcommand, and never touches a remote).
//!
//! Parsing and counting are pure functions, tested directly against
//! hand-written `git log` output — no throwaway git repo needed for that
//! part. The `git log` invocation itself is a thin wrapper around them.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// `%H` alone is ambiguous against a file whose name happens to look like a
/// hex hash, so the format uses a marker no filename can start a line with.
const COMMIT_MARKER: &str = "COMMIT:";

/// Splits `git log --name-only --pretty=format:COMMIT:%H` output into one
/// file list per commit. A line starting with the marker begins a new
/// commit; every other non-blank line is a file path in that commit.
pub(crate) fn parse_log_output(output: &str) -> Vec<Vec<String>> {
    let mut commits: Vec<Vec<String>> = Vec::new();
    for line in output.lines() {
        if let Some(_hash) = line.strip_prefix(COMMIT_MARKER) {
            commits.push(Vec::new());
        } else if !line.trim().is_empty() {
            if let Some(files) = commits.last_mut() {
                files.push(line.to_string());
            }
        }
    }
    commits
}

/// Every unordered pair of files that appear together in the same commit,
/// counted across all commits — the canonical key orders the pair
/// lexicographically so `(a, b)` and `(b, a)` accumulate into one entry.
pub(crate) fn pair_counts(commits: &[Vec<String>]) -> HashMap<(String, String), u32> {
    let mut counts = HashMap::new();
    for files in commits {
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                let key = if files[i] <= files[j] {
                    (files[i].clone(), files[j].clone())
                } else {
                    (files[j].clone(), files[i].clone())
                };
                *counts.entry(key).or_insert(0) += 1;
            }
        }
    }
    counts
}

/// Runs `git log` (read-only, local, on the CH-14 allowlist) over
/// `repo_root` and returns the same co-occurrence counts `pair_counts`
/// computes. `None` if `repo_root` is not a git repository at all — that is
/// not a partial result, it is the absence of a signal that only applies to
/// version-controlled repos.
pub(crate) fn co_change_counts(repo_root: &Path) -> Option<HashMap<(String, String), u32>> {
    // `-- .` scopes the log to commits that touched something under
    // `repo_root`: without it, a `repo_root` that is a subdirectory of a
    // larger repository (as the test fixtures below are, nested inside
    // Trestle's own repo) would pull in that larger repo's entire,
    // unrelated history. Git still reports paths relative to the
    // repository's real root rather than to `repo_root` in that case;
    // `modules.rs` handles the mismatch by only keeping pairs that resolve
    // to a module this survey actually found, so a nested `repo_root`
    // degrades to zero co-change edges rather than wrong ones.
    let output = Command::new("git")
        .current_dir(repo_root)
        .args([
            "log",
            "--name-only",
            &format!("--pretty=format:{COMMIT_MARKER}%H"),
            "--",
            ".",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Some(pair_counts(&parse_log_output(&text)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_log_output_splits_into_one_file_list_per_commit() {
        let output = "COMMIT:aaa\napp_a/main.py\nshared/helper.py\n\nCOMMIT:bbb\napp_a/main.py\n";
        let commits = parse_log_output(output);
        assert_eq!(
            commits,
            vec![
                vec!["app_a/main.py".to_string(), "shared/helper.py".to_string()],
                vec!["app_a/main.py".to_string()],
            ]
        );
    }

    #[test]
    fn pair_counts_counts_files_changed_together_and_ignores_solo_commits() {
        let commits = vec![
            vec!["app_a/main.py".to_string(), "shared/helper.py".to_string()],
            vec!["app_a/main.py".to_string()], // solo change: contributes no pair
            vec!["shared/helper.py".to_string(), "app_b/main.py".to_string()],
        ];
        let counts = pair_counts(&commits);
        assert_eq!(
            counts.get(&("app_a/main.py".to_string(), "shared/helper.py".to_string())),
            Some(&1)
        );
        assert_eq!(
            counts.get(&("app_b/main.py".to_string(), "shared/helper.py".to_string())),
            Some(&1)
        );
        assert_eq!(counts.len(), 2);
    }

    #[test]
    fn pair_counts_orders_each_pair_canonically_regardless_of_commit_order() {
        // "shared" listed before "app_a" here, reversed from the test above —
        // the key must come out identical either way, or two commits that
        // touch the same two files in different listing order would be
        // undercounted as separate pairs.
        let commits = vec![vec![
            "shared/helper.py".to_string(),
            "app_a/main.py".to_string(),
        ]];
        let counts = pair_counts(&commits);
        assert_eq!(
            counts.get(&("app_a/main.py".to_string(), "shared/helper.py".to_string())),
            Some(&1)
        );
    }

    #[test]
    fn co_change_counts_runs_against_a_real_repo_without_erroring() {
        // A smoke test against this crate's own repository, deliberately
        // weak on specifics (real git history is data, not a fixture, and
        // will keep changing) — it exists to prove the `git log` invocation
        // itself works and returns something, not to pin exact counts.
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("crates/trestle-survey has two ancestors: crates/ and the repo root");
        let counts = co_change_counts(repo_root);
        assert!(
            counts.is_some(),
            "expected a git repository at {repo_root:?}"
        );
    }

    #[test]
    fn non_git_directory_yields_no_signal_rather_than_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(co_change_counts(dir.path()).is_none());
    }
}

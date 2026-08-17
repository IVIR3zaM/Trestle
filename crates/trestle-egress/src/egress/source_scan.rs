//! Grep-based checks over Trestle's own product source (never this crate's
//! own — `repo_paths::product_crate_dirs` excludes it, or every one of
//! these checks would trip on its own doc comments). Several of the
//! channels below name code that has not been built yet (the survey's git
//! invocations, the dashboard's assets, `trestle init`'s override
//! directory); until it exists, these checks are real but vacuously true —
//! grepping a source tree that does not yet contain the capability finds
//! nothing, which is the same thing a real absence would produce. That is
//! noted at each call site in `mod.rs`, not asserted away here.

use std::path::{Path, PathBuf};

/// Subcommands the survey (T05) is allowed to invoke `git` with — read-only
/// and local. Adding `fetch`, `pull`, `push`, `clone` or `ls-remote` here
/// would need a human to decide `git_invocations_are_local_read_only`
/// should widen, which is the point: the list lives in this test, not in
/// product code, so widening it is a reviewable diff.
pub(super) const GIT_ALLOWLIST: &[&str] = &[
    "rev-parse",
    "log",
    "diff",
    "diff-tree",
    "status",
    "ls-files",
    "show",
    "branch",
    "describe",
    "cat-file",
    "rev-list",
    "shortlog",
    "blame",
    "tag",
    "config",
    "name-rev",
];

/// Every `Command::new("git")` call found, paired with the subcommand it
/// was given — the first quoted string literal in the same statement,
/// which is where `.arg("subcommand")` and `.args(["subcommand", ...])`
/// both put it.
pub(super) fn git_invocations(files: &[PathBuf]) -> Vec<(PathBuf, String)> {
    let mut found = Vec::new();
    for file in files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        let needle = "Command::new(\"git\")";
        let mut search_from = 0;
        while let Some(rel_pos) = text[search_from..].find(needle) {
            let after = search_from + rel_pos + needle.len();
            let statement_end = text[after..].find(';').map_or(text.len(), |p| after + p);
            if let Some(subcommand) = first_string_literal(&text[after..statement_end]) {
                found.push((file.clone(), subcommand));
            }
            search_from = after;
        }
    }
    found
}

fn first_string_literal(text: &str) -> Option<String> {
    let start = text.find('"')? + 1;
    let end = text[start..].find('"')?;
    Some(text[start..start + end].to_string())
}

/// Case-insensitive substring search across `files`, returning `(file,
/// line number, line text)` for every match — used for the several
/// "this capability must not exist at all" checks below.
pub(super) fn find_mentions(files: &[PathBuf], needle: &str) -> Vec<(PathBuf, usize, String)> {
    let needle_lower = needle.to_lowercase();
    let mut found = Vec::new();
    for file in files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        for (n, line) in text.lines().enumerate() {
            if line.to_lowercase().contains(&needle_lower) {
                found.push((file.clone(), n + 1, line.to_string()));
            }
        }
    }
    found
}

/// Whether `dir`'s content contains a bare `http://` or `https://`
/// reference — the signature of a dashboard asset fetching something
/// remote rather than shipping compiled into the binary (`D4`).
pub(super) fn asset_files_with_external_urls(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut offenders = Vec::new();
    for dir in dirs {
        for file in asset_files_under(dir) {
            if let Ok(text) = std::fs::read_to_string(&file) {
                if text.contains("http://") || text.contains("https://") {
                    offenders.push(file);
                }
            }
        }
    }
    offenders
}

fn asset_files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|n| n.to_str()) != Some("target") {
                    stack.push(path);
                }
            } else if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("html" | "js" | "css")
            ) {
                files.push(path);
            }
        }
    }
    files
}

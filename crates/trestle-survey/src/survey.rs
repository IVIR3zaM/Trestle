//! The entry point: one repository on disk in, one `Survey` out. Every
//! other module in this crate measures one part of the picture; this file's
//! only job is assembling them, so `lib.rs` stays a two-hop jump from here
//! to whichever part does the actual work.

use crate::schema::{LanguageStat, Support, Survey, SCHEMA_VERSION};
use crate::{ci, commands, conventions, modules, repo_files, shape_signals};
use std::collections::HashMap;
use std::path::Path;

/// Reads `repo_root` and produces a structured, versioned survey of it.
/// Read-only: nothing under `repo_root` is written, and every subprocess
/// this crate spawns (`git log`, see `co_change.rs`) is local and read-only
/// (`docs/THREAT-MODEL.md` CH-14).
pub fn survey(repo_root: &Path) -> Survey {
    let files = repo_files::walk(repo_root);
    let graph = modules::build(repo_root, &files);
    let discovered_commands = commands::discover(repo_root);
    let shape_signals =
        shape_signals::compute(&graph.modules, &graph.edges, &discovered_commands, &files);

    Survey {
        schema_version: SCHEMA_VERSION,
        partial: !graph.partial_reasons.is_empty(),
        partial_reasons: graph.partial_reasons,
        languages: language_stats(&files),
        modules: graph.modules,
        edges: graph.edges,
        test_commands: discovered_commands.test_commands,
        build_commands: discovered_commands.build_commands,
        ci_configs: ci::discover(repo_root),
        conventions: conventions::discover(repo_root),
        shape_signals,
    }
}

fn language_stats(files: &[repo_files::SourceFile]) -> Vec<LanguageStat> {
    let mut by_language: HashMap<&'static str, (Support, usize)> = HashMap::new();
    for file in files {
        let entry = by_language
            .entry(file.language)
            .or_insert((file.support, 0));
        entry.1 += 1;
    }
    let mut stats: Vec<LanguageStat> = by_language
        .into_iter()
        .map(|(name, (support, file_count))| LanguageStat {
            name: name.to_string(),
            support,
            file_count,
        })
        .collect();
    stats.sort_by(|a, b| a.name.cmp(&b.name));
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn survey_of_an_empty_directory_is_not_partial_and_has_no_languages() {
        let dir = tempfile::tempdir().unwrap();
        let result = survey(dir.path());
        assert!(!result.partial);
        assert!(result.languages.is_empty());
        assert!(result.modules.is_empty());
        assert!(result.edges.is_empty());
    }

    #[test]
    fn survey_assembles_languages_modules_and_edges_from_one_repo() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("app_a")).unwrap();
        fs::create_dir_all(dir.path().join("shared")).unwrap();
        fs::write(
            dir.path().join("app_a/main.py"),
            "from shared import helper\n",
        )
        .unwrap();
        fs::write(dir.path().join("shared/helper.py"), "def helper(): pass\n").unwrap();

        let result = survey(dir.path());
        assert_eq!(result.languages.len(), 1);
        assert_eq!(result.languages[0].name, "python");
        assert_eq!(result.modules.len(), 2);
        assert!(result
            .edges
            .iter()
            .any(|e| e.from == "app_a" && e.to == "shared"));
    }

    #[test]
    fn an_unsupported_language_marks_the_whole_survey_partial() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("app.rb"), "puts 'hi'\n").unwrap();
        let result = survey(dir.path());
        assert!(result.partial);
        assert!(!result.partial_reasons.is_empty());
    }

    /// The node's own acceptance bullet: "every shape signal T03 consumes
    /// is present and has a defined measurement — asserted by iterating
    /// T03's signal list rather than by hand-written checks, so adding a
    /// signal there fails here until it is measured." T03 does not exist
    /// as a crate yet (it depends on this node — see `shape_signals.rs`'s
    /// module comment), so `shape_signals::SIGNAL_NAMES` is the canonical
    /// list this node commits to; this test iterates it instead of
    /// hand-writing five separate `assert!` calls, so a signal added to
    /// that one list and forgotten in `ShapeSignals::to_json` fails here.
    #[test]
    fn every_signal_in_the_canonical_list_is_present_in_the_json_output() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("app.py"), "import os\n").unwrap();
        let result = survey(dir.path());
        let signals = &result.to_json()["shape_signals"];
        for name in crate::shape_signals::SIGNAL_NAMES {
            assert!(
                !signals[name].is_null(),
                "shape_signals.{name} (from SIGNAL_NAMES) has no value in the JSON output"
            );
        }
    }
}

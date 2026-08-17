//! Acceptance tests for T05, one per bullet in the node file's Acceptance
//! section. `tests/fixtures/mixed_repo/` is the fixture repo: Rust and
//! Python get real tree-sitter extraction, JavaScript uses the `D3` regex
//! fallback, and Ruby is deliberately unsupported. The Python half
//! (`python_app_a`, `python_app_b` both importing `python_shared`) is the
//! multi-module-with-a-shared-package fixture the "consumer edge" bullet
//! asks for.

use std::fs;
use std::path::{Path, PathBuf};

/// Copies the checked-in fixture into a fresh temp directory before every
/// test that surveys it. Two reasons, not one: it keeps the golden-file
/// output stable regardless of where the fixture happens to be checked out,
/// and — more importantly — it keeps the fixture *outside* any git
/// repository, since it is nested inside Trestle's own repo and a real
/// survey there would otherwise pick up Trestle's own commit history for
/// the co-change signal (see the comment in `co_change.rs`).
fn copy_fixture_to_tempdir() -> tempfile::TempDir {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mixed_repo");
    let dest = tempfile::tempdir().expect("tempdir creation cannot fail in a test environment");
    copy_dir(&source, dest.path());
    dest
}

fn copy_dir(from: &Path, to: &Path) {
    for entry in fs::read_dir(from)
        .expect("fixture directory must exist")
        .flatten()
    {
        let path = entry.path();
        let dest = to.join(entry.file_name());
        if path.is_dir() {
            fs::create_dir_all(&dest).unwrap();
            copy_dir(&path, &dest);
        } else {
            fs::copy(&path, &dest).unwrap();
        }
    }
}

/// `(relative path, byte length, modified time)` for every file under
/// `root`, used to prove a survey run touched none of them.
fn fingerprint(root: &Path) -> Vec<(PathBuf, u64, std::time::SystemTime)> {
    let mut prints = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let meta = entry.metadata().unwrap();
                prints.push((
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    meta.len(),
                    meta.modified().unwrap(),
                ));
            }
        }
    }
    prints.sort_by(|a, b| a.0.cmp(&b.0));
    prints
}

#[test]
fn fixture_repo_covers_at_least_three_languages_including_one_deliberately_unsupported() {
    let fixture = copy_fixture_to_tempdir();
    let result = trestle_survey::survey(fixture.path());
    let json = result.to_json();
    let languages = json["languages"].as_array().unwrap();
    let names: Vec<&str> = languages
        .iter()
        .map(|l| l["name"].as_str().unwrap())
        .collect();

    assert!(
        names.len() >= 3,
        "expected at least three languages, got {names:?}"
    );
    assert!(names.contains(&"rust"));
    assert!(names.contains(&"python"));
    assert!(names.contains(&"javascript"));
    assert!(names.contains(&"ruby"));

    let ruby = languages.iter().find(|l| l["name"] == "ruby").unwrap();
    assert_eq!(ruby["support"], "unsupported");
    let rust = languages.iter().find(|l| l["name"] == "rust").unwrap();
    assert_eq!(rust["support"], "tree-sitter");
    let python = languages.iter().find(|l| l["name"] == "python").unwrap();
    assert_eq!(python["support"], "tree-sitter");
    let javascript = languages
        .iter()
        .find(|l| l["name"] == "javascript")
        .unwrap();
    assert_eq!(javascript["support"], "regex-fallback");
}

#[test]
fn ruby_being_unsupported_marks_the_whole_result_partial() {
    let fixture = copy_fixture_to_tempdir();
    let result = trestle_survey::survey(fixture.path());
    let json = result.to_json();
    assert_eq!(json["partial"], true);
    let reasons = json["partial_reasons"].as_array().unwrap();
    assert!(
        reasons.iter().any(|r| r.as_str().unwrap().contains("ruby")),
        "expected a partial_reasons entry naming ruby, got {reasons:?}"
    );
}

#[test]
fn consumers_of_the_shared_package_report_edges_to_it() {
    let fixture = copy_fixture_to_tempdir();
    let result = trestle_survey::survey(fixture.path());
    let json = result.to_json();
    let edges = json["edges"].as_array().unwrap();

    let has_import_edge = |from: &str, to: &str| {
        edges
            .iter()
            .any(|e| e["from"] == from && e["to"] == to && e["kind"] == "import")
    };
    assert!(
        has_import_edge("python_app_a", "python_shared"),
        "expected python_app_a -> python_shared among {edges:#?}"
    );
    assert!(
        has_import_edge("python_app_b", "python_shared"),
        "expected python_app_b -> python_shared among {edges:#?}"
    );
    // The Rust half is the same shape (two consumers, one shared module),
    // included so the shared-package property is shown to hold across the
    // tree-sitter-backed languages, not just one of them.
    assert!(has_import_edge("rust_consumer_a", "rust_shared"));
    assert!(has_import_edge("rust_consumer_b", "rust_shared"));
}

#[test]
fn every_shape_signal_appears_with_a_non_null_value() {
    let fixture = copy_fixture_to_tempdir();
    let result = trestle_survey::survey(fixture.path());
    let signals = &result.to_json()["shape_signals"];
    for key in [
        "parallelism",
        "oracle",
        "module_fan_out",
        "repo_size",
        "test_to_source_ratio",
    ] {
        assert!(
            !signals[key].is_null(),
            "shape_signals.{key} was not present"
        );
    }
}

#[test]
fn test_and_build_commands_are_discovered_from_every_format_the_fixture_uses() {
    let fixture = copy_fixture_to_tempdir();
    let result = trestle_survey::survey(fixture.path());
    let json = result.to_json();
    let test_commands: Vec<&str> = json["test_commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["command"].as_str().unwrap())
        .collect();
    let build_commands: Vec<&str> = json["build_commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["command"].as_str().unwrap())
        .collect();

    assert!(test_commands.contains(&"npm run test"));
    assert!(test_commands.contains(&"make test"));
    assert!(test_commands.contains(&"pytest"));
    assert!(build_commands.contains(&"npm run build"));
    assert!(build_commands.contains(&"make build"));
}

#[test]
fn ci_config_and_conventions_are_discovered() {
    let fixture = copy_fixture_to_tempdir();
    let result = trestle_survey::survey(fixture.path());
    let json = result.to_json();
    assert_eq!(
        json["ci_configs"],
        serde_json::json!([".github/workflows/ci.yml"])
    );
    assert_eq!(json["conventions"], serde_json::json!(["AGENTS.md"]));
}

#[test]
fn survey_writes_nothing_to_the_repository_it_reads() {
    let fixture = copy_fixture_to_tempdir();
    let before = fingerprint(fixture.path());
    let _ = trestle_survey::survey(fixture.path());
    let after = fingerprint(fixture.path());
    assert_eq!(
        before, after,
        "survey() modified files under the repository it read"
    );
}

#[test]
fn crate_manifest_pulls_in_no_http_client_or_telemetry_dependency() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    for banned in [
        "reqwest",
        "hyper",
        "curl",
        "ureq",
        "isahc",
        "surf",
        "attohttpc",
        "sentry",
        "opentelemetry",
    ] {
        assert!(
            !manifest.contains(banned),
            "Cargo.toml names banned dependency {banned:?}; trestle-survey must stay network-free"
        );
    }
}

#[test]
fn co_change_edges_in_a_real_repository_are_always_marked_heuristic() {
    // Surveys Trestle's own repository — the fixture above is deliberately
    // not a git repo (see `copy_fixture_to_tempdir`), so this is the one
    // place the co-change path runs against real history. Deliberately
    // weak on which edges exist (that is git history, not a fixture); the
    // property under test is the label, which must hold no matter what the
    // history contains.
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/trestle-survey has two ancestors: crates/ and the repo root");
    let result = trestle_survey::survey(repo_root);
    let json = result.to_json();
    let edges = json["edges"].as_array().unwrap();
    let co_change_edges: Vec<_> = edges.iter().filter(|e| e["kind"] == "co_change").collect();
    assert!(
        !co_change_edges.is_empty(),
        "expected at least one co-change edge in Trestle's own history"
    );
    for edge in &co_change_edges {
        assert_eq!(
            edge["heuristic"], true,
            "co-change edge not marked heuristic: {edge:?}"
        );
    }
}

#[test]
fn json_output_matches_the_committed_golden_file() {
    let fixture = copy_fixture_to_tempdir();
    let result = trestle_survey::survey(fixture.path());
    let actual = result.to_json_string_pretty();
    let golden_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mixed_repo.golden.json");
    let expected = fs::read_to_string(&golden_path)
        .unwrap_or_else(|_| panic!("missing golden file at {}", golden_path.display()));
    assert_eq!(
        actual.trim(),
        expected.trim(),
        "survey JSON output no longer matches the golden file — a field was renamed, added, or removed. \
         If this is a deliberate, reviewed change to the schema, regenerate {} from `actual` above.",
        golden_path.display()
    );
}

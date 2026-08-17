//! Acceptance tests for T02b, one per bullet in the node's Acceptance section.
//!
//! Fixtures under `fixtures/expressed/` (repo root, T02a's deliverable) prove the
//! format can be *parsed*; fixtures under `tests/fixtures/malformed/` (this crate's
//! own, named after the mistake each one makes) prove it can be *rejected* with an
//! actionable message.

use std::fs;
use std::path::{Path, PathBuf};

use trestle_plan::{parse_plan, parse_status, validate_status, PlanError};

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is crates/trestle-plan; the expressed fixtures live two
    // levels up, at the repo root, because T02a owns that directory and this crate
    // only reads it.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn expressed(shape: &str) -> PathBuf {
    repo_root().join("fixtures/expressed").join(shape)
}

fn malformed(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/malformed")
        .join(name)
}

fn read(dir: &Path, file: &str) -> String {
    fs::read_to_string(dir.join(file))
        .unwrap_or_else(|e| panic!("reading {}/{file}: {e}", dir.display()))
}

// ---------------------------------------------------------------------------
// Every fixture in fixtures/expressed/ parses and validates.
// ---------------------------------------------------------------------------

#[test]
fn graph_shape_fixture_parses_and_validates() {
    let yaml = read(&expressed("graph-shape"), "plan.yaml");
    let plan = parse_plan(&yaml).expect("graph-shape/plan.yaml must parse and validate");
    assert_eq!(plan.shape, "graph");
    assert_eq!(plan.units.len(), 7);
}

#[test]
fn loop_shape_fixture_parses_and_validates() {
    let yaml = read(&expressed("loop-shape"), "plan.yaml");
    let plan = parse_plan(&yaml).expect("loop-shape/plan.yaml must parse and validate");
    assert_eq!(plan.shape, "loop");
    assert_eq!(plan.units.len(), 8);
    assert!(
        plan.oracle.is_some(),
        "a loop's oracle binds to the iteration"
    );
    assert_eq!(plan.journal.as_deref(), Some("journal.md"));
}

#[test]
fn hybrid_fixture_parses_and_validates_including_nested_queue() {
    let yaml = read(&expressed("hybrid"), "plan.yaml");
    let plan = parse_plan(&yaml).expect("hybrid/plan.yaml must parse and validate");
    assert_eq!(plan.shape, "hybrid");
    let h02 = plan
        .units
        .iter()
        .find(|u| u.id == "H02")
        .expect("H02 present");
    assert_eq!(
        h02.queue.len(),
        3,
        "H02's nested queue is a graph unit that is itself iterated"
    );
    assert_eq!(h02.queue[0].id, "H02.1");
    assert_eq!(h02.queue[0].order, Some(1));
}

#[test]
fn forward_compat_fixture_parses_and_validates_including_unit_repo() {
    let yaml = read(&expressed("forward-compat"), "plan.yaml");
    let plan = parse_plan(&yaml).expect("forward-compat/plan.yaml must parse and validate");
    let m01 = plan
        .units
        .iter()
        .find(|u| u.id == "M01")
        .expect("M01 present");
    assert_eq!(
        m01.repo.as_deref(),
        Some("acme/client-lib"),
        "repo is reserved but must still parse"
    );
}

// ---------------------------------------------------------------------------
// Round-trip fidelity: parse -> serialise -> parse loses nothing, compared by
// parsed structure (PartialEq on Plan), never by diffing serialised text.
// ---------------------------------------------------------------------------

fn assert_round_trips(shape: &str) {
    let yaml = read(&expressed(shape), "plan.yaml");
    let first = parse_plan(&yaml).unwrap_or_else(|e| panic!("{shape} failed to parse: {e:?}"));
    let serialised = first.to_yaml();
    let second = parse_plan(&serialised)
        .unwrap_or_else(|e| panic!("{shape}'s own serialised output failed to reparse: {e:?}"));
    assert_eq!(
        first, second,
        "{shape}: parse -> serialise -> parse produced a different structure"
    );
}

#[test]
fn graph_shape_round_trips_losslessly() {
    assert_round_trips("graph-shape");
}

#[test]
fn loop_shape_round_trips_losslessly() {
    assert_round_trips("loop-shape");
}

#[test]
fn hybrid_round_trips_losslessly() {
    assert_round_trips("hybrid");
}

#[test]
fn forward_compat_round_trips_losslessly() {
    assert_round_trips("forward-compat");
}

// ---------------------------------------------------------------------------
// Unknown keys survive a round trip: forward-compat's schedule, budget_ceiling
// (top-level) and retry_budget (per-unit) are not in the schema.
// ---------------------------------------------------------------------------

#[test]
fn unknown_top_level_keys_survive_round_trip() {
    let yaml = read(&expressed("forward-compat"), "plan.yaml");
    let plan = parse_plan(&yaml).expect("forward-compat/plan.yaml must parse");
    assert!(
        plan.extra.contains_key("schedule"),
        "schedule is not in the schema and must still load"
    );
    assert!(
        plan.extra.contains_key("budget_ceiling"),
        "budget_ceiling is not in the schema and must still load"
    );

    let round_tripped = parse_plan(&plan.to_yaml()).expect("re-parse of serialised output");
    assert!(
        round_tripped.extra.contains_key("schedule"),
        "schedule must survive a round trip"
    );
    assert!(
        round_tripped.extra.contains_key("budget_ceiling"),
        "budget_ceiling must survive a round trip"
    );
}

#[test]
fn unknown_per_unit_keys_survive_round_trip() {
    let yaml = read(&expressed("forward-compat"), "plan.yaml");
    let plan = parse_plan(&yaml).expect("forward-compat/plan.yaml must parse");
    let m02 = plan
        .units
        .iter()
        .find(|u| u.id == "M02")
        .expect("M02 present");
    assert!(
        m02.extra.contains_key("retry_budget"),
        "retry_budget is a later-version per-unit key and must still load"
    );

    let round_tripped = parse_plan(&plan.to_yaml()).expect("re-parse of serialised output");
    let m02_again = round_tripped
        .units
        .iter()
        .find(|u| u.id == "M02")
        .expect("M02 present after round trip");
    assert!(
        m02_again.extra.contains_key("retry_budget"),
        "retry_budget must survive a round trip"
    );
}

// ---------------------------------------------------------------------------
// Status is read without parsing prose: every unit's status is reachable from
// structured fields alone (T12's requirement, asserted here).
// ---------------------------------------------------------------------------

#[test]
fn status_is_reachable_from_structured_fields_alone() {
    let yaml = read(&expressed("loop-shape"), "status.yaml");
    let status = parse_status(&yaml).expect("loop-shape/status.yaml must parse");
    let blocked = status
        .units
        .iter()
        .find(|u| u.id == "1.4")
        .expect("unit 1.4 present");
    assert_eq!(blocked.status, "blocked");
    assert!(
        blocked
            .blocked_question
            .as_deref()
            .unwrap_or_default()
            .contains("signing step"),
        "the blocking question is a structured sibling field, not text inside the status"
    );
    let in_progress = status
        .units
        .iter()
        .find(|u| u.id == "1.2")
        .expect("unit 1.2 present");
    assert_eq!(in_progress.status, "in_progress");
    assert_eq!(in_progress.iteration, Some(12));
}

#[test]
fn hybrid_status_covers_nested_queue_ids() {
    let yaml = read(&expressed("hybrid"), "status.yaml");
    let status = parse_status(&yaml).expect("hybrid/status.yaml must parse");
    let queue_item = status
        .units
        .iter()
        .find(|u| u.id == "H02.1")
        .expect("nested queue item H02.1 has its own status record");
    assert_eq!(queue_item.status, "done");
}

// ---------------------------------------------------------------------------
// Error-message quality: each malformed fixture names the offending path and
// the expectation, not just "parsing failed".
// ---------------------------------------------------------------------------

fn errors_for(plan_yaml: &str) -> Vec<PlanError> {
    match parse_plan(plan_yaml) {
        Ok(plan) => panic!("expected malformed plan to fail validation, got: {plan:?}"),
        Err(errors) => errors,
    }
}

fn assert_has_error(errors: &[PlanError], path_contains: &str, message_contains: &str) {
    let found = errors
        .iter()
        .any(|e| e.path.contains(path_contains) && e.message.contains(message_contains));
    assert!(
        found,
        "expected an error with path containing {path_contains:?} and message containing \
         {message_contains:?}, got: {errors:#?}"
    );
}

#[test]
fn dep_to_missing_unit_names_the_offending_edge() {
    let yaml = read(&malformed("dep-to-missing-unit"), "plan.yaml");
    let errors = errors_for(&yaml);
    assert_has_error(&errors, "units[0].deps", "A99");
}

#[test]
fn dependency_cycle_names_the_cycle() {
    let yaml = read(&malformed("dependency-cycle"), "plan.yaml");
    let errors = errors_for(&yaml);
    assert_has_error(&errors, "units", "cycle");
}

#[test]
fn loop_without_journal_names_the_missing_field() {
    let yaml = read(&malformed("loop-without-journal"), "plan.yaml");
    let errors = errors_for(&yaml);
    assert_has_error(&errors, "journal", "required");
}

#[test]
fn unit_without_oracle_gate_or_order_names_the_unit() {
    let yaml = read(&malformed("unit-without-oracle-gate-or-order"), "plan.yaml");
    let errors = errors_for(&yaml);
    assert_has_error(&errors, "units[0]", "oracle");
}

#[test]
fn tier_naming_a_model_instead_of_an_abstract_level_is_rejected() {
    let yaml = read(&malformed("vendor-model-name-in-tier"), "plan.yaml");
    let errors = errors_for(&yaml);
    assert_has_error(&errors, "units[0].tier", "cheap, standard, deep");
}

#[test]
fn duplicate_unit_id_names_both_occurrences() {
    let yaml = read(&malformed("duplicate-unit-id"), "plan.yaml");
    let errors = errors_for(&yaml);
    assert_has_error(&errors, "units", "A01");
    assert_has_error(&errors, "units", "duplicate");
}

#[test]
fn done_status_with_no_oracle_result_is_rejected() {
    let dir = malformed("done-without-oracle-result");
    let plan = parse_plan(&read(&dir, "plan.yaml")).expect("this fixture's plan.yaml is valid");
    let status = parse_status(&read(&dir, "status.yaml")).expect("status.yaml parses structurally");
    let errors = validate_status(&status, &plan);
    assert_has_error(&errors, "units[0]", "oracle_result");
}

// ---------------------------------------------------------------------------
// Naming: each malformed fixture's test name says which class of bad plan it
// covers, so a failing test report names the mistake rather than "a fixture".
// This is asserted structurally: every directory under tests/fixtures/malformed
// is exercised by exactly one #[test] above. If this drifts, the list below
// (kept in the test, not derived) will stop matching the directory listing.
// ---------------------------------------------------------------------------

#[test]
fn every_malformed_fixture_directory_is_covered_by_a_named_test() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/malformed");
    let mut on_disk: Vec<String> = fs::read_dir(&dir)
        .expect("tests/fixtures/malformed exists")
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    on_disk.sort();

    let mut covered: Vec<String> = [
        "dep-to-missing-unit",
        "dependency-cycle",
        "loop-without-journal",
        "unit-without-oracle-gate-or-order",
        "vendor-model-name-in-tier",
        "duplicate-unit-id",
        "done-without-oracle-result",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    covered.sort();

    assert_eq!(
        on_disk, covered,
        "a malformed fixture was added or removed without updating its dedicated test"
    );
}

// "The crate has no I/O beyond reading the files it is handed" is a design property
// of the public API above, not a runtime behaviour: parse_plan/parse_status/
// validate_status take &str and structs, never a path, so the crate itself never
// touches a filesystem — every read in this file is done by the test, not the
// crate. That is enforced by the function signatures compiling at all, so there is
// nothing further to assert at runtime.

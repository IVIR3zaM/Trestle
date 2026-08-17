//! `status.yaml`: the only file execution writes. Every unit's status is
//! reachable from structured fields alone — no consumer parses prose to find
//! out where a unit stands.

use serde_yaml::{Mapping, Value};

use crate::decode::{decode_list, insert_extra, Decoder};
use crate::error::PlanError;
use crate::plan::Plan;

pub const STATUSES: &[&str] = &[
    "draft",
    "todo",
    "in_progress",
    "blocked",
    "verified",
    "done",
    "superseded",
    "n_a",
];

#[derive(Debug, Clone, PartialEq)]
pub struct OracleResult {
    pub command: String,
    pub exit: i64,
    pub at: Option<String>,
    pub extra: Mapping,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Override {
    pub reason: String,
    pub by: Option<String>,
    pub at: String,
    pub extra: Mapping,
}

/// `blocked` carries its question in a sibling field, never inside the status
/// value itself: `status: blocked` plus `blocked_question:`, never
/// `blocked(user): <question>`. This crate only ever stores and reads the
/// structured form; the parenthesised display form is a later node's concern.
#[derive(Debug, Clone, PartialEq)]
pub struct StatusRecord {
    pub id: String,
    pub status: String,
    pub blocked_question: Option<String>,
    pub note: Option<String>,
    pub iteration: Option<i64>,
    pub oracle_result: Option<OracleResult>,
    pub override_record: Option<Override>,
    pub extra: Mapping,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    pub plan: String,
    pub units: Vec<StatusRecord>,
    pub extra: Mapping,
}

const STATUS_RECORD_KNOWN_KEYS: &[&str] = &[
    "id",
    "status",
    "blocked_question",
    "note",
    "iteration",
    "oracle_result",
    "override",
];

const STATUS_KNOWN_KEYS: &[&str] = &["plan", "units"];

/// Parses and validates a `status.yaml` document. String in, `Status` or the
/// full list of what is wrong with it out — the same shape as `parse_plan`,
/// and the same reason: no path is ever touched here.
pub fn parse_status(yaml: &str) -> Result<Status, Vec<PlanError>> {
    let value: Value = serde_yaml::from_str(yaml)
        .map_err(|e| vec![PlanError::new("<document>", format!("not valid YAML: {e}"))])?;
    let Some(map) = value.as_mapping() else {
        return Err(vec![PlanError::new(
            "<document>",
            "must be a YAML mapping at the top level",
        )]);
    };

    let mut errors = Vec::new();
    let decoder = Decoder::new(map, "");
    let plan = decoder.string("plan", &mut errors);
    if decoder.sequence_opt("units").is_none() {
        errors.push(PlanError::new("units", "required"));
    }
    let units = decode_list(&decoder, "units", &mut errors, decode_record);
    let extra = decoder.extra(STATUS_KNOWN_KEYS);

    let status = Status { plan, units, extra };
    if errors.is_empty() {
        Ok(status)
    } else {
        Err(errors)
    }
}

fn decode_record(value: &Value, path: &str, errors: &mut Vec<PlanError>) -> StatusRecord {
    let Some(map) = value.as_mapping() else {
        errors.push(PlanError::new(path, "must be a mapping"));
        return StatusRecord {
            id: String::new(),
            status: String::new(),
            blocked_question: None,
            note: None,
            iteration: None,
            oracle_result: None,
            override_record: None,
            extra: Mapping::new(),
        };
    };
    let decoder = Decoder::new(map, path);
    let id = decoder.string("id", errors);
    let status = decoder.string("status", errors);
    if !status.is_empty() && !STATUSES.contains(&status.as_str()) {
        errors.push(PlanError::new(
            decoder.path_for("status"),
            format!("must be one of {}, got {status:?}", STATUSES.join(", ")),
        ));
    }
    let blocked_question = decoder.string_opt("blocked_question");
    if status == "blocked" && blocked_question.is_none() {
        errors.push(PlanError::new(
            decoder.path_for("blocked_question"),
            "required when status is \"blocked\" — a human owes an answer, and the question is a sibling field, never text inside the status value",
        ));
    }
    let note = decoder.string_opt("note");
    let iteration = decoder.int_opt("iteration");

    let oracle_result = decoder
        .mapping_opt("oracle_result")
        .map(|m| decode_oracle_result(m, decoder.path_for("oracle_result"), errors));
    let override_record = decoder
        .mapping_opt("override")
        .map(|m| decode_override(m, decoder.path_for("override"), errors));

    let extra = decoder.extra(STATUS_RECORD_KNOWN_KEYS);

    StatusRecord {
        id,
        status,
        blocked_question,
        note,
        iteration,
        oracle_result,
        override_record,
        extra,
    }
}

fn decode_oracle_result(map: &Mapping, path: String, errors: &mut Vec<PlanError>) -> OracleResult {
    let d = Decoder::new(map, path);
    let command = d.string("command", errors);
    let exit = d.int("exit", errors);
    let at = d.string_opt("at");
    let extra = d.extra(&["command", "exit", "at"]);
    OracleResult {
        command,
        exit,
        at,
        extra,
    }
}

fn decode_override(map: &Mapping, path: String, errors: &mut Vec<PlanError>) -> Override {
    let d = Decoder::new(map, path);
    let reason = d.string("reason", errors);
    let by = d.string_opt("by");
    let at = d.string("at", errors);
    let extra = d.extra(&["reason", "by", "at"]);
    Override {
        reason,
        by,
        at,
        extra,
    }
}

fn record_to_value(record: &StatusRecord) -> Value {
    let mut map = Mapping::new();
    map.insert(Value::from("id"), Value::from(record.id.as_str()));
    map.insert(Value::from("status"), Value::from(record.status.as_str()));
    if let Some(v) = &record.blocked_question {
        map.insert(Value::from("blocked_question"), Value::from(v.as_str()));
    }
    if let Some(v) = &record.note {
        map.insert(Value::from("note"), Value::from(v.as_str()));
    }
    if let Some(v) = record.iteration {
        map.insert(Value::from("iteration"), Value::from(v));
    }
    if let Some(o) = &record.oracle_result {
        let mut om = Mapping::new();
        om.insert(Value::from("command"), Value::from(o.command.as_str()));
        om.insert(Value::from("exit"), Value::from(o.exit));
        if let Some(at) = &o.at {
            om.insert(Value::from("at"), Value::from(at.as_str()));
        }
        insert_extra(&mut om, &o.extra);
        map.insert(Value::from("oracle_result"), Value::Mapping(om));
    }
    if let Some(o) = &record.override_record {
        let mut om = Mapping::new();
        om.insert(Value::from("reason"), Value::from(o.reason.as_str()));
        if let Some(by) = &o.by {
            om.insert(Value::from("by"), Value::from(by.as_str()));
        }
        om.insert(Value::from("at"), Value::from(o.at.as_str()));
        insert_extra(&mut om, &o.extra);
        map.insert(Value::from("override"), Value::Mapping(om));
    }
    insert_extra(&mut map, &record.extra);
    Value::Mapping(map)
}

impl Status {
    pub(crate) fn to_value(&self) -> Value {
        let mut map = Mapping::new();
        map.insert(Value::from("plan"), Value::from(self.plan.as_str()));
        map.insert(
            Value::from("units"),
            Value::Sequence(self.units.iter().map(record_to_value).collect()),
        );
        insert_extra(&mut map, &self.extra);
        Value::Mapping(map)
    }

    /// Serialises back to a `status.yaml` document.
    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(&self.to_value()).expect("a Status always serialises to valid YAML")
    }
}

/// Cross-checks a `status.yaml` against the `plan.yaml` it belongs to — the
/// one validation that needs both files at once. A `done` status for a unit
/// that has its own oracle must carry the `oracle_result` that proved it; a
/// unit whose only verification is a human `gate` or a loop's `order` has no
/// oracle to have run, so it is exempt.
pub fn validate_status(status: &Status, plan: &Plan) -> Vec<PlanError> {
    let mut errors = Vec::new();
    for (i, record) in status.units.iter().enumerate() {
        if record.status != "done" || record.oracle_result.is_some() {
            continue;
        }
        let Some(unit) = plan.find_unit(&record.id) else {
            continue;
        };
        if unit.oracle.is_some() {
            errors.push(PlanError::new(
                format!("units[{i}].oracle_result"),
                format!(
                    "oracle_result is required when unit {:?} is done and its plan definition has an oracle — the agent's own report is not evidence",
                    record.id
                ),
            ));
        }
    }
    errors
}

//! Plan-level invariants that hold for every unit. Superseded in place, never
//! deleted — the audit trail is the point.

use serde_yaml::{Mapping, Value};

use crate::decode::{insert_extra, Decoder};
use crate::error::PlanError;

pub const RULE_STATUSES: &[&str] = &["active", "superseded"];

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub id: String,
    pub text: String,
    pub status: String,
    pub superseded_by: Option<String>,
    pub replaces: Option<String>,
    pub reason: Option<String>,
    pub extra: Mapping,
}

impl Rule {
    pub(crate) fn decode(value: &Value, path: &str, errors: &mut Vec<PlanError>) -> Rule {
        let Some(map) = value.as_mapping() else {
            errors.push(PlanError::new(path, "must be a mapping"));
            return Rule {
                id: String::new(),
                text: String::new(),
                status: String::new(),
                superseded_by: None,
                replaces: None,
                reason: None,
                extra: Mapping::new(),
            };
        };
        let decoder = Decoder::new(map, path);
        let id = decoder.string("id", errors);
        let text = decoder.string("text", errors);
        let status = decoder.string("status", errors);
        if !status.is_empty() && !RULE_STATUSES.contains(&status.as_str()) {
            errors.push(PlanError::new(
                decoder.path_for("status"),
                format!(
                    "must be one of {}, got {status:?}",
                    RULE_STATUSES.join(", ")
                ),
            ));
        }
        let superseded_by = decoder.string_opt("superseded_by");
        let replaces = decoder.string_opt("replaces");
        let reason = decoder.string_opt("reason");
        if status == "superseded" {
            if superseded_by.is_none() {
                errors.push(PlanError::new(
                    decoder.path_for("superseded_by"),
                    "required when status is \"superseded\" — a struck-through rule must say what replaces it",
                ));
            }
            if reason.is_none() {
                errors.push(PlanError::new(
                    decoder.path_for("reason"),
                    "required when status is \"superseded\" — the reasoning is what stops the same mistake twice",
                ));
            }
        }
        let extra = decoder.extra(&[
            "id",
            "text",
            "status",
            "superseded_by",
            "replaces",
            "reason",
        ]);
        Rule {
            id,
            text,
            status,
            superseded_by,
            replaces,
            reason,
            extra,
        }
    }

    pub(crate) fn to_value(&self) -> Value {
        let mut map = Mapping::new();
        map.insert(Value::from("id"), Value::from(self.id.as_str()));
        map.insert(Value::from("text"), Value::from(self.text.as_str()));
        map.insert(Value::from("status"), Value::from(self.status.as_str()));
        if let Some(v) = &self.superseded_by {
            map.insert(Value::from("superseded_by"), Value::from(v.as_str()));
        }
        if let Some(v) = &self.replaces {
            map.insert(Value::from("replaces"), Value::from(v.as_str()));
        }
        if let Some(v) = &self.reason {
            map.insert(Value::from("reason"), Value::from(v.as_str()));
        }
        insert_extra(&mut map, &self.extra);
        Value::Mapping(map)
    }
}

//! The command that decides done, and where it came from.

use serde_yaml::{Mapping, Value};

use crate::decode::{insert_extra, Decoder};
use crate::error::PlanError;

pub const PROVENANCE_SOURCES: &[&str] = &["unit", "discovered", "standard", "convention"];

/// Where an oracle came from. Required for standards-derived oracles so a
/// reviewer can trace an extra command back to the clause that attached it.
#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    pub source: String,
    pub rule_id: Option<String>,
    pub citation: Option<String>,
    pub extra: Mapping,
}

/// The command that decides done. Never the agent's opinion — an oracle is by
/// definition external to the thing it checks.
#[derive(Debug, Clone, PartialEq)]
pub struct Oracle {
    pub command: String,
    pub provenance: Option<Provenance>,
    pub extra: Mapping,
}

impl Oracle {
    pub(crate) fn decode(value: &Value, path: &str, errors: &mut Vec<PlanError>) -> Oracle {
        let Some(map) = value.as_mapping() else {
            errors.push(PlanError::new(
                path,
                "must be a mapping with at least a `command`",
            ));
            return Oracle {
                command: String::new(),
                provenance: None,
                extra: Mapping::new(),
            };
        };
        let decoder = Decoder::new(map, path);
        let command = decoder.string("command", errors);
        let provenance = decoder
            .mapping_opt("provenance")
            .map(|m| Provenance::decode(m, &decoder.path_for("provenance"), errors));
        let extra = decoder.extra(&["command", "provenance"]);
        Oracle {
            command,
            provenance,
            extra,
        }
    }

    pub(crate) fn to_value(&self) -> Value {
        let mut map = Mapping::new();
        map.insert(Value::from("command"), Value::from(self.command.as_str()));
        if let Some(p) = &self.provenance {
            map.insert(Value::from("provenance"), p.to_value());
        }
        insert_extra(&mut map, &self.extra);
        Value::Mapping(map)
    }
}

impl Provenance {
    fn decode(map: &Mapping, path: &str, errors: &mut Vec<PlanError>) -> Provenance {
        let decoder = Decoder::new(map, path);
        let source = decoder.string("source", errors);
        if !source.is_empty() && !PROVENANCE_SOURCES.contains(&source.as_str()) {
            errors.push(PlanError::new(
                decoder.path_for("source"),
                format!(
                    "must be one of {}, got {source:?}",
                    PROVENANCE_SOURCES.join(", ")
                ),
            ));
        }
        let rule_id = decoder.string_opt("rule_id");
        let citation = decoder.string_opt("citation");
        if source == "standard" {
            if rule_id.is_none() {
                errors.push(PlanError::new(
                    decoder.path_for("rule_id"),
                    "required when provenance.source is \"standard\", so a reviewer can trace the oracle back to its clause",
                ));
            }
            if citation.is_none() {
                errors.push(PlanError::new(
                    decoder.path_for("citation"),
                    "required when provenance.source is \"standard\", so a reviewer can trace the oracle back to its clause",
                ));
            }
        }
        let extra = decoder.extra(&["source", "rule_id", "citation"]);
        Provenance {
            source,
            rule_id,
            citation,
            extra,
        }
    }

    fn to_value(&self) -> Value {
        let mut map = Mapping::new();
        map.insert(Value::from("source"), Value::from(self.source.as_str()));
        if let Some(v) = &self.rule_id {
            map.insert(Value::from("rule_id"), Value::from(v.as_str()));
        }
        if let Some(v) = &self.citation {
            map.insert(Value::from("citation"), Value::from(v.as_str()));
        }
        insert_extra(&mut map, &self.extra);
        Value::Mapping(map)
    }
}

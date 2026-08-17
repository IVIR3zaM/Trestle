//! Pulls typed fields out of a `serde_yaml::Mapping` while tracking the path to
//! each one, so a validation failure can name exactly where it happened.
//!
//! `serde`'s derive macros are not used here (see `Cargo.toml`): deriving
//! `Deserialize` would pull in `proc-macro2` -> `unicode-ident`, whose license
//! includes a Unicode-3.0 component `deny.toml`'s allow-list does not cover.
//! Decoding by hand from `serde_yaml::Value` avoids that dependency entirely,
//! and as a side effect gives every field its own path-qualified error instead
//! of whatever message a derived `Deserialize` would produce.
//!
//! One `Decoder` per mapping level; every unit, oracle, rule and so on gets one
//! when it is decoded, which is the second (and third, and fourth...) caller
//! that justifies this as shared code rather than a one-off (`AGENTS.md` §1).

use serde_yaml::{Mapping, Value};

use crate::error::PlanError;

pub(crate) struct Decoder<'a> {
    map: &'a Mapping,
    path: String,
}

impl<'a> Decoder<'a> {
    pub(crate) fn new(map: &'a Mapping, path: impl Into<String>) -> Self {
        Decoder {
            map,
            path: path.into(),
        }
    }

    pub(crate) fn path_for(&self, key: &str) -> String {
        if self.path.is_empty() {
            key.to_string()
        } else {
            format!("{}.{}", self.path, key)
        }
    }

    fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(Value::from(key))
    }

    /// The raw value at `key`, whatever its type — for fields whose own
    /// decoder needs to report its own "must be a mapping" error rather than
    /// this one silently treating a wrong type as absent.
    pub(crate) fn value_opt(&self, key: &str) -> Option<&Value> {
        self.get(key)
    }

    /// A required string field. Missing or wrong-typed both produce an error
    /// naming the path; an empty string is returned so the caller can keep
    /// decoding the rest of the mapping and surface every problem at once.
    pub(crate) fn string(&self, key: &str, errors: &mut Vec<PlanError>) -> String {
        match self.get(key) {
            Some(Value::String(s)) => s.clone(),
            Some(_) => {
                errors.push(PlanError::new(self.path_for(key), "must be a string"));
                String::new()
            }
            None => {
                errors.push(PlanError::new(self.path_for(key), "required"));
                String::new()
            }
        }
    }

    pub(crate) fn string_opt(&self, key: &str) -> Option<String> {
        match self.get(key) {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    pub(crate) fn int_opt(&self, key: &str) -> Option<i64> {
        match self.get(key) {
            Some(Value::Number(n)) => n.as_i64(),
            _ => None,
        }
    }

    /// A required integer field, e.g. `oracle_result.exit`. Missing or
    /// wrong-typed both produce an error naming the path; `0` is returned so
    /// the caller can keep decoding the rest of the mapping.
    pub(crate) fn int(&self, key: &str, errors: &mut Vec<PlanError>) -> i64 {
        match self.get(key) {
            Some(Value::Number(n)) => n.as_i64().unwrap_or_else(|| {
                errors.push(PlanError::new(self.path_for(key), "must be an integer"));
                0
            }),
            Some(_) => {
                errors.push(PlanError::new(self.path_for(key), "must be an integer"));
                0
            }
            None => {
                errors.push(PlanError::new(self.path_for(key), "required"));
                0
            }
        }
    }

    pub(crate) fn mapping_opt(&self, key: &str) -> Option<&Mapping> {
        match self.get(key) {
            Some(Value::Mapping(m)) => Some(m),
            _ => None,
        }
    }

    pub(crate) fn sequence_opt(&self, key: &str) -> Option<&Vec<Value>> {
        match self.get(key) {
            Some(Value::Sequence(s)) => Some(s),
            _ => None,
        }
    }

    /// A list of strings, e.g. `deps`. Absent is an empty list, not an error —
    /// `deps` is optional on a unit.
    pub(crate) fn string_list(&self, key: &str) -> Vec<String> {
        self.sequence_opt(key)
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Every key in this mapping that isn't one of `known`, preserved verbatim
    /// so a round trip does not drop a field the parser does not understand.
    pub(crate) fn extra(&self, known: &[&str]) -> Mapping {
        let mut extra = Mapping::new();
        for (k, v) in self.map {
            let is_known = k.as_str().is_some_and(|k| known.contains(&k));
            if !is_known {
                extra.insert(k.clone(), v.clone());
            }
        }
        extra
    }
}

/// Inserts the extra/unknown fields captured at decode time back into a
/// mapping being built for serialisation. Shared by every `to_value` in this
/// crate — the other half of the `Decoder::extra` contract.
pub(crate) fn insert_extra(map: &mut Mapping, extra: &Mapping) {
    for (k, v) in extra {
        map.insert(k.clone(), v.clone());
    }
}

/// Decodes an optional array field into a `Vec<T>`, indexing each item's path
/// as `<key>[<i>]` under `decoder`'s own path. Absent is an empty list, not an
/// error — every array field in this format (`rules`, `units`, `queue`,
/// `extra_oracles`, ...) is optional at the container level, and it is each
/// item's own decoder that enforces what it requires. Shared across `Plan`,
/// `Unit` and `Status`, which is what makes this worth pulling out rather than
/// repeating the same `sequence_opt().map().unwrap_or_default()` four times.
pub(crate) fn decode_list<T>(
    decoder: &Decoder,
    key: &str,
    errors: &mut Vec<PlanError>,
    mut decode_item: impl FnMut(&Value, &str, &mut Vec<PlanError>) -> T,
) -> Vec<T> {
    decoder
        .sequence_opt(key)
        .map(|seq| {
            seq.iter()
                .enumerate()
                .map(|(i, v)| decode_item(v, &format!("{}[{i}]", decoder.path_for(key)), errors))
                .collect()
        })
        .unwrap_or_default()
}

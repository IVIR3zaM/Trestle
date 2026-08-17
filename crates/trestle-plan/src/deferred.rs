//! Consciously postponed work — distinct from forgotten, and not a unit: no
//! oracle, no status, so unit counts and readiness are not polluted by work
//! nobody intends to do this time.

use serde_yaml::{Mapping, Value};

use crate::decode::{insert_extra, Decoder};
use crate::error::PlanError;

#[derive(Debug, Clone, PartialEq)]
pub struct Deferred {
    pub id: String,
    pub item: String,
    pub why: String,
    pub revisit_when: String,
    pub extra: Mapping,
}

impl Deferred {
    pub(crate) fn decode(value: &Value, path: &str, errors: &mut Vec<PlanError>) -> Deferred {
        let Some(map) = value.as_mapping() else {
            errors.push(PlanError::new(path, "must be a mapping"));
            return Deferred {
                id: String::new(),
                item: String::new(),
                why: String::new(),
                revisit_when: String::new(),
                extra: Mapping::new(),
            };
        };
        let decoder = Decoder::new(map, path);
        let id = decoder.string("id", errors);
        let item = decoder.string("item", errors);
        let why = decoder.string("why", errors);
        let revisit_when = decoder.string("revisit_when", errors);
        let extra = decoder.extra(&["id", "item", "why", "revisit_when"]);
        Deferred {
            id,
            item,
            why,
            revisit_when,
            extra,
        }
    }

    pub(crate) fn to_value(&self) -> Value {
        let mut map = Mapping::new();
        map.insert(Value::from("id"), Value::from(self.id.as_str()));
        map.insert(Value::from("item"), Value::from(self.item.as_str()));
        map.insert(Value::from("why"), Value::from(self.why.as_str()));
        map.insert(
            Value::from("revisit_when"),
            Value::from(self.revisit_when.as_str()),
        );
        insert_extra(&mut map, &self.extra);
        Value::Mapping(map)
    }
}

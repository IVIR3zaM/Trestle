//! An ordered group over the queue: a label plus an order, not a second kind
//! of unit.

use serde_yaml::{Mapping, Value};

use crate::decode::{insert_extra, Decoder};
use crate::error::PlanError;

#[derive(Debug, Clone, PartialEq)]
pub struct Phase {
    pub id: String,
    pub title: String,
    pub extra: Mapping,
}

impl Phase {
    pub(crate) fn decode(value: &Value, path: &str, errors: &mut Vec<PlanError>) -> Phase {
        let Some(map) = value.as_mapping() else {
            errors.push(PlanError::new(path, "must be a mapping"));
            return Phase {
                id: String::new(),
                title: String::new(),
                extra: Mapping::new(),
            };
        };
        let decoder = Decoder::new(map, path);
        let id = decoder.string("id", errors);
        let title = decoder.string("title", errors);
        let extra = decoder.extra(&["id", "title"]);
        Phase { id, title, extra }
    }

    pub(crate) fn to_value(&self) -> Value {
        let mut map = Mapping::new();
        map.insert(Value::from("id"), Value::from(self.id.as_str()));
        map.insert(Value::from("title"), Value::from(self.title.as_str()));
        insert_extra(&mut map, &self.extra);
        Value::Mapping(map)
    }
}

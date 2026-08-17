//! The work itself, and — via `queue` — the hybrid shape's iterated units.

use serde_yaml::{Mapping, Value};

use crate::decode::{decode_list, insert_extra, Decoder};
use crate::error::PlanError;
use crate::oracle::Oracle;

pub const TIERS: &[&str] = &["cheap", "standard", "deep"];
pub const GATES: &[&str] = &["human"];

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub id: String,
    pub title: String,
    pub body: Option<String>,
    pub done_when: Option<String>,
    pub deps: Vec<String>,
    pub order: Option<i64>,
    pub phase: Option<String>,
    pub tier: Option<String>,
    pub gate: Option<String>,
    pub oracle: Option<Oracle>,
    pub extra_oracles: Vec<Oracle>,
    pub repo: Option<String>,
    pub queue: Vec<Unit>,
    pub journal: Option<String>,
    pub extra: Mapping,
}

const KNOWN_KEYS: &[&str] = &[
    "id",
    "title",
    "body",
    "done_when",
    "deps",
    "order",
    "phase",
    "tier",
    "gate",
    "oracle",
    "extra_oracles",
    "repo",
    "queue",
    "journal",
];

impl Unit {
    pub(crate) fn decode(value: &Value, path: &str, errors: &mut Vec<PlanError>) -> Unit {
        let Some(map) = value.as_mapping() else {
            errors.push(PlanError::new(path, "must be a mapping"));
            return Unit {
                id: String::new(),
                title: String::new(),
                body: None,
                done_when: None,
                deps: Vec::new(),
                order: None,
                phase: None,
                tier: None,
                gate: None,
                oracle: None,
                extra_oracles: Vec::new(),
                repo: None,
                queue: Vec::new(),
                journal: None,
                extra: Mapping::new(),
            };
        };
        let decoder = Decoder::new(map, path);

        let id = decoder.string("id", errors);
        let title = decoder.string("title", errors);
        let body = decoder.string_opt("body");
        let done_when = decoder.string_opt("done_when");
        let deps = decoder.string_list("deps");
        let order = decoder.int_opt("order");
        let phase = decoder.string_opt("phase");
        let repo = decoder.string_opt("repo");
        let journal = decoder.string_opt("journal");

        let tier = decoder.string_opt("tier");
        check_enum(&tier, &decoder, "tier", TIERS, errors, " — never a vendor model name, a plan naming one stops being portable between harnesses");

        let gate = decoder.string_opt("gate");
        check_enum(&gate, &decoder, "gate", GATES, errors, "");

        let oracle = decoder
            .value_opt("oracle")
            .map(|v| Oracle::decode(v, &decoder.path_for("oracle"), errors));

        let extra_oracles = decode_list(&decoder, "extra_oracles", errors, Oracle::decode);
        let queue: Vec<Unit> = decode_list(&decoder, "queue", errors, Unit::decode);

        check_verifiable(&oracle, &gate, order, &queue, &decoder, errors);

        let extra = decoder.extra(KNOWN_KEYS);

        Unit {
            id,
            title,
            body,
            done_when,
            deps,
            order,
            phase,
            tier,
            gate,
            oracle,
            extra_oracles,
            repo,
            queue,
            journal,
            extra,
        }
    }

    pub(crate) fn to_value(&self) -> Value {
        let mut map = Mapping::new();
        map.insert(Value::from("id"), Value::from(self.id.as_str()));
        map.insert(Value::from("title"), Value::from(self.title.as_str()));
        if let Some(v) = &self.body {
            map.insert(Value::from("body"), Value::from(v.as_str()));
        }
        if let Some(v) = &self.done_when {
            map.insert(Value::from("done_when"), Value::from(v.as_str()));
        }
        if !self.deps.is_empty() {
            map.insert(
                Value::from("deps"),
                Value::Sequence(self.deps.iter().map(|d| Value::from(d.as_str())).collect()),
            );
        }
        if let Some(v) = self.order {
            map.insert(Value::from("order"), Value::from(v));
        }
        if let Some(v) = &self.phase {
            map.insert(Value::from("phase"), Value::from(v.as_str()));
        }
        if let Some(v) = &self.tier {
            map.insert(Value::from("tier"), Value::from(v.as_str()));
        }
        if let Some(v) = &self.gate {
            map.insert(Value::from("gate"), Value::from(v.as_str()));
        }
        if let Some(o) = &self.oracle {
            map.insert(Value::from("oracle"), o.to_value());
        }
        if !self.extra_oracles.is_empty() {
            map.insert(
                Value::from("extra_oracles"),
                Value::Sequence(self.extra_oracles.iter().map(Oracle::to_value).collect()),
            );
        }
        if let Some(v) = &self.repo {
            map.insert(Value::from("repo"), Value::from(v.as_str()));
        }
        if !self.queue.is_empty() {
            map.insert(
                Value::from("queue"),
                Value::Sequence(self.queue.iter().map(Unit::to_value).collect()),
            );
        }
        if let Some(v) = &self.journal {
            map.insert(Value::from("journal"), Value::from(v.as_str()));
        }
        insert_extra(&mut map, &self.extra);
        Value::Mapping(map)
    }
}

/// Rejects an optional enum-valued field that was present but not one of
/// `allowed`. `suffix` carries a field-specific reason to append, e.g. why a
/// tier is never a vendor model name.
fn check_enum(
    value: &Option<String>,
    decoder: &Decoder,
    key: &str,
    allowed: &[&str],
    errors: &mut Vec<PlanError>,
    suffix: &str,
) {
    let Some(v) = value else { return };
    if !allowed.contains(&v.as_str()) {
        errors.push(PlanError::new(
            decoder.path_for(key),
            format!("must be one of {}{suffix}; got {v:?}", allowed.join(", ")),
        ));
    }
}

/// The old rule was "no oracle, no node"; a loop or hybrid queue item satisfies
/// it with `order` instead, since its verification binds to the plan-level
/// iteration oracle rather than to itself. A hybrid unit that itself carries a
/// `queue` (docs/PLAN-FORMAT.md's own H02 worked example, and
/// fixtures/expressed/hybrid/plan.yaml's H02) is not covered by the prose rule
/// or the schema's `anyOf` as written — neither names `queue` as a fourth
/// satisfying option, yet the worked example has no oracle, gate, or order on
/// that unit either. Treating a non-empty queue as satisfying the rule is the
/// only reading under which the committed fixture and the spec's own example
/// are valid; see the T02b report for the amendment this implies for T02a.
fn check_verifiable(
    oracle: &Option<Oracle>,
    gate: &Option<String>,
    order: Option<i64>,
    queue: &[Unit],
    decoder: &Decoder,
    errors: &mut Vec<PlanError>,
) {
    if oracle.is_none() && gate.is_none() && order.is_none() && queue.is_empty() {
        errors.push(PlanError::new(
            decoder.path_for("oracle"),
            "required when neither gate, order, nor a non-empty queue is present — a unit needs an oracle, a human gate, a queue position, or its own queue of iterated work, or it is unverifiable work with no human holding it",
        ));
    }
}

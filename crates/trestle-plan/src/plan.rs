//! `plan.yaml`: the definition of the work. Never mutated by execution — only
//! `status.yaml` (see `status.rs`) changes while work proceeds.

use std::collections::{HashMap, HashSet};

use serde_yaml::{Mapping, Value};

use crate::decode::{decode_list, insert_extra, Decoder};
use crate::deferred::Deferred;
use crate::error::PlanError;
use crate::oracle::Oracle;
use crate::phase::Phase;
use crate::rule::Rule;
use crate::unit::Unit;

pub const SHAPES: &[&str] = &["graph", "loop", "hybrid"];

#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub trestle_plan: String,
    pub shape: String,
    pub name: String,
    pub goal: String,
    pub done_when: Option<String>,
    pub oracle: Option<Oracle>,
    pub journal: Option<String>,
    pub rules: Vec<Rule>,
    pub phases: Vec<Phase>,
    pub units: Vec<Unit>,
    pub deferred: Vec<Deferred>,
    pub extra: Mapping,
}

const KNOWN_KEYS: &[&str] = &[
    "trestle_plan",
    "shape",
    "name",
    "goal",
    "done_when",
    "oracle",
    "journal",
    "rules",
    "phases",
    "units",
    "deferred",
];

/// Parses and fully validates a `plan.yaml` document. The only entry point
/// this crate has for plan definitions — a string in, a `Plan` or the full
/// list of everything wrong with it out. No path is ever touched: the caller
/// already read the file.
pub fn parse_plan(yaml: &str) -> Result<Plan, Vec<PlanError>> {
    let value: Value = serde_yaml::from_str(yaml)
        .map_err(|e| vec![PlanError::new("<document>", format!("not valid YAML: {e}"))])?;
    let Some(map) = value.as_mapping() else {
        return Err(vec![PlanError::new(
            "<document>",
            "must be a YAML mapping at the top level",
        )]);
    };

    let mut errors = Vec::new();
    let plan = Plan::decode(map, &mut errors);
    plan.validate(&mut errors);

    if errors.is_empty() {
        Ok(plan)
    } else {
        Err(errors)
    }
}

impl Plan {
    fn decode(map: &Mapping, errors: &mut Vec<PlanError>) -> Plan {
        let decoder = Decoder::new(map, "");

        let trestle_plan = decoder.string("trestle_plan", errors);
        let name = decoder.string("name", errors);
        let goal = decoder.string("goal", errors);

        let shape = decoder.string("shape", errors);
        if !shape.is_empty() && !SHAPES.contains(&shape.as_str()) {
            errors.push(PlanError::new(
                "shape",
                format!("must be one of {}, got {shape:?}", SHAPES.join(", ")),
            ));
        }

        let done_when = decoder.string_opt("done_when");
        let oracle = decoder
            .value_opt("oracle")
            .map(|v| Oracle::decode(v, "oracle", errors));
        let journal = decoder.string_opt("journal");

        let rules = decode_list(&decoder, "rules", errors, Rule::decode);
        let phases = decode_list(&decoder, "phases", errors, Phase::decode);
        let units = decode_list(&decoder, "units", errors, Unit::decode);
        let deferred = decode_list(&decoder, "deferred", errors, Deferred::decode);

        let extra = decoder.extra(KNOWN_KEYS);

        Plan {
            trestle_plan,
            shape,
            name,
            goal,
            done_when,
            oracle,
            journal,
            rules,
            phases,
            units,
            deferred,
            extra,
        }
    }

    /// Checks that need the whole plan in view, rather than one field at a
    /// time: shape-conditional requirements, duplicate ids, dangling
    /// dependency edges, and dependency cycles.
    fn validate(&self, errors: &mut Vec<PlanError>) {
        if matches!(self.shape.as_str(), "loop" | "hybrid") {
            if self.oracle.is_none() {
                errors.push(PlanError::new(
                    "oracle",
                    format!(
                        "required for shape {:?} — verification binds to the iteration, not to a queue item, which is how \"no oracle, no node\" stays true for this shape",
                        self.shape
                    ),
                ));
            }
            if self.journal.is_none() {
                errors.push(PlanError::new(
                    "journal",
                    "required for shape \"loop\" and \"hybrid\" — a loop without a journal is a list someone will lose track of",
                ));
            }
        }
        if self.shape == "graph" && self.units.is_empty() {
            errors.push(PlanError::new(
                "units",
                "required and must be non-empty for shape \"graph\"",
            ));
        }

        let flat = flatten_units(&self.units, "units");
        check_duplicate_ids(&flat, errors);
        check_dependency_edges(&flat, errors);
        check_dependency_cycle(&flat, errors);
    }

    pub(crate) fn to_value(&self) -> Value {
        let mut map = Mapping::new();
        map.insert(
            Value::from("trestle_plan"),
            Value::from(self.trestle_plan.as_str()),
        );
        map.insert(Value::from("shape"), Value::from(self.shape.as_str()));
        map.insert(Value::from("name"), Value::from(self.name.as_str()));
        map.insert(Value::from("goal"), Value::from(self.goal.as_str()));
        if let Some(v) = &self.done_when {
            map.insert(Value::from("done_when"), Value::from(v.as_str()));
        }
        if let Some(o) = &self.oracle {
            map.insert(Value::from("oracle"), o.to_value());
        }
        if let Some(v) = &self.journal {
            map.insert(Value::from("journal"), Value::from(v.as_str()));
        }
        if !self.rules.is_empty() {
            map.insert(
                Value::from("rules"),
                Value::Sequence(self.rules.iter().map(Rule::to_value).collect()),
            );
        }
        if !self.phases.is_empty() {
            map.insert(
                Value::from("phases"),
                Value::Sequence(self.phases.iter().map(Phase::to_value).collect()),
            );
        }
        if !self.units.is_empty() {
            map.insert(
                Value::from("units"),
                Value::Sequence(self.units.iter().map(Unit::to_value).collect()),
            );
        }
        if !self.deferred.is_empty() {
            map.insert(
                Value::from("deferred"),
                Value::Sequence(self.deferred.iter().map(Deferred::to_value).collect()),
            );
        }
        insert_extra(&mut map, &self.extra);
        Value::Mapping(map)
    }

    /// Serialises back to a `plan.yaml` document. `parse_plan(&plan.to_yaml())`
    /// is the round trip every fixture is checked against.
    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(&self.to_value()).expect("a Plan always serialises to valid YAML")
    }

    /// Finds a unit by id anywhere in the tree, including nested queue items —
    /// `status.yaml` addresses both a hybrid unit and its queue items in one
    /// flat namespace.
    pub fn find_unit(&self, id: &str) -> Option<&Unit> {
        fn search<'a>(units: &'a [Unit], id: &str) -> Option<&'a Unit> {
            for u in units {
                if u.id == id {
                    return Some(u);
                }
                if let Some(found) = search(&u.queue, id) {
                    return Some(found);
                }
            }
            None
        }
        search(&self.units, id)
    }
}

struct FlatUnit<'a> {
    path: String,
    unit: &'a Unit,
}

fn flatten_units<'a>(units: &'a [Unit], prefix: &str) -> Vec<FlatUnit<'a>> {
    let mut out = Vec::new();
    flatten_into(units, prefix, &mut out);
    out
}

fn flatten_into<'a>(units: &'a [Unit], prefix: &str, out: &mut Vec<FlatUnit<'a>>) {
    for (i, unit) in units.iter().enumerate() {
        let path = format!("{prefix}[{i}]");
        if !unit.queue.is_empty() {
            flatten_into(&unit.queue, &format!("{path}.queue"), out);
        }
        out.push(FlatUnit { path, unit });
    }
}

fn check_duplicate_ids(flat: &[FlatUnit], errors: &mut Vec<PlanError>) {
    let mut first_seen: HashMap<&str, &str> = HashMap::new();
    for f in flat {
        let id = f.unit.id.as_str();
        if let Some(&first_path) = first_seen.get(id) {
            errors.push(PlanError::new(
                format!("{}.id", f.path),
                format!("duplicate unit id {id:?}; already used at {first_path}.id"),
            ));
        } else {
            first_seen.insert(id, &f.path);
        }
    }
}

fn check_dependency_edges(flat: &[FlatUnit], errors: &mut Vec<PlanError>) {
    let ids: HashSet<&str> = flat.iter().map(|f| f.unit.id.as_str()).collect();
    for f in flat {
        for (i, dep) in f.unit.deps.iter().enumerate() {
            if !ids.contains(dep.as_str()) {
                errors.push(PlanError::new(
                    format!("{}.deps[{i}]", f.path),
                    format!("names unit {dep:?}, which does not exist in this plan"),
                ));
            }
        }
    }
}

fn check_dependency_cycle(flat: &[FlatUnit], errors: &mut Vec<PlanError>) {
    let graph: HashMap<&str, &[String]> = flat
        .iter()
        .map(|f| (f.unit.id.as_str(), f.unit.deps.as_slice()))
        .collect();

    let mut state: HashMap<&str, u8> = HashMap::new();
    for &start in graph.keys() {
        if state.get(start).copied().unwrap_or(0) == 0 {
            if let Some(cycle) = dfs_find_cycle(start, &graph, &mut state) {
                errors.push(PlanError::new(
                    "units",
                    format!("dependency cycle: {}", cycle.join(" -> ")),
                ));
                return;
            }
        }
    }
}

/// 0 = unvisited, 1 = on the current path, 2 = fully explored.
fn dfs_find_cycle<'a>(
    node: &'a str,
    graph: &HashMap<&'a str, &'a [String]>,
    state: &mut HashMap<&'a str, u8>,
) -> Option<Vec<String>> {
    state.insert(node, 1);
    if let Some(deps) = graph.get(node) {
        for dep in *deps {
            match state.get(dep.as_str()).copied().unwrap_or(0) {
                0 => {
                    if let Some(mut cycle) = dfs_find_cycle(dep, graph, state) {
                        cycle.insert(0, node.to_string());
                        return Some(cycle);
                    }
                }
                1 => return Some(vec![node.to_string(), dep.clone()]),
                _ => {}
            }
        }
    }
    state.insert(node, 2);
    None
}

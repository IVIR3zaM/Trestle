//! The `Survey` type and its JSON shape — the stable contract T05's node file
//! calls out explicitly: "shipped prompts reference its field names, so
//! carrying `schema_version` and breaking a field is a product-breaking
//! change." No `#[derive(Serialize)]` here (see the comment on the
//! `serde_json` dependency in `Cargo.toml`): every type builds its own
//! `serde_json::Value` by hand.

use serde_json::{json, Value};

/// Bumped only on a breaking change to the shape below, never on an addition.
pub(crate) const SCHEMA_VERSION: &str = "1";

/// How a language's import edges were produced. `TreeSitter` is a real parse;
/// `RegexFallback` and `Unsupported` are the two heuristic-or-absent cases
/// `D3` requires every partial result to name honestly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Support {
    TreeSitter,
    RegexFallback,
    Unsupported,
}

impl Support {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Support::TreeSitter => "tree-sitter",
            Support::RegexFallback => "regex-fallback",
            Support::Unsupported => "unsupported",
        }
    }
}

pub(crate) struct LanguageStat {
    pub(crate) name: String,
    pub(crate) support: Support,
    pub(crate) file_count: usize,
}

impl LanguageStat {
    fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "support": self.support.as_str(),
            "file_count": self.file_count,
        })
    }
}

pub(crate) struct Module {
    pub(crate) name: String,
    pub(crate) language: String,
    pub(crate) file_count: usize,
}

impl Module {
    fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "language": self.language,
            "file_count": self.file_count,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EdgeKind {
    Import,
    CoChange,
}

impl EdgeKind {
    fn as_str(self) -> &'static str {
        match self {
            EdgeKind::Import => "import",
            EdgeKind::CoChange => "co_change",
        }
    }
}

/// One dependency-shaped fact between two modules. `heuristic` is `D3`'s
/// second required label: an import edge from a tree-sitter parse is not
/// heuristic, one from the regex fallback is, and every co-change edge is,
/// always — co-change is coupling evidence, never a dependency edge, and
/// conflating the two is the mistake `D3` calls out by name.
pub(crate) struct Edge {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) kind: EdgeKind,
    pub(crate) heuristic: bool,
    pub(crate) weight: u32,
}

impl Edge {
    fn to_json(&self) -> Value {
        json!({
            "from": self.from,
            "to": self.to,
            "kind": self.kind.as_str(),
            "heuristic": self.heuristic,
            "weight": self.weight,
        })
    }
}

pub(crate) struct DiscoveredCommand {
    pub(crate) kind: String,
    pub(crate) command: String,
    pub(crate) source: String,
}

impl DiscoveredCommand {
    fn to_json(&self) -> Value {
        json!({
            "kind": self.kind,
            "command": self.command,
            "source": self.source,
        })
    }
}

/// Parallelism, measured as connected components over the module graph
/// (import + co-change edges, direction ignored) — modules with no edge
/// between them are the independent tracks T03's rubric asks about.
pub(crate) struct Parallelism {
    pub(crate) independent_clusters: usize,
    pub(crate) cluster_sizes: Vec<usize>,
}

impl Parallelism {
    fn to_json(&self) -> Value {
        json!({
            "independent_clusters": self.independent_clusters,
            "cluster_sizes": self.cluster_sizes,
        })
    }
}

/// Oracle presence is measured: whether a candidate test/build command was
/// discovered, and how many. `measured_runtime_seconds` is always `null` in
/// v0.1.0 — running a discovered command would write into the surveyed repo
/// (build artifacts, caches) and could reach the network, both of which the
/// survey is required not to do. The field exists so the JSON shape is
/// stable when a later version can measure it; see the doc comment on
/// `ShapeSignals` for the same pattern `PRODUCT.md` uses for token cost.
pub(crate) struct OracleSignal {
    pub(crate) present: bool,
    pub(crate) commands_found: usize,
    pub(crate) measured_runtime_seconds: Option<f64>,
}

impl OracleSignal {
    fn to_json(&self) -> Value {
        json!({
            "present": self.present,
            "commands_found": self.commands_found,
            "measured_runtime_seconds": self.measured_runtime_seconds,
        })
    }
}

pub(crate) struct FanOut {
    pub(crate) max: usize,
    pub(crate) mean: f64,
    pub(crate) per_module: Vec<(String, usize)>,
}

impl FanOut {
    fn to_json(&self) -> Value {
        json!({
            "max": self.max,
            "mean": self.mean,
            "per_module": self.per_module.iter().map(|(name, n)| json!({
                "module": name,
                "fan_out": n,
            })).collect::<Vec<_>>(),
        })
    }
}

pub(crate) struct RepoSize {
    pub(crate) file_count: usize,
    pub(crate) total_lines: usize,
}

impl RepoSize {
    fn to_json(&self) -> Value {
        json!({
            "file_count": self.file_count,
            "total_lines": self.total_lines,
        })
    }
}

pub(crate) struct TestRatio {
    pub(crate) test_files: usize,
    pub(crate) source_files: usize,
    pub(crate) ratio: f64,
}

impl TestRatio {
    fn to_json(&self) -> Value {
        json!({
            "test_files": self.test_files,
            "source_files": self.source_files,
            "ratio": self.ratio,
        })
    }
}

/// Every signal T03's rubric weighs, each with the measurement that produced
/// it — see `shape_signals.rs`. The list of field names here is also the
/// canonical signal list `tests/survey_test.rs` iterates for the "every
/// signal T03 consumes has a defined measurement" acceptance criterion.
pub(crate) struct ShapeSignals {
    pub(crate) parallelism: Parallelism,
    pub(crate) oracle: OracleSignal,
    pub(crate) module_fan_out: FanOut,
    pub(crate) repo_size: RepoSize,
    pub(crate) test_to_source_ratio: TestRatio,
}

impl ShapeSignals {
    fn to_json(&self) -> Value {
        json!({
            "parallelism": self.parallelism.to_json(),
            "oracle": self.oracle.to_json(),
            "module_fan_out": self.module_fan_out.to_json(),
            "repo_size": self.repo_size.to_json(),
            "test_to_source_ratio": self.test_to_source_ratio.to_json(),
        })
    }
}

/// The whole survey result. `partial` is `true` the moment any part of the
/// picture is incomplete (an unsupported language, a fallback extraction, a
/// git-log co-change edge) — `partial_reasons` says which parts and why, so
/// the flag is never a confident-looking wrapper around an incomplete graph.
pub struct Survey {
    pub(crate) schema_version: &'static str,
    pub(crate) partial: bool,
    pub(crate) partial_reasons: Vec<String>,
    pub(crate) languages: Vec<LanguageStat>,
    pub(crate) modules: Vec<Module>,
    pub(crate) edges: Vec<Edge>,
    pub(crate) test_commands: Vec<DiscoveredCommand>,
    pub(crate) build_commands: Vec<DiscoveredCommand>,
    pub(crate) ci_configs: Vec<String>,
    pub(crate) conventions: Vec<String>,
    pub(crate) shape_signals: ShapeSignals,
}

impl Survey {
    pub fn to_json(&self) -> Value {
        json!({
            "schema_version": self.schema_version,
            "partial": self.partial,
            "partial_reasons": self.partial_reasons,
            "languages": self.languages.iter().map(LanguageStat::to_json).collect::<Vec<_>>(),
            "modules": self.modules.iter().map(Module::to_json).collect::<Vec<_>>(),
            "edges": self.edges.iter().map(Edge::to_json).collect::<Vec<_>>(),
            "test_commands": self.test_commands.iter().map(DiscoveredCommand::to_json).collect::<Vec<_>>(),
            "build_commands": self.build_commands.iter().map(DiscoveredCommand::to_json).collect::<Vec<_>>(),
            "ci_configs": self.ci_configs,
            "conventions": self.conventions,
            "shape_signals": self.shape_signals.to_json(),
        })
    }

    /// Pretty-printed, so a golden-file diff reads as an actual field
    /// change rather than a one-line blob. `serde_json::Value`'s map is a
    /// `BTreeMap` (no `preserve_order` feature enabled), so key order is
    /// alphabetical and therefore stable across runs.
    pub fn to_json_string_pretty(&self) -> String {
        serde_json::to_string_pretty(&self.to_json()).expect("Value serialization cannot fail")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_is_present_in_output() {
        let survey = Survey {
            schema_version: SCHEMA_VERSION,
            partial: false,
            partial_reasons: vec![],
            languages: vec![],
            modules: vec![],
            edges: vec![],
            test_commands: vec![],
            build_commands: vec![],
            ci_configs: vec![],
            conventions: vec![],
            shape_signals: ShapeSignals {
                parallelism: Parallelism {
                    independent_clusters: 0,
                    cluster_sizes: vec![],
                },
                oracle: OracleSignal {
                    present: false,
                    commands_found: 0,
                    measured_runtime_seconds: None,
                },
                module_fan_out: FanOut {
                    max: 0,
                    mean: 0.0,
                    per_module: vec![],
                },
                repo_size: RepoSize {
                    file_count: 0,
                    total_lines: 0,
                },
                test_to_source_ratio: TestRatio {
                    test_files: 0,
                    source_files: 0,
                    ratio: 0.0,
                },
            },
        };
        assert_eq!(survey.to_json()["schema_version"], SCHEMA_VERSION);
    }
}

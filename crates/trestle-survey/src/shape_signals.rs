//! The five shape signals T03's rubric consumes, each with the measurement
//! that produces it (the node's own requirement: "a signal with no defined
//! measurement is not a signal"). `tests/survey_test.rs` iterates
//! `SIGNAL_NAMES` below to assert every one of them is present in a real
//! survey's output, rather than hand-writing five separate checks that could
//! silently drift from this list.

use crate::commands::Discovered;
use crate::repo_files::SourceFile;
use crate::schema::{
    Edge, EdgeKind, FanOut, Module, OracleSignal, Parallelism, RepoSize, ShapeSignals, TestRatio,
};
use std::collections::{HashMap, HashSet};

/// The canonical signal list this node commits to measuring — see the
/// module doc comment. T03 does not exist as a crate yet (it depends on
/// this node), so there is no shared source to import this from; when it
/// lands, point this list or its test at whichever one is authoritative.
/// `#[cfg(test)]`: nothing in production code needs this list, only the
/// acceptance test in `survey.rs` that iterates it.
#[cfg(test)]
pub(crate) const SIGNAL_NAMES: &[&str] = &[
    "parallelism",
    "oracle",
    "module_fan_out",
    "repo_size",
    "test_to_source_ratio",
];

pub(crate) fn compute(
    modules: &[Module],
    edges: &[Edge],
    commands: &Discovered,
    files: &[SourceFile],
) -> ShapeSignals {
    ShapeSignals {
        parallelism: parallelism(modules, edges),
        oracle: oracle(commands),
        module_fan_out: module_fan_out(modules, edges),
        repo_size: repo_size(files),
        test_to_source_ratio: test_to_source_ratio(files),
    }
}

/// Connected components over the module graph, both edge kinds counted as
/// connectivity (a real import and a strong co-change pattern both argue
/// against two modules being independent work). A module with no edge at
/// all is its own cluster of size one — that is exactly the independent
/// track T03's rubric is asking about.
fn parallelism(modules: &[Module], edges: &[Edge]) -> Parallelism {
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for module in modules {
        adjacency.entry(module.name.as_str()).or_default();
    }
    for edge in edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
        adjacency
            .entry(edge.to.as_str())
            .or_default()
            .push(edge.from.as_str());
    }

    let mut visited: HashSet<&str> = HashSet::new();
    let mut cluster_sizes = Vec::new();
    for module in modules {
        if visited.contains(module.name.as_str()) {
            continue;
        }
        let mut stack = vec![module.name.as_str()];
        let mut size = 0;
        while let Some(node) = stack.pop() {
            if !visited.insert(node) {
                continue;
            }
            size += 1;
            for neighbour in adjacency.get(node).into_iter().flatten() {
                if !visited.contains(neighbour) {
                    stack.push(neighbour);
                }
            }
        }
        cluster_sizes.push(size);
    }
    cluster_sizes.sort_unstable();

    Parallelism {
        independent_clusters: cluster_sizes.len(),
        cluster_sizes,
    }
}

/// Presence and count are measured directly; `measured_runtime_seconds` is
/// always `None` — see the doc comment on `OracleSignal` in `schema.rs` for
/// why running a discovered command is out of scope for this node.
fn oracle(commands: &Discovered) -> OracleSignal {
    let commands_found = commands.test_commands.len() + commands.build_commands.len();
    OracleSignal {
        present: commands_found > 0,
        commands_found,
        measured_runtime_seconds: None,
    }
}

/// Fan-out counts only import edges — co-change is coupling, not a
/// dependency, and folding it in here would make the number answer a
/// different question than "how many modules does this one depend on".
fn module_fan_out(modules: &[Module], edges: &[Edge]) -> FanOut {
    let mut per_module: Vec<(String, usize)> = modules
        .iter()
        .map(|m| {
            let fan_out = edges
                .iter()
                .filter(|e| e.kind == EdgeKind::Import && e.from == m.name)
                .count();
            (m.name.clone(), fan_out)
        })
        .collect();
    per_module.sort_by(|a, b| a.0.cmp(&b.0));

    let max = per_module.iter().map(|(_, n)| *n).max().unwrap_or(0);
    let mean = if per_module.is_empty() {
        0.0
    } else {
        per_module.iter().map(|(_, n)| *n).sum::<usize>() as f64 / per_module.len() as f64
    };

    FanOut {
        max,
        mean,
        per_module,
    }
}

fn repo_size(files: &[SourceFile]) -> RepoSize {
    RepoSize {
        file_count: files.len(),
        total_lines: files.iter().map(|f| f.line_count).sum(),
    }
}

fn test_to_source_ratio(files: &[SourceFile]) -> TestRatio {
    let test_files = files.iter().filter(|f| f.is_test).count();
    let source_files = files.iter().filter(|f| !f.is_test).count();
    let ratio = if source_files == 0 {
        0.0
    } else {
        test_files as f64 / source_files as f64
    };
    TestRatio {
        test_files,
        source_files,
        ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::DiscoveredCommand;

    fn module(name: &str) -> Module {
        Module {
            name: name.to_string(),
            language: "python".to_string(),
            file_count: 1,
        }
    }

    fn import_edge(from: &str, to: &str) -> Edge {
        Edge {
            from: from.to_string(),
            to: to.to_string(),
            kind: EdgeKind::Import,
            heuristic: false,
            weight: 1,
        }
    }

    #[test]
    fn modules_with_no_edge_between_them_are_separate_clusters() {
        let modules = vec![module("a"), module("b")];
        let signals = parallelism(&modules, &[]);
        assert_eq!(signals.independent_clusters, 2);
        assert_eq!(signals.cluster_sizes, vec![1, 1]);
    }

    #[test]
    fn an_import_edge_merges_two_modules_into_one_cluster() {
        let modules = vec![module("a"), module("b")];
        let edges = vec![import_edge("a", "b")];
        let signals = parallelism(&modules, &edges);
        assert_eq!(signals.independent_clusters, 1);
        assert_eq!(signals.cluster_sizes, vec![2]);
    }

    #[test]
    fn oracle_presence_reflects_discovered_commands() {
        let none = Discovered {
            test_commands: vec![],
            build_commands: vec![],
        };
        assert!(!oracle(&none).present);

        let some = Discovered {
            test_commands: vec![DiscoveredCommand {
                kind: "pytest".to_string(),
                command: "pytest".to_string(),
                source: "pyproject.toml".to_string(),
            }],
            build_commands: vec![],
        };
        let signal = oracle(&some);
        assert!(signal.present);
        assert_eq!(signal.commands_found, 1);
        assert_eq!(signal.measured_runtime_seconds, None);
    }

    #[test]
    fn fan_out_counts_import_edges_only_not_co_change() {
        let modules = vec![module("a"), module("b")];
        let edges = vec![
            import_edge("a", "b"),
            Edge {
                from: "a".to_string(),
                to: "b".to_string(),
                kind: EdgeKind::CoChange,
                heuristic: true,
                weight: 5,
            },
        ];
        let fan_out = module_fan_out(&modules, &edges);
        assert_eq!(
            fan_out.per_module,
            vec![("a".to_string(), 1), ("b".to_string(), 0)]
        );
        assert_eq!(fan_out.max, 1);
    }

    #[test]
    fn test_to_source_ratio_divides_test_files_by_source_files() {
        let files = vec![
            SourceFile {
                relative_path: "app.py".to_string(),
                language: "python",
                support: crate::schema::Support::TreeSitter,
                line_count: 1,
                is_test: false,
            },
            SourceFile {
                relative_path: "test_app.py".to_string(),
                language: "python",
                support: crate::schema::Support::TreeSitter,
                line_count: 1,
                is_test: true,
            },
        ];
        let ratio = test_to_source_ratio(&files);
        assert_eq!(ratio.test_files, 1);
        assert_eq!(ratio.source_files, 1);
        assert!((ratio.ratio - 1.0).abs() < f64::EPSILON);
    }
}

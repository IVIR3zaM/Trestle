//! Groups files into modules and resolves import/co-change edges between
//! them — the load-bearing output T05's node file names explicitly, since
//! it feeds the blast-radius overlay (T15) and the parallelism signal (T03).
//! Module granularity is deliberately coarse (`D3`: useful, not accurate):
//! a module is a file's top-level directory under the repo root, or the
//! file itself when it has none.

use crate::schema::{Edge, EdgeKind, Module, Support};
use crate::{co_change, regex_imports, repo_files::SourceFile, tree_sitter_imports};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(crate) struct ModuleGraph {
    pub(crate) modules: Vec<Module>,
    pub(crate) edges: Vec<Edge>,
    /// One entry per language whose import edges are not a real parse —
    /// `D3`'s labelling requirement, in prose specific enough to act on.
    pub(crate) partial_reasons: Vec<String>,
}

/// A file's module is the first path segment (`shared/helper.py` → `shared`)
/// or, when the file sits directly under the repo root, its own stem
/// (`rust_shared.rs` → `rust_shared`) — the same rule for every language,
/// which is what makes a Python package and a bare Rust file both resolve
/// the same way.
fn module_name_for(relative_path: &str) -> String {
    let path = Path::new(relative_path);
    let mut components = path.components();
    let first = components
        .next()
        .map(|c| c.as_os_str().to_string_lossy().into_owned());
    match (first, components.next()) {
        (Some(first), Some(_second)) => first,
        _ => path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
    }
}

pub(crate) fn build(repo_root: &Path, files: &[SourceFile]) -> ModuleGraph {
    let modules = group_into_modules(files);
    let module_names: HashSet<&str> = modules.iter().map(|m| m.name.as_str()).collect();

    let mut edge_weights: HashMap<(String, String), (u32, bool)> = HashMap::new();
    let mut partial_languages: HashSet<&'static str> = HashSet::new();

    for file in files {
        if file.support == Support::Unsupported {
            partial_languages.insert(file.language);
            continue;
        }
        let from_module = module_name_for(&file.relative_path);
        let Ok(source) = std::fs::read_to_string(repo_root.join(&file.relative_path)) else {
            continue;
        };
        let heuristic = file.support == Support::RegexFallback;
        if heuristic {
            partial_languages.insert(file.language);
        }
        for target in raw_import_targets(file.language, file.support, &source) {
            if target == from_module || !module_names.contains(target.as_str()) {
                continue;
            }
            let entry = edge_weights
                .entry((from_module.clone(), target))
                .or_insert((0, heuristic));
            entry.0 += 1;
            entry.1 |= heuristic;
        }
    }

    let mut edges: Vec<Edge> = edge_weights
        .into_iter()
        .map(|((from, to), (weight, heuristic))| Edge {
            from,
            to,
            kind: EdgeKind::Import,
            heuristic,
            weight,
        })
        .collect();
    edges.extend(co_change_edges(repo_root, &module_names));
    edges.sort_by(|a, b| (&a.from, &a.to, a.kind as u8).cmp(&(&b.from, &b.to, b.kind as u8)));

    let mut partial_reasons: Vec<String> = partial_languages
        .into_iter()
        .map(|language| {
            format!("{language}: import edges are a heuristic fallback or unavailable (D3)")
        })
        .collect();
    partial_reasons.sort();

    ModuleGraph {
        modules,
        edges,
        partial_reasons,
    }
}

fn raw_import_targets(language: &'static str, support: Support, source: &str) -> Vec<String> {
    match (language, support) {
        ("rust", Support::TreeSitter) => tree_sitter_imports::rust_use_targets(source),
        ("python", Support::TreeSitter) => tree_sitter_imports::python_import_targets(source),
        (_, Support::RegexFallback) => regex_imports::js_require_targets(source),
        _ => Vec::new(),
    }
}

fn group_into_modules(files: &[SourceFile]) -> Vec<Module> {
    let mut by_name: HashMap<String, Module> = HashMap::new();
    for file in files {
        let name = module_name_for(&file.relative_path);
        by_name
            .entry(name.clone())
            .or_insert_with(|| Module {
                name,
                language: file.language.to_string(),
                file_count: 0,
            })
            .file_count += 1;
    }
    let mut modules: Vec<Module> = by_name.into_values().collect();
    modules.sort_by(|a, b| a.name.cmp(&b.name));
    modules
}

/// File-level co-change counts (`co_change.rs`), rolled up to the same
/// module granularity as import edges and always marked `heuristic` —
/// `D3`'s explicit requirement that coupling evidence is never presented as
/// a dependency edge. `None` (no git repository) produces no edges at all,
/// silently — that is not a partial result, it is a signal that does not
/// apply outside version control.
///
/// Only pairs where both sides resolve to a module this survey actually
/// found are kept. That is not just noise reduction: `git log` reports
/// paths relative to the repository's real root, which differs from
/// `repo_root` when `repo_root` is a subdirectory of a larger repository
/// (see the comment in `co_change.rs`) — in that case every path carries an
/// extra prefix, resolves to a module name nothing else produced, and is
/// dropped here rather than surfaced as a wrong edge.
fn co_change_edges(repo_root: &Path, module_names: &HashSet<&str>) -> Vec<Edge> {
    let Some(file_pair_counts) = co_change::co_change_counts(repo_root) else {
        return Vec::new();
    };
    let mut module_pair_counts: HashMap<(String, String), u32> = HashMap::new();
    for ((file_a, file_b), count) in file_pair_counts {
        let module_a = module_name_for(&file_a);
        let module_b = module_name_for(&file_b);
        if module_a == module_b
            || !module_names.contains(module_a.as_str())
            || !module_names.contains(module_b.as_str())
        {
            continue;
        }
        let key = if module_a <= module_b {
            (module_a, module_b)
        } else {
            (module_b, module_a)
        };
        *module_pair_counts.entry(key).or_insert(0) += count;
    }
    module_pair_counts
        .into_iter()
        .map(|((from, to), weight)| Edge {
            from,
            to,
            kind: EdgeKind::CoChange,
            heuristic: true,
            weight,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo_files;

    #[test]
    fn module_name_for_uses_first_directory_component() {
        assert_eq!(module_name_for("shared/helper.py"), "shared");
        assert_eq!(module_name_for("app_a/deep/nested.py"), "app_a");
    }

    #[test]
    fn module_name_for_root_level_file_uses_its_own_stem() {
        assert_eq!(module_name_for("rust_shared.rs"), "rust_shared");
    }

    #[test]
    fn consumers_importing_a_shared_package_produce_edges_to_it() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("app_a")).unwrap();
        std::fs::create_dir_all(dir.path().join("app_b")).unwrap();
        std::fs::create_dir_all(dir.path().join("shared")).unwrap();
        std::fs::write(
            dir.path().join("app_a/main.py"),
            "from shared import helper\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("app_b/main.py"),
            "from shared import helper\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("shared/helper.py"), "def helper(): pass\n").unwrap();

        let files = repo_files::walk(dir.path());
        let graph = build(dir.path(), &files);

        let has_edge = |from: &str, to: &str| {
            graph
                .edges
                .iter()
                .any(|e| e.from == from && e.to == to && e.kind == EdgeKind::Import)
        };
        assert!(
            has_edge("app_a", "shared"),
            "expected app_a -> shared: {:?}",
            graph
                .edges
                .iter()
                .map(|e| (&e.from, &e.to))
                .collect::<Vec<_>>()
        );
        assert!(has_edge("app_b", "shared"), "expected app_b -> shared");
    }

    #[test]
    fn import_of_a_module_not_in_the_repo_produces_no_edge() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("app.py"), "import os\n").unwrap();
        let files = repo_files::walk(dir.path());
        let graph = build(dir.path(), &files);
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn unsupported_language_file_is_named_in_partial_reasons() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("app.rb"), "require 'shared'\n").unwrap();
        let files = repo_files::walk(dir.path());
        let graph = build(dir.path(), &files);
        assert!(graph.partial_reasons.iter().any(|r| r.contains("ruby")));
    }

    #[test]
    fn regex_fallback_import_edge_is_marked_heuristic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("consumer.js"),
            "const shared = require('./shared');\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("shared.js"), "module.exports = {};\n").unwrap();
        let files = repo_files::walk(dir.path());
        let graph = build(dir.path(), &files);
        let edge = graph
            .edges
            .iter()
            .find(|e| e.from == "consumer" && e.to == "shared")
            .expect("expected consumer -> shared edge");
        assert!(edge.heuristic);
    }

    #[test]
    fn tree_sitter_import_edge_is_not_marked_heuristic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("shared")).unwrap();
        std::fs::write(dir.path().join("app.py"), "from shared import helper\n").unwrap();
        std::fs::write(dir.path().join("shared/helper.py"), "def helper(): pass\n").unwrap();
        let files = repo_files::walk(dir.path());
        let graph = build(dir.path(), &files);
        let edge = graph
            .edges
            .iter()
            .find(|e| e.kind == EdgeKind::Import)
            .expect("expected an import edge");
        assert!(!edge.heuristic);
    }
}

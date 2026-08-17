//! Real parse extraction for the two `D3` tree-sitter-supported languages.
//! Tree-sitter finds the import statements reliably (so a `use` inside a
//! string literal or comment is never mistaken for one); a plain text slice
//! of each statement is then enough to pull out the first path segment —
//! the "useful, not accurate" bar `D3` sets means a full path-resolution
//! pass buys nothing the module-level edge doesn't already give T03 and T15.

use std::sync::LazyLock;
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

static RUST_LANGUAGE: LazyLock<Language> = LazyLock::new(|| tree_sitter_rust::LANGUAGE.into());
static PYTHON_LANGUAGE: LazyLock<Language> = LazyLock::new(|| tree_sitter_python::LANGUAGE.into());

/// The raw first path segment named by every `use` declaration in `source`,
/// e.g. `use crate::shared::helper;` yields `"shared"` and `use std::fmt;`
/// yields `"std"` — module resolution (deciding which of these are internal
/// to the surveyed repo) happens one layer up, in `survey.rs`.
pub(crate) fn rust_use_targets(source: &str) -> Vec<String> {
    query_first_segments(&RUST_LANGUAGE, source, "(use_declaration) @use", |text| {
        let path = text.trim_start_matches("use ").trim_end_matches(';').trim();
        let path = path
            .trim_start_matches("crate::")
            .trim_start_matches("self::")
            .trim_start_matches("super::");
        first_segment(path, "::")
    })
}

/// The raw module named by every `import`/`from ... import` statement, e.g.
/// `from shared import helper` yields `"shared"` and `import a.b` yields
/// `"a"`. Relative imports (`from . import x`) have no name to resolve
/// against and are skipped.
pub(crate) fn python_import_targets(source: &str) -> Vec<String> {
    let imports = query_first_segments(
        &PYTHON_LANGUAGE,
        source,
        "(import_statement) @import",
        |text| first_segment(text.trim_start_matches("import ").trim(), "."),
    );
    let from_imports = query_first_segments(
        &PYTHON_LANGUAGE,
        source,
        "(import_from_statement) @import",
        |text| {
            let rest = text.trim_start_matches("from ").trim();
            if rest.starts_with('.') {
                return None;
            }
            // The captured text is the whole statement ("shared import
            // helper"), not just the module name, so the module name ends
            // at the first `.` (a dotted module) or the first space (the
            // `import` keyword) — whichever comes first.
            let module_name_end = rest.find(['.', ' ']).unwrap_or(rest.len());
            first_segment(&rest[..module_name_end], ".")
        },
    );
    imports.into_iter().chain(from_imports).collect()
}

/// Runs `query_source` over `source`, hands each matched node's raw text to
/// `parse`, and keeps the results that parsed to something. One tiny query
/// per call site keeps the tree-sitter plumbing (parse, query, cursor,
/// streaming-iterator) in this one place rather than duplicated per
/// language, while `parse` stays the only language-specific part.
fn query_first_segments(
    language: &Language,
    source: &str,
    query_source: &str,
    parse: impl Fn(&str) -> Option<String>,
) -> Vec<String> {
    let mut parser = Parser::new();
    parser
        .set_language(language)
        .expect("statically linked tree-sitter grammar is always compatible");
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };
    let query =
        Query::new(language, query_source).expect("query source above is a fixed, valid pattern");
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    let mut found = Vec::new();
    while let Some(m) = matches.next() {
        for capture in m.captures {
            let text = capture.node.utf8_text(source.as_bytes()).unwrap_or("");
            if let Some(target) = parse(text) {
                found.push(target);
            }
        }
    }
    found
}

/// The substring of `path` up to (not including) the first `separator`, or
/// all of `path` if `separator` never appears. `None` for an empty segment —
/// a bare `use {a, b};` or similar has no single first identifier to name.
fn first_segment(path: &str, separator: &str) -> Option<String> {
    let segment = path.split(separator).next()?.trim();
    if segment.is_empty() || segment.contains(['{', '(', ' ']) {
        None
    } else {
        Some(segment.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_use_declaration_yields_first_segment_after_crate_prefix() {
        let source = "use crate::shared::helper;\nfn main() {}\n";
        assert_eq!(rust_use_targets(source), vec!["shared".to_string()]);
    }

    #[test]
    fn rust_use_declaration_without_crate_prefix_yields_first_segment() {
        let source = "use std::collections::HashMap;\n";
        assert_eq!(rust_use_targets(source), vec!["std".to_string()]);
    }

    #[test]
    fn rust_use_inside_a_string_literal_is_not_mistaken_for_a_declaration() {
        let source = "fn f() { let s = \"use crate::not_real;\"; }\n";
        assert!(rust_use_targets(source).is_empty());
    }

    #[test]
    fn python_import_statement_yields_first_dotted_segment() {
        let source = "import shared.helper\n";
        assert_eq!(python_import_targets(source), vec!["shared".to_string()]);
    }

    #[test]
    fn python_from_import_yields_module_name() {
        let source = "from shared import helper\n";
        assert_eq!(python_import_targets(source), vec!["shared".to_string()]);
    }

    #[test]
    fn python_relative_import_has_no_resolvable_name() {
        let source = "from . import helper\n";
        assert!(python_import_targets(source).is_empty());
    }
}

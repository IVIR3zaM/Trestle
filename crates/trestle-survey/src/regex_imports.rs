//! `D3`'s fallback for a language tree-sitter is not wired up for in this
//! survey: `import ... from '<path>'` and `require('<path>')`, matched by
//! substring search rather than a real parse. Every edge this produces is
//! marked `heuristic` by the caller (`survey.rs`) — a regex has no idea
//! whether it matched inside a comment or a template string, so it is
//! useful evidence, never presented as a resolved dependency.

/// The raw relative path named by every `import ... from '<path>'` or
/// `require('<path>')` in `source`. Only relative paths (`./x`, `../x`) are
/// returned — a bare `require('react')` names a package, not a module in
/// this repo, and module resolution one layer up only wants candidates that
/// could plausibly be internal.
pub(crate) fn js_require_targets(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in source.lines() {
        for marker in ["require(", "from "] {
            if let Some(start) = line.find(marker) {
                if let Some(path) = quoted_path_after(&line[start + marker.len()..]) {
                    if let Some(name) = relative_module_name(&path) {
                        found.push(name);
                    }
                }
            }
        }
    }
    found
}

/// The text between the first matching pair of `'` or `"` in `text`, which
/// is where both `require('./x')` and `from './x'` put the module path.
fn quoted_path_after(text: &str) -> Option<String> {
    let text = text.trim_start();
    let quote = text.chars().next().filter(|c| *c == '\'' || *c == '"')?;
    let rest = &text[1..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

/// `./shared` and `../shared/thing` both name the top-level module
/// `shared`; a bare package name (`react`, `lodash/fp`) is external and
/// returns `None`.
fn relative_module_name(path: &str) -> Option<String> {
    if !path.starts_with('.') {
        return None;
    }
    let trimmed = path.trim_start_matches("../").trim_start_matches("./");
    let first = trimmed.split('/').next()?;
    if first.is_empty() {
        None
    } else {
        Some(first.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_of_a_relative_path_yields_the_top_level_module() {
        let source = "const shared = require('./shared');\n";
        assert_eq!(js_require_targets(source), vec!["shared".to_string()]);
    }

    #[test]
    fn es_import_of_a_relative_path_yields_the_top_level_module() {
        let source = "import { helper } from '../shared/helper';\n";
        assert_eq!(js_require_targets(source), vec!["shared".to_string()]);
    }

    #[test]
    fn bare_package_import_is_not_treated_as_internal() {
        let source = "import React from 'react';\n";
        assert!(js_require_targets(source).is_empty());
    }
}

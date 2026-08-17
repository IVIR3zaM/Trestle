//! Which language a source file is, and which extraction tier `D3` assigns
//! it. Three tiers, not two, because "unsupported" and "regex fallback" are
//! different honesty claims: a fallback edge is heuristic but real, an
//! unsupported language has none at all.

use crate::schema::Support;
use std::path::Path;

/// `None` means the extension is not source code this survey tracks at all
/// (docs, config, data) — excluded from every language/module count rather
/// than counted as a fourth, silent category.
pub(crate) fn detect(path: &Path) -> Option<(&'static str, Support)> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some(("rust", Support::TreeSitter)),
        "py" => Some(("python", Support::TreeSitter)),
        "js" | "jsx" | "mjs" | "cjs" => Some(("javascript", Support::RegexFallback)),
        "rb" => Some(("ruby", Support::Unsupported)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_and_python_are_tree_sitter_supported() {
        assert_eq!(
            detect(Path::new("a.rs")),
            Some(("rust", Support::TreeSitter))
        );
        assert_eq!(
            detect(Path::new("a.py")),
            Some(("python", Support::TreeSitter))
        );
    }

    #[test]
    fn javascript_is_regex_fallback() {
        assert_eq!(
            detect(Path::new("a.js")),
            Some(("javascript", Support::RegexFallback))
        );
    }

    #[test]
    fn ruby_is_deliberately_unsupported() {
        assert_eq!(
            detect(Path::new("a.rb")),
            Some(("ruby", Support::Unsupported))
        );
    }

    #[test]
    fn unrecognised_extension_is_not_a_language_at_all() {
        assert_eq!(detect(Path::new("README.md")), None);
    }
}

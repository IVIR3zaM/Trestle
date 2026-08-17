//! One walk of the repository, producing the per-file facts every other
//! measurement (module boundaries, import edges, repo size, test-to-source
//! ratio) is built from — so the tree is only ever walked once, and a
//! change to what counts as "a source file" only has to happen here.

use crate::language::detect;
use crate::schema::Support;
use std::path::Path;

pub(crate) struct SourceFile {
    /// Relative to the repo root, always forward-slash-separated so module
    /// names and edge endpoints don't depend on the host OS.
    pub(crate) relative_path: String,
    pub(crate) language: &'static str,
    pub(crate) support: Support,
    pub(crate) line_count: usize,
    pub(crate) is_test: bool,
}

const SKIPPED_DIRS: &[&str] = &[".git", "node_modules", "target", "__pycache__", "venv"];

/// Every recognised source file under `repo_root`, walked by hand rather
/// than via a directory-walking crate — the same call this crate's own
/// `AGENTS.md` §1 makes for `trestle-egress`: the tree is small enough that
/// the dependency would be indirection with one caller.
pub(crate) fn walk(repo_root: &Path) -> Vec<SourceFile> {
    let mut files = Vec::new();
    let mut stack = vec![repo_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !SKIPPED_DIRS.contains(&name) {
                    stack.push(path);
                }
            } else if let Some(file) = classify(repo_root, &path) {
                files.push(file);
            }
        }
    }
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    files
}

fn classify(repo_root: &Path, path: &Path) -> Option<SourceFile> {
    let (language, support) = detect(path)?;
    let relative_path = path
        .strip_prefix(repo_root)
        .ok()?
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    let text = std::fs::read_to_string(path).ok()?;
    Some(SourceFile {
        is_test: looks_like_a_test_file(&relative_path),
        line_count: text.lines().count(),
        relative_path,
        language,
        support,
    })
}

/// A file counts as a test file if a path segment is a conventional test
/// directory, or the file name itself follows one of the common test-naming
/// conventions across the languages this survey recognises.
fn looks_like_a_test_file(relative_path: &str) -> bool {
    let path = Path::new(relative_path);
    let in_test_dir = path.components().any(|c| {
        matches!(
            c.as_os_str().to_str(),
            Some("test" | "tests" | "spec" | "specs" | "__tests__")
        )
    });
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    in_test_dir
        || stem.starts_with("test_")
        || stem.ends_with("_test")
        || file_name.contains(".test.")
        || file_name.contains(".spec.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn walk_finds_source_files_and_skips_git_and_vendor_dirs() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.py"), "import os\n").unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/config"), "not source").unwrap();
        fs::create_dir(dir.path().join("node_modules")).unwrap();
        fs::write(
            dir.path().join("node_modules/lib.js"),
            "module.exports = {};",
        )
        .unwrap();
        let files = walk(dir.path());
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].relative_path, "main.py");
    }

    #[test]
    fn nested_path_becomes_forward_slash_relative_path() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("shared")).unwrap();
        fs::write(dir.path().join("shared/helper.py"), "x = 1\n").unwrap();
        let files = walk(dir.path());
        assert_eq!(files[0].relative_path, "shared/helper.py");
    }

    #[test]
    fn line_count_matches_the_file_content() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.py"), "one\ntwo\nthree\n").unwrap();
        let files = walk(dir.path());
        assert_eq!(files[0].line_count, 3);
    }

    #[test]
    fn file_under_a_tests_directory_is_classified_as_a_test_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("tests")).unwrap();
        fs::write(dir.path().join("tests/check.py"), "assert True\n").unwrap();
        fs::write(dir.path().join("app.py"), "x = 1\n").unwrap();
        let files = walk(dir.path());
        let test_file = files
            .iter()
            .find(|f| f.relative_path.contains("tests"))
            .unwrap();
        let source_file = files.iter().find(|f| f.relative_path == "app.py").unwrap();
        assert!(test_file.is_test);
        assert!(!source_file.is_test);
    }

    #[test]
    fn test_prefixed_file_name_is_classified_as_a_test_file_without_a_test_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test_app.py"), "assert True\n").unwrap();
        let files = walk(dir.path());
        assert!(files[0].is_test);
    }
}

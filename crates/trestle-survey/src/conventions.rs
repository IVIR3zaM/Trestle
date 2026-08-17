//! Existing agent conventions — `AGENTS.md`, `CLAUDE.md`, `.cursorrules`,
//! `.claude/` — surfaced so T08 can fold them into a plan's rules rather
//! than proposing something that contradicts what the repo already says.
//! Presence only: reading and classifying the content is T08's job, named
//! out of scope on this node.

use std::path::Path;

const CONVENTION_PATHS: &[&str] = &["AGENTS.md", "CLAUDE.md", ".cursorrules", ".claude"];

pub(crate) fn discover(repo_root: &Path) -> Vec<String> {
    CONVENTION_PATHS
        .iter()
        .filter(|name| repo_root.join(name).exists())
        .map(|name| (*name).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn finds_agents_md_and_claude_dir_but_not_absent_ones() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("AGENTS.md"), "rules").unwrap();
        fs::create_dir(dir.path().join(".claude")).unwrap();
        let found = discover(dir.path());
        assert!(found.contains(&"AGENTS.md".to_string()));
        assert!(found.contains(&".claude".to_string()));
        assert!(!found.contains(&"CLAUDE.md".to_string()));
        assert!(!found.contains(&".cursorrules".to_string()));
    }

    #[test]
    fn no_conventions_present_yields_empty_list() {
        let dir = tempfile::tempdir().unwrap();
        assert!(discover(dir.path()).is_empty());
    }
}

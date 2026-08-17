//! CI configuration files, if any — named in the Goal alongside test/build
//! commands because a CI workflow is often the most reliable place to find
//! the *actual* command a repo runs, not just a candidate one. This node
//! only records which files exist; reading the commands out of them is left
//! to T08's convention classification.

use std::path::Path;

const CI_GLOBS: &[(&str, &[&str])] = &[
    (".github/workflows", &["yml", "yaml"]),
    (".circleci", &["yml", "yaml"]),
];

pub(crate) fn discover(repo_root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for (dir, extensions) in CI_GLOBS {
        let Ok(entries) = std::fs::read_dir(repo_root.join(dir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| extensions.contains(&ext))
            {
                found.push(format!(
                    "{dir}/{}",
                    path.file_name().unwrap().to_string_lossy()
                ));
            }
        }
    }
    for single_file in [".gitlab-ci.yml", ".travis.yml"] {
        if repo_root.join(single_file).exists() {
            found.push(single_file.to_string());
        }
    }
    found.sort();
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn github_actions_workflow_is_discovered() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".github/workflows")).unwrap();
        fs::write(dir.path().join(".github/workflows/ci.yml"), "on: push").unwrap();
        assert_eq!(
            discover(dir.path()),
            vec![".github/workflows/ci.yml".to_string()]
        );
    }

    #[test]
    fn gitlab_ci_file_is_discovered() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".gitlab-ci.yml"), "stages: []").unwrap();
        assert_eq!(discover(dir.path()), vec![".gitlab-ci.yml".to_string()]);
    }

    #[test]
    fn no_ci_config_yields_empty_list() {
        let dir = tempfile::tempdir().unwrap();
        assert!(discover(dir.path()).is_empty());
    }
}

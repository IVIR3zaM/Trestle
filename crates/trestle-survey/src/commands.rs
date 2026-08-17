//! Test and build commands, discovered rather than assumed — the node's own
//! requirement, because these become candidate oracles during synthesis
//! (T07) and a wrong guess there is expensive. Each source format gets its
//! own small reader; a command is classified as `test` or `build` by name,
//! since none of these formats separate the two structurally.

use crate::schema::DiscoveredCommand;
use std::path::Path;

pub(crate) struct Discovered {
    pub(crate) test_commands: Vec<DiscoveredCommand>,
    pub(crate) build_commands: Vec<DiscoveredCommand>,
}

pub(crate) fn discover(repo_root: &Path) -> Discovered {
    let mut test_commands = Vec::new();
    let mut build_commands = Vec::new();

    package_json_scripts(repo_root, &mut test_commands, &mut build_commands);
    makefile_targets(repo_root, &mut test_commands, &mut build_commands);
    pyproject_toml(repo_root, &mut test_commands, &mut build_commands);
    xcodeproj_schemes(repo_root, &mut build_commands);

    Discovered {
        test_commands,
        build_commands,
    }
}

/// `true` if `name` looks like a test target/script by the conventions
/// these ecosystems actually use, rather than requiring an exact match —
/// `pretest`, `test:unit` and `check` all show up in real repos.
fn looks_like_test(name: &str) -> bool {
    name.contains("test") || name == "check"
}

fn looks_like_build(name: &str) -> bool {
    name.contains("build") || name == "all" || name == "compile"
}

fn classify(
    name: &str,
    command: DiscoveredCommand,
    test_commands: &mut Vec<DiscoveredCommand>,
    build_commands: &mut Vec<DiscoveredCommand>,
) {
    if looks_like_test(name) {
        test_commands.push(command);
    } else if looks_like_build(name) {
        build_commands.push(command);
    }
}

fn package_json_scripts(
    repo_root: &Path,
    test_commands: &mut Vec<DiscoveredCommand>,
    build_commands: &mut Vec<DiscoveredCommand>,
) {
    let Ok(text) = std::fs::read_to_string(repo_root.join("package.json")) else {
        return;
    };
    let Ok(parsed) = text.parse::<serde_json::Value>() else {
        return;
    };
    let Some(scripts) = parsed.get("scripts").and_then(|s| s.as_object()) else {
        return;
    };
    for name in scripts.keys() {
        classify(
            name,
            DiscoveredCommand {
                kind: "npm-script".to_string(),
                command: format!("npm run {name}"),
                source: "package.json".to_string(),
            },
            test_commands,
            build_commands,
        );
    }
}

/// A line of the form `target: prereqs`, at the start of the line (no
/// leading whitespace, which is what distinguishes a target header from a
/// recipe line) and not one of Make's own special targets.
fn makefile_targets(
    repo_root: &Path,
    test_commands: &mut Vec<DiscoveredCommand>,
    build_commands: &mut Vec<DiscoveredCommand>,
) {
    let Ok(text) = std::fs::read_to_string(repo_root.join("Makefile")) else {
        return;
    };
    for line in text.lines() {
        let Some(colon) = line.find(':') else {
            continue;
        };
        let name = &line[..colon];
        if name.is_empty() || name.starts_with([' ', '\t', '.']) {
            continue;
        }
        classify(
            name,
            DiscoveredCommand {
                kind: "make-target".to_string(),
                command: format!("make {name}"),
                source: "Makefile".to_string(),
            },
            test_commands,
            build_commands,
        );
    }
}

/// `pyproject.toml` gets two treatments: `[tool.pytest.ini_options]`
/// presence is itself the signal that `pytest` is this project's oracle
/// (there is no script name to classify), and `[project.scripts]` /
/// `[tool.poetry.scripts]` entries are classified by name like every other
/// format here.
fn pyproject_toml(
    repo_root: &Path,
    test_commands: &mut Vec<DiscoveredCommand>,
    build_commands: &mut Vec<DiscoveredCommand>,
) {
    let Ok(text) = std::fs::read_to_string(repo_root.join("pyproject.toml")) else {
        return;
    };
    // `toml::Table`, not `toml::Value` — `Value::from_str` parses a single
    // bare value literal (`{ a = 1 }`), not a whole document with `[section]`
    // headers, and a `pyproject.toml` is a document.
    let Ok(parsed) = text.parse::<toml::Table>() else {
        return;
    };

    if parsed.get("tool").and_then(|t| t.get("pytest")).is_some() {
        test_commands.push(DiscoveredCommand {
            kind: "pytest".to_string(),
            command: "pytest".to_string(),
            source: "pyproject.toml".to_string(),
        });
    }

    let script_tables = [
        parsed.get("project").and_then(|p| p.get("scripts")),
        parsed
            .get("tool")
            .and_then(|t| t.get("poetry"))
            .and_then(|p| p.get("scripts")),
    ];
    for scripts in script_tables.into_iter().flatten() {
        let Some(table) = scripts.as_table() else {
            continue;
        };
        for name in table.keys() {
            classify(
                name,
                DiscoveredCommand {
                    kind: "python-script".to_string(),
                    command: name.clone(),
                    source: "pyproject.toml".to_string(),
                },
                test_commands,
                build_commands,
            );
        }
    }
}

/// Xcode schemes aren't test or build by name convention the way the other
/// formats are, so every scheme found is recorded as a build command —
/// `xcodebuild -scheme <name>` is the command that would run it, whichever
/// action it's configured for.
fn xcodeproj_schemes(repo_root: &Path, build_commands: &mut Vec<DiscoveredCommand>) {
    let Ok(entries) = std::fs::read_dir(repo_root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("xcodeproj") {
            continue;
        }
        let schemes_dir = path.join("xcshareddata").join("xcschemes");
        let Ok(schemes) = std::fs::read_dir(&schemes_dir) else {
            continue;
        };
        let project_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        for scheme in schemes.flatten() {
            let scheme_path = scheme.path();
            if scheme_path.extension().and_then(|e| e.to_str()) != Some("xcscheme") {
                continue;
            }
            let Some(scheme_name) = scheme_path.file_stem().and_then(|n| n.to_str()) else {
                continue;
            };
            build_commands.push(DiscoveredCommand {
                kind: "xcode-scheme".to_string(),
                command: format!("xcodebuild -scheme {scheme_name}"),
                source: project_name.to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn package_json_test_script_is_discovered_as_a_test_command() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts": {"test": "jest", "build": "webpack"}}"#,
        )
        .unwrap();
        let discovered = discover(dir.path());
        assert!(discovered
            .test_commands
            .iter()
            .any(|c| c.command == "npm run test"));
        assert!(discovered
            .build_commands
            .iter()
            .any(|c| c.command == "npm run build"));
    }

    #[test]
    fn makefile_test_target_is_discovered() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Makefile"),
            "test:\n\tcargo test\n\nbuild:\n\tcargo build\n",
        )
        .unwrap();
        let discovered = discover(dir.path());
        assert!(discovered
            .test_commands
            .iter()
            .any(|c| c.command == "make test"));
        assert!(discovered
            .build_commands
            .iter()
            .any(|c| c.command == "make build"));
    }

    #[test]
    fn makefile_recipe_lines_are_not_mistaken_for_targets() {
        let dir = tempfile::tempdir().unwrap();
        // The recipe line below starts with a tab and contains a colon
        // (`echo "note: done"`) — it must not be read as a target header.
        fs::write(
            dir.path().join("Makefile"),
            "test:\n\techo \"note: done\"\n",
        )
        .unwrap();
        let discovered = discover(dir.path());
        assert_eq!(discovered.test_commands.len(), 1);
    }

    #[test]
    fn pyproject_pytest_config_is_discovered_as_the_test_oracle() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("pyproject.toml"),
            "[tool.pytest.ini_options]\nminversion = \"6.0\"\n",
        )
        .unwrap();
        let discovered = discover(dir.path());
        assert!(discovered.test_commands.iter().any(|c| c.kind == "pytest"));
    }

    #[test]
    fn xcodeproj_scheme_is_discovered() {
        let dir = tempfile::tempdir().unwrap();
        let schemes_dir = dir
            .path()
            .join("App.xcodeproj")
            .join("xcshareddata")
            .join("xcschemes");
        fs::create_dir_all(&schemes_dir).unwrap();
        fs::write(schemes_dir.join("App.xcscheme"), "<Scheme/>").unwrap();
        let discovered = discover(dir.path());
        assert!(discovered
            .build_commands
            .iter()
            .any(|c| c.command == "xcodebuild -scheme App"));
    }

    #[test]
    fn absent_files_discover_nothing_rather_than_erroring() {
        let dir = tempfile::tempdir().unwrap();
        let discovered = discover(dir.path());
        assert!(discovered.test_commands.is_empty());
        assert!(discovered.build_commands.is_empty());
    }
}

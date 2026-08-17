//! Where things live, relative to this crate — every other module reads a
//! file somewhere in the repository, and this is the one place that says
//! where the repository root is.

use std::path::{Path, PathBuf};

/// `CARGO_MANIFEST_DIR` for this crate is `crates/trestle-egress`, so two
/// ancestors up is the repository root — the same fixed layout
/// `deny.toml` and `scripts/check-workspace.sh` already assume.
pub(super) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/trestle-egress has two ancestors: crates/ and the repo root")
        .to_path_buf()
}

pub(super) fn threat_model_path() -> PathBuf {
    repo_root().join("docs/THREAT-MODEL.md")
}

pub(super) fn cargo_lock_path() -> PathBuf {
    repo_root().join("Cargo.lock")
}

/// The debug binary `cargo build -p trestle-cli` produces. Built on demand by
/// `command_surface::trestle_binary`, not assumed to already exist — the
/// oracle is `cargo test -p trestle-egress`, which does not build
/// `trestle-cli` on its own.
pub(super) fn trestle_binary_path() -> PathBuf {
    repo_root().join("target/debug/trestle")
}

/// Crate directories that hold Trestle's own product code, as opposed to this
/// crate. The grep-based checks scan these and never this crate's own source
/// — otherwise a channel's name appearing in one of *these* doc comments
/// (as it does, repeatedly) would trip its own absence check.
pub(super) fn product_crate_dirs() -> Vec<PathBuf> {
    let crates_dir = repo_root().join("crates");
    let mut dirs = Vec::new();
    let Ok(entries) = std::fs::read_dir(&crates_dir) else {
        return dirs;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.file_name().and_then(|n| n.to_str()) != Some("trestle-egress") {
            dirs.push(path);
        }
    }
    dirs
}

/// Every `.rs` file under `dir`, walked by hand — the tree is small enough
/// that pulling in a directory-walking crate would be indirection with one
/// caller.
pub(super) fn rust_files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|n| n.to_str()) != Some("target") {
                    stack.push(path);
                }
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files
}

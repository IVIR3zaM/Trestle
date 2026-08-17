//! A minimal, portable filesystem sandbox for the "write outside the
//! declared path set" guarantee (CH-11): a directory made non-writable via
//! ordinary POSIX permission bits.
//!
//! That is a real, kernel-enforced denial (`EACCES`, not an
//! application-level check this suite wrote itself), and unlike network
//! denial it needs no OS-specific mechanism — Unix permission bits work
//! identically on macOS and Linux, so there is no per-platform branch here
//! the way there is in `network_denial`.
//!
//! Permission bits only deny a process that does not have root's blanket
//! DAC-override, so `is_available` refuses to claim a denial is real when
//! running as root — the same "fail loudly rather than pass for the wrong
//! reason" rule `network_denial` follows.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) enum Availability {
    Available,
    Unavailable(String),
}

/// Root bypasses Unix permission checks entirely, so a directory with its
/// write bit cleared is not actually protected — checked with `id -u`
/// rather than a Rust crate, to avoid a dependency this one call does not
/// justify.
pub(super) fn is_available() -> Availability {
    match Command::new("id").arg("-u").output() {
        Ok(output) if output.status.success() => {
            let uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if uid == "0" {
                Availability::Unavailable(
                    "running as root: permission bits do not deny root, so this mechanism cannot \
                     enforce a real denial here"
                        .to_string(),
                )
            } else {
                Availability::Available
            }
        }
        Ok(output) => Availability::Unavailable(format!(
            "`id -u` exited with {:?}: cannot confirm this process is unprivileged",
            output.status.code()
        )),
        Err(e) => Availability::Unavailable(format!("could not run `id -u`: {e}")),
    }
}

/// A directory that is readable but not writable by the current process —
/// the "outside the declared path set" side of the fixture.
pub(super) struct DeniedDir {
    pub(super) path: PathBuf,
}

impl DeniedDir {
    pub(super) fn create_under(parent: &Path, name: &str) -> Self {
        let path = parent.join(name);
        fs::create_dir_all(&path).expect("create scratch dir for the denied side of the fixture");
        let mut perms = fs::metadata(&path).expect("stat scratch dir").permissions();
        perms.set_readonly(true); // clears the write bits for all classes on Unix
        fs::set_permissions(&path, perms).expect("remove write permission from scratch dir");
        Self { path }
    }
}

impl Drop for DeniedDir {
    fn drop(&mut self) {
        // `remove_dir_all` needs to delete the directory entry itself, which
        // needs write permission on it — restore owner read+write+execute
        // (not `set_readonly(false)`, which on Unix would leave the
        // directory world-writable) before cleanup, so a dropped fixture
        // does not leak a read-only or over-permissive directory into the
        // system temp dir.
        if let Ok(metadata) = fs::metadata(&self.path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o700);
            let _ = fs::set_permissions(&self.path, perms);
        }
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Attempts to create `name` inside `dir`, returning whether the write was
/// denied and by what `io::ErrorKind`.
pub(super) fn attempt_write(dir: &Path, name: &str) -> std::io::Result<()> {
    fs::write(dir.join(name), b"planted violation")
}

//! Denying network access to a child process, not merely observing whether
//! it happened to use one — an observer can be raced, a denial cannot
//! (`docs/THREAT-MODEL.md`, CH-02/CH-03).
//!
//! The two platform mechanisms are verified below by their own *denial
//! signature*: attempting a raw-IP connect that bypasses DNS, then checking
//! the specific error each mechanism leaves. That distinction matters
//! because a machine with no route to the internet at all (true of the
//! sandbox this suite was developed in) fails a connect attempt identically
//! to a real denial unless the signature is checked — a guard "working"
//! because there was never anything to guard against is not known to work
//! (`AGENTS.md` §4, `scripts/check-workspace.sh`'s own comment on the same
//! trap).

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// What happened when a command was run wrapped by the platform's
/// network-denial mechanism.
pub(super) enum Denial {
    /// The mechanism produced its own specific denial signature.
    Denied,
    /// The command did not show that signature — it may have succeeded, or
    /// failed for a reason unrelated to the mechanism. Either way, this is
    /// not evidence the mechanism denied anything.
    NotDenied(String),
    /// The mechanism itself could not run here.
    Unavailable(String),
}

/// A subprocess that connects to a raw IP literal — never a hostname, so DNS
/// resolution (itself a channel, CH-02) never enters into whether the
/// *connect* was denied — and prints the OS error number it received, if
/// any. `python3` because `DEVELOPING.md` already requires it for
/// `make status`, so it is guaranteed present without a new dependency.
fn connect_probe_script(host: &str, port: u16) -> String {
    format!(
        "import socket\n\
         s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)\n\
         s.settimeout(3)\n\
         try:\n\
         \u{20}\u{20}\u{20}\u{20}s.connect((\"{host}\", {port}))\n\
         \u{20}\u{20}\u{20}\u{20}print(\"CONNECTED\")\n\
         except OSError as e:\n\
         \u{20}\u{20}\u{20}\u{20}print(f\"ERRNO={{e.errno}}\")\n\
         except Exception:\n\
         \u{20}\u{20}\u{20}\u{20}print(\"ERRNO=timeout\")\n"
    )
}

/// Wraps `program`/`args` to run under the platform's network-denial
/// mechanism. macOS: `sandbox-exec` with a profile that denies everything by
/// default and allows no network predicate, so no network rule is ever
/// reached. Linux: a network namespace with only `lo`, brought up via `sudo`
/// rather than the unprivileged `--user` path — Ubuntu 23.10+ restricts
/// unprivileged user-namespace creation by default (AppArmor), which would
/// make the *mechanism itself* unavailable before it ever denied anything.
/// `sudo -n` turns a missing passwordless sudo into an immediate failure
/// instead of a hang on a password prompt.
fn wrap(program: &str, args: &[String]) -> Result<Command, String> {
    if cfg!(target_os = "macos") {
        let profile_path = write_macos_profile()?;
        let mut cmd = Command::new("sandbox-exec");
        cmd.arg("-f").arg(profile_path).arg(program).args(args);
        Ok(cmd)
    } else if cfg!(target_os = "linux") {
        let mut cmd = Command::new("sudo");
        cmd.args(["-n", "unshare", "--net", "--"])
            .arg(program)
            .args(args);
        Ok(cmd)
    } else {
        Err(format!(
            "no network-denial mechanism implemented for target_os = \"{}\"",
            std::env::consts::OS
        ))
    }
}

/// `deny default` with no network-outbound/network-inbound rule anywhere in
/// the profile denies all networking; the rest of the profile only grants
/// what an arbitrary program (a shell, `python3`, `cargo`, `trestle`) needs
/// to run at all. This profile does not restrict the filesystem — that is a
/// separate guarantee (`filesystem_denial`), and conflating the two would
/// make a filesystem failure look like a network failure.
fn write_macos_profile() -> Result<PathBuf, String> {
    let profile = "(version 1)\n\
                    (deny default)\n\
                    (allow process-fork)\n\
                    (allow process-exec*)\n\
                    (allow file-read*)\n\
                    (allow file-write*)\n\
                    (allow file-ioctl)\n\
                    (allow sysctl-read)\n\
                    (allow mach-lookup)\n\
                    (allow mach-priv-task-port)\n\
                    (allow signal)\n\
                    (allow iokit-open)\n\
                    (allow ipc-posix-shm)\n";
    let path = std::env::temp_dir().join(format!(
        "trestle-egress-network-deny-{}.sb",
        std::process::id()
    ));
    let mut file =
        std::fs::File::create(&path).map_err(|e| format!("writing sandbox profile: {e}"))?;
    file.write_all(profile.as_bytes())
        .map_err(|e| format!("writing sandbox profile: {e}"))?;
    Ok(path)
}

/// Runs `program`/`args` under the network-denial mechanism and classifies
/// what happened, by wrapping a raw-IP connect probe as the payload of an
/// arbitrary command run — used both by the planted-violation self-test and
/// by every real command-surface test, so both exercise the identical
/// wrapping code path.
pub(super) fn probe() -> Denial {
    // 203.0.113.0/24 is TEST-NET-3 (RFC 5737): reserved for documentation, so
    // it is never actually routable. That is deliberate — a real destination
    // would make the test's result depend on whether the network happens to
    // be reachable from wherever it runs, which is exactly the ambient
    // failure this probe exists to distinguish from a real denial.
    run_probe_via(wrap)
}

fn run_probe_via(wrap_fn: impl Fn(&str, &[String]) -> Result<Command, String>) -> Denial {
    let script = connect_probe_script("203.0.113.1", 80);
    let args = vec!["-c".to_string(), script];
    let mut cmd = match wrap_fn("python3", &args) {
        Ok(cmd) => cmd,
        Err(reason) => return Denial::Unavailable(reason),
    };
    classify(cmd.output())
}

fn classify(output: std::io::Result<std::process::Output>) -> Denial {
    let output = match output {
        Ok(output) => output,
        Err(e) => return Denial::Unavailable(format!("could not spawn the wrapping command: {e}")),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() && stdout.trim().is_empty() {
        // The wrapper itself failed to run the probe at all (e.g. `sudo -n`
        // found no passwordless sudo, or `sandbox-exec` rejected the
        // profile) — that is the mechanism being unavailable, not a denial.
        return Denial::Unavailable(format!(
            "the wrapping command exited with {:?} before the probe could run: {stderr}",
            output.status.code()
        ));
    }
    if cfg!(target_os = "macos") && stdout.contains("ERRNO=1") {
        // EPERM: sandbox-exec's own signature for a syscall its profile
        // vetoed outright.
        return Denial::Denied;
    }
    if cfg!(target_os = "linux") && (stdout.contains("ERRNO=101") || stdout.contains("ERRNO=113")) {
        // ENETUNREACH (101) or EHOSTUNREACH (113): what a connect attempt
        // gets in a network namespace with no interface but a downed `lo`
        // and therefore no route at all.
        return Denial::Denied;
    }
    Denial::NotDenied(format!(
        "stdout={stdout:?} stderr={stderr:?} status={:?}",
        output.status
    ))
}

/// Runs `program`/`args` under the network-denial mechanism, for use by
/// tests that need a real command to succeed with the network denied rather
/// than assert on the denial signature itself. `dir`, if given, becomes the
/// child's working directory (needed for `cargo build`, which must run from
/// the workspace root).
pub(super) fn run_denied(
    program: &str,
    args: &[String],
    dir: Option<&std::path::Path>,
) -> Result<std::process::Output, String> {
    let mut cmd = wrap(program, args)?;
    if let Some(dir) = dir {
        cmd.current_dir(dir);
    }
    cmd.output()
        .map_err(|e| format!("spawning {program} under network denial: {e}"))
}

/// Runs the identical probe with no network-denial wrapper at all, so a
/// test can show the wrapped and unwrapped runs behave differently — proof
/// the wrapper is the cause, not an accident of the machine's own
/// connectivity (the same second half `scripts/check-workspace.sh` insists
/// on: a guard that fails is not evidence unless it failed for the stated
/// reason).
pub(super) fn probe_unwrapped() -> Denial {
    run_probe_via(|program, args| {
        Ok({
            let mut cmd = Command::new(program);
            cmd.args(args);
            cmd
        })
    })
}

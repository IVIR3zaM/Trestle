//! Enumerating a process's own sockets from the outside — via the OS's
//! process/socket tables, never by reading the process's source — because
//! CH-08/CH-09 require the listener claim to be checked the same way an
//! attacker would check it, not the way the code hopes it behaves.

use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Listener {
    pub(super) local_address: String,
}

/// Every socket the process at `pid` currently holds in `LISTEN` state.
/// macOS: `lsof`, which ships with the OS. Linux: `/proc/<pid>/net/tcp{,6}`
/// cross-referenced against that pid's open file descriptors, so no extra
/// tool (`lsof`, `ss`) needs to be present on the CI image.
pub(super) fn listeners_of(pid: u32) -> Vec<Listener> {
    if cfg!(target_os = "macos") {
        listeners_via_lsof(pid)
    } else if cfg!(target_os = "linux") {
        listeners_via_proc(pid)
    } else {
        Vec::new()
    }
}

fn listeners_via_lsof(pid: u32) -> Vec<Listener> {
    let Ok(output) = Command::new("lsof")
        .args(["-a", "-p", &pid.to_string(), "-n", "-P", "-i"])
        .output()
    else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .filter(|line| line.contains("(LISTEN)"))
        .filter_map(|line| {
            // Columns are whitespace-separated; the address:port is the one
            // immediately before the literal "(LISTEN)".
            let fields: Vec<&str> = line.split_whitespace().collect();
            let listen_pos = fields.iter().position(|f| *f == "(LISTEN)")?;
            fields.get(listen_pos.checked_sub(1)?).map(|addr| Listener {
                local_address: (*addr).to_string(),
            })
        })
        .collect()
}

fn listeners_via_proc(pid: u32) -> Vec<Listener> {
    let mut inodes = std::collections::HashSet::new();
    if let Ok(fds) = std::fs::read_dir(format!("/proc/{pid}/fd")) {
        for fd in fds.flatten() {
            if let Ok(target) = std::fs::read_link(fd.path()) {
                if let Some(name) = target.to_str() {
                    if let Some(inode) = name
                        .strip_prefix("socket:[")
                        .and_then(|s| s.strip_suffix(']'))
                    {
                        inodes.insert(inode.to_string());
                    }
                }
            }
        }
    }
    let mut listeners = Vec::new();
    for table in ["/proc/net/tcp", "/proc/net/tcp6"] {
        let Ok(text) = std::fs::read_to_string(table) else {
            continue;
        };
        for line in text.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            // local_address, st, ... inode are columns 1, 3, 9 (0-indexed).
            let (Some(local), Some(state), Some(inode)) =
                (fields.first(), fields.get(3), fields.get(9))
            else {
                continue;
            };
            if *state == "0A" && inodes.contains(*inode) {
                listeners.push(Listener {
                    local_address: (*local).to_string(),
                });
            }
        }
    }
    listeners
}

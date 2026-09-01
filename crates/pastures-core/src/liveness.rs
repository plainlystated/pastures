//! Which agent processes are alive right now, from Claude Code's own per-process records.
//!
//! Claude Code (2.1.2xx) writes `~/.claude/sessions/<pid>.json` for every running process and
//! removes it on exit. Older processes may only have a `<pid>.<hash>.key` file, which carries no
//! session id; those are reported as alive with `Liveness::Unknown` rather than hidden.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::record::Liveness;

#[derive(Debug, Clone)]
pub struct LiveProcess {
    pub pid: u32,
    pub session_id: Option<String>,
    /// The launch directory as recorded by the agent. May be stale after a cross-directory resume.
    pub cwd: Option<PathBuf>,
    pub liveness: Liveness,
    pub name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionFile {
    pid: u32,
    session_id: Option<String>,
    cwd: Option<PathBuf>,
    proc_start: Option<String>,
    status: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeyFile {
    proc_start: Option<String>,
}

/// Every live agent process under `claude_home/sessions`.
pub fn live_processes(claude_home: &Path) -> Vec<LiveProcess> {
    let dir = claude_home.join("sessions");
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut with_record: Vec<LiveProcess> = Vec::new();
    let mut key_only: Vec<(u32, Option<String>)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if let Some(stem) = name.strip_suffix(".json") {
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(rec) = serde_json::from_str::<SessionFile>(&text) else {
                continue;
            };
            let pid = stem.parse().unwrap_or(rec.pid);
            if !process_alive(pid, rec.proc_start.as_deref()) {
                continue;
            }
            with_record.push(LiveProcess {
                pid,
                session_id: rec.session_id,
                cwd: rec.cwd,
                liveness: match rec.status.as_deref() {
                    Some("busy") => Liveness::Busy,
                    Some("idle") => Liveness::Idle,
                    Some("shell") => Liveness::Shell,
                    _ => Liveness::Unknown,
                },
                name: rec.name,
            });
        } else if name.ends_with(".key") {
            let Some(pid) = name.split('.').next().and_then(|p| p.parse::<u32>().ok()) else {
                continue;
            };
            let proc_start = fs::read_to_string(&path)
                .ok()
                .and_then(|t| serde_json::from_str::<KeyFile>(&t).ok())
                .and_then(|k| k.proc_start);
            key_only.push((pid, proc_start));
        }
    }

    for (pid, proc_start) in key_only {
        if with_record.iter().any(|p| p.pid == pid) {
            continue;
        }
        if !process_alive(pid, proc_start.as_deref()) {
            continue;
        }
        with_record.push(LiveProcess {
            pid,
            session_id: None,
            cwd: process_cwd(pid),
            liveness: Liveness::Unknown,
            name: None,
        });
    }

    with_record
}

/// True if `pid` is running and, when a start time is known, is the same process that wrote the
/// record (guards against pid reuse). `proc_start` is `/proc/<pid>/stat` field 22 verbatim.
fn process_alive(pid: u32, proc_start: Option<&str>) -> bool {
    #[cfg(target_os = "linux")]
    {
        let Ok(stat) = fs::read_to_string(format!("/proc/{pid}/stat")) else {
            return false;
        };
        let Some(expected) = proc_start else {
            return true;
        };
        // The comm field is parenthesised and may contain spaces; fields are counted after it.
        let Some(after_comm) = stat.rsplit_once(')').map(|(_, rest)| rest) else {
            return false;
        };
        // after_comm starts at field 3 (state); starttime is field 22.
        after_comm.split_whitespace().nth(19) == Some(expected)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = proc_start;
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

fn process_cwd(pid: u32) -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        fs::read_link(format!("/proc/{pid}/cwd")).ok()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_process_is_alive_with_matching_start() {
        let pid = std::process::id();
        let stat = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
        let start = stat
            .rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .nth(19)
            .unwrap()
            .to_string();
        assert!(process_alive(pid, Some(&start)));
        assert!(!process_alive(pid, Some("1")));
    }

    #[test]
    fn missing_dir_yields_nothing() {
        assert!(live_processes(Path::new("/nonexistent/pastures")).is_empty());
    }
}

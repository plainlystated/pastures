//! Reads local AI coding-agent sessions and ranks them by the person's own engagement.
//!
//! The core knows nothing about terminals or multiplexers. It answers one question — which of my
//! sessions are alive, and how warm is each — and hands back neutral [`SessionRecord`]s for a CLI
//! or an adapter to label and focus.

pub mod config;
pub mod liveness;
pub mod record;
pub mod transcript;
pub mod turns;
pub mod warmth;

use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;

pub use config::Config;
pub use record::{Labels, Liveness, SessionRecord};

/// Every live session, warmest first.
pub fn scan(config: &Config) -> Result<Vec<SessionRecord>> {
    let claude_home = config.claude_home()?;
    let now = Utc::now();
    let mut records = merge_by_session(
        liveness::live_processes(&claude_home)
            .into_iter()
            .map(|p| build_record(&claude_home, p, config, now)),
    );
    records.sort_by(|a, b| {
        b.warmth
            .partial_cmp(&a.warmth)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.last_turn_at.cmp(&a.last_turn_at))
            .then_with(|| a.label.cmp(&b.label))
    });
    Ok(records)
}

fn build_record(
    claude_home: &Path,
    proc_: liveness::LiveProcess,
    config: &Config,
    now: chrono::DateTime<Utc>,
) -> SessionRecord {
    let transcript_path = proc_
        .session_id
        .as_deref()
        .and_then(|id| transcript::find_transcript(claude_home, id));
    let summary = transcript_path
        .as_deref()
        .and_then(|p| transcript::summarize(p).ok())
        .unwrap_or_default();

    let cwd: PathBuf = summary
        .cwd
        .clone()
        .or(proc_.cwd.clone())
        .unwrap_or_else(|| PathBuf::from("?"));
    let dir = cwd
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| cwd.display().to_string());

    let staleness_hours = summary
        .last_turn_at
        .map(|t| (now - t).num_milliseconds() as f64 / 3_600_000.0);

    let labels = Labels {
        custom_title: summary.custom_title,
        ai_title: summary.ai_title,
        first_turn: summary.first_turn_text.map(|t| first_line(&t, 80)),
        dir,
    };
    let label = labels
        .custom_title
        .clone()
        .or_else(|| labels.ai_title.clone())
        .or_else(|| labels.first_turn.clone())
        .unwrap_or_else(|| labels.dir.clone());

    SessionRecord {
        session_id: proc_.session_id,
        pids: vec![proc_.pid],
        liveness: proc_.liveness,
        cwd,
        git_branch: summary.git_branch,
        turns: summary.turns,
        first_turn_at: summary.first_turn_at,
        last_turn_at: summary.last_turn_at,
        staleness_hours,
        warmth: warmth::warmth(summary.turns, staleness_hours, &config.warmth),
        label,
        labels,
        transcript: transcript_path,
        agent_name: proc_.name,
    }
}

/// One row per session. A conversation resumed in several processes keeps every pid and the
/// liveliest status; processes with no session id stay as their own rows.
fn merge_by_session(records: impl Iterator<Item = SessionRecord>) -> Vec<SessionRecord> {
    let mut out: Vec<SessionRecord> = Vec::new();
    for r in records {
        let existing = r.session_id.as_ref().and_then(|id| {
            out.iter_mut()
                .find(|o| o.session_id.as_deref() == Some(id.as_str()))
        });
        match existing {
            Some(o) => {
                o.pids.extend(r.pids);
                if liveness_rank(r.liveness) > liveness_rank(o.liveness) {
                    o.liveness = r.liveness;
                    o.agent_name = r.agent_name;
                }
            }
            None => out.push(r),
        }
    }
    out
}

fn liveness_rank(l: Liveness) -> u8 {
    match l {
        Liveness::Busy => 3,
        Liveness::Shell => 2,
        Liveness::Idle => 1,
        Liveness::Unknown => 0,
    }
}

fn first_line(text: &str, max_chars: usize) -> String {
    let line = text.lines().next().unwrap_or("").trim();
    if line.chars().count() <= max_chars {
        return line.to_string();
    }
    let cut: String = line.chars().take(max_chars - 1).collect();
    format!("{}…", cut.trim_end())
}

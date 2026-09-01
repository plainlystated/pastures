use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::Serialize;

/// What the agent is doing right now. Never a sort key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Liveness {
    /// Mid-turn: working on the person's last request.
    Busy,
    /// Waiting for the person.
    Idle,
    /// The person has dropped to a shell inside the session.
    Shell,
    /// The process is alive but published no status (older agent versions).
    Unknown,
}

/// The label candidates, in precedence order. `SessionRecord::label` is the first present.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Labels {
    /// Set by the person with `/rename`.
    pub custom_title: Option<String>,
    /// The agent's own stored title for the session.
    pub ai_title: Option<String>,
    /// The person's first typed message, truncated.
    pub first_turn: Option<String>,
    /// Basename of the working directory.
    pub dir: String,
}

/// One live session, with everything an adapter or renderer needs.
#[derive(Debug, Clone, Serialize)]
pub struct SessionRecord {
    /// The agent's session id. Absent for processes that published no record.
    pub session_id: Option<String>,
    /// Every live process attached to this session (a conversation can be resumed in several).
    pub pids: Vec<u32>,
    pub liveness: Liveness,
    /// The session's current working directory (from its latest transcript entry, not its launch dir).
    pub cwd: PathBuf,
    pub git_branch: Option<String>,
    /// Investment: messages the person typed.
    pub turns: u32,
    pub first_turn_at: Option<DateTime<Utc>>,
    pub last_turn_at: Option<DateTime<Utc>>,
    /// Staleness: hours since the person's last turn.
    pub staleness_hours: Option<f64>,
    /// The ranking score. Higher is warmer.
    pub warmth: f64,
    pub label: String,
    pub labels: Labels,
    pub transcript: Option<PathBuf>,
    /// The agent's own short name for the process (Claude Code's `name`).
    pub agent_name: Option<String>,
}

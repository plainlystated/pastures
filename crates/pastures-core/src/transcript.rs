//! Reads one Claude Code transcript and reduces it to the few facts pastures needs.
//!
//! Transcripts are `~/.claude/projects/<slug>/<session-id>.jsonl`. The slug is a lossy encoding of
//! the *launch* directory and is never used as data: the working directory comes from the entries,
//! which record it per line and can change after a cross-directory resume.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::turns::human_turn_text;

#[derive(Debug, Default, Clone)]
pub struct TranscriptSummary {
    pub turns: u32,
    pub first_turn_at: Option<DateTime<Utc>>,
    pub last_turn_at: Option<DateTime<Utc>>,
    pub first_turn_text: Option<String>,
    /// From the latest entry that carried one.
    pub cwd: Option<PathBuf>,
    pub git_branch: Option<String>,
    pub custom_title: Option<String>,
    pub ai_title: Option<String>,
}

/// Locates `<session_id>.jsonl` under any project slug.
pub fn find_transcript(claude_home: &Path, session_id: &str) -> Option<PathBuf> {
    let projects = claude_home.join("projects");
    let file = format!("{session_id}.jsonl");
    for entry in std::fs::read_dir(projects).ok()?.flatten() {
        let candidate = entry.path().join(&file);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Single pass over the file. Malformed lines are skipped; unknown entry types are ignored.
pub fn summarize(path: &Path) -> std::io::Result<TranscriptSummary> {
    let reader = BufReader::new(File::open(path)?);
    let mut s = TranscriptSummary::default();

    for line in reader.split(b'\n') {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let Ok(entry) = serde_json::from_slice::<Value>(&line) else {
            continue;
        };
        if let Some(cwd) = entry.get("cwd").and_then(Value::as_str) {
            s.cwd = Some(PathBuf::from(cwd));
        }
        if let Some(branch) = entry.get("gitBranch").and_then(Value::as_str) {
            if !branch.is_empty() {
                s.git_branch = Some(branch.to_string());
            }
        }
        match entry.get("type").and_then(Value::as_str) {
            Some("custom-title") => {
                s.custom_title = entry
                    .get("customTitle")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
            Some("ai-title") => {
                s.ai_title = entry
                    .get("aiTitle")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
            Some("user") => {
                if let Some(text) = human_turn_text(&entry) {
                    s.turns += 1;
                    let at = entry
                        .get("timestamp")
                        .and_then(Value::as_str)
                        .and_then(|t| t.parse::<DateTime<Utc>>().ok());
                    if s.first_turn_at.is_none() {
                        s.first_turn_at = at;
                        s.first_turn_text = Some(text);
                    }
                    if at.is_some() {
                        s.last_turn_at = at;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn summarises_a_small_transcript() {
        let dir = std::env::temp_dir().join(format!("pastures-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.jsonl");
        let mut f = File::create(&path).unwrap();
        writeln!(f, r#"{{"type":"user","cwd":"/a","gitBranch":"main","timestamp":"2026-09-01T10:00:00.000Z","message":{{"role":"user","content":"first question"}}}}"#).unwrap();
        writeln!(f, r#"{{"type":"assistant","cwd":"/a","timestamp":"2026-09-01T10:00:05.000Z","message":{{"role":"assistant","content":[]}}}}"#).unwrap();
        writeln!(f, r#"{{"type":"user","cwd":"/a","timestamp":"2026-09-01T10:00:06.000Z","toolUseResult":{{}},"message":{{"role":"user","content":[{{"type":"tool_result","content":"x"}}]}}}}"#).unwrap();
        writeln!(f, "not json").unwrap();
        writeln!(f, r#"{{"type":"ai-title","aiTitle":"A title","sessionId":"t"}}"#).unwrap();
        writeln!(f, r#"{{"type":"user","cwd":"/b","gitBranch":"feature","timestamp":"2026-09-01T11:00:00.000Z","message":{{"role":"user","content":"second"}}}}"#).unwrap();
        writeln!(f, r#"{{"type":"last-prompt","lastPrompt":"second","sessionId":"t"}}"#).unwrap();

        let s = summarize(&path).unwrap();
        assert_eq!(s.turns, 2);
        assert_eq!(s.first_turn_text.as_deref(), Some("first question"));
        assert_eq!(s.cwd.as_deref(), Some(Path::new("/b")));
        assert_eq!(s.git_branch.as_deref(), Some("feature"));
        assert_eq!(s.ai_title.as_deref(), Some("A title"));
        assert_eq!(
            s.last_turn_at.unwrap().to_rfc3339(),
            "2026-09-01T11:00:00+00:00"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

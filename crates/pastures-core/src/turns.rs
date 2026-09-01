//! The rule for "did the person type this?"
//!
//! A Claude Code transcript records tool results, injected context, and local slash-command echoes
//! as `user` entries alongside the person's real messages. No single field distinguishes them
//! across versions (`userType` is always `external`; `promptSource`/`origin` only exist in newer
//! ones), so this is a structural rule, validated against `~/.claude/history.jsonl`.

use serde_json::Value;

/// Text prefixes that mark a `user` entry the person did not type.
const INJECTED_PREFIXES: &[&str] = &[
    "<local-command-stdout>",
    "<local-command-caveat>",
    "<system-reminder>",
    "<task-notification>",
    "<persisted-output>",
    "[Request interrupted",
];

/// Returns the typed text if `entry` is a turn the person typed in the parent session.
///
/// Slash commands that reach the model (skills) count as turns and are rendered as
/// `/name args`. Local builtins (`/model`, `/clear`) never reach the model and don't count.
pub fn human_turn_text(entry: &Value) -> Option<String> {
    if entry.get("type").and_then(Value::as_str) != Some("user") {
        return None;
    }
    if entry.get("isSidechain").and_then(Value::as_bool) == Some(true)
        || entry.get("isMeta").and_then(Value::as_bool) == Some(true)
        || entry.get("toolUseResult").is_some()
    {
        return None;
    }
    let content = entry.get("message")?.get("content")?;
    let text = match content {
        Value::String(s) => s.clone(),
        Value::Array(blocks) => {
            if blocks
                .iter()
                .any(|b| b.get("type").and_then(Value::as_str) == Some("tool_result"))
            {
                return None;
            }
            blocks
                .iter()
                .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|b| b.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => return None,
    };
    let trimmed = text.trim_start();
    if trimmed.is_empty() || INJECTED_PREFIXES.iter().any(|p| trimmed.starts_with(p)) {
        return None;
    }
    if trimmed.starts_with("<command-name>") {
        // A local builtin echo; the model never saw it.
        return None;
    }
    if let Some(name) = tag_body(trimmed, "command-name") {
        let args = tag_body(trimmed, "command-args").unwrap_or("");
        return Some(format!("{name} {args}").trim().to_string());
    }
    Some(trimmed.to_string())
}

fn tag_body<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = text.find(&open)? + open.len();
    let end = text[start..].find(&close)? + start;
    Some(text[start..end].trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn user(content: Value) -> Value {
        json!({"type": "user", "isSidechain": false, "message": {"role": "user", "content": content}})
    }

    #[test]
    fn plain_string_is_a_turn() {
        assert_eq!(
            human_turn_text(&user(json!("fix the tests"))).as_deref(),
            Some("fix the tests")
        );
    }

    #[test]
    fn tool_results_are_not_turns() {
        let mut e = user(json!([{"type": "tool_result", "tool_use_id": "x", "content": "ok"}]));
        assert!(human_turn_text(&e).is_none());
        e["toolUseResult"] = json!({"stdout": ""});
        assert!(human_turn_text(&e).is_none());
    }

    #[test]
    fn injected_context_is_not_a_turn() {
        for prefix in INJECTED_PREFIXES {
            assert!(human_turn_text(&user(json!(format!("{prefix}stuff")))).is_none());
        }
        let mut e = user(json!([{"type": "text", "text": "skill body"}]));
        e["isMeta"] = json!(true);
        assert!(human_turn_text(&e).is_none());
    }

    #[test]
    fn sidechain_and_assistant_are_not_turns() {
        let mut e = user(json!("hi"));
        e["isSidechain"] = json!(true);
        assert!(human_turn_text(&e).is_none());
        let a = json!({"type": "assistant", "message": {"content": "hi"}});
        assert!(human_turn_text(&a).is_none());
    }

    #[test]
    fn skill_invocation_counts_and_is_rendered() {
        let e = user(json!(
            "<command-message>grill</command-message>\n<command-name>/grill</command-name>\n<command-args>the plan</command-args>"
        ));
        assert_eq!(human_turn_text(&e).as_deref(), Some("/grill the plan"));
    }

    #[test]
    fn local_builtin_does_not_count() {
        let e = user(json!("<command-name>/model</command-name>\n<command-message>model</command-message>"));
        assert!(human_turn_text(&e).is_none());
    }

    #[test]
    fn text_blocks_are_joined() {
        let e = user(json!([{"type": "text", "text": "a"}, {"type": "image"}, {"type": "text", "text": "b"}]));
        assert_eq!(human_turn_text(&e).as_deref(), Some("a\nb"));
    }
}

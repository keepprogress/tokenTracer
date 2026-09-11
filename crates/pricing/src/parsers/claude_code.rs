//! Claude Code JSONL parser (`EC-claude-code-v1`).
//!
//! - Only `type == "assistant"` lines with `message.usage`
//! - Dedup by `message.id` (streaming may repeat usage)
//! - Map cache_read / cache_write from Anthropic usage fields

use crate::models::{AgentId, UsageEvent};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

pub const SOURCE_TAG: &str = "EC-claude-code-v1";

#[derive(Debug, Error)]
pub enum ClaudeParseError {
    #[error("JSONL line {line}: {message}")]
    Line { line: usize, message: String },
}

/// Parse Claude Code session JSONL text into deduplicated UsageEvents.
pub fn parse_claude_code_jsonl(text: &str) -> Result<Vec<UsageEvent>, ClaudeParseError> {
    parse_claude_code_jsonl_with_source(text, SOURCE_TAG)
}

pub fn parse_claude_code_jsonl_with_source(
    text: &str,
    source: &str,
) -> Result<Vec<UsageEvent>, ClaudeParseError> {
    // Keep insertion order of first-seen message.id; later duplicates overwrite
    // with the latest usage (streaming ends with final usage).
    let mut by_id: HashMap<String, (usize, UsageEvent)> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for (idx, raw) in text.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(trimmed).map_err(|e| ClaudeParseError::Line {
            line: line_no,
            message: e.to_string(),
        })?;

        let ty = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if ty != "assistant" {
            continue;
        }
        let message = match v.get("message") {
            Some(m) => m,
            None => continue,
        };
        let usage = match message.get("usage") {
            Some(u) if u.is_object() => u,
            _ => continue,
        };

        let id = message
            .get("id")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                v.get("uuid")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| format!("claude_line_{line_no}"));

        let ts = v
            .get("timestamp")
            .and_then(|x| x.as_str())
            .unwrap_or("1970-01-01T00:00:00.000Z")
            .to_string();

        let model = message
            .get("model")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());

        let input_tokens = usage
            .get("input_tokens")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let output_tokens = usage
            .get("output_tokens")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let cache_read = usage
            .get("cache_read_input_tokens")
            .and_then(|x| x.as_u64());
        let cache_write = usage
            .get("cache_creation_input_tokens")
            .and_then(|x| x.as_u64())
            .or_else(|| {
                // Fallback: sum ephemeral 5m+1h if top-level missing.
                let cc = usage.get("cache_creation")?;
                let a = cc
                    .get("ephemeral_5m_input_tokens")
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0);
                let b = cc
                    .get("ephemeral_1h_input_tokens")
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0);
                Some(a + b)
            });

        let meta = serde_json::json!({
            "sessionId": v.get("sessionId").or_else(|| v.get("session_id")),
            "requestId": v.get("requestId"),
            "uuid": v.get("uuid"),
            "cwd": v.get("cwd"),
            "evidence": SOURCE_TAG,
        });

        let event = UsageEvent {
            id: id.clone(),
            agent: AgentId::ClaudeCode,
            source: source.to_string(),
            ts,
            model,
            usage_pool: None,
            input_tokens,
            output_tokens,
            cache_read_tokens: cache_read,
            cache_write_tokens: cache_write,
            raw_cost_usd: None,
            meta: Some(meta),
        };

        if let Some((ord, slot)) = by_id.get_mut(&id) {
            let _ = ord;
            *slot = event;
        } else {
            order.push(id.clone());
            by_id.insert(id, (order.len() - 1, event));
        }
    }

    let mut out = Vec::with_capacity(order.len());
    for id in order {
        if let Some((_, ev)) = by_id.remove(&id) {
            out.push(ev);
        }
    }
    Ok(out)
}

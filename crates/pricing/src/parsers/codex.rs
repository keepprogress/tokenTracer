//! Codex rollout JSONL parser (`EC-codex-v1`).
//!
//! - Only `type == "token_usage_record"` (do NOT count `event_msg` / `token_count`)
//! - Prefer `payload.response_id` as id
//! - input_tokens = usage.input_tokens − cached_input_tokens (CodexScope)
//! - Model from latest preceding `turn_context.payload.model`

use crate::models::{AgentId, UsageEvent};
use serde_json::Value;
use thiserror::Error;

pub const SOURCE_TAG: &str = "EC-codex-v1";

#[derive(Debug, Error)]
pub enum CodexParseError {
    #[error("JSONL line {line}: {message}")]
    Line { line: usize, message: String },
}

/// Parse a Codex rollout JSONL text into UsageEvents.
pub fn parse_codex_rollout_jsonl(text: &str) -> Result<Vec<UsageEvent>, CodexParseError> {
    parse_codex_rollout_jsonl_with_source(text, SOURCE_TAG)
}

pub fn parse_codex_rollout_jsonl_with_source(
    text: &str,
    source: &str,
) -> Result<Vec<UsageEvent>, CodexParseError> {
    let mut events = Vec::new();
    let mut latest_model: Option<String> = None;

    for (idx, raw) in text.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(trimmed).map_err(|e| CodexParseError::Line {
            line: line_no,
            message: e.to_string(),
        })?;

        let ty = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

        // Track model from turn_context; never emit usage from it.
        if ty == "turn_context" {
            if let Some(model) = v
                .pointer("/payload/model")
                .and_then(|m| m.as_str())
            {
                latest_model = Some(model.to_string());
            }
            continue;
        }

        // Explicitly ignore cumulative snapshots to avoid double-count.
        if ty == "event_msg" {
            continue;
        }

        if ty != "token_usage_record" {
            continue;
        }

        let payload = match v.get("payload") {
            Some(p) => p,
            None => continue,
        };

        // Prefer turn_token_usage when present; else usage.
        let usage = payload
            .get("turn_token_usage")
            .filter(|u| u.is_object())
            .or_else(|| payload.get("usage"))
            .ok_or_else(|| CodexParseError::Line {
                line: line_no,
                message: "token_usage_record missing usage".into(),
            })?;

        let raw_input = usage
            .get("input_tokens")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let cached = usage
            .get("cached_input_tokens")
            .or_else(|| usage.get("cache_read_input_tokens"))
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let cache_write = usage
            .get("cache_write_input_tokens")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let output = usage
            .get("output_tokens")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let reasoning = usage
            .get("reasoning_output_tokens")
            .and_then(|x| x.as_u64())
            .unwrap_or(0);

        let non_cached_input = raw_input.saturating_sub(cached);
        let output_tokens = output.saturating_add(reasoning);

        let id = payload
            .get("response_id")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                let session = payload
                    .get("session_id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("unknown");
                let turn = payload
                    .get("turn_id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("unknown");
                let ordinal = v
                    .get("ordinal")
                    .and_then(|x| x.as_u64())
                    .unwrap_or(line_no as u64);
                format!("{session}:{turn}:{ordinal}")
            });

        let ts = v
            .get("timestamp")
            .and_then(|x| x.as_str())
            .unwrap_or("1970-01-01T00:00:00.000Z")
            .to_string();

        let meta = serde_json::json!({
            "thread_id": payload.get("thread_id"),
            "turn_id": payload.get("turn_id"),
            "session_id": payload.get("session_id"),
            "raw_input_tokens": raw_input,
            "reasoning_output_tokens": reasoning,
            "evidence": SOURCE_TAG,
        });

        events.push(UsageEvent {
            id,
            agent: AgentId::Codex,
            source: source.to_string(),
            ts,
            model: latest_model.clone(),
            usage_pool: None,
            input_tokens: non_cached_input,
            output_tokens,
            cache_read_tokens: Some(cached),
            cache_write_tokens: Some(cache_write),
            raw_cost_usd: None,
            meta: Some(meta),
        });
    }

    Ok(events)
}

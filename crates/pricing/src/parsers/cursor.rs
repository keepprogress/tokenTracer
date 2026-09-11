//! Cursor local parser — **PARTIAL stub** (`EC-cursor-v1` + AC v1.3a).
//!
//! # Hard constraint (AC v1.1a / EC-cursor-v1)
//! Local bubble `tokenCount` is **unreliable** (often 0) and must **never** be
//! treated as billed usage. This stub therefore:
//! - Does **not** invent spend from local SQLite / bubbles
//! - Returns `Err` for opaque local dumps, OR accepts an explicit
//!   vendor-reported aggregate JSON fixture (Admin API / CSV-shaped)
//!
//! When pricing includes cursor events from this path, `by_agent.status`
//! must be `partial`.
//!
//! # AC v1.3a
//! - Preserve concrete model ids (`modelIntent` / etc.); never emit `__other__`
//! - Missing model → leave `null` (pricing maps to `cursor:unknown` in by_model)
//! - Derive `usage_pool` from explicit field or `meta.tier` / `meta.cursor_tier`
//!   (PARTIAL: tier 1→other_models, tier 2→cursor_models)
//! - Never treat literal `"Other"` / `"Other model"` as model or pool authority

use crate::models::{AgentId, UsageEvent, UsagePool, FORBIDDEN_OTHER_MODEL_LITERALS};
use thiserror::Error;

pub const SOURCE_TAG: &str = "EC-cursor-v1";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CursorParseError {
    #[error(
        "cursor local bubble tokenCount is unreliable (EC-cursor-v1); \
         refusing to treat as billed usage. Use vendor-reported aggregates \
         (Admin API / dashboard CSV) or mark status=partial with empty events."
    )]
    BubbleTokensForbidden,
    #[error("cursor stub: {0}")]
    Message(String),
}

fn normalize_cursor_event(mut ev: UsageEvent) -> UsageEvent {
    // Strip forbidden Other literals from model (not a model id / not pool authority).
    if let Some(ref m) = ev.model {
        let t = m.trim();
        if t.is_empty()
            || FORBIDDEN_OTHER_MODEL_LITERALS
                .iter()
                .any(|lit| t.eq_ignore_ascii_case(lit))
        {
            ev.model = None;
        } else {
            ev.model = Some(t.to_string());
        }
    }

    // Derive usage_pool if missing: tier → PARTIAL map; else unknown.
    if ev.usage_pool.is_none() {
        if let Some(tier) = ev
            .meta
            .as_ref()
            .and_then(|m| m.get("cursor_tier").or_else(|| m.get("tier")))
            .and_then(|v| v.as_i64())
        {
            ev.usage_pool = Some(UsagePool::from_cursor_tier(tier));
        } else {
            ev.usage_pool = Some(UsagePool::Unknown);
        }
    }
    ev
}

/// Parse Cursor local data.
///
/// - Empty input → empty Ok (no invented events)
/// - JSON array of UsageEvent with `agent=cursor` and `meta.cost_nature=vendor_reported`
///   → accepted as vendor-reported aggregates (model ids preserved; pool derived)
/// - Anything that looks like bubble `tokenCount` billing → Err
pub fn parse_cursor_local(text: &str) -> Result<Vec<UsageEvent>, CursorParseError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    // Reject obvious bubble payloads that try to bill from tokenCount.
    if trimmed.contains("\"tokenCount\"") && !trimmed.contains("vendor_reported") {
        return Err(CursorParseError::BubbleTokensForbidden);
    }

    // Accept explicit vendor-reported UsageEvent JSON array.
    if let Ok(events) = serde_json::from_str::<Vec<UsageEvent>>(trimmed) {
        let mut out = Vec::with_capacity(events.len());
        for ev in events {
            if ev.agent != AgentId::Cursor {
                return Err(CursorParseError::Message(
                    "vendor aggregate events must have agent=cursor".into(),
                ));
            }
            let nature = ev
                .meta
                .as_ref()
                .and_then(|m| m.get("cost_nature"))
                .and_then(|v| v.as_str());
            if nature != Some("vendor_reported") {
                return Err(CursorParseError::Message(
                    "cursor fixtures must set meta.cost_nature=vendor_reported \
                     (local bubbles are not billable)"
                        .into(),
                ));
            }
            out.push(normalize_cursor_event(ev));
        }
        return Ok(out);
    }

    Err(CursorParseError::Message(
        "unsupported cursor local input; provide empty, or vendor_reported UsageEvent JSON"
            .into(),
    ))
}

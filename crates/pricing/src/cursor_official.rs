//! Cursor official Admin API parse + reconcile (AC v1.4 F14 / F16).
//!
//! Pure JSON → `UsageEvent` / `SpendSnapshot` / `ReconcileReport`.
//! Tests are fixture-only (no live network). Credential env aligns with bridge:
//! `TOKENTRACER_CURSOR_ADMIN_API_KEY` (no invented secret storage).

use crate::models::{
    AgentId, CursorOfficialConfig, CursorSourceMode, ReconcileReport, SpendSnapshot,
    TeamMemberSpend, UsageEvent, UsagePool, CURSOR_ADMIN_API_KEY_ENV, FORBIDDEN_OTHER_MODEL_LITERALS,
    RECONCILE_TOLERANCE_USD, UNDOCUMENTED_DASHBOARD_BANNER,
};
use serde::Deserialize;
use thiserror::Error;

pub const SOURCE_TAG: &str = "EC-cursor-official-api-v1";

/// Diagnostic code aligned with bridge `TT-C14-MISSING-KEY`.
pub const CODE_MISSING_KEY: &str = "TT-C14-MISSING-KEY";

#[derive(Debug, Error, PartialEq)]
pub enum CursorOfficialError {
    #[error(
        "{CODE_MISSING_KEY}: Team Admin API key required for official_admin live fetch. \
         Set env {CURSOR_ADMIN_API_KEY_ENV} (Basic username, empty password; same as bridge). \
         Pricing/bridge do not store this key. Use --events/--spend fixtures for offline reconcile."
    )]
    AdminKeyMissing,
    #[error("cursor official JSON: {0}")]
    Json(String),
    #[error("undocumented_dashboard is disabled (default off); refuse GetCurrentPeriodUsage path")]
    UndocumentedDisabled,
    #[error("{UNDOCUMENTED_DASHBOARD_BANNER} {0}")]
    Undocumented(String),
}

/// Require Admin key for any live network path. Fixture CLI does not call this.
pub fn require_admin_api_key() -> Result<String, CursorOfficialError> {
    match std::env::var(CURSOR_ADMIN_API_KEY_ENV) {
        Ok(k) if !k.trim().is_empty() => Ok(k),
        _ => Err(CursorOfficialError::AdminKeyMissing),
    }
}

/// Gate undocumented dashboard paths (AC-F16). Default config refuses.
pub fn gate_undocumented_dashboard(
    cfg: &CursorOfficialConfig,
) -> Result<(), CursorOfficialError> {
    if cfg.undocumented_dashboard {
        Err(CursorOfficialError::Undocumented(
            "GetCurrentPeriodUsage / dashboard RPC must never be labeled official_admin".into(),
        ))
    } else {
        Err(CursorOfficialError::UndocumentedDisabled)
    }
}

#[derive(Debug, Deserialize)]
struct AdminEventsEnvelope {
    #[serde(default)]
    #[allow(dead_code)]
    #[serde(rename = "totalUsageEventsCount")]
    total_usage_events_count: Option<u64>,
    #[serde(default)]
    #[serde(alias = "usageEventsDisplay", alias = "usageEvents")]
    usage_events_display: Option<Vec<AdminUsageEvent>>,
}

#[derive(Debug, Deserialize)]
struct AdminUsageEvent {
    #[serde(default)]
    timestamp: Option<serde_json::Value>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default, rename = "conversationId")]
    conversation_id: Option<String>,
    #[serde(default, rename = "tokenUsage")]
    token_usage: Option<AdminTokenUsage>,
    #[serde(default, rename = "chargedCents")]
    charged_cents: Option<f64>,
    #[serde(default, rename = "cursorTokenFee")]
    cursor_token_fee: Option<f64>,
    #[serde(default)]
    tier: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct AdminTokenUsage {
    #[serde(default, rename = "inputTokens")]
    input_tokens: Option<u64>,
    #[serde(default, rename = "outputTokens")]
    output_tokens: Option<u64>,
    #[serde(default, rename = "cacheWriteTokens")]
    cache_write_tokens: Option<u64>,
    #[serde(default, rename = "cacheReadTokens")]
    cache_read_tokens: Option<u64>,
    #[serde(default, rename = "totalCents")]
    total_cents: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct AdminSpendEnvelope {
    #[serde(default, rename = "teamMemberSpend")]
    team_member_spend: Option<Vec<AdminMemberSpend>>,
    #[serde(default, rename = "subscriptionCycleStart")]
    subscription_cycle_start: Option<serde_json::Value>,
    #[serde(default, rename = "totalMembers")]
    total_members: Option<u64>,
    #[serde(default, rename = "overallSpendCents")]
    overall_spend_cents: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct AdminMemberSpend {
    #[serde(default, rename = "userId")]
    user_id: Option<i64>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default, rename = "spendCents")]
    spend_cents: Option<f64>,
    #[serde(default, rename = "overallSpendCents")]
    overall_spend_cents: Option<f64>,
}

fn ts_to_iso(v: &Option<serde_json::Value>) -> String {
    match v {
        Some(serde_json::Value::String(s)) if !s.is_empty() => s.clone(),
        Some(serde_json::Value::Number(n)) => {
            if let Some(ms) = n.as_i64() {
                use chrono::{TimeZone, Utc};
                Utc.timestamp_millis_opt(ms)
                    .single()
                    .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
                    .unwrap_or_else(|| "1970-01-01T00:00:00.000Z".into())
            } else {
                "1970-01-01T00:00:00.000Z".into()
            }
        }
        _ => "1970-01-01T00:00:00.000Z".into(),
    }
}

fn cycle_start_to_string(v: &Option<serde_json::Value>) -> Option<String> {
    match v {
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(serde_json::Value::Number(n)) => Some(n.to_string()),
        _ => None,
    }
}

fn normalize_model(raw: Option<String>) -> Option<String> {
    let t = raw?.trim().to_string();
    if t.is_empty() {
        return None;
    }
    if FORBIDDEN_OTHER_MODEL_LITERALS
        .iter()
        .any(|lit| t.eq_ignore_ascii_case(lit))
    {
        return None;
    }
    Some(t)
}

/// Parse Admin `filtered-usage-events` JSON → `UsageEvent` (AC-F14 / F19).
pub fn parse_admin_filtered_usage_events(json: &str) -> Result<Vec<UsageEvent>, CursorOfficialError> {
    let trimmed = json.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let events_raw: Vec<AdminUsageEvent> = if let Ok(env) =
        serde_json::from_str::<AdminEventsEnvelope>(trimmed)
    {
        env.usage_events_display.unwrap_or_default()
    } else if let Ok(arr) = serde_json::from_str::<Vec<AdminUsageEvent>>(trimmed) {
        arr
    } else {
        return Err(CursorOfficialError::Json(
            "expected filtered-usage-events envelope or event array".into(),
        ));
    };

    let mut out = Vec::with_capacity(events_raw.len());
    for (idx, raw) in events_raw.into_iter().enumerate() {
        let model = normalize_model(raw.model);
        let usage_pool = raw.tier.map(UsagePool::from_cursor_tier);
        let tu = raw.token_usage.as_ref();
        let charged = raw.charged_cents.unwrap_or(0.0);
        let raw_cost_usd = Some(charged / 100.0);
        let id = raw
            .conversation_id
            .clone()
            .map(|c| format!("cursor-admin:{c}:{idx}"))
            .unwrap_or_else(|| format!("cursor-admin:event:{idx}"));

        let meta = serde_json::json!({
            "cost_nature": "vendor_reported",
            "cursor_source_mode": CursorSourceMode::OfficialAdmin.as_str(),
            "charged_cents": charged,
            "cursor_token_fee": raw.cursor_token_fee,
            "kind": raw.kind,
            "conversation_id": raw.conversation_id,
            "tier": raw.tier,
            "token_usage_total_cents": tu.and_then(|t| t.total_cents),
        });

        out.push(UsageEvent {
            id,
            agent: AgentId::Cursor,
            source: SOURCE_TAG.into(),
            ts: ts_to_iso(&raw.timestamp),
            model,
            usage_pool,
            input_tokens: tu.and_then(|t| t.input_tokens).unwrap_or(0),
            output_tokens: tu.and_then(|t| t.output_tokens).unwrap_or(0),
            cache_read_tokens: tu.and_then(|t| t.cache_read_tokens),
            cache_write_tokens: tu.and_then(|t| t.cache_write_tokens),
            raw_cost_usd,
            meta: Some(meta),
        });
    }
    Ok(out)
}

/// Parse Admin `/teams/spend` JSON → `SpendSnapshot`.
pub fn parse_teams_spend(json: &str) -> Result<SpendSnapshot, CursorOfficialError> {
    let trimmed = json.trim();
    let env: AdminSpendEnvelope = serde_json::from_str(trimmed)
        .map_err(|e| CursorOfficialError::Json(format!("teams/spend: {e}")))?;

    let members: Vec<TeamMemberSpend> = env
        .team_member_spend
        .unwrap_or_default()
        .into_iter()
        .map(|m| TeamMemberSpend {
            user_id: m.user_id,
            name: m.name,
            email: m.email,
            spend_cents: m.spend_cents.unwrap_or(0.0),
            overall_spend_cents: m.overall_spend_cents,
        })
        .collect();

    let overall = if let Some(top) = env.overall_spend_cents {
        top
    } else {
        let sum_overall: f64 = members
            .iter()
            .filter_map(|m| m.overall_spend_cents)
            .sum();
        if sum_overall > 0.0 {
            sum_overall
        } else {
            members.iter().map(|m| m.spend_cents).sum()
        }
    };

    Ok(SpendSnapshot {
        members,
        overall_spend_cents: overall,
        subscription_cycle_start: cycle_start_to_string(&env.subscription_cycle_start),
        total_members: env.total_members,
    })
}

/// Sum `meta.charged_cents` or derive from `raw_cost_usd * 100`.
pub fn sum_charged_cents(events: &[UsageEvent]) -> f64 {
    events
        .iter()
        .map(|ev| {
            if let Some(c) = ev
                .meta
                .as_ref()
                .and_then(|m| m.get("charged_cents"))
                .and_then(|v| v.as_f64())
            {
                c
            } else {
                ev.raw_cost_usd.unwrap_or(0.0) * 100.0
            }
        })
        .sum()
}

/// Reconcile Σ chargedCents ↔ spend overall (abs tol $0.01).
pub fn reconcile_charged_cents(events: &[UsageEvent], spend: &SpendSnapshot) -> ReconcileReport {
    reconcile_charged_cents_with_tol(events, spend, RECONCILE_TOLERANCE_USD)
}

pub fn reconcile_charged_cents_with_tol(
    events: &[UsageEvent],
    spend: &SpendSnapshot,
    tolerance_usd: f64,
) -> ReconcileReport {
    let events_charged_cents_sum = sum_charged_cents(events);
    let events_charged_usd = events_charged_cents_sum / 100.0;
    let spend_overall_cents = spend.overall_spend_cents;
    let spend_overall_usd = spend_overall_cents / 100.0;
    let delta_usd = (events_charged_usd - spend_overall_usd).abs();
    ReconcileReport {
        events_charged_cents_sum,
        events_charged_usd,
        spend_overall_cents,
        spend_overall_usd,
        delta_usd,
        tolerance_usd,
        matched: delta_usd <= tolerance_usd,
        event_count: events.len(),
        spend_member_count: spend.members.len(),
    }
}

/// Trait hook for optional live Admin HTTP (tests never call live).
pub trait AdminApiClient {
    fn fetch_filtered_usage_events(&self) -> Result<String, CursorOfficialError>;
    fn fetch_teams_spend(&self) -> Result<String, CursorOfficialError>;
}

/// Placeholder client that only checks for Admin key — no network in v0.
pub struct EnvAdminApiClient;

impl AdminApiClient for EnvAdminApiClient {
    fn fetch_filtered_usage_events(&self) -> Result<String, CursorOfficialError> {
        let _key = require_admin_api_key()?;
        Err(CursorOfficialError::Json(
            "live Admin HTTP not enabled in this build; use fixtures --events/--spend".into(),
        ))
    }

    fn fetch_teams_spend(&self) -> Result<String, CursorOfficialError> {
        let _key = require_admin_api_key()?;
        Err(CursorOfficialError::Json(
            "live Admin HTTP not enabled in this build; use fixtures --events/--spend".into(),
        ))
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn admin_key_missing_message_has_bridge_code() {
        let s = CursorOfficialError::AdminKeyMissing.to_string();
        assert!(s.contains(CODE_MISSING_KEY));
        assert!(s.contains(CURSOR_ADMIN_API_KEY_ENV));
    }

    #[test]
    fn undocumented_default_off() {
        let cfg = CursorOfficialConfig::default();
        assert!(!cfg.undocumented_dashboard);
        let err = gate_undocumented_dashboard(&cfg).unwrap_err();
        assert_eq!(err, CursorOfficialError::UndocumentedDisabled);
    }
}

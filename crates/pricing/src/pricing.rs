//! Pure pricing: no filesystem / network / global mutability.
//!
//! BY-MODEL v0.2 / AC v1.3 ∪ v1.3a: aggregates `by_model` (concrete model ids only)
//! and `by_usage_pool` (Cursor dual pool). Never emits `__other__` as a model sentinel.

use crate::models::*;
use crate::price_table::PriceTable;
use crate::range::ResolvedWindow;
use chrono::Utc;
use std::collections::BTreeMap;

const TOKENS_PER_MILLION: f64 = 1_000_000.0;

/// Round to 6 decimal places to avoid binary float display noise (still << $0.01 tol).
pub fn round_money(x: f64) -> f64 {
    (x * 1_000_000.0).round() / 1_000_000.0
}

/// Cost for one event from the price table (notional).
/// Returns `None` if model is missing/unknown (caller treats as unpriced).
pub fn event_notional_usd(event: &UsageEvent, table: &PriceTable) -> Option<f64> {
    let model = event.model.as_deref()?.trim();
    if model.is_empty() {
        return None;
    }
    let (rates, _row) = table.lookup_row(model)?;
    let input = event.input_tokens as f64;
    let output = event.output_tokens as f64;
    let cache_read = event.cache_read_tokens.unwrap_or(0) as f64;
    let cache_write = event.cache_write_tokens.unwrap_or(0) as f64;

    let usd = (input * rates.input
        + output * rates.output
        + cache_read * rates.cache_read_rate()
        + cache_write * rates.cache_write_rate())
        / TOKENS_PER_MILLION;
    Some(usd)
}

fn is_vendor_reported(event: &UsageEvent) -> bool {
    event
        .meta
        .as_ref()
        .and_then(|m| m.get("cost_nature"))
        .and_then(|v| v.as_str())
        == Some("vendor_reported")
}

/// (amount_usd, used_vendor_reported, unknown_model).
pub fn event_amount_usd_parts(
    event: &UsageEvent,
    table: &PriceTable,
) -> (f64, bool, bool) {
    if event.raw_cost_usd.is_some() && is_vendor_reported(event) {
        return (event.raw_cost_usd.unwrap_or(0.0), true, false);
    }
    match event_notional_usd(event, table) {
        Some(v) => (v, false, false),
        None => {
            let unknown = event
                .model
                .as_ref()
                .map(|m| !m.trim().is_empty())
                .unwrap_or(false);
            (0.0, false, unknown)
        }
    }
}

/// Resolve usage_pool for an event (AC v1.3a priority).
///
/// 1. Explicit `event.usage_pool`
/// 2. `meta.cursor_tier` / `meta.tier` via PARTIAL community map (1→other_models, 2→cursor_models)
/// 3. Cursor agent with no signal → `unknown`
/// 4. Non-cursor → `None`
pub fn resolve_usage_pool(event: &UsageEvent) -> Option<UsagePool> {
    if let Some(pool) = event.usage_pool {
        return Some(pool);
    }
    if let Some(tier) = event
        .meta
        .as_ref()
        .and_then(|m| m.get("cursor_tier").or_else(|| m.get("tier")))
        .and_then(|v| v.as_i64())
    {
        return Some(UsagePool::from_cursor_tier(tier));
    }
    if matches!(event.agent, AgentId::Cursor) {
        return Some(UsagePool::Unknown);
    }
    None
}

/// Concrete model label for by_model rows.
/// Missing / empty / forbidden `"Other"` literals → [`CURSOR_UNKNOWN_MODEL`].
/// Never returns `__other__`.
fn model_key(event: &UsageEvent) -> String {
    let raw = event
        .model
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if let Some(m) = raw {
        if FORBIDDEN_OTHER_MODEL_LITERALS
            .iter()
            .any(|lit| m.eq_ignore_ascii_case(lit))
        {
            return CURSOR_UNKNOWN_MODEL.to_string();
        }
        return m.to_string();
    }
    CURSOR_UNKNOWN_MODEL.to_string()
}

fn default_status(agent: &AgentId) -> AgentSpendStatus {
    match agent {
        AgentId::Cursor => AgentSpendStatus::Partial,
        AgentId::ClaudeCode | AgentId::Codex => AgentSpendStatus::Ok,
        _ => AgentSpendStatus::Unsupported,
    }
}

#[derive(Default)]
struct ModelAgg {
    agent: Option<AgentId>,
    usage_pool: Option<UsagePool>,
    amount_usd: f64,
    input_tokens: u64,
    output_tokens: u64,
    priced: bool,
    price_table_row: Option<String>,
}

#[derive(Default)]
struct PoolAgg {
    amount_usd: f64,
    has_priced: bool,
    percent_used: Option<f64>,
}

/// Pure function: `price(events, price_table, fx?) -> SpendSummary`.
pub fn price(
    events: &[UsageEvent],
    price_table: &PriceTable,
    fx: Option<&FxSnapshot>,
) -> SpendSummary {
    price_with_range(events, price_table, fx, None)
}

/// Like `price`, but attach a resolved `SpendRange` (e.g. after filter).
pub fn price_with_range(
    events: &[UsageEvent],
    price_table: &PriceTable,
    fx: Option<&FxSnapshot>,
    window: Option<&ResolvedWindow>,
) -> SpendSummary {
    let mut by_agent_amt: BTreeMap<String, (AgentId, f64, AgentSpendStatus)> = BTreeMap::new();
    let mut by_model_amt: BTreeMap<(String, String), ModelAgg> = BTreeMap::new();
    let mut by_pool_amt: BTreeMap<&'static str, PoolAgg> = BTreeMap::new();
    let mut unknown_models: u64 = 0;
    let mut vendor_count: u64 = 0;
    let mut notional_count: u64 = 0;
    let mut unpriced: Vec<UnpricedTokens> = Vec::new();

    for ev in events {
        let (amt_usd, used_vendor, unknown) = event_amount_usd_parts(ev, price_table);
        if unknown {
            unknown_models += 1;
        }

        let mkey = model_key(ev);
        let pool = resolve_usage_pool(ev);
        let row_key = ev
            .model
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter(|m| {
                !FORBIDDEN_OTHER_MODEL_LITERALS
                    .iter()
                    .any(|lit| m.eq_ignore_ascii_case(lit))
            })
            .and_then(|m| price_table.lookup_row(m).map(|(_, k)| k.to_string()));

        // Priced = vendor-reported raw OR price-table hit. Unpriced → amount null, excluded from total.
        let priced = used_vendor || row_key.is_some();

        if used_vendor {
            vendor_count += 1;
        } else if priced {
            notional_count += 1;
        }

        let agent_add = if priced { amt_usd } else { 0.0 };
        let akey = ev.agent.as_str().to_string();
        let entry = by_agent_amt
            .entry(akey)
            .or_insert_with(|| (ev.agent.clone(), 0.0, default_status(&ev.agent)));
        entry.1 += agent_add;
        if matches!(ev.agent, AgentId::Cursor) {
            entry.2 = AgentSpendStatus::Partial;
        }

        let mk = (ev.agent.as_str().to_string(), mkey.clone());
        let magg = by_model_amt.entry(mk).or_insert_with(|| ModelAgg {
            agent: Some(ev.agent.clone()),
            usage_pool: pool,
            ..Default::default()
        });
        if magg.usage_pool.is_none() {
            magg.usage_pool = pool;
        } else if magg.usage_pool == Some(UsagePool::Unknown) && pool.is_some() {
            magg.usage_pool = pool;
        }
        magg.input_tokens += ev.input_tokens;
        magg.output_tokens += ev.output_tokens;
        if priced {
            magg.amount_usd += amt_usd;
            magg.priced = true;
            if magg.price_table_row.is_none() {
                magg.price_table_row = row_key.clone();
            }
        } else {
            unpriced.push(UnpricedTokens {
                model: mkey.clone(),
                input_tokens: ev.input_tokens,
                output_tokens: ev.output_tokens,
            });
        }

        if matches!(ev.agent, AgentId::Cursor) {
            let p = pool.unwrap_or(UsagePool::Unknown);
            let pagg = by_pool_amt.entry(p.as_str()).or_default();
            if priced {
                pagg.amount_usd += amt_usd;
                pagg.has_priced = true;
            }
            if pagg.percent_used.is_none() {
                let pct_key = match p {
                    UsagePool::CursorModels => "autoPercentUsed",
                    UsagePool::OtherModels => "apiPercentUsed",
                    UsagePool::Unknown => "",
                };
                if !pct_key.is_empty() {
                    pagg.percent_used = ev
                        .meta
                        .as_ref()
                        .and_then(|m| m.get(pct_key))
                        .and_then(|v| v.as_f64());
                }
            }
        }
    }

    let total_usd: f64 = round_money(by_agent_amt.values().map(|(_, a, _)| *a).sum());

    let (currency, total, fx_out) = match fx {
        Some(snap)
            if snap.pair.eq_ignore_ascii_case("USD/TWD")
                || snap.pair.eq_ignore_ascii_case("USDTWD") =>
        {
            (
                Currency::Twd,
                round_money(total_usd * snap.rate),
                Some(snap.clone()),
            )
        }
        _ => (Currency::Usd, total_usd, fx.cloned()),
    };

    let scale = if matches!(currency, Currency::Twd) {
        fx_out.as_ref().map(|s| s.rate).unwrap_or(1.0)
    } else {
        1.0
    };

    let by_agent: Vec<AgentSpend> = by_agent_amt
        .into_values()
        .map(|(agent, amount, status)| AgentSpend {
            agent,
            amount: round_money(amount * scale),
            status,
        })
        .collect();

    let mut by_model: Vec<ModelSpend> = by_model_amt
        .into_iter()
        .map(|((_agent_str, model), agg)| {
            let amount = if agg.priced {
                Some(round_money(agg.amount_usd * scale))
            } else {
                None
            };
            ModelSpend {
                model,
                agent: agg.agent,
                usage_pool: agg.usage_pool,
                amount,
                input_tokens: agg.input_tokens,
                output_tokens: agg.output_tokens,
                priced: agg.priced,
                price_table_row: agg.price_table_row,
            }
        })
        .collect();

    by_model.sort_by(|a, b| {
        let aa = a.amount.unwrap_or(f64::NEG_INFINITY);
        let bb = b.amount.unwrap_or(f64::NEG_INFINITY);
        bb.partial_cmp(&aa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.model.cmp(&b.model))
    });

    let mut by_usage_pool: Vec<UsagePoolSpend> = by_pool_amt
        .into_iter()
        .map(|(pool_str, agg)| {
            let pool = match pool_str {
                "cursor_models" => UsagePool::CursorModels,
                "other_models" => UsagePool::OtherModels,
                _ => UsagePool::Unknown,
            };
            UsagePoolSpend {
                agent: AgentId::Cursor,
                pool,
                amount: Some(round_money(agg.amount_usd * scale)),
                percent_used: agg.percent_used,
            }
        })
        .collect();
    by_usage_pool.sort_by(|a, b| a.pool.as_str().cmp(b.pool.as_str()));

    let unpriced_tokens = if unpriced.is_empty() {
        None
    } else {
        let mut map: BTreeMap<String, (u64, u64)> = BTreeMap::new();
        for u in unpriced {
            let e = map.entry(u.model).or_default();
            e.0 += u.input_tokens;
            e.1 += u.output_tokens;
        }
        Some(
            map.into_iter()
                .map(|(model, (input_tokens, output_tokens))| UnpricedTokens {
                    model,
                    input_tokens,
                    output_tokens,
                })
                .collect(),
        )
    };

    let cost_nature = if vendor_count > 0 && notional_count > 0 {
        CostNature::Mixed
    } else if vendor_count > 0 {
        CostNature::VendorReported
    } else {
        CostNature::NotionalApiEstimate
    };

    let meta = if unknown_models > 0 {
        Some(serde_json::json!({
            "unknown_model_events": unknown_models,
            "note": "unknown models unpriced (amount null); no invented rates"
        }))
    } else {
        None
    };

    let range = window.map(|w| w.to_spend_range()).or(Some(SpendRange {
        kind: RangeKind::All,
        start: None,
        end: None,
    }));

    SpendSummary {
        currency,
        total,
        by_agent,
        by_model,
        by_usage_pool,
        unpriced_tokens,
        price_table_version: price_table.version.clone(),
        pricing_mode: PricingMode::NotionalApiEstimate,
        disclaimer: NOTIONAL_DISCLAIMER.to_string(),
        fx_snapshot: fx_out,
        range,
        computed_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        cost_nature,
        price_table_source: price_table.source_citation(),
        meta,
    }
}

#[cfg(test)]
mod pure_tests {
    use super::*;

    #[test]
    fn price_does_not_require_filesystem() {
        let table = PriceTable::embedded();
        let events = vec![UsageEvent {
            id: "t1".into(),
            agent: AgentId::ClaudeCode,
            source: "test".into(),
            ts: "2026-09-11T00:00:00.000Z".into(),
            model: Some("claude-sonnet-4-20250514".into()),
            usage_pool: None,
            input_tokens: 1_000_000,
            output_tokens: 0,
            cache_read_tokens: None,
            cache_write_tokens: None,
            raw_cost_usd: None,
            meta: None,
        }];
        let summary = price(&events, &table, None);
        assert!((summary.total - 3.0).abs() < 0.01);
        assert_eq!(summary.cost_nature, CostNature::NotionalApiEstimate);
        assert_eq!(summary.pricing_mode, PricingMode::NotionalApiEstimate);
        assert!(summary.disclaimer.to_lowercase().contains("notional"));
        assert_eq!(summary.by_model.len(), 1);
        assert_eq!(summary.by_model[0].model, "claude-sonnet-4-20250514");
        assert!(summary.by_model[0].priced);
    }

    #[test]
    fn never_emits_dunder_other_sentinel() {
        let table = PriceTable::embedded();
        let events = vec![UsageEvent {
            id: "c1".into(),
            agent: AgentId::Cursor,
            source: "test".into(),
            ts: "2026-09-11T00:00:00.000Z".into(),
            model: None,
            usage_pool: Some(UsagePool::OtherModels),
            input_tokens: 100,
            output_tokens: 0,
            cache_read_tokens: None,
            cache_write_tokens: None,
            raw_cost_usd: Some(1.5),
            meta: Some(serde_json::json!({"cost_nature": "vendor_reported"})),
        }];
        let summary = price(&events, &table, None);
        let json = serde_json::to_string(&summary).unwrap();
        assert!(!json.contains("__other__"));
        assert_eq!(summary.by_model[0].model, CURSOR_UNKNOWN_MODEL);
        assert_eq!(
            summary.by_model[0].usage_pool,
            Some(UsagePool::OtherModels)
        );
        assert!(summary
            .by_usage_pool
            .iter()
            .any(|p| p.pool == UsagePool::OtherModels));
    }

    #[test]
    fn tier_maps_to_usage_pool_partial() {
        assert_eq!(UsagePool::from_cursor_tier(1), UsagePool::OtherModels);
        assert_eq!(UsagePool::from_cursor_tier(2), UsagePool::CursorModels);
        assert_eq!(UsagePool::from_cursor_tier(99), UsagePool::Unknown);
    }

    #[test]
    fn by_model_sum_matches_total() {
        let table = PriceTable::embedded();
        let events = vec![
            UsageEvent {
                id: "a".into(),
                agent: AgentId::ClaudeCode,
                source: "t".into(),
                ts: "2026-09-11T00:00:00.000Z".into(),
                model: Some("claude-sonnet-4-20250514".into()),
                usage_pool: None,
                input_tokens: 1_000_000,
                output_tokens: 0,
                cache_read_tokens: None,
                cache_write_tokens: None,
                raw_cost_usd: None,
                meta: None,
            },
            UsageEvent {
                id: "b".into(),
                agent: AgentId::Codex,
                source: "t".into(),
                ts: "2026-09-11T00:00:00.000Z".into(),
                model: Some("gpt-5".into()),
                usage_pool: None,
                input_tokens: 1_000_000,
                output_tokens: 0,
                cache_read_tokens: None,
                cache_write_tokens: None,
                raw_cost_usd: None,
                meta: None,
            },
        ];
        let s = price(&events, &table, None);
        let sum_model: f64 = s
            .by_model
            .iter()
            .filter(|m| m.priced)
            .map(|m| m.amount.unwrap_or(0.0))
            .sum();
        let sum_agent: f64 = s.by_agent.iter().map(|a| a.amount).sum();
        assert!((sum_model - s.total).abs() <= 0.01);
        assert!((sum_agent - s.total).abs() <= 0.01);
    }

    #[test]
    fn concrete_model_plus_other_models_pool_distinct() {
        let table = PriceTable::embedded();
        let events = vec![UsageEvent {
            id: "c".into(),
            agent: AgentId::Cursor,
            source: "t".into(),
            ts: "2026-09-11T00:00:00.000Z".into(),
            model: Some("grok-4.6".into()),
            usage_pool: Some(UsagePool::OtherModels),
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: None,
            cache_write_tokens: None,
            raw_cost_usd: Some(0.42),
            meta: Some(serde_json::json!({
                "cost_nature": "vendor_reported",
                "tier": 1
            })),
        }];
        let s = price(&events, &table, None);
        assert_eq!(s.by_model[0].model, "grok-4.6");
        assert_eq!(s.by_model[0].usage_pool, Some(UsagePool::OtherModels));
        assert!(!s.by_model[0].model.contains("Other"));
        assert!(!s.by_model.iter().any(|m| m.model == "__other__"));
    }
}

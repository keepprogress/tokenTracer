//! AC v1.4 cursor-official — F14 reconcile, F15 local guard, F16 default off, F19 no Other.

use pricing::parsers::parse_cursor_local;
use pricing::{
    gate_undocumented_dashboard, parse_admin_filtered_usage_events, parse_teams_spend, price,
    reconcile_charged_cents, require_admin_api_key, AgentId, AgentSpendStatus, CursorOfficialConfig,
    CursorOfficialError, CursorSourceMode, OnDemandAlign, PriceTable, SpendingAlignSummary,
    UsageEvent, UsagePool, CODE_MISSING_KEY, CURSOR_ADMIN_API_KEY_ENV, CURSOR_LOCAL_ONLY_DISCLAIMER,
    FORBIDDEN_OTHER_MODEL_LITERALS, RECONCILE_TOLERANCE_USD,
};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/ac-v1.4")
}

fn read_fix(name: &str) -> String {
    std::fs::read_to_string(fixtures_dir().join(name))
        .unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
}

#[test]
fn f14_parse_admin_events_maps_vendor_reported_and_pools() {
    let events = parse_admin_filtered_usage_events(&read_fix("admin-events.json"))
        .expect("parse admin events");
    assert_eq!(events.len(), 3);
    for ev in &events {
        assert_eq!(ev.agent, AgentId::Cursor);
        assert_eq!(ev.source, "EC-cursor-official-api-v1");
        assert!(ev.raw_cost_usd.is_some());
        let meta = ev.meta.as_ref().expect("meta");
        assert_eq!(
            meta.get("cost_nature").and_then(|v| v.as_str()),
            Some("vendor_reported")
        );
        assert_eq!(
            meta.get("cursor_source_mode").and_then(|v| v.as_str()),
            Some(CursorSourceMode::OfficialAdmin.as_str())
        );
        let model = ev.model.as_deref().expect("concrete model");
        for lit in FORBIDDEN_OTHER_MODEL_LITERALS {
            assert!(!model.eq_ignore_ascii_case(lit));
        }
    }
    assert_eq!(events[0].model.as_deref(), Some("claude-4.5-sonnet"));
    assert_eq!(events[0].usage_pool, Some(UsagePool::OtherModels));
    assert!((events[0].raw_cost_usd.unwrap() - 12.34).abs() < 1e-9);
    assert_eq!(events[1].model.as_deref(), Some("composer-1.5"));
    assert_eq!(events[1].usage_pool, Some(UsagePool::CursorModels));
    assert_eq!(events[2].model.as_deref(), Some("grok-4.6"));
    assert_eq!(events[2].usage_pool, Some(UsagePool::OtherModels));
}

#[test]
fn f14_reconcile_charged_cents_matches_spend_fixture() {
    let events = parse_admin_filtered_usage_events(&read_fix("admin-events.json")).unwrap();
    let spend = parse_teams_spend(&read_fix("admin-spend.json")).unwrap();
    let report = reconcile_charged_cents(&events, &spend);
    let expected: serde_json::Value =
        serde_json::from_str(&read_fix("admin-reconcile.expected.json")).unwrap();

    assert!((report.events_charged_cents_sum - 2000.0).abs() < 1e-9);
    assert!((report.events_charged_usd - 20.0).abs() < 1e-9);
    assert!((report.spend_overall_cents - 2000.0).abs() < 1e-9);
    assert!((report.spend_overall_usd - 20.0).abs() < 1e-9);
    assert!((report.delta_usd - 0.0).abs() < 1e-9);
    assert_eq!(report.tolerance_usd, RECONCILE_TOLERANCE_USD);
    assert!(report.matched);
    assert_eq!(report.event_count, 3);
    assert_eq!(report.spend_member_count, 2);
    assert_eq!(report.matched, expected["matched"].as_bool().unwrap());
    assert!(
        (report.events_charged_cents_sum - expected["events_charged_cents_sum"].as_f64().unwrap())
            .abs()
            < 1e-9
    );
}

#[test]
fn f14_admin_key_missing_is_clear_error() {
    let prev = std::env::var_os(CURSOR_ADMIN_API_KEY_ENV);
    std::env::remove_var(CURSOR_ADMIN_API_KEY_ENV);
    let err = require_admin_api_key().expect_err("must err without key");
    let msg = err.to_string();
    assert!(msg.contains("Team Admin API key required"), "{msg}");
    assert!(msg.contains(CURSOR_ADMIN_API_KEY_ENV), "{msg}");
    assert!(msg.contains(CODE_MISSING_KEY), "{msg}");
    assert!(msg.contains("fixtures") || msg.contains("--events"), "{msg}");
    match prev {
        Some(v) => std::env::set_var(CURSOR_ADMIN_API_KEY_ENV, v),
        None => std::env::remove_var(CURSOR_ADMIN_API_KEY_ENV),
    }
}

#[test]
fn f15_local_enrichment_not_sole_authoritative_usd() {
    let table = PriceTable::embedded();
    let events = vec![UsageEvent {
        id: "local1".into(),
        agent: AgentId::Cursor,
        source: "EC-cursor-v1".into(),
        ts: "2026-09-11T00:00:00.000Z".into(),
        model: Some("claude-sonnet-4-20250514".into()),
        usage_pool: Some(UsagePool::OtherModels),
        input_tokens: 1_000_000,
        output_tokens: 0,
        cache_read_tokens: None,
        cache_write_tokens: None,
        raw_cost_usd: None,
        meta: Some(serde_json::json!({
            "cursor_source_mode": "local_enrichment"
        })),
    }];
    let s = price(&events, &table, None);
    let cursor = s
        .by_agent
        .iter()
        .find(|a| a.agent == AgentId::Cursor)
        .expect("cursor row");
    assert_eq!(cursor.status, AgentSpendStatus::Partial);
    assert!(
        s.disclaimer.contains("PARTIAL") || s.disclaimer.contains(CURSOR_LOCAL_ONLY_DISCLAIMER),
        "disclaimer={}",
        s.disclaimer
    );
    let lower = s.disclaimer.to_lowercase();
    assert!(
        (lower.contains("not") && lower.contains("invoice"))
            || s.disclaimer.contains("authoritative"),
        "must not present as sole invoice: {}",
        s.disclaimer
    );
    let meta = s.meta.as_ref().expect("meta for local cursor");
    assert_eq!(
        meta.get("cursor_authoritative_invoice_usd")
            .and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn f15_bubble_tokencount_still_refused() {
    let bubble = r#"{"bubbleId":"x","tokenCount":{"inputTokens":10,"outputTokens":5}}"#;
    let err = parse_cursor_local(bubble).expect_err("must reject bubble tokenCount");
    let msg = err.to_string().to_lowercase();
    assert!(msg.contains("tokencount") || msg.contains("unreliable"), "{msg}");
}

#[test]
fn f16_undocumented_dashboard_default_off() {
    let cfg = CursorOfficialConfig::default();
    assert!(!cfg.undocumented_dashboard);
    assert!(!cfg.is_undocumented_dashboard_enabled());
    let err = gate_undocumented_dashboard(&cfg).unwrap_err();
    assert!(
        matches!(err, CursorOfficialError::UndocumentedDisabled)
            || err.to_string().to_lowercase().contains("disabled"),
        "{err}"
    );

    let mut on = CursorOfficialConfig::default();
    on.undocumented_dashboard = true;
    let err2 = gate_undocumented_dashboard(&on).unwrap_err();
    let s = err2.to_string();
    assert!(s.contains("UNSUPPORTED") || s.contains("undocumented"), "{s}");
}

#[test]
fn f19_no_literal_other_model_from_admin_parse() {
    let events = parse_admin_filtered_usage_events(&read_fix("admin-events.json")).unwrap();
    let table = PriceTable::embedded();
    let s = price(&events, &table, None);
    for m in &s.by_model {
        assert_ne!(m.model, "Other");
        assert_ne!(m.model, "Other model");
        assert_ne!(m.model, "__other__");
        assert_ne!(m.model, "cursor:other");
        for lit in FORBIDDEN_OTHER_MODEL_LITERALS {
            assert!(!m.model.eq_ignore_ascii_case(lit), "model={}", m.model);
        }
    }
    assert!(s
        .by_usage_pool
        .iter()
        .any(|p| p.pool == UsagePool::CursorModels));
    assert!(s
        .by_usage_pool
        .iter()
        .any(|p| p.pool == UsagePool::OtherModels));
    assert!(s.by_model.iter().any(|m| m.model == "composer-1.5"));
    assert!(s.by_model.iter().any(|m| m.model == "claude-4.5-sonnet"));
}

#[test]
fn spending_align_summary_is_comparison_not_usage_event() {
    let align = SpendingAlignSummary {
        cursor_models_percent: Some(65.0),
        other_models_percent: Some(100.0),
        reset_note: Some("resets ~2026-09-12".into()),
        on_demand: Some(OnDemandAlign {
            enabled: false,
            monthly_limit_note: Some("Disabled".into()),
        }),
        partial: true,
        no_public_personal_usage_api: true,
        note: Some("Personal Ultra: Spending UI align target; not local token sum".into()),
    };
    let v = serde_json::to_value(&align).unwrap();
    assert!(v.get("cursor_models_percent").is_some());
    assert_eq!(v["partial"], true);
    assert_eq!(v["no_public_personal_usage_api"], true);
    assert!(v.get("input_tokens").is_none());
    assert!(v.get("raw_cost_usd").is_none());
}

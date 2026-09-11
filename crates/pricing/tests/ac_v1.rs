//! AC v1 fixture + parser + pricing integration tests.

use chrono::{TimeZone, Utc};
use pricing::parsers::{parse_claude_code_jsonl, parse_codex_rollout_jsonl, parse_cursor_local};
use pricing::{
    daily_spend_series, filter_events_by_range, price, price_with_range, read_import_meta,
    record_import, AgentId, AgentSpendStatus, CostNature, DayBoundary, PriceTable, PricingMode,
    RangeFilterOpts, RangeKind, SeriesOpts, UsageEvent, UsagePool, CURSOR_UNKNOWN_MODEL,
};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    // crates/pricing/tests → ../../../fixtures/ac-v1
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/ac-v1")
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixtures_dir().join(name))
        .unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
}

#[test]
fn parse_claude_fixture_dedups_message_id() {
    let text = read_fixture("claude-sample.jsonl");
    let events = parse_claude_code_jsonl(&text).expect("parse claude");
    assert_eq!(events.len(), 2, "expected 2 after dedup, got {}", events.len());
    assert_eq!(events[0].id, "msg_dedup_test");
    assert_eq!(events[1].id, "msg_second");
    assert_eq!(events[0].agent, AgentId::ClaudeCode);
    assert_eq!(events[0].input_tokens, 10_000);
    assert_eq!(events[0].output_tokens, 2_000);
    assert_eq!(events[0].cache_read_tokens, Some(100_000));
    assert_eq!(events[0].cache_write_tokens, Some(8_000));
    assert_eq!(events[1].input_tokens, 5_000);
    assert_eq!(events[1].output_tokens, 1_000);
}

#[test]
fn parse_codex_fixture_no_double_count() {
    let text = read_fixture("codex-sample.jsonl");
    let events = parse_codex_rollout_jsonl(&text).expect("parse codex");
    assert_eq!(events.len(), 1, "token_count snapshot must not be counted");
    assert_eq!(events[0].id, "resp_fixture_codex_1");
    assert_eq!(events[0].agent, AgentId::Codex);
    assert_eq!(events[0].model.as_deref(), Some("gpt-5-codex"));
    // 20000 - 8000
    assert_eq!(events[0].input_tokens, 12_000);
    assert_eq!(events[0].cache_read_tokens, Some(8_000));
    assert_eq!(events[0].cache_write_tokens, Some(0));
    assert_eq!(events[0].output_tokens, 400);
}

#[test]
fn price_matches_expected_usd_total() {
    let events: Vec<UsageEvent> =
        serde_json::from_str(&read_fixture("events.json")).expect("events.json");
    let table = PriceTable::embedded();
    let summary = price(&events, &table, None);
    assert!(
        (summary.total - 0.17).abs() <= 0.01,
        "total {} != 0.17",
        summary.total
    );
    assert_eq!(summary.price_table_version, "2026-09-11.v2");
    assert_eq!(summary.pricing_mode, PricingMode::NotionalApiEstimate);
    assert!(summary.disclaimer.to_lowercase().contains("notional"));
    assert_eq!(summary.cost_nature, CostNature::NotionalApiEstimate);
    assert!(summary.price_table_source.contains("platform.claude.com"));
    assert!(summary.price_table_source.contains("openai.com"));

    let claude = summary
        .by_agent
        .iter()
        .find(|a| a.agent == AgentId::ClaudeCode)
        .expect("claude_code");
    let codex = summary
        .by_agent
        .iter()
        .find(|a| a.agent == AgentId::Codex)
        .expect("codex");
    assert!((claude.amount - 0.15).abs() <= 0.01);
    assert!((codex.amount - 0.02).abs() <= 0.01);
    assert_eq!(claude.status, AgentSpendStatus::Ok);
    assert_eq!(codex.status, AgentSpendStatus::Ok);
}

#[test]
fn cursor_stub_rejects_bubble_tokencount() {
    let bubble = r#"{"bubbleId":"x","tokenCount":12345,"type":"ai"}"#;
    let err = parse_cursor_local(bubble).expect_err("must reject bubble tokenCount");
    let msg = err.to_string();
    assert!(
        msg.contains("tokenCount") || msg.contains("unreliable") || msg.contains("refusing"),
        "msg={msg}"
    );
}

#[test]
fn cursor_stub_empty_ok_no_invented_billing() {
    let events = parse_cursor_local("").expect("empty ok");
    assert!(events.is_empty());
    let table = PriceTable::embedded();
    let summary = price(&events, &table, None);
    assert!((summary.total - 0.0).abs() < 0.001);
}

#[test]
fn pure_pricing_does_not_touch_filesystem() {
    // Construct everything in-memory; if this passes, price() needs no FS.
    let table = PriceTable::from_json(pricing::price_table::EMBEDDED_PRICE_TABLE_JSON).unwrap();
    let events = vec![UsageEvent {
        id: "mem".into(),
        agent: AgentId::Codex,
        source: "mem".into(),
        ts: "2026-09-11T00:00:00.000Z".into(),
        model: Some("gpt-5".into()),
        usage_pool: None,
        input_tokens: 1_000_000,
        output_tokens: 0,
        cache_read_tokens: Some(0),
        cache_write_tokens: Some(0),
        raw_cost_usd: None,
        meta: None,
    }];
    let s = price(&events, &table, None);
    assert!((s.total - 1.25).abs() < 0.01);
}

#[test]
fn today_range_filters_by_local_day() {
    // Asia/Taipei 2026-09-11 = UTC [2026-09-10T16:00:00Z, 2026-09-11T16:00:00Z)
    let events: Vec<UsageEvent> =
        serde_json::from_str(&read_fixture("events.json")).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 11, 8, 0, 0).unwrap();
    let opts = RangeFilterOpts {
        kind: RangeKind::Today,
        day_boundary: DayBoundary::Local,
        timezone: Some("Asia/Taipei"),
        now,
        custom_from: None,
        custom_to: None,
    };
    let (filtered, window) = filter_events_by_range(&events, &opts).unwrap();
    assert_eq!(window.kind, RangeKind::Today);
    assert!(window.start_utc.is_some());
    assert!(window.end_utc.is_some());
    // All three fixture events fall on 2026-09-11 Taipei local.
    assert_eq!(filtered.len(), 3);
    let table = PriceTable::embedded();
    let summary = price_with_range(&filtered, &table, None, Some(&window));
    assert!((summary.total - 0.17).abs() <= 0.01);
    assert_eq!(summary.range.as_ref().unwrap().kind, RangeKind::Today);
}

#[test]
fn series_emits_zero_days_and_sums_to_total() {
    let events: Vec<UsageEvent> =
        serde_json::from_str(&read_fixture("events.json")).unwrap();
    let table = PriceTable::embedded();
    let now = Utc.with_ymd_and_hms(2026, 9, 11, 12, 0, 0).unwrap();
    let series = daily_spend_series(
        &events,
        &table,
        None,
        &SeriesOpts {
            day_boundary: DayBoundary::Local,
            timezone: Some("Asia/Taipei"),
            now,
            range_kind: RangeKind::Custom,
            from: Some("2026-09-09"),
            to: Some("2026-09-11"),
        },
    )
    .unwrap();
    assert_eq!(series.grain, "day");
    assert_eq!(series.points.len(), 3);
    assert_eq!(series.points[0].date, "2026-09-09");
    assert!((series.points[0].amount - 0.0).abs() < 1e-9);
    assert_eq!(series.points[1].date, "2026-09-10");
    assert!((series.points[1].amount - 0.0).abs() < 1e-9);
    assert_eq!(series.points[2].date, "2026-09-11");
    let sum: f64 = series.points.iter().map(|p| p.amount).sum();
    assert!((sum - 0.17).abs() <= 0.01);

    let (filtered, window) = filter_events_by_range(
        &events,
        &RangeFilterOpts {
            kind: RangeKind::Custom,
            day_boundary: DayBoundary::Local,
            timezone: Some("Asia/Taipei"),
            now,
            custom_from: Some(chrono::NaiveDate::from_ymd_opt(2026, 9, 9).unwrap()),
            custom_to: Some(chrono::NaiveDate::from_ymd_opt(2026, 9, 11).unwrap()),
        },
    )
    .unwrap();
    let total = price_with_range(&filtered, &table, None, Some(&window));
    assert!((sum - total.total).abs() <= 0.01);
}

#[test]
fn import_meta_roundtrip_state_file() {
    let dir = std::env::temp_dir().join(format!(
        "token-tracer-import-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let state = dir.join("import-meta.json");

    let empty = read_import_meta(&state).unwrap();
    assert!(empty.last_imported_at.is_none());
    assert_eq!(empty.schema_version, "import-meta/v0");

    let written = record_import(
        &state,
        vec!["EC-claude-code-v1".into(), "EC-codex-v1".into()],
        3,
        0,
        Some(state.display().to_string()),
    )
    .unwrap();
    assert!(written.last_imported_at.is_some());
    assert_eq!(written.events_upserted, 3);
    assert_eq!(written.parse_error_count, 0);

    let loaded = read_import_meta(&state).unwrap();
    assert_eq!(loaded.events_upserted, 3);
    assert_eq!(loaded.last_import_source_ids.len(), 2);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn unknown_model_priced_zero() {
    let table = PriceTable::embedded();
    let events = vec![UsageEvent {
        id: "u".into(),
        agent: AgentId::Other,
        source: "t".into(),
        ts: "2026-09-11T00:00:00.000Z".into(),
        model: Some("totally-unknown-model-xyz".into()),
        usage_pool: None,
        input_tokens: 999_999,
        output_tokens: 999_999,
        cache_read_tokens: None,
        cache_write_tokens: None,
        raw_cost_usd: None,
        meta: None,
    }];
    let s = price(&events, &table, None);
    assert!((s.total - 0.0).abs() < 1e-9);
    assert_eq!(
        s.meta
            .as_ref()
            .unwrap()
            .get("unknown_model_events")
            .and_then(|v| v.as_u64()),
        Some(1)
    );
}


fn fixtures_v13() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/ac-v1.3")
}

fn fixtures_v13a() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/ac-v1.3a")
}

#[test]
fn ac_f11_by_model_fixture() {
    let text = std::fs::read_to_string(fixtures_v13().join("by-model.json")).unwrap();
    let events: Vec<UsageEvent> = serde_json::from_str(&text).unwrap();
    let table = PriceTable::embedded();
    let s = price(&events, &table, None);
    assert_eq!(s.pricing_mode, PricingMode::NotionalApiEstimate);
    assert!(s.disclaimer.to_lowercase().contains("notional"));
    assert!(s.by_model.len() >= 2);
    // concrete models present
    assert!(s.by_model.iter().any(|m| m.model == "claude-sonnet-4-20250514"));
    assert!(s.by_model.iter().any(|m| m.model == "gpt-5-codex"));
    // other_models pool with concrete model id
    let other = s
        .by_model
        .iter()
        .find(|m| m.usage_pool == Some(UsagePool::OtherModels))
        .expect("usage_pool=other_models row");
    assert_eq!(other.model, "claude-4.6-opus-high-thinking");
    assert!(!other.model.contains("Other"));
    assert_ne!(other.model, "__other__");
    assert_ne!(other.model, "cursor:other");
    let json = serde_json::to_string(&s).unwrap();
    assert!(!json.contains("__other__"));
    assert!(!json.contains("cursor:other"));
    let sum: f64 = s
        .by_model
        .iter()
        .filter(|m| m.priced)
        .map(|m| m.amount.unwrap_or(0.0))
        .sum();
    assert!((sum - s.total).abs() <= 0.01);
}

#[test]
fn ac_f13_cursor_pools_fixture() {
    let text = std::fs::read_to_string(fixtures_v13a().join("cursor-pools.json")).unwrap();
    let events: Vec<UsageEvent> = serde_json::from_str(&text).unwrap();
    let table = PriceTable::embedded();
    let s = price(&events, &table, None);
    assert!(s.by_usage_pool.iter().any(|p| p.pool == UsagePool::CursorModels));
    assert!(s.by_usage_pool.iter().any(|p| p.pool == UsagePool::OtherModels));
    for m in &s.by_model {
        assert_ne!(m.model, "Other");
        assert_ne!(m.model, "Other model");
        assert_ne!(m.model, "__other__");
        assert_ne!(m.model, "cursor:other");
    }
    // grok + other_models both distinct
    let grok = s.by_model.iter().find(|m| m.model == "grok-4.6").expect("grok");
    assert_eq!(grok.usage_pool, Some(UsagePool::OtherModels));
    let composer = s.by_model.iter().find(|m| m.model == "composer-1.5").expect("composer");
    assert_eq!(composer.usage_pool, Some(UsagePool::CursorModels));
    let json = serde_json::to_string(&s).unwrap();
    assert!(!json.contains("__other__"));
}

#[test]
fn cursor_weird_slug_unpriced_unless_vendor() {
    let table = PriceTable::embedded();
    let events = vec![UsageEvent {
        id: "w".into(),
        agent: AgentId::Cursor,
        source: "t".into(),
        ts: "2026-09-11T00:00:00.000Z".into(),
        model: Some("some-weird-slug".into()),
        usage_pool: Some(UsagePool::Unknown),
        input_tokens: 1000,
        output_tokens: 1000,
        cache_read_tokens: None,
        cache_write_tokens: None,
        raw_cost_usd: None,
        meta: Some(serde_json::json!({"cost_nature": "vendor_reported"})),
    }];
    // vendor_reported without raw_cost → still not priced via table
    let s = price(&events, &table, None);
    let row = s.by_model.iter().find(|m| m.model == "some-weird-slug").expect("slug");
    assert!(!row.priced);
    assert!(row.amount.is_none());
    assert_ne!(row.model, "__other__");

    let events2 = vec![UsageEvent {
        id: "w2".into(),
        agent: AgentId::Cursor,
        source: "t".into(),
        ts: "2026-09-11T00:00:00.000Z".into(),
        model: Some("some-weird-slug".into()),
        usage_pool: Some(UsagePool::OtherModels),
        input_tokens: 1000,
        output_tokens: 1000,
        cache_read_tokens: None,
        cache_write_tokens: None,
        raw_cost_usd: Some(0.77),
        meta: Some(serde_json::json!({"cost_nature": "vendor_reported"})),
    }];
    let s2 = price(&events2, &table, None);
    let row2 = s2.by_model.iter().find(|m| m.model == "some-weird-slug").unwrap();
    assert!(row2.priced);
    assert!((row2.amount.unwrap() - 0.77).abs() < 0.01);
    assert_eq!(row2.usage_pool, Some(UsagePool::OtherModels));
}

#[test]
fn cursor_missing_model_is_cursor_unknown_not_dunder_other() {
    let table = PriceTable::embedded();
    let events = vec![UsageEvent {
        id: "m".into(),
        agent: AgentId::Cursor,
        source: "t".into(),
        ts: "2026-09-11T00:00:00.000Z".into(),
        model: None,
        usage_pool: Some(UsagePool::Unknown),
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: None,
        cache_write_tokens: None,
        raw_cost_usd: Some(0.1),
        meta: Some(serde_json::json!({"cost_nature": "vendor_reported"})),
    }];
    let s = price(&events, &table, None);
    assert_eq!(s.by_model[0].model, CURSOR_UNKNOWN_MODEL);
    assert!(!serde_json::to_string(&s).unwrap().contains("__other__"));
}

#[test]
fn forbid_other_literal_as_model() {
    let parsed = pricing::parsers::parse_cursor_local(
        r#"[{
            "id":"x","agent":"cursor","source":"t","ts":"2026-09-11T00:00:00.000Z",
            "model":"Other","input_tokens":0,"output_tokens":0,
            "raw_cost_usd":1.0,
            "meta":{"cost_nature":"vendor_reported","tier":1}
        }]"#,
    )
    .unwrap();
    assert!(parsed[0].model.is_none());
    assert_eq!(parsed[0].usage_pool, Some(UsagePool::OtherModels));
}

#[test]
fn ac_v1_by_model_breakdown() {
    let events: Vec<UsageEvent> =
        serde_json::from_str(&read_fixture("events.json")).unwrap();
    let table = PriceTable::embedded();
    let s = price(&events, &table, None);
    assert!(s.by_model.len() >= 2);
    let sum: f64 = s
        .by_model
        .iter()
        .filter(|m| m.priced)
        .map(|m| m.amount.unwrap_or(0.0))
        .sum();
    assert!((sum - s.total).abs() <= 0.01);
    assert!((sum - 0.17).abs() <= 0.01);
}

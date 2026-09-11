//! OPEN-BIND spending-align v0 — CLI + state file (AC-F17 comparison layer).
//!
//! Hard bans: notional SpendSummary / by_usage_pool.amount must not drive pct;
//! Official Admin reconcile cents must not land in pct fields.

use pricing::{
    parse_teams_spend, price, read_spending_align, set_spending_align_from_json, AgentId,
    PriceTable, SpendingAlignSourceMode, PERSONAL_NO_PUBLIC_USAGE_API_MSG,
    SPENDING_ALIGN_SCHEMA_VERSION,
};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/ac-v1.4")
}

fn pools_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/ac-v1.3a/cursor-pools.json")
}

fn read_fix(name: &str) -> String {
    std::fs::read_to_string(fixtures_dir().join(name))
        .unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
}

static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

fn tmp_workdir() -> PathBuf {
    let n = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "tt-spending-align-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&dir).expect("mkdir tmp");
    dir
}

fn spend_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spend"))
}

fn run_spend(args: &[&str]) -> (bool, String, String) {
    let out = spend_bin()
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("spawn spend: {e}"));
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn assert_open_bind_shape(v: &Value) {
    assert_eq!(v["schema_version"], SPENDING_ALIGN_SCHEMA_VERSION);
    assert!(v.get("cursor_models_pct").is_some());
    assert!(v.get("other_models_pct").is_some());
    assert!(v.get("reset_label").is_some());
    assert!(v.get("on_demand").is_some());
    assert!(v.get("source_mode").is_some());
    assert!(v.get("partial").is_some());
    assert!(v.get("partial_reason").is_some());
    assert!(v.get("grok_bot_week_note").is_some());
    assert!(v.get("computed_at").is_some());
    assert!(v.get("cursor_models_percent").is_none());
    assert!(v.get("input_tokens").is_none());
    assert!(v.get("raw_cost_usd").is_none());
    assert!(v.get("local_enrichment").is_none());
    assert_ne!(v["source_mode"], "local_enrichment");
}

#[test]
fn fixture_matches_ui_mock_shape() {
    let v: Value = serde_json::from_str(&read_fix("spending-align-manual-p1.json")).unwrap();
    assert_open_bind_shape(&v);
    assert_eq!(v["source_mode"], "manual_p1");
    assert!((v["cursor_models_pct"].as_f64().unwrap() - 65.0).abs() < 1e-9);
    assert!((v["other_models_pct"].as_f64().unwrap() - 100.0).abs() < 1e-9);
    assert_eq!(v["partial"], true);
    let reason = v["partial_reason"].as_str().unwrap();
    assert!(
        reason.contains(PERSONAL_NO_PUBLIC_USAGE_API_MSG),
        "{reason}"
    );
    assert_eq!(v["on_demand"], "Disabled");
    assert_eq!(v["reset_label"], "≈ 9/12");
}

#[test]
fn lib_set_then_read_round_trip() {
    let dir = tmp_workdir();
    let state = dir.join("spending-align.json");
    let payload = read_fix("spending-align-manual-p1.json");

    let written = set_spending_align_from_json(&state, &payload).expect("set");
    assert_eq!(written.source_mode, SpendingAlignSourceMode::ManualP1);
    assert_eq!(written.cursor_models_pct, Some(65.0));
    assert_eq!(written.other_models_pct, Some(100.0));
    assert!(written.partial);
    assert_ne!(
        written.computed_at, "2026-09-11T09:00:00Z",
        "set must stamp computed_at"
    );
    assert!(state.is_file());

    let read = read_spending_align(&state).expect("read");
    assert_eq!(read.schema_version, written.schema_version);
    assert_eq!(read.source_mode, written.source_mode);
    assert_eq!(read.cursor_models_pct, written.cursor_models_pct);
    assert_eq!(read.other_models_pct, written.other_models_pct);
    assert_eq!(read.reset_label, written.reset_label);
    assert_eq!(read.on_demand, written.on_demand);
    assert_eq!(read.partial, written.partial);
    assert_eq!(read.partial_reason, written.partial_reason);
    assert_eq!(read.grok_bot_week_note, written.grok_bot_week_note);
    assert_eq!(read.computed_at, written.computed_at);
}

#[test]
fn missing_state_emits_none_placeholder() {
    let state = tmp_workdir().join("missing.json");
    assert!(!state.exists());
    let align = read_spending_align(&state).expect("placeholder");
    assert_eq!(align.source_mode, SpendingAlignSourceMode::None);
    assert_eq!(align.schema_version, SPENDING_ALIGN_SCHEMA_VERSION);
    assert!(align.cursor_models_pct.is_none());
    assert!(align.other_models_pct.is_none());
    assert!(align.reset_label.is_none());
    assert!(align.on_demand.is_none());
    assert!(align.partial);
    let reason = align.partial_reason.expect("reason");
    assert!(
        reason.contains("尚無") || reason.to_lowercase().contains("no align"),
        "{reason}"
    );
    assert!(
        reason.contains(PERSONAL_NO_PUBLIC_USAGE_API_MSG),
        "{reason}"
    );
    assert!(!align.computed_at.is_empty());
}

#[test]
fn notional_and_admin_cents_do_not_drive_pct() {
    let table = PriceTable::embedded();
    let events: Vec<pricing::UsageEvent> =
        serde_json::from_str(&std::fs::read_to_string(pools_fixture()).unwrap()).unwrap();
    let summary = price(&events, &table, None);
    assert!(summary.total > 0.0, "notional total must be nonzero");
    assert!(
        summary
            .by_usage_pool
            .iter()
            .any(|p| p.amount.unwrap_or(0.0) > 0.0),
        "pool amounts must be nonzero so a mistaken f(amount)→pct would be visible"
    );
    let cursor = summary
        .by_agent
        .iter()
        .find(|a| a.agent == AgentId::Cursor)
        .expect("cursor row");
    assert!(cursor.amount > 0.0);

    let spend = parse_teams_spend(&read_fix("admin-spend.json")).unwrap();
    assert!((spend.overall_spend_cents - 2000.0).abs() < 1e-9);

    let dir = tmp_workdir();
    let missing = dir.join("no-align.json");
    let before = read_spending_align(&missing).unwrap();
    assert_eq!(before.source_mode, SpendingAlignSourceMode::None);
    assert!(before.cursor_models_pct.is_none());
    assert!(before.other_models_pct.is_none());

    let state = dir.join("spending-align.json");
    let written =
        set_spending_align_from_json(&state, &read_fix("spending-align-manual-p1.json")).unwrap();
    let after_notional = read_spending_align(&state).unwrap();
    assert_eq!(after_notional.cursor_models_pct, Some(65.0));
    assert_eq!(after_notional.other_models_pct, Some(100.0));
    assert_eq!(
        after_notional.source_mode,
        SpendingAlignSourceMode::ManualP1
    );

    // Ban: loading notional / Admin cents after set must not change stored pct.
    let _ = price(&events, &table, None);
    let _ = parse_teams_spend(&read_fix("admin-spend.json")).unwrap();
    let again = read_spending_align(&state).unwrap();
    assert_eq!(again.cursor_models_pct, written.cursor_models_pct);
    assert_eq!(again.other_models_pct, written.other_models_pct);
    assert_ne!(again.cursor_models_pct, Some(spend.overall_spend_cents));
    assert_ne!(again.other_models_pct, Some(spend.overall_spend_cents));
    for pool in &summary.by_usage_pool {
        if let Some(amt) = pool.amount {
            assert_ne!(again.cursor_models_pct, Some(amt));
            assert_ne!(again.other_models_pct, Some(amt));
        }
    }
}

#[test]
fn set_rejects_notional_spend_summary_json() {
    let dir = tmp_workdir();
    let state = dir.join("spending-align.json");
    let events: Vec<pricing::UsageEvent> =
        serde_json::from_str(&std::fs::read_to_string(pools_fixture()).unwrap()).unwrap();
    let summary = price(&events, &PriceTable::embedded(), None);
    let json = serde_json::to_string_pretty(&summary).unwrap();
    let err = set_spending_align_from_json(&state, &json).expect_err("notional must not set align");
    assert!(!state.exists(), "failed set must not create state");
    let msg = err.to_string();
    assert!(
        msg.contains("schema_version")
            || msg.contains("source_mode")
            || msg.contains("missing")
            || msg.contains("json"),
        "{msg}"
    );
}

#[test]
fn cli_set_then_read_round_trip() {
    let dir = tmp_workdir();
    let state = dir.join("spending-align.json");
    let fixture = fixtures_dir().join("spending-align-manual-p1.json");

    let (ok, stdout, stderr) = run_spend(&[
        "spending-align",
        "set",
        "--json",
        fixture.to_str().unwrap(),
        "--state",
        state.to_str().unwrap(),
    ]);
    assert!(ok, "set failed: {stderr}\n{stdout}");
    let set_v: Value = serde_json::from_str(&stdout).expect("set JSON");
    assert_open_bind_shape(&set_v);
    assert_eq!(set_v["source_mode"], "manual_p1");
    assert!((set_v["cursor_models_pct"].as_f64().unwrap() - 65.0).abs() < 1e-9);
    assert!((set_v["other_models_pct"].as_f64().unwrap() - 100.0).abs() < 1e-9);
    assert_eq!(set_v["partial"], true);
    assert!(state.is_file());

    let (ok, stdout, stderr) = run_spend(&[
        "spending-align",
        "--json",
        "--state",
        state.to_str().unwrap(),
    ]);
    assert!(ok, "read failed: {stderr}\n{stdout}");
    let read_v: Value = serde_json::from_str(&stdout).expect("read JSON");
    assert_open_bind_shape(&read_v);
    assert_eq!(read_v["source_mode"], set_v["source_mode"]);
    assert_eq!(read_v["cursor_models_pct"], set_v["cursor_models_pct"]);
    assert_eq!(read_v["other_models_pct"], set_v["other_models_pct"]);
    assert_eq!(read_v["computed_at"], set_v["computed_at"]);
}

#[test]
fn cli_missing_state_none_placeholder() {
    let state = tmp_workdir().join("does-not-exist.json");
    let (ok, stdout, stderr) = run_spend(&[
        "spending-align",
        "--json",
        "--state",
        state.to_str().unwrap(),
    ]);
    assert!(ok, "read missing failed: {stderr}\n{stdout}");
    let v: Value = serde_json::from_str(&stdout).expect("placeholder JSON");
    assert_open_bind_shape(&v);
    assert_eq!(v["source_mode"], "none");
    assert!(v["cursor_models_pct"].is_null());
    assert!(v["other_models_pct"].is_null());
    assert_eq!(v["partial"], true);
    let reason = v["partial_reason"].as_str().unwrap_or("");
    assert!(
        reason.contains(PERSONAL_NO_PUBLIC_USAGE_API_MSG),
        "{reason}"
    );
    assert!(!Path::new(&state).exists());
}

#[test]
fn cli_help_lists_spending_align() {
    let (ok, stdout, stderr) = run_spend(&["spending-align", "--help"]);
    assert!(ok, "help failed: {stderr}\n{stdout}");
    let text = format!("{stdout}{stderr}");
    assert!(
        text.contains("spending-align") || text.contains("SpendingAlign"),
        "{text}"
    );
    assert!(text.contains("--state"), "{text}");
    assert!(text.contains(".token-tracer/spending-align.json"), "{text}");
}

#[test]
fn cli_set_rejects_local_enrichment() {
    let dir = tmp_workdir();
    let bad = dir.join("bad.json");
    std::fs::write(
        &bad,
        r#"{
          "schema_version": "spending-align/v0",
          "cursor_models_pct": 10,
          "other_models_pct": null,
          "reset_label": null,
          "on_demand": null,
          "source_mode": "local_enrichment",
          "partial": true,
          "partial_reason": null,
          "grok_bot_week_note": null,
          "computed_at": "2026-09-11T00:00:00Z"
        }"#,
    )
    .unwrap();
    let state = dir.join("spending-align.json");
    let (ok, stdout, stderr) = run_spend(&[
        "spending-align",
        "set",
        "--json",
        bad.to_str().unwrap(),
        "--state",
        state.to_str().unwrap(),
    ]);
    assert!(!ok, "local_enrichment must fail set: {stdout}");
    let msg = format!("{stdout}{stderr}");
    assert!(
        msg.contains("local_enrichment")
            || msg.contains("not allowed")
            || msg.contains("source_mode"),
        "{msg}"
    );
    assert!(!state.exists());
}

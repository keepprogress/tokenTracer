//! Integration: spend import from-discover vs bridge fixture paths.

use pricing::{
    import_from_discover_result, read_import_meta, DiscoverError, DiscoverResult, DiscoverSource,
    FromDiscoverOpts,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn bridge_fixtures() -> PathBuf {
    PathBuf::from("/workspace/tokenTracer-bridge/tests/fixtures")
}

fn tmp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tt-from-discover-{}-{}",
        tag,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn claude_win_source(root: &str) -> DiscoverSource {
    DiscoverSource {
        id: format!("EC-claude-code-v1:windows:{root}"),
        agent: "claude_code".into(),
        evidence_card: Some("EC-claude-code-v1".into()),
        host: "windows".into(),
        root_path: root.into(),
        glob: "projects/**/*.jsonl".into(),
        files: None,
        file_count: 2,
        truncated: None,
        readable: true,
        status: "ok".into(),
        meta: None,
    }
}

fn codex_win_source(root: &str) -> DiscoverSource {
    DiscoverSource {
        id: format!("EC-codex-v1:windows:{root}"),
        agent: "codex".into(),
        evidence_card: Some("EC-codex-v1".into()),
        host: "windows".into(),
        root_path: root.into(),
        glob: "**/rollout-*.jsonl".into(),
        files: None,
        file_count: 1,
        truncated: None,
        readable: true,
        status: "ok".into(),
        meta: None,
    }
}

#[test]
fn from_discover_bridge_fixtures_events_and_meta() {
    let fixtures = bridge_fixtures();
    if !fixtures.is_dir() {
        eprintln!("skip: bridge fixtures missing at {}", fixtures.display());
        return;
    }

    let claude_root = fixtures.join("win_home/.claude");
    let codex_root = fixtures.join("win_home/.codex/sessions");
    let discover = DiscoverResult {
        discovered_at: "2026-09-11T00:00:00Z".into(),
        host_os: "linux".into(),
        sources: vec![
            claude_win_source(&claude_root.to_string_lossy()),
            codex_win_source(&codex_root.to_string_lossy()),
        ],
        errors: vec![],
    };

    let dir = tmp_dir("ok");
    let state = dir.join("import-meta.json");
    let events_out = dir.join("events.json");
    let report_out = dir.join("report.json");

    let result = import_from_discover_result(
        &discover,
        &FromDiscoverOpts {
            discover_path: dir.join("unused.json"),
            limit: 0,
            state_path: state.clone(),
            events_out: Some(events_out.clone()),
            report_out: Some(report_out.clone()),
        },
    )
    .expect("import");

    assert!(
        result.report.events_upserted > 0,
        "expected non-empty events, got {}",
        result.report.events_upserted
    );
    assert!(result.events.iter().any(|e| e.agent.as_str() == "claude_code"));
    assert!(result.events.iter().any(|e| e.agent.as_str() == "codex"));
    assert!(result
        .events
        .iter()
        .all(|e| e.meta.as_ref().and_then(|m| m.get("host_os")).is_some()));
    assert_eq!(result.report.truncated_sources.len(), 0);
    assert!(!result.exit_nonzero);

    let meta = read_import_meta(&state).unwrap();
    assert!(meta.last_imported_at.is_some());
    assert_eq!(meta.events_upserted, result.report.events_upserted);
    assert_eq!(meta.last_import_source_ids.len(), 2);

    let loaded: Vec<serde_json::Value> =
        serde_json::from_str(&fs::read_to_string(&events_out).unwrap()).unwrap();
    assert_eq!(loaded.len() as u64, result.report.events_upserted);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn truncated_source_does_not_silently_import_partial() {
    let fixtures = bridge_fixtures();
    if !fixtures.is_dir() {
        eprintln!("skip: bridge fixtures missing");
        return;
    }
    let claude_root = fixtures.join("win_home/.claude");
    let f1 = claude_root
        .join("projects/encoded-cwd/session-win.jsonl")
        .to_string_lossy()
        .to_string();
    // Pretend discover returned only 1 of 2 files with truncated:true
    let mut src = claude_win_source(&claude_root.to_string_lossy());
    src.files = Some(vec![f1]);
    src.file_count = 2;
    src.truncated = Some(true);

    let discover = DiscoverResult {
        discovered_at: "2026-09-11T00:00:00Z".into(),
        host_os: "linux".into(),
        sources: vec![src.clone()],
        errors: vec![DiscoverError::truncated_list(Some(src.id.clone()))],
    };

    let dir = tmp_dir("trunc");
    let result = import_from_discover_result(
        &discover,
        &FromDiscoverOpts {
            discover_path: dir.join("unused.json"),
            limit: 0,
            state_path: dir.join("import-meta.json"),
            events_out: Some(dir.join("events.json")),
            report_out: None,
        },
    )
    .expect("import");

    assert!(
        result.report.truncated_sources.contains(&src.id),
        "truncated source must be listed"
    );
    assert_eq!(
        result.report.events_upserted, 0,
        "must not import truncated files[] as complete"
    );
    assert!(result.exit_nonzero);
    assert!(result
        .report
        .errors
        .iter()
        .any(|e| e.code == "TT-F2-006"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn local_expand_limit_truncates_and_fails_source() {
    let fixtures = bridge_fixtures();
    if !fixtures.is_dir() {
        eprintln!("skip: bridge fixtures missing");
        return;
    }
    let claude_root = fixtures.join("win_home/.claude");
    let src = claude_win_source(&claude_root.to_string_lossy());
    let discover = DiscoverResult {
        discovered_at: "2026-09-11T00:00:00Z".into(),
        host_os: "linux".into(),
        sources: vec![src.clone()],
        errors: vec![],
    };

    let dir = tmp_dir("limit");
    let result = import_from_discover_result(
        &discover,
        &FromDiscoverOpts {
            discover_path: dir.join("unused.json"),
            limit: 1, // 2 files on disk → truncate
            state_path: dir.join("import-meta.json"),
            events_out: None,
            report_out: None,
        },
    )
    .expect("import");

    assert!(result.report.truncated_sources.contains(&src.id));
    assert_eq!(result.report.events_upserted, 0);
    assert!(result.exit_nonzero);
    assert!(result
        .report
        .errors
        .iter()
        .any(|e| e.code == "TT-F2-006"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn cursor_source_skipped_local_billing() {
    let src = DiscoverSource {
        id: "EC-cursor-v1:windows:/tmp/cursor".into(),
        agent: "cursor".into(),
        evidence_card: Some("EC-cursor-v1".into()),
        host: "windows".into(),
        root_path: "/tmp/cursor".into(),
        glob: "state.vscdb".into(),
        files: None,
        file_count: 1,
        truncated: None,
        readable: true,
        status: "partial".into(),
        meta: None,
    };
    let discover = DiscoverResult {
        discovered_at: "2026-09-11T00:00:00Z".into(),
        host_os: "linux".into(),
        sources: vec![src.clone()],
        errors: vec![],
    };
    let dir = tmp_dir("cursor");
    let result = import_from_discover_result(
        &discover,
        &FromDiscoverOpts {
            discover_path: dir.join("unused.json"),
            limit: 0,
            state_path: dir.join("import-meta.json"),
            events_out: None,
            report_out: None,
        },
    )
    .expect("import");

    assert_eq!(result.report.events_upserted, 0);
    assert!(result
        .report
        .source_ids
        .contains(&src.id));
    let skips = result.report.cursor_sources_skipped.unwrap();
    assert_eq!(skips[0].note, "cursor_skipped_local_billing");
    assert!(!result.exit_nonzero);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn wsl_import_path_meta_preferred() {
    let fixtures = bridge_fixtures();
    if !fixtures.is_dir() {
        return;
    }
    let import_path = fixtures
        .join("wsl_ubuntu/home/t3261/.claude")
        .to_string_lossy()
        .to_string();
    let mut meta = HashMap::new();
    meta.insert("import_path".into(), import_path.clone());
    meta.insert("posix_path".into(), "/home/t3261/.claude".into());
    meta.insert("distro".into(), "Ubuntu-Work".into());

    let src = DiscoverSource {
        id: "EC-claude-code-v1:wsl2:wsl:Ubuntu-Work:/home/t3261/.claude".into(),
        agent: "claude_code".into(),
        evidence_card: Some("EC-claude-code-v1".into()),
        host: "wsl2".into(),
        root_path: "wsl:Ubuntu-Work:/home/t3261/.claude".into(),
        glob: "projects/**/*.jsonl".into(),
        files: None,
        file_count: 1,
        truncated: None,
        readable: true,
        status: "ok".into(),
        meta: Some(meta),
    };
    let discover = DiscoverResult {
        discovered_at: "2026-09-11T00:00:00Z".into(),
        host_os: "linux".into(),
        sources: vec![src],
        errors: vec![],
    };
    let dir = tmp_dir("wsl");
    let result = import_from_discover_result(
        &discover,
        &FromDiscoverOpts {
            discover_path: dir.join("unused.json"),
            limit: 0,
            state_path: dir.join("import-meta.json"),
            events_out: None,
            report_out: None,
        },
    )
    .expect("import");

    assert!(result.report.events_upserted >= 1);
    assert!(result
        .events
        .iter()
        .any(|e| e.meta.as_ref().and_then(|m| m.get("host_os")).and_then(|v| v.as_str())
            == Some("wsl2")));

    let _ = fs::remove_dir_all(&dir);
}

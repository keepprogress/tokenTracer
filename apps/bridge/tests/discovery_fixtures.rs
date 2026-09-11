use std::collections::HashSet;
use std::path::PathBuf;
use tokentracer_bridge::errors::catalog;
use tokentracer_bridge::types::{AgentId, HostOs, SourceHost, SourceStatus};
use tokentracer_bridge::{
    config_from_fixture_root, discover_paths, list_files_for_source, DiscoverConfig,
};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn fixture_discover_finds_all_three_agents() {
    let result = discover_paths(&config_from_fixture_root(&fixtures()));
    assert_eq!(result.host_os, HostOs::Linux);

    let agents: HashSet<_> = result
        .sources
        .iter()
        .filter(|s| {
            (s.status == SourceStatus::Ok || s.status == SourceStatus::Partial) && s.file_count > 0
        })
        .map(|s| s.agent)
        .collect();

    assert!(agents.contains(&AgentId::ClaudeCode), "{:?}", result.sources);
    assert!(agents.contains(&AgentId::Codex), "{:?}", result.sources);
    assert!(agents.contains(&AgentId::Cursor), "{:?}", result.sources);
}

#[test]
fn dual_scan_emits_windows_and_wsl_for_claude_and_codex() {
    let result = discover_paths(&config_from_fixture_root(&fixtures()));

    let claude_hosts: HashSet<_> = result
        .sources
        .iter()
        .filter(|s| s.agent == AgentId::ClaudeCode && s.file_count > 0)
        .map(|s| s.host)
        .collect();
    assert!(claude_hosts.contains(&SourceHost::Windows));
    assert!(claude_hosts.contains(&SourceHost::Wsl2));

    let codex_hosts: HashSet<_> = result
        .sources
        .iter()
        .filter(|s| s.agent == AgentId::Codex && s.file_count > 0)
        .map(|s| s.host)
        .collect();
    assert!(codex_hosts.contains(&SourceHost::Windows));
    assert!(codex_hosts.contains(&SourceHost::Wsl2));
}

#[test]
fn source_id_format_is_card_host_canonical_root() {
    let result = discover_paths(&config_from_fixture_root(&fixtures()));
    for s in &result.sources {
        let parts: Vec<_> = s.id.splitn(3, ':').collect();
        assert_eq!(parts.len(), 3, "id={}", s.id);
        assert_eq!(parts[0], s.evidence_card.as_str());
        assert_eq!(parts[1], s.host.as_str());
        assert_eq!(parts[2], s.root_path.as_str());
        assert!(!s.root_path.ends_with('/') || s.root_path.len() <= 3);
        assert!(!s.root_path.contains('\\'));
    }

    let wsl_claude = result
        .sources
        .iter()
        .find(|s| s.agent == AgentId::ClaudeCode && s.host == SourceHost::Wsl2)
        .expect("wsl claude");
    assert_eq!(
        wsl_claude.id,
        "EC-claude-code-v1:wsl2:wsl:Ubuntu-Work:/home/t3261/.claude"
    );
    assert_eq!(
        wsl_claude.root_path,
        "wsl:Ubuntu-Work:/home/t3261/.claude"
    );
    let meta = wsl_claude.meta.as_ref().expect("meta");
    assert_eq!(meta.get("distro").map(String::as_str), Some("Ubuntu-Work"));
    assert_eq!(
        meta.get("posix_path").map(String::as_str),
        Some("/home/t3261/.claude")
    );
    assert!(meta.contains_key("unc_path") || meta.contains_key("access_unc"));
    // UNC must not be the sole id / root_path
    assert!(!wsl_claude.root_path.contains("wsl$"));
    assert!(!wsl_claude.id.contains("wsl$"));
}

#[test]
fn default_discover_omits_files_array() {
    let result = discover_paths(&config_from_fixture_root(&fixtures()));
    for s in &result.sources {
        assert!(s.files.is_none(), "files should be omitted by default: {}", s.id);
        assert!(s.truncated.is_none());
    }
}

#[test]
fn list_files_flag_populates_files() {
    let mut cfg = config_from_fixture_root(&fixtures());
    cfg.list_files = true;
    let result = discover_paths(&cfg);
    let any = result
        .sources
        .iter()
        .any(|s| s.files.as_ref().map(|f| !f.is_empty()).unwrap_or(false));
    assert!(any, "expected some files when list_files=true");
}

#[test]
fn files_expand_limit_zero_returns_all_fixture_files() {
    let cfg = config_from_fixture_root(&fixtures());
    let win_claude = "EC-claude-code-v1:windows:".to_string()
        + &cfg
            .win_user_profile
            .as_ref()
            .unwrap()
            .join(".claude")
            .display()
            .to_string()
            .replace('\\', "/");
    // Prefer stable WSL id (absolute canonical, fixture-independent)
    let wsl_id = "EC-claude-code-v1:wsl2:wsl:Ubuntu-Work:/home/t3261/.claude";
    let result = list_files_for_source(&cfg, wsl_id, 0).expect("expand");
    assert!(!result.truncated);
    assert_eq!(result.files.len() as u64, result.file_count);
    assert!(result.file_count >= 1);
    assert!(result.errors.is_empty());

    // Win Claude fixture has 2 jsonl files — uncapped returns both
    let win_src = discover_paths(&cfg)
        .sources
        .into_iter()
        .find(|s| s.agent == AgentId::ClaudeCode && s.host == SourceHost::Windows)
        .expect("win claude");
    let all = list_files_for_source(&cfg, &win_src.id, 0).expect("win expand");
    assert_eq!(all.file_count, 2);
    assert_eq!(all.files.len(), 2);
    assert!(!all.truncated);
    let _ = win_claude; // constructed for clarity; id comes from discover
}

#[test]
fn files_expand_limit_one_sets_truncated_when_more_exist() {
    let cfg = config_from_fixture_root(&fixtures());
    let win_src = discover_paths(&cfg)
        .sources
        .into_iter()
        .find(|s| s.agent == AgentId::ClaudeCode && s.host == SourceHost::Windows)
        .expect("win claude");
    assert!(win_src.file_count >= 2, "fixture needs ≥2 files for truncate test");

    let capped = list_files_for_source(&cfg, &win_src.id, 1).expect("expand");
    assert!(capped.truncated);
    assert_eq!(capped.files.len(), 1);
    assert!(capped.file_count > 1);
    assert!(capped.errors.iter().any(|e| e.code == "TT-F2-006"));
}

#[test]
fn cursor_stays_partial_and_warns_tokencount() {
    let result = discover_paths(&config_from_fixture_root(&fixtures()));
    let cursor = result
        .sources
        .iter()
        .find(|s| s.agent == AgentId::Cursor && s.file_count > 0)
        .expect("cursor source");
    assert_eq!(cursor.status, SourceStatus::Partial);
    let meta = cursor.meta.as_ref().expect("meta");
    let blob = format!("{meta:?}").to_ascii_lowercase();
    assert!(blob.contains("tokencount") || blob.contains("billed") || blob.contains("partial"));
}

#[test]
fn missing_path_emits_tt_f2_003_with_next_step() {
    let mut cfg = config_from_fixture_root(&fixtures());
    cfg.win_user_profile = Some(fixtures().join("does-not-exist-profile"));
    let result = discover_paths(&cfg);
    assert!(result.errors.iter().any(|e| e.code == "TT-F2-003"));
    for e in &result.errors {
        assert!(!e.next_step.is_empty());
    }
}

#[test]
fn error_catalog_contains_required_codes() {
    let codes: HashSet<_> = catalog().iter().map(|c| c.code).collect();
    for required in [
        "TT-F1-001",
        "TT-F1-002",
        "TT-F1-003",
        "TT-F2-001",
        "TT-F2-002",
        "TT-F2-003",
        "TT-F2-004",
        "TT-F2-005",
        "TT-F2-006",
    ] {
        assert!(codes.contains(required), "missing {required}");
    }
}

#[test]
fn discover_result_serializes_to_contract_json() {
    let result = discover_paths(&config_from_fixture_root(&fixtures()));
    let v = serde_json::to_value(&result).expect("json");
    assert!(v.get("discovered_at").is_some());
    assert_eq!(v["host_os"], "linux");
    assert!(v["sources"].is_array());
    assert!(v["errors"].is_array());
    let src = &v["sources"][0];
    for key in [
        "id",
        "agent",
        "evidence_card",
        "host",
        "root_path",
        "glob",
        "file_count",
        "readable",
        "status",
    ] {
        assert!(src.get(key).is_some(), "missing {key}");
    }
    assert!(src.get("files").is_none(), "files omitted by default in JSON");
}

#[test]
fn empty_config_does_not_panic() {
    let cfg = DiscoverConfig {
        host_os: HostOs::Linux,
        require_wsl_scan: false,
        ..DiscoverConfig::default()
    };
    let result = discover_paths(&cfg);
    assert!(result.sources.is_empty());
}

#[test]
fn host_enum_matches_usage_event_meta_host_os() {
    // windows | wsl2 | macos — same strings as UsageEvent.meta.host_os
    assert_eq!(SourceHost::Windows.as_str(), "windows");
    assert_eq!(SourceHost::Wsl2.as_str(), "wsl2");
    assert_eq!(SourceHost::Macos.as_str(), "macos");
}

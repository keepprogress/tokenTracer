//! Dual-scan discovery for Claude Code, Codex, and Cursor (Win + WSL2).
//!
//! **Hard constraint (AC v1.1a):** Win and WSL install trees are separate —
//! discovery **must** scan both. Never silent-skip; emit diagnostic codes.
//! Specs: AC v1.1a ∪ v1.2b. Live probe: docs/live-probe-NB-T3261.md.
//! Contract: path-list-v0.2 — WSL ids embed `wsl:<Distro>:<posix>`.

use crate::errors::{make_error, TT_F2_001, TT_F2_002, TT_F2_003, TT_F2_006};
use crate::fsutil::probe_glob;
use crate::types::{
    canonical_root, make_source_id, make_source_id_from_canonical, wsl_canonical_root, AgentId,
    DiscoverConfig, DiscoverError, DiscoverResult, DiscoverSource, EvidenceCard, FileListResult,
    HostOs, SourceHost, SourceStatus, WslDistroProbe, DISCOVER_FILES_CAP,
    FILES_EXPAND_DEFAULT_LIMIT, FILES_EXPAND_WARN_THRESHOLD,
};
use crate::wsl::{try_live_wsl_list, wsl_unc_home};
use chrono::Utc;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const CLAUDE_GLOB: &str = "projects/**/*.jsonl";
const CODEX_SESSIONS_GLOB: &str = "sessions/**/rollout-*.jsonl";
const CODEX_ARCHIVED_GLOB: &str = "archived_sessions/rollout-*.jsonl";
const CURSOR_STATE_REL: &str = "Cursor/User/globalStorage/state.vscdb";
const CURSOR_WS_GLOB: &str = "**/state.vscdb";

/// Public entry: discover agent path sources for 帳本.
pub fn discover_paths(config: &DiscoverConfig) -> DiscoverResult {
    let mut sources: Vec<DiscoverSource> = Vec::new();
    let mut errors: Vec<DiscoverError> = Vec::new();

    // --- Windows native trees ---
    if let Some(profile) = &config.win_user_profile {
        scan_claude_root(
            &mut sources,
            &mut errors,
            &profile.join(".claude"),
            SourceHost::Windows,
            None,
            config,
        );
        scan_codex_root(
            &mut sources,
            &mut errors,
            &profile.join(".codex"),
            SourceHost::Windows,
            None,
            config,
        );
        scan_cursor_agent_home(
            &mut sources,
            &mut errors,
            &profile.join(".cursor"),
            SourceHost::Windows,
            config,
        );
    }

    if let Some(appdata) = &config.win_appdata {
        scan_cursor_appdata(&mut sources, &mut errors, appdata, SourceHost::Windows, config);
    }

    // --- WSL2 trees (mandatory dual-scan) ---
    let mut distros = config.wsl_distros.clone();
    if distros.is_empty() && config.host_os == HostOs::Windows {
        let live = try_live_wsl_list();
        errors.extend(live.errors);
        distros = live.distros;
    }

    if distros.is_empty() && config.require_wsl_scan {
        if config.host_os == HostOs::Linux
            && config.win_user_profile.is_some()
            && config.wsl_distros.is_empty()
        {
            errors.push(make_error(
                TT_F2_001,
                "No WSL distros injected for dual-scan on this Linux fixture host",
                None,
            ));
        } else if config.host_os == HostOs::Windows && config.wsl_distros.is_empty() {
            if !errors.iter().any(|e| e.code.starts_with("TT-F2-00")) {
                errors.push(make_error(
                    TT_F2_001,
                    "WSL dual-scan requested but no distros discovered",
                    None,
                ));
            }
        }
    }

    for distro in &distros {
        scan_wsl_distro(&mut sources, &mut errors, distro, config);
    }

    // --- macOS (document-level; live verification UNKNOWN per AC v1.2b) ---
    if let Some(home) = &config.macos_home {
        scan_macos(&mut sources, &mut errors, home, config);
    }

    DiscoverResult {
        discovered_at: Utc::now(),
        host_os: config.host_os,
        sources,
        errors,
    }
}

fn scan_wsl_distro(
    sources: &mut Vec<DiscoverSource>,
    errors: &mut Vec<DiscoverError>,
    distro: &WslDistroProbe,
    config: &DiscoverConfig,
) {
    let Some(user) = distro.wsl_user.clone() else {
        errors.push(make_error(
            TT_F2_002,
            format!(
                "WSL distro '{}': wsl_user unresolved — refusing to invent 'unknown' (path would be wrong)",
                distro.name
            ),
            Some(distro.linux_home.display().to_string()),
        ));
        return;
    };
    let home = &distro.linux_home;

    if !home.exists() {
        errors.push(make_error(
            TT_F2_003,
            format!(
                "WSL distro '{}' linux home not found at {}",
                distro.name,
                home.display()
            ),
            Some(home.display().to_string()),
        ));
        return;
    }

    // Stable CONTRACT id embeds distro: EC-claude-code-v1:wsl2:wsl:Ubuntu-Work:/home/t3261/.claude
    let logical_home = PathBuf::from(format!("/home/{user}"));
    let claude_logical = logical_home.join(".claude");
    let claude_canonical = wsl_canonical_root(&distro.name, &claude_logical);
    let claude_id =
        make_source_id_from_canonical(EvidenceCard::ClaudeCodeV1, SourceHost::Wsl2, &claude_canonical);
    if sources.iter().any(|s| s.id == claude_id) {
        // Same distro+logical home already emitted.
        return;
    }

    let mut base_meta = BTreeMap::new();
    base_meta.insert("distro".into(), distro.name.clone());
    base_meta.insert("wsl_user".into(), user.clone());
    base_meta.insert("wsl_state".into(), distro.state.clone());
    if let Some(v) = &distro.version {
        base_meta.insert("wsl_version".into(), v.clone());
    }
    let unc = wsl_unc_home(&distro.name, &user);
    base_meta.insert("unc_path".into(), unc.clone());
    base_meta.insert("access_unc".into(), unc);
    base_meta.insert("access_exec".into(), format!("wsl -d {} --", distro.name));

    let before = sources.len();
    scan_claude_root(
        sources,
        errors,
        &home.join(".claude"),
        SourceHost::Wsl2,
        Some(base_meta.clone()),
        config,
    );
    scan_codex_root(
        sources,
        errors,
        &home.join(".codex"),
        SourceHost::Wsl2,
        Some(base_meta),
        config,
    );
    rewrite_wsl_canonical_ids(&mut sources[before..], home, &logical_home, &distro.name);
}

/// After probing via mount/UNC, rewrite root_path + id to
/// `wsl:<Distro>:<posix-abs>`. Default `meta.import_path` to the **posix** path
/// (same as `meta.posix_path`) so WSL-side `spend import from-discover` works
/// without remapping. Keep UNC in `meta.unc_path` / `meta.access_unc` for Win open.
fn rewrite_wsl_canonical_ids(
    sources: &mut [DiscoverSource],
    mount_home: &Path,
    logical_home: &Path,
    distro: &str,
) {
    let mount_prefix = canonical_root(mount_home);
    let logical_prefix = canonical_root(logical_home);
    for s in sources.iter_mut() {
        if s.host != SourceHost::Wsl2 {
            continue;
        }
        if s.root_path.starts_with(&mount_prefix) {
            let suffix = &s.root_path[mount_prefix.len()..];
            let mut posix_root = format!("{logical_prefix}{suffix}");
            while posix_root.len() > 1 && posix_root.ends_with('/') {
                posix_root.pop();
            }
            if posix_root.is_empty() {
                posix_root = logical_prefix.clone();
            }

            let mut meta = s.meta.take().unwrap_or_default();
            // Default import_path to posix so importers inside the distro need no remap.
            meta.insert("import_path".into(), posix_root.clone());
            meta.insert("posix_path".into(), posix_root.clone());
            meta.insert("distro".into(), distro.to_string());

            s.root_path = wsl_canonical_root(distro, Path::new(&posix_root));
            s.id = make_source_id_from_canonical(s.evidence_card, s.host, &s.root_path);
            s.meta = Some(meta);
        }
    }
}

fn scan_claude_root(
    sources: &mut Vec<DiscoverSource>,
    errors: &mut Vec<DiscoverError>,
    claude_root: &Path,
    host: SourceHost,
    meta: Option<BTreeMap<String, String>>,
    config: &DiscoverConfig,
) {
    // canonical_root for id is the agent config root (~/.claude), glob under it.
    push_source(
        sources,
        errors,
        AgentId::ClaudeCode,
        host,
        claude_root,
        CLAUDE_GLOB,
        meta,
        config,
    );
}

fn scan_codex_root(
    sources: &mut Vec<DiscoverSource>,
    errors: &mut Vec<DiscoverError>,
    codex_root: &Path,
    host: SourceHost,
    meta: Option<BTreeMap<String, String>>,
    config: &DiscoverConfig,
) {
    // Prefer distinct roots: ~/.codex/sessions and ~/.codex/archived_sessions.
    let sessions = codex_root.join("sessions");
    push_source(
        sources,
        errors,
        AgentId::Codex,
        host,
        &sessions,
        "**/rollout-*.jsonl",
        meta.clone(),
        config,
    );
    let archived = codex_root.join("archived_sessions");
    push_source(
        sources,
        errors,
        AgentId::Codex,
        host,
        &archived,
        "rollout-*.jsonl",
        meta,
        config,
    );
}

fn scan_cursor_appdata(
    sources: &mut Vec<DiscoverSource>,
    errors: &mut Vec<DiscoverError>,
    appdata: &Path,
    host: SourceHost,
    config: &DiscoverConfig,
) {
    let mut meta = BTreeMap::new();
    meta.insert(
        "billing_note".into(),
        "NEVER treat bubble tokenCount as billed usage; prefer API/CSV/Admin for spend".into(),
    );
    meta.insert("support_status".into(), "partial".into());
    meta.insert("evidence".into(), "EC-cursor-v1".into());

    // Root = globalStorage directory containing state.vscdb
    let global = appdata.join("Cursor").join("User").join("globalStorage");
    push_source(
        sources,
        errors,
        AgentId::Cursor,
        host,
        &global,
        "state.vscdb",
        Some(meta.clone()),
        config,
    );

    let workspace = appdata.join("Cursor").join("User").join("workspaceStorage");
    push_source(
        sources,
        errors,
        AgentId::Cursor,
        host,
        &workspace,
        CURSOR_WS_GLOB,
        Some(meta),
        config,
    );
}

fn scan_cursor_agent_home(
    sources: &mut Vec<DiscoverSource>,
    errors: &mut Vec<DiscoverError>,
    cursor_home: &Path,
    host: SourceHost,
    config: &DiscoverConfig,
) {
    let mut m = BTreeMap::new();
    m.insert(
        "note".into(),
        "~/.cursor trees are enrichment (ai-tracking/chats); not authoritative billed tokens".into(),
    );
    m.insert("support_status".into(), "partial".into());
    let tracking = cursor_home.join("ai-tracking");
    push_source(
        sources,
        errors,
        AgentId::Cursor,
        host,
        &tracking,
        "*.db",
        Some(m),
        config,
    );
}

fn scan_macos(
    sources: &mut Vec<DiscoverSource>,
    errors: &mut Vec<DiscoverError>,
    home: &Path,
    config: &DiscoverConfig,
) {
    let mut note = BTreeMap::new();
    note.insert(
        "live_mac_verification".into(),
        "UNKNOWN — AC v1.2b document paths only; no live Mac dump".into(),
    );

    // Claude: ~/.claude/projects/**/*.jsonl → root ~/.claude
    let mut m = note.clone();
    m.insert("ac_v1_2b".into(), "CONFIRMED* path/format".into());
    scan_claude_root(
        sources,
        errors,
        &home.join(".claude"),
        SourceHost::Macos,
        Some(m),
        config,
    );

    // Codex: ~/.codex/sessions/**/rollout-*.jsonl
    let mut m = note.clone();
    m.insert("ac_v1_2b".into(), "CONFIRMED* path/format".into());
    scan_codex_root(
        sources,
        errors,
        &home.join(".codex"),
        SourceHost::Macos,
        Some(m),
        config,
    );

    // Cursor: ~/Library/Application Support/Cursor/User/globalStorage/state.vscdb
    let mut m = note;
    m.insert("ac_v1_2b".into(), "PARTIAL — path CONFIRMED; billable not local".into());
    m.insert(
        "billing_note".into(),
        "NEVER treat bubble tokenCount as billed usage".into(),
    );
    let global = home
        .join("Library")
        .join("Application Support")
        .join("Cursor")
        .join("User")
        .join("globalStorage");
    push_source(
        sources,
        errors,
        AgentId::Cursor,
        SourceHost::Macos,
        &global,
        "state.vscdb",
        Some(m),
        config,
    );

    let _ = CURSOR_STATE_REL;
    let _ = CODEX_SESSIONS_GLOB;
    let _ = CODEX_ARCHIVED_GLOB;
}

fn push_source(
    sources: &mut Vec<DiscoverSource>,
    errors: &mut Vec<DiscoverError>,
    agent: AgentId,
    host: SourceHost,
    root: &Path,
    glob: &str,
    meta: Option<BTreeMap<String, String>>,
    config: &DiscoverConfig,
) {
    let probe = probe_glob(root, glob, config.file_list_cap);
    if let Some(err) = probe.error {
        errors.push(err);
    }
    if config.list_files && probe.truncated {
        errors.push(make_error(
            TT_F2_006,
            format!(
                "files[] truncated at cap {} for {} (file_count={})",
                config.file_list_cap,
                root.display(),
                probe.file_count
            ),
            Some(root.display().to_string()),
        ));
    }

    // Cursor local sources stay partial even when readable (AC v1.1a).
    let status = if agent == AgentId::Cursor && probe.status == SourceStatus::Ok {
        SourceStatus::Partial
    } else {
        probe.status
    };

    let (files, truncated) = if config.list_files {
        let files = if probe.files.is_empty() {
            None
        } else {
            Some(probe.files)
        };
        (files, Some(probe.truncated))
    } else {
        (None, None)
    };

    sources.push(DiscoverSource {
        id: make_source_id(agent.evidence_card(), host, root),
        agent,
        evidence_card: agent.evidence_card(),
        host,
        root_path: canonical_root(root),
        glob: glob.to_string(),
        files,
        file_count: probe.file_count,
        truncated,
        readable: probe.readable,
        status,
        meta,
    });
}

/// Expand files for a single source id (CONTRACT expand path for 帳本 import).
///
/// `limit`:
/// - omit / use [`FILES_EXPAND_DEFAULT_LIMIT`] (500) via CLI default
/// - `0` = unlimited; if `file_count > 5000` a warning is set (and stderr by CLI)
/// - `N > 0` = hard cap; truncated listings set `truncated: true` + TT-F2-006
pub fn list_files_for_source(
    config: &DiscoverConfig,
    source_id: &str,
    limit: usize,
) -> Result<FileListResult, DiscoverError> {
    let mut cfg = config.clone();
    cfg.list_files = true;
    cfg.file_list_cap = limit; // 0 = unlimited
    let result = discover_paths(&cfg);
    if let Some(src) = result.sources.iter().find(|s| s.id == source_id) {
        let truncated = src.truncated.unwrap_or(false);
        let files = src.files.clone().unwrap_or_default();
        let mut errors: Vec<DiscoverError> = Vec::new();
        if truncated {
            errors.push(make_error(
                TT_F2_006,
                format!(
                    "File list truncated at limit {limit} for source {source_id} (file_count={})",
                    src.file_count
                ),
                Some(src.root_path.clone()),
            ));
        }
        let warning = if limit == 0 && src.file_count > FILES_EXPAND_WARN_THRESHOLD {
            Some(format!(
                "Unlimited expand returned {} files (> {}); consider paging or confirming import scope",
                src.file_count, FILES_EXPAND_WARN_THRESHOLD
            ))
        } else {
            None
        };
        Ok(FileListResult {
            source_id: source_id.to_string(),
            files,
            file_count: src.file_count,
            truncated,
            errors,
            warning,
        })
    } else {
        Err(make_error(
            TT_F2_003,
            format!("source id not found: {source_id}"),
            None,
        ))
    }
}

/// Fill host path defaults from environment when CLI flags / config fields are omitted.
///
/// - **Windows** (`host_os == Windows`): `USERPROFILE` → `win_user_profile`, `APPDATA` (Roaming) → `win_appdata`
/// - **macOS** (`host_os == Macos`): `HOME` → `macos_home`
///
/// No-op when values are already set. Safe to call on Linux with `host_os` set to Windows/Macos for tests.
pub fn apply_host_env_defaults(cfg: &mut DiscoverConfig) {
    match cfg.host_os {
        HostOs::Windows => {
            if cfg.win_user_profile.is_none() {
                if let Some(p) = std::env::var_os("USERPROFILE") {
                    if !p.is_empty() {
                        cfg.win_user_profile = Some(PathBuf::from(p));
                    }
                }
            }
            if cfg.win_appdata.is_none() {
                if let Some(p) = std::env::var_os("APPDATA") {
                    if !p.is_empty() {
                        cfg.win_appdata = Some(PathBuf::from(p));
                    }
                }
            }
        }
        HostOs::Macos => {
            if cfg.macos_home.is_none() {
                if let Some(p) = std::env::var_os("HOME") {
                    if !p.is_empty() {
                        cfg.macos_home = Some(PathBuf::from(p));
                    }
                }
            }
        }
        HostOs::Linux => {}
    }
}

/// Load fixture layout for Linux-box POC / tests.
pub fn config_from_fixture_root(fixture_root: &Path) -> DiscoverConfig {
    let distro_home = fixture_root.join("wsl_ubuntu").join("home").join("t3261");
    let distros = if distro_home.exists() {
        // NB-T3261 has Ubuntu-Work (Running) + Ubuntu (Stopped). Fixtures only
        // mount the Running default to avoid duplicate logical ids on one tree.
        vec![WslDistroProbe {
            name: "Ubuntu-Work".into(),
            state: "Running".into(),
            version: Some("2".into()),
            wsl_user: Some("t3261".into()),
            linux_home: distro_home,
        }]
    } else {
        Vec::new()
    };

    DiscoverConfig {
        host_os: HostOs::Linux,
        win_user_profile: Some(fixture_root.join("win_home")),
        win_appdata: Some(fixture_root.join("win_appdata")),
        macos_home: None,
        wsl_distros: distros,
        file_list_cap: DISCOVER_FILES_CAP,
        list_files: false,
        require_wsl_scan: true,
    }
}

pub fn discover_fixtures(fixture_root: impl AsRef<Path>) -> DiscoverResult {
    let cfg = config_from_fixture_root(fixture_root.as_ref());
    discover_paths(&cfg)
}

pub fn resolve_path(p: PathBuf) -> PathBuf {
    if p.is_absolute() {
        p
    } else if let Ok(cwd) = std::env::current_dir() {
        cwd.join(p)
    } else {
        p
    }
}

/// Default limit for `files --source-id` when caller does not override.
pub fn default_files_expand_limit() -> usize {
    FILES_EXPAND_DEFAULT_LIMIT
}


#[cfg(test)]
mod env_defaults_tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize env-mutating tests (process-global env).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn windows_defaults_fill_from_userprofile_and_appdata() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var("USERPROFILE", r"C:\Users\T3261");
        std::env::set_var("APPDATA", r"C:\Users\T3261\AppData\Roaming");
        let mut cfg = DiscoverConfig {
            host_os: HostOs::Windows,
            require_wsl_scan: false,
            ..DiscoverConfig::default()
        };
        apply_host_env_defaults(&mut cfg);
        assert_eq!(
            cfg.win_user_profile.as_deref(),
            Some(Path::new(r"C:\Users\T3261"))
        );
        assert_eq!(
            cfg.win_appdata.as_deref(),
            Some(Path::new(r"C:\Users\T3261\AppData\Roaming"))
        );
        // Explicit values win over env
        cfg.win_user_profile = Some(PathBuf::from(r"D:\Other"));
        apply_host_env_defaults(&mut cfg);
        assert_eq!(cfg.win_user_profile.as_deref(), Some(Path::new(r"D:\Other")));
        std::env::remove_var("USERPROFILE");
        std::env::remove_var("APPDATA");
    }

    #[test]
    fn macos_defaults_fill_from_home() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var("HOME", "/Users/demo");
        let mut cfg = DiscoverConfig {
            host_os: HostOs::Macos,
            require_wsl_scan: false,
            ..DiscoverConfig::default()
        };
        apply_host_env_defaults(&mut cfg);
        assert_eq!(cfg.macos_home.as_deref(), Some(Path::new("/Users/demo")));
        std::env::remove_var("HOME");
    }

    #[test]
    fn missing_wsl_user_emits_tt_f2_002_not_unknown() {
        let mut sources = Vec::new();
        let mut errors = Vec::new();
        let distro = WslDistroProbe {
            name: "Ubuntu-Work".into(),
            state: "Running".into(),
            version: Some("2".into()),
            wsl_user: None,
            linux_home: PathBuf::from(r"\\wsl$\Ubuntu-Work"),
        };
        let cfg = DiscoverConfig {
            require_wsl_scan: false,
            ..DiscoverConfig::default()
        };
        scan_wsl_distro(&mut sources, &mut errors, &distro, &cfg);
        assert!(sources.is_empty());
        assert!(errors.iter().any(|e| e.code == "TT-F2-002"));
        assert!(!errors.iter().any(|e| e.message.contains("unknown") && !e.message.contains("invent")));
    }
}

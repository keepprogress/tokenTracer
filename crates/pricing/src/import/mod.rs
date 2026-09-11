//! Discover → UsageEvent import pipeline (bridge path-list-v0.2).
//!
//! Performs filesystem I/O (read discover JSON, expand globs, parse agent
//! files, write events / ImportMeta). Pure `price()` stays I/O-free.

pub mod discover;
pub mod expand;

pub use discover::{
    files_list_complete, has_truncation_error, source_eligible, DiscoverError, DiscoverResult,
    DiscoverSource, FileListResult,
};
pub use expand::{expand_files_locally, resolve_import_root, strip_wsl_root};

use crate::import_meta::record_import;
use crate::models::{ImportMeta, UsageEvent};
use crate::parsers::{
    parse_claude_code_jsonl_with_source, parse_codex_rollout_jsonl_with_source,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("import: {0}")]
    Message(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportReport {
    pub events_upserted: u64,
    pub parse_error_count: u64,
    pub source_ids: Vec<String>,
    pub truncated_sources: Vec<String>,
    pub errors: Vec<DiscoverError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events_path: Option<String>,
    pub import_meta: ImportMeta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_sources_skipped: Option<Vec<CursorSkipNote>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CursorSkipNote {
    pub source_id: String,
    pub note: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct FromDiscoverOpts {
    pub discover_path: PathBuf,
    /// Cap on local expand / incidental files[]. `0` = unlimited.
    pub limit: u64,
    pub state_path: PathBuf,
    pub events_out: Option<PathBuf>,
    pub report_out: Option<PathBuf>,
}

#[derive(Debug)]
pub struct FromDiscoverResult {
    pub report: ImportReport,
    pub events: Vec<UsageEvent>,
    /// Non-zero recommended when truncated sources blocked a complete import.
    pub exit_nonzero: bool,
}

/// Load DiscoverResult JSON from a file (timestamps as strings).
pub fn load_discover_result(path: &Path) -> Result<DiscoverResult, ImportError> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

/// Import UsageEvents from a bridge DiscoverResult JSON file.
pub fn import_from_discover(opts: &FromDiscoverOpts) -> Result<FromDiscoverResult, ImportError> {
    let discover = load_discover_result(&opts.discover_path)?;
    import_from_discover_result(&discover, opts)
}

pub fn import_from_discover_result(
    discover: &DiscoverResult,
    opts: &FromDiscoverOpts,
) -> Result<FromDiscoverResult, ImportError> {
    let mut errors: Vec<DiscoverError> = discover.errors.clone();
    let mut truncated_sources: Vec<String> = Vec::new();
    let mut source_ids: Vec<String> = Vec::new();
    let mut cursor_skipped: Vec<CursorSkipNote> = Vec::new();
    let mut parse_error_count: u64 = 0;
    let mut by_id: HashMap<String, UsageEvent> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for src in &discover.sources {
        if !source_eligible(src) {
            continue;
        }

        // Truncated files[] from discover — never treat as complete.
        if src.truncated == Some(true)
            || has_truncation_error(src, &discover.errors)
            || source_errors_have_f2006(src, &errors)
        {
            truncated_sources.push(src.id.clone());
            if !errors.iter().any(|e| {
                e.code == "TT-F2-006" && e.path.as_deref() == Some(src.id.as_str())
            }) {
                errors.push(DiscoverError::truncated_list(Some(src.id.clone())));
            }
            continue;
        }

        // Cursor: do not parse bubble/DB as billing.
        if src.agent == "cursor" {
            cursor_skipped.push(CursorSkipNote {
                source_id: src.id.clone(),
                note: "cursor_skipped_local_billing".into(),
                status: "partial".into(),
            });
            source_ids.push(src.id.clone());
            continue;
        }

        let file_list = resolve_file_list(src, opts.limit, &mut errors, &mut truncated_sources);
        let Some(files) = file_list else {
            // truncated or unresolvable — already recorded
            continue;
        };

        match src.agent.as_str() {
            "claude_code" | "codex" => {
                for path in &files {
                    match parse_agent_file(src, path) {
                        Ok(events) => {
                            for mut ev in events {
                                attach_host_os(&mut ev, &src.host);
                                if !by_id.contains_key(&ev.id) {
                                    order.push(ev.id.clone());
                                    by_id.insert(ev.id.clone(), ev);
                                }
                            }
                        }
                        Err(msg) => {
                            parse_error_count += 1;
                            errors.push(DiscoverError {
                                code: "TT-F2-005".into(),
                                message: msg,
                                next_step: "Inspect transcript; fix parse or skip file".into(),
                                path: Some(path.clone()),
                            });
                        }
                    }
                }
                source_ids.push(src.id.clone());
            }
            other => {
                errors.push(DiscoverError {
                    code: "TT-F2-005".into(),
                    message: format!("Unsupported agent '{other}' for local import"),
                    next_step: "Skip or add parser".into(),
                    path: Some(src.id.clone()),
                });
            }
        }
    }

    let events: Vec<UsageEvent> = order
        .into_iter()
        .filter_map(|id| by_id.remove(&id))
        .collect();
    let events_upserted = events.len() as u64;

    let events_path = if let Some(ref out) = opts.events_out {
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(&events)?;
        fs::write(out, text)?;
        Some(out.display().to_string())
    } else {
        None
    };

    let import_meta = record_import(
        &opts.state_path,
        source_ids.clone(),
        events_upserted,
        parse_error_count,
        events_path.clone().or_else(|| Some(opts.state_path.display().to_string())),
    )
    .map_err(|e| ImportError::Message(e.to_string()))?;

    let report = ImportReport {
        events_upserted,
        parse_error_count,
        source_ids,
        truncated_sources: truncated_sources.clone(),
        errors,
        events_path,
        import_meta,
        cursor_sources_skipped: if cursor_skipped.is_empty() {
            None
        } else {
            Some(cursor_skipped)
        },
    };

    if let Some(ref report_out) = opts.report_out {
        if let Some(parent) = report_out.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(report_out, serde_json::to_string_pretty(&report)?)?;
    }

    let exit_nonzero = !truncated_sources.is_empty();
    Ok(FromDiscoverResult {
        report,
        events,
        exit_nonzero,
    })
}

fn source_errors_have_f2006(src: &DiscoverSource, errors: &[DiscoverError]) -> bool {
    // If discover embedded files that look truncated without flag but TT-F2-006
    // appears globally without path, only treat as truncated when source.truncated.
    let _ = (src, errors);
    false
}

/// Resolve the file list for a source. Returns `None` if truncated / failed
/// (caller must not import as complete).
fn resolve_file_list(
    src: &DiscoverSource,
    limit: u64,
    errors: &mut Vec<DiscoverError>,
    truncated_sources: &mut Vec<String>,
) -> Option<Vec<String>> {
    if files_list_complete(src) {
        let files = src.files.clone().unwrap_or_default();
        // Optional local cap when limit > 0 and discover gave a full list larger than limit
        if limit > 0 && files.len() as u64 > limit {
            truncated_sources.push(src.id.clone());
            errors.push(DiscoverError::truncated_list(Some(src.id.clone())));
            return None;
        }
        return Some(files);
    }

    // files present but incomplete / missing → local expand
    let outcome = expand_files_locally(src, limit);
    for e in &outcome.errors {
        if !errors.iter().any(|x| x.code == e.code && x.path == e.path) {
            errors.push(e.clone());
        }
    }
    if outcome.truncated {
        truncated_sources.push(src.id.clone());
        return None;
    }
    if outcome.import_root.is_none() && outcome.files.is_empty() {
        // root missing — already in errors; skip source (not truncated)
        return None;
    }
    Some(outcome.files)
}

fn parse_agent_file(src: &DiscoverSource, path: &str) -> Result<Vec<UsageEvent>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    match src.agent.as_str() {
        "claude_code" => parse_claude_code_jsonl_with_source(&text, &src.id)
            .map_err(|e| e.to_string()),
        "codex" => parse_codex_rollout_jsonl_with_source(&text, &src.id).map_err(|e| e.to_string()),
        _ => Err(format!("no parser for agent {}", src.agent)),
    }
}

fn attach_host_os(ev: &mut UsageEvent, host: &str) {
    let mut meta = ev
        .meta
        .take()
        .unwrap_or_else(|| serde_json::json!({}));
    if let Some(obj) = meta.as_object_mut() {
        obj.insert("host_os".into(), serde_json::Value::String(host.to_string()));
    }
    ev.meta = Some(meta);
}

/// Helper used by tests: build a minimal DiscoverSource.
pub fn test_source(
    id: &str,
    agent: &str,
    host: &str,
    root: &str,
    glob: &str,
    file_count: u64,
) -> DiscoverSource {
    DiscoverSource {
        id: id.into(),
        agent: agent.into(),
        evidence_card: Some(format!("EC-{}-v1", agent.replace('_', "-"))),
        host: host.into(),
        root_path: root.into(),
        glob: glob.into(),
        files: None,
        file_count,
        truncated: None,
        readable: true,
        status: "ok".into(),
        meta: None,
    }
}

/// Dedup helper exposed for unit tests.
pub fn dedup_events_by_id(events: Vec<UsageEvent>) -> Vec<UsageEvent> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for ev in events {
        if seen.insert(ev.id.clone()) {
            out.push(ev);
        }
    }
    out
}

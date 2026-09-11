//! ImportMeta state file helpers (OPEN-UI-3).
//!
//! I/O lives here only — pure pricing never calls these.

use crate::models::ImportMeta;
use chrono::Utc;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImportMetaError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn read_import_meta(path: &Path) -> Result<ImportMeta, ImportMetaError> {
    if !path.exists() {
        return Ok(ImportMeta::empty());
    }
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

pub fn write_import_meta(path: &Path, meta: &ImportMeta) -> Result<(), ImportMetaError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(meta)?;
    fs::write(path, text)?;
    Ok(())
}

/// Record a successful (possibly fake) import into the state file.
pub fn record_import(
    path: &Path,
    source_ids: Vec<String>,
    events_upserted: u64,
    parse_error_count: u64,
    ledger_path: Option<String>,
) -> Result<ImportMeta, ImportMetaError> {
    let meta = ImportMeta {
        last_imported_at: Some(
            Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        ),
        last_import_source_ids: source_ids,
        parse_error_count,
        events_upserted,
        ledger_path,
        schema_version: ImportMeta::SCHEMA_VERSION.to_string(),
    };
    write_import_meta(path, &meta)?;
    Ok(meta)
}

/// Default relative state path used by the CLI when none is given.
pub fn default_state_path() -> std::path::PathBuf {
    std::path::PathBuf::from(".token-tracer/import-meta.json")
}

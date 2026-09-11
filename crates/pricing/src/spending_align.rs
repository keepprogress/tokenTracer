//! SpendingAlign state I/O (OPEN-BIND spending-align v0 / AC-F17).
//!
//! Comparison layer only. This module **must not**:
//! - compute `*_pct` from notional `SpendSummary.total` or `by_usage_pool.amount`
//! - copy Official Admin reconcile cents into pct fields
//! - accept `local_enrichment` as `SpendingAlign.source_mode`
//!
//! I/O lives here (like `import_meta`); pure pricing never calls these.

use crate::models::{
    SpendingAlign, SpendingAlignSourceMode, DEFAULT_SPENDING_ALIGN_STATE,
    SPENDING_ALIGN_SCHEMA_VERSION,
};
use chrono::Utc;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub fn default_spending_align_state_path() -> PathBuf {
    PathBuf::from(DEFAULT_SPENDING_ALIGN_STATE)
}

#[derive(Debug, Error)]
pub enum SpendingAlignError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error(
        "invalid spending-align schema_version {found:?}; expected {expected}",
        expected = SPENDING_ALIGN_SCHEMA_VERSION
    )]
    SchemaVersion { found: String },
    #[error("{field} must be null or a finite number in 0..=100 (got {value})")]
    PctOutOfRange { field: &'static str, value: f64 },
    #[error(
        "source_mode {0:?} is not allowed on SpendingAlign \
         (local_enrichment must not appear; undocumented_dashboard maps to undocumented_opt_in)"
    )]
    ForbiddenSourceMode(String),
    #[error(
        "invalid source_mode {0:?}; expected none|manual_p1|official_admin|undocumented_opt_in"
    )]
    InvalidSourceMode(String),
    #[error("spend spending-align set requires source_mode=manual_p1 (manual path); got {0}")]
    SetRequiresManualP1(SpendingAlignSourceMode),
    #[error("spending-align JSON missing source_mode")]
    MissingSourceMode,
}

pub fn now_iso_utc() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Parse and schema-validate SpendingAlign JSON (read path).
pub fn parse_spending_align_json(text: &str) -> Result<SpendingAlign, SpendingAlignError> {
    let value: serde_json::Value = serde_json::from_str(text)?;
    reject_forbidden_source_mode(&value)?;
    let align: SpendingAlign = serde_json::from_value(value)?;
    validate_spending_align(&align)?;
    Ok(align)
}

fn reject_forbidden_source_mode(value: &serde_json::Value) -> Result<(), SpendingAlignError> {
    match value.get("source_mode") {
        None => Ok(()),
        Some(serde_json::Value::Null) => Err(SpendingAlignError::MissingSourceMode),
        Some(serde_json::Value::String(mode)) => match mode.as_str() {
            "none" | "manual_p1" | "official_admin" | "undocumented_opt_in" => Ok(()),
            "local_enrichment" | "undocumented_dashboard" => {
                Err(SpendingAlignError::ForbiddenSourceMode(mode.clone()))
            }
            other => Err(SpendingAlignError::InvalidSourceMode(other.to_string())),
        },
        Some(other) => Err(SpendingAlignError::InvalidSourceMode(other.to_string())),
    }
}

pub fn validate_pct(field: &'static str, pct: Option<f64>) -> Result<(), SpendingAlignError> {
    match pct {
        None => Ok(()),
        Some(v) if v.is_finite() && (0.0..=100.0).contains(&v) => Ok(()),
        Some(v) => Err(SpendingAlignError::PctOutOfRange { field, value: v }),
    }
}

pub fn validate_spending_align(align: &SpendingAlign) -> Result<(), SpendingAlignError> {
    if align.schema_version != SPENDING_ALIGN_SCHEMA_VERSION {
        return Err(SpendingAlignError::SchemaVersion {
            found: align.schema_version.clone(),
        });
    }
    validate_pct("cursor_models_pct", align.cursor_models_pct)?;
    validate_pct("other_models_pct", align.other_models_pct)?;
    Ok(())
}

pub fn validate_spending_align_for_set(align: &SpendingAlign) -> Result<(), SpendingAlignError> {
    validate_spending_align(align)?;
    if align.source_mode != SpendingAlignSourceMode::ManualP1 {
        return Err(SpendingAlignError::SetRequiresManualP1(align.source_mode));
    }
    Ok(())
}

/// Read state file. Missing path → `source_mode=none` placeholder (pcts null).
/// Does **not** consult SpendSummary / by_usage_pool.
pub fn read_spending_align(path: &Path) -> Result<SpendingAlign, SpendingAlignError> {
    if !path.exists() {
        return Ok(SpendingAlign::none_placeholder(now_iso_utc()));
    }
    let text = fs::read_to_string(path)?;
    parse_spending_align_json(&text)
}

fn atomic_write(path: &Path, contents: &str) -> io::Result<()> {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    if let Some(dir) = parent {
        fs::create_dir_all(dir)?;
    }
    let dir = parent.unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("spending-align.json");
    let tmp = dir.join(format!(".{file_name}.{}.tmp", std::process::id()));
    if let Err(e) = fs::write(&tmp, contents) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            Err(e)
        }
    }
}

pub fn write_spending_align(path: &Path, align: &SpendingAlign) -> Result<(), SpendingAlignError> {
    validate_spending_align(align)?;
    let text = serde_json::to_string_pretty(align)?;
    atomic_write(path, &text)?;
    Ok(())
}

/// Validate `manual_p1` payload, stamp `computed_at`, atomically write, return stored value.
pub fn set_spending_align_from_json(
    path: &Path,
    json: &str,
) -> Result<SpendingAlign, SpendingAlignError> {
    let mut align = parse_spending_align_json(json)?;
    validate_spending_align_for_set(&align)?;
    align.computed_at = now_iso_utc();
    write_spending_align(path, &align)?;
    Ok(align)
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn local_enrichment_is_forbidden_source_mode() {
        let err = parse_spending_align_json(
            r#"{
              "schema_version": "spending-align/v0",
              "source_mode": "local_enrichment",
              "partial": true
            }"#,
        )
        .unwrap_err();
        assert!(matches!(err, SpendingAlignError::ForbiddenSourceMode(_)));
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn pct_out_of_range_rejected() {
        let err = parse_spending_align_json(
            r#"{
              "schema_version": "spending-align/v0",
              "cursor_models_pct": 2000,
              "source_mode": "manual_p1",
              "partial": true
            }"#,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            SpendingAlignError::PctOutOfRange { value, .. } if (value - 2000.0).abs() < 1e-9
        ));
    }

    #[test]
    fn set_rejects_none_source_mode() {
        let align = SpendingAlign::none_placeholder("2026-09-11T00:00:00Z");
        let err = validate_spending_align_for_set(&align).unwrap_err();
        assert!(matches!(
            err,
            SpendingAlignError::SetRequiresManualP1(SpendingAlignSourceMode::None)
        ));
    }
}

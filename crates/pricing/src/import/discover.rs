//! Bridge DiscoverResult / FileListResult types (path-list-v0.2).
//!
//! Timestamps stay as strings so chrono ISO variants deserialize without a
//! custom visitor.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoverResult {
    /// ISO-8601 UTC (string; tolerant of chrono/bridge variants).
    pub discovered_at: String,
    pub host_os: String,
    #[serde(default)]
    pub sources: Vec<DiscoverSource>,
    #[serde(default)]
    pub errors: Vec<DiscoverError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoverSource {
    pub id: String,
    pub agent: String,
    #[serde(default)]
    pub evidence_card: Option<String>,
    pub host: String,
    pub root_path: String,
    pub glob: String,
    #[serde(default)]
    pub files: Option<Vec<String>>,
    #[serde(default)]
    pub file_count: u64,
    #[serde(default)]
    pub truncated: Option<bool>,
    pub readable: bool,
    pub status: String,
    #[serde(default)]
    pub meta: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoverError {
    pub code: String,
    pub message: String,
    pub next_step: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl DiscoverError {
    pub fn truncated_list(path: Option<String>) -> Self {
        Self {
            code: "TT-F2-006".into(),
            message: "File list truncated; do not treat as complete".into(),
            next_step: "Raise --limit or use --limit 0; do not treat list as complete".into(),
            path,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileListResult {
    pub source_id: String,
    pub files: Vec<String>,
    pub file_count: u64,
    pub truncated: bool,
    #[serde(default)]
    pub errors: Vec<DiscoverError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// True when this source should be walked by the importer.
pub fn source_eligible(src: &DiscoverSource) -> bool {
    src.readable && (src.status == "ok" || src.status == "partial")
}

/// Whether an embedded `files[]` is safe to treat as the complete set.
pub fn files_list_complete(src: &DiscoverSource) -> bool {
    if src.truncated == Some(true) {
        return false;
    }
    let Some(files) = src.files.as_ref() else {
        return false;
    };
    files.len() as u64 == src.file_count
}

/// Discover-level or source-scoped TT-F2-006 present for this source.
pub fn has_truncation_error(src: &DiscoverSource, global: &[DiscoverError]) -> bool {
    if src.truncated == Some(true) {
        return true;
    }
    let in_src_files = false; // no per-source errors field on DiscoverSource
    let _ = in_src_files;
    global.iter().any(|e| {
        e.code == "TT-F2-006"
            && e.path
                .as_ref()
                .map(|p| p == &src.id || p == &src.root_path || src.id.contains(p))
                .unwrap_or(false)
    })
}

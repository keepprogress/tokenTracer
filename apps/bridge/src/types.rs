//! Path-list API types shared with 帳本 (ledger). See CONTRACT-path-list-v0.md.
//! Specs in force: AC v1.1a ∪ v1.2b. Contract revision: path-list-v0.2.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Incidental `files[]` cap for discover / preview (`--list-files`).
pub const DISCOVER_FILES_CAP: usize = 32;
/// Product default for `files --source-id` expand (帳本 may pass `--limit 0` for full).
pub const FILES_EXPAND_DEFAULT_LIMIT: usize = 500;
/// When `--limit 0` (unlimited) returns more than this many files, emit a warn.
pub const FILES_EXPAND_WARN_THRESHOLD: u64 = 5000;

/// Host OS where discovery ran (importer perspective).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostOs {
    Windows,
    Macos,
    Linux,
}

impl HostOs {
    pub fn as_str(self) -> &'static str {
        match self {
            HostOs::Windows => "windows",
            HostOs::Macos => "macos",
            HostOs::Linux => "linux",
        }
    }
}

/// Agent identifiers (AC v1.1a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentId {
    ClaudeCode,
    Codex,
    Cursor,
}

impl AgentId {
    pub fn as_str(self) -> &'static str {
        match self {
            AgentId::ClaudeCode => "claude_code",
            AgentId::Codex => "codex",
            AgentId::Cursor => "cursor",
        }
    }

    pub fn evidence_card(self) -> EvidenceCard {
        match self {
            AgentId::ClaudeCode => EvidenceCard::ClaudeCodeV1,
            AgentId::Codex => EvidenceCard::CodexV1,
            AgentId::Cursor => EvidenceCard::CursorV1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceCard {
    #[serde(rename = "EC-claude-code-v1")]
    ClaudeCodeV1,
    #[serde(rename = "EC-codex-v1")]
    CodexV1,
    #[serde(rename = "EC-cursor-v1")]
    CursorV1,
}

impl EvidenceCard {
    pub fn as_str(self) -> &'static str {
        match self {
            EvidenceCard::ClaudeCodeV1 => "EC-claude-code-v1",
            EvidenceCard::CodexV1 => "EC-codex-v1",
            EvidenceCard::CursorV1 => "EC-cursor-v1",
        }
    }
}

/// Where source files live — MUST match UsageEvent.meta.host_os:
/// `"windows" | "wsl2" | "macos"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceHost {
    Windows,
    Wsl2,
    Macos,
}

impl SourceHost {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceHost::Windows => "windows",
            SourceHost::Wsl2 => "wsl2",
            SourceHost::Macos => "macos",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceStatus {
    Ok,
    Partial,
    Unsupported,
    Error,
}

/// Top-level discovery result for 帳本 consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverResult {
    pub discovered_at: DateTime<Utc>,
    pub host_os: HostOs,
    pub sources: Vec<DiscoverSource>,
    pub errors: Vec<DiscoverError>,
}

/// One discoverable agent data root + glob.
///
/// Default discover returns `root_path` + `glob` + `file_count` only.
/// `files` is omitted unless expand/`list_files` was requested.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverSource {
    /// Stable id: `{evidence_card}:{host}:{canonical_root}`
    /// WSL example: `EC-claude-code-v1:wsl2:wsl:Ubuntu-Work:/home/t3261/.claude`
    pub id: String,
    pub agent: AgentId,
    pub evidence_card: EvidenceCard,
    /// Same enum values as UsageEvent.meta.host_os.
    pub host: SourceHost,
    /// Absolute canonical root (no trailing slash; Windows uses `/`).
    /// For `host=wsl2`: `wsl:<Distro>:<posix-abs>` (distro embedded; NOT UNC).
    pub root_path: String,
    /// Relative glob under the probe/import root (see `meta.import_path` / `meta.posix_path`).
    pub glob: String,
    /// Optional concrete file list — only when expand requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,
    pub file_count: u64,
    /// Set when `files` was requested and the listing hit `file_list_cap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    pub readable: bool,
    pub status: SourceStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<BTreeMap<String, String>>,
}

/// Result of `files --source-id` / `list_files_for_source` expand path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListResult {
    pub source_id: String,
    pub files: Vec<String>,
    /// Total matches under the source glob (may exceed `files.len()` when truncated).
    pub file_count: u64,
    /// True when the returned `files` list was capped — importers MUST NOT treat as complete.
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<DiscoverError>,
    /// Non-fatal advisory (e.g. unlimited expand returned >5000 files).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// Diagnostic error with next_step (never silent skip).
/// TT-F2-* codes pass through to the import report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverError {
    pub code: String,
    pub message: String,
    pub next_step: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Injectable discovery configuration (real Win/WSL or Linux fixtures).
#[derive(Debug, Clone)]
pub struct DiscoverConfig {
    pub host_os: HostOs,
    pub win_user_profile: Option<PathBuf>,
    pub win_appdata: Option<PathBuf>,
    /// macOS home for document-level path discovery (live Mac = UNKNOWN).
    pub macos_home: Option<PathBuf>,
    pub wsl_distros: Vec<WslDistroProbe>,
    /// Cap for incidental `files[]` when `list_files` is true.
    /// `0` = unlimited. Discover default = [`DISCOVER_FILES_CAP`] (32).
    pub file_list_cap: usize,
    /// When true, populate `files` (subject to `file_list_cap`). Default false per CONTRACT.
    pub list_files: bool,
    pub require_wsl_scan: bool,
}

impl Default for DiscoverConfig {
    fn default() -> Self {
        Self {
            host_os: detect_host_os(),
            win_user_profile: None,
            win_appdata: None,
            macos_home: None,
            wsl_distros: Vec::new(),
            file_list_cap: DISCOVER_FILES_CAP,
            list_files: false,
            require_wsl_scan: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslDistroProbe {
    pub name: String,
    pub state: String,
    pub version: Option<String>,
    pub wsl_user: Option<String>,
    /// Absolute path on the bridge host mapping to the Linux home directory.
    pub linux_home: PathBuf,
}

pub fn detect_host_os() -> HostOs {
    if cfg!(target_os = "windows") {
        HostOs::Windows
    } else if cfg!(target_os = "macos") {
        HostOs::Macos
    } else {
        HostOs::Linux
    }
}

/// Normalize a filesystem path for native (windows/macos) `root_path`:
/// - forward slashes
/// - no trailing slash (except drive root like `C:/`)
pub fn canonical_root(path: &Path) -> String {
    let mut s = path.display().to_string().replace('\\', "/");
    while s.len() > 1 && s.ends_with('/') {
        // Keep "C:/" style
        if s.len() == 3 && s.as_bytes()[1] == b':' {
            break;
        }
        s.pop();
    }
    s
}

/// WSL canonical root embedded in `source.id` / `root_path`:
/// `wsl:<Distro>:<posix-abs>` — parseable, cross-platform, multi-distro safe.
/// UNC is NEVER the sole id (optional `meta.unc_path` for Win open).
pub fn wsl_canonical_root(distro: &str, posix_root: &Path) -> String {
    format!("wsl:{}:{}", distro, canonical_root(posix_root))
}

/// Build stable source id: `{evidence_card}:{host}:{canonical_root}`
pub fn make_source_id(card: EvidenceCard, host: SourceHost, root: &Path) -> String {
    format!("{}:{}:{}", card.as_str(), host.as_str(), canonical_root(root))
}

/// Build stable source id from an already-formed canonical_root string
/// (native path or `wsl:<Distro>:<posix>`).
pub fn make_source_id_from_canonical(card: EvidenceCard, host: SourceHost, canonical: &str) -> String {
    format!("{}:{}:{}", card.as_str(), host.as_str(), canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wsl_canonical_embeds_distro() {
        let c = wsl_canonical_root("Ubuntu-Work", Path::new("/home/t3261/.claude"));
        assert_eq!(c, "wsl:Ubuntu-Work:/home/t3261/.claude");
        let id = make_source_id_from_canonical(
            EvidenceCard::ClaudeCodeV1,
            SourceHost::Wsl2,
            &c,
        );
        assert_eq!(
            id,
            "EC-claude-code-v1:wsl2:wsl:Ubuntu-Work:/home/t3261/.claude"
        );
        let parts: Vec<_> = id.splitn(3, ':').collect();
        assert_eq!(parts[2], c);
    }
}

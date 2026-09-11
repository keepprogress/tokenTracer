//! tokenTracer bridge — Windows/WSL path discovery (AC-F1 / AC-F2 POC)
//! plus Cursor official Admin credential stub (AC v1.4-cursor-official).
//!
//! Dual-scan of Win + WSL agent trees is **mandatory** (AC v1.1a ∪ v1.2b).
//! Library crate for the Tauri host shell; CLI bin is separate for headless use.
//! Contract: path-list-v0.2; cursor-official-admin-v0.

pub mod cursor_admin;
pub mod discover;
pub mod errors;
pub mod fsutil;
pub mod types;
pub mod wsl;

pub use discover::{
    apply_host_env_defaults, config_from_fixture_root, default_files_expand_limit, discover_fixtures,
    discover_paths, list_files_for_source, resolve_path,
};
pub use types::{
    canonical_root, make_source_id, make_source_id_from_canonical, wsl_canonical_root, AgentId,
    DiscoverConfig, DiscoverError, DiscoverResult, DiscoverSource, EvidenceCard, FileListResult,
    HostOs, SourceHost, SourceStatus, WslDistroProbe, DISCOVER_FILES_CAP,
    FILES_EXPAND_DEFAULT_LIMIT, FILES_EXPAND_WARN_THRESHOLD,
};

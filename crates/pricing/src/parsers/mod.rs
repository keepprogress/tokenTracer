//! Agent-specific parsers → `UsageEvent`.
//!
//! I/O (reading files) lives at the edges; parsers accept `&str` / bytes so
//! the pricing core stays testable without the filesystem.

pub mod claude_code;
pub mod codex;
pub mod cursor;

pub use claude_code::{parse_claude_code_jsonl, parse_claude_code_jsonl_with_source};
pub use codex::{parse_codex_rollout_jsonl, parse_codex_rollout_jsonl_with_source};
pub use cursor::parse_cursor_local;

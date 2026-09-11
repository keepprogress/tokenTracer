//! Spawn the workspace `spend` CLI and parse JSON — same contract as
//! `apps/ui/scripts/spend-dev-bridge.mjs` (no invented prices).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SpendPaths {
    pub root: PathBuf,
    pub ranged: PathBuf,
    pub pool: PathBuf,
    pub model: PathBuf,
    pub import_state: PathBuf,
    /// OPEN-BIND SpendingAlign state (separate from ImportMeta).
    pub spending_align_state: PathBuf,
}

impl SpendPaths {
    pub fn resolve() -> Self {
        let root = resolve_ledger_root();
        Self {
            ranged: env_or(
                &root,
                "TOKENTRACER_EVENTS_RANGED",
                "fixtures/ac-v1.3a/cursor-pools-ranged.json",
            ),
            pool: env_or(
                &root,
                "TOKENTRACER_EVENTS_POOL",
                "fixtures/ac-v1.3a/cursor-pools.json",
            ),
            model: env_or(
                &root,
                "TOKENTRACER_EVENTS_MODEL",
                "fixtures/ac-v1.3/by-model.json",
            ),
            import_state: env_or(
                &root,
                "TOKENTRACER_IMPORT_STATE",
                ".token-tracer/import-meta.json",
            ),
            spending_align_state: env_or(
                &root,
                "TOKENTRACER_SPENDING_ALIGN_STATE",
                ".token-tracer/spending-align.json",
            ),
            root,
        }
    }
}

fn resolve_ledger_root() -> PathBuf {
    if let Ok(p) = std::env::var("TOKENTRACER_LEDGER_ROOT") {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("LEDGER_ROOT") {
        return PathBuf::from(p);
    }
    // apps/tokenTracer-host → repo root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn env_or(root: &Path, key: &str, rel: &str) -> PathBuf {
    if let Ok(v) = std::env::var(key) {
        let p = PathBuf::from(v);
        if p.is_absolute() {
            return p;
        }
        return root.join(p);
    }
    root.join(rel)
}

enum Launcher {
    Bin { path: PathBuf },
    Cargo,
}

fn resolve_launcher(root: &Path) -> Launcher {
    if let Ok(bin) = std::env::var("TOKENTRACER_SPEND_BIN") {
        let p = PathBuf::from(&bin);
        if p.exists() {
            return Launcher::Bin { path: p };
        }
    }
    let candidates = [
        "target/release/spend",
        "target/release/spend.exe",
        "target/debug/spend",
        "target/debug/spend.exe",
    ];
    for rel in candidates {
        let p = root.join(rel);
        if p.exists() {
            return Launcher::Bin { path: p };
        }
    }
    Launcher::Cargo
}

/// Extract first top-level JSON object/array from possibly mixed CLI stdout.
pub fn extract_json(raw: &str) -> Result<Value> {
    let start_obj = raw.find('{');
    let start_arr = raw.find('[');
    let start = match (start_obj, start_arr) {
        (None, None) => return Err(anyhow!("no JSON found in spend CLI stdout")),
        (Some(o), None) => o,
        (None, Some(a)) => a,
        (Some(o), Some(a)) => o.min(a),
    };
    let open = raw.as_bytes()[start] as char;
    let close = if open == '{' { '}' } else { ']' };
    let mut depth = 0i32;
    let mut in_str = false;
    let mut esc = false;
    let mut end = None;
    for (i, ch) in raw[start..].char_indices() {
        let idx = start + i;
        if in_str {
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == '"' {
                in_str = false;
            }
            continue;
        }
        if ch == '"' {
            in_str = true;
        } else if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                end = Some(idx + ch.len_utf8());
                break;
            }
        }
    }
    let end = end.ok_or_else(|| anyhow!("unterminated JSON in spend CLI stdout"))?;
    serde_json::from_str(&raw[start..end]).context("parse spend CLI JSON")
}

pub fn run_spend(cli_args: &[&str]) -> Result<Value> {
    let paths = SpendPaths::resolve();
    let launcher = resolve_launcher(&paths.root);
    let output = match &launcher {
        Launcher::Bin { path } => Command::new(path)
            .args(cli_args)
            .current_dir(&paths.root)
            .output()
            .with_context(|| format!("spawn spend bin {}", path.display()))?,
        Launcher::Cargo => Command::new("cargo")
            .args(["run", "-q", "-p", "pricing", "--bin", "spend", "--"])
            .args(cli_args)
            .current_dir(&paths.root)
            .output()
            .context("spawn cargo run spend")?,
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = {
            let s = stderr.trim();
            if s.is_empty() {
                stdout.trim()
            } else {
                s
            }
        };
        return Err(anyhow!("spend exited {}: {}", output.status, detail));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    extract_json(&stdout)
}


/// Soft timeout note: `Command::output` is synchronous; callers should keep
/// IPC work off the UI hot path when possible. (v0: acceptable for tray refresh.)
#[allow(dead_code)]
pub const SPEND_SOFT_TIMEOUT: Duration = Duration::from_secs(120);

pub fn spend_total(range: &str, currency: &str) -> Result<Value> {
    let paths = SpendPaths::resolve();
    run_spend(&[
        "by-pool",
        "--currency",
        currency,
        "--events",
        paths.ranged.to_str().unwrap_or_default(),
        "--range",
        range,
    ])
}

pub fn spend_series(grain: &str, range: &str, currency: &str) -> Result<Value> {
    if grain != "day" {
        return Err(anyhow!("v0 grain is day only"));
    }
    let paths = SpendPaths::resolve();
    run_spend(&[
        "series",
        "--grain",
        "day",
        "--currency",
        currency,
        "--events",
        paths.ranged.to_str().unwrap_or_default(),
        "--range",
        range,
    ])
}

pub fn spend_by_model(range: &str, currency: &str) -> Result<Value> {
    let paths = SpendPaths::resolve();
    run_spend(&[
        "by-model",
        "--currency",
        currency,
        "--events",
        paths.model.to_str().unwrap_or_default(),
        "--range",
        range,
    ])
}

pub fn spend_by_pool(range: &str, currency: &str) -> Result<Value> {
    let paths = SpendPaths::resolve();
    let events = if range == "all" {
        &paths.pool
    } else {
        &paths.ranged
    };
    run_spend(&[
        "by-pool",
        "--currency",
        currency,
        "--events",
        events.to_str().unwrap_or_default(),
        "--range",
        range,
    ])
}

pub fn import_status() -> Result<Value> {
    let paths = SpendPaths::resolve();
    run_spend(&[
        "import",
        "status",
        "--json",
        "--state",
        paths.import_state.to_str().unwrap_or_default(),
    ])
}

/// `spend spending-align --json [--state .token-tracer/spending-align.json]`
/// Comparison layer only — never invents pct from notional / by_usage_pool.
pub fn spending_align() -> Result<Value> {
    let paths = SpendPaths::resolve();
    run_spend(&[
        "spending-align",
        "--json",
        "--state",
        paths
            .spending_align_state
            .to_str()
            .unwrap_or(".token-tracer/spending-align.json"),
    ])
}

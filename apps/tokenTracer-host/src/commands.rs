//! SHELL-HOST v0.1 IPC commands exposed to the webview via `invoke`.

use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::panel::{PanelMode, PanelState};
use crate::spend_cli;

#[derive(Debug, Serialize)]
pub struct IpcError {
    pub error: String,
    pub source: &'static str,
}

impl IpcError {
    fn cli(e: impl ToString) -> Self {
        Self {
            error: e.to_string(),
            source: "cli",
        }
    }
    fn stub(msg: &str) -> Self {
        Self {
            error: msg.to_string(),
            source: "stub",
        }
    }
}

fn default_currency(c: Option<String>) -> String {
    c.unwrap_or_else(|| "USD".into())
}

fn default_range(r: Option<String>) -> String {
    r.unwrap_or_else(|| "all".into())
}

#[tauri::command]
pub fn spend_total(range: Option<String>, currency: Option<String>) -> Result<Value, IpcError> {
    let range = default_range(range);
    let currency = default_currency(currency);
    spend_cli::spend_total(&range, &currency).map_err(IpcError::cli)
}

#[tauri::command]
pub fn spend_series(
    grain: Option<String>,
    range: Option<String>,
    currency: Option<String>,
) -> Result<Value, IpcError> {
    let grain = grain.unwrap_or_else(|| "day".into());
    let range = range.unwrap_or_else(|| "30d".into());
    let currency = default_currency(currency);
    spend_cli::spend_series(&grain, &range, &currency).map_err(IpcError::cli)
}

#[tauri::command]
pub fn spend_by_model(
    currency: Option<String>,
    range: Option<String>,
) -> Result<Value, IpcError> {
    let range = default_range(range);
    let currency = default_currency(currency);
    spend_cli::spend_by_model(&range, &currency).map_err(IpcError::cli)
}

#[tauri::command]
pub fn spend_by_pool(
    currency: Option<String>,
    range: Option<String>,
) -> Result<Value, IpcError> {
    let range = default_range(range);
    let currency = default_currency(currency);
    spend_cli::spend_by_pool(&range, &currency).map_err(IpcError::cli)
}

#[tauri::command]
pub fn import_status() -> Result<Value, IpcError> {
    spend_cli::import_status().map_err(IpcError::cli)
}

/// B-surface: `spend spending-align --json` (OPEN-BIND). UI falls back to fixture on Err.
#[tauri::command]
pub fn spending_align() -> Result<Value, IpcError> {
    spend_cli::spending_align().map_err(IpcError::cli)
}

/// In-process bridge discover (tokentracer-bridge lib — not a subprocess).
#[tauri::command]
pub fn discover() -> Result<Value, IpcError> {
    let mut cfg = tokentracer_bridge::DiscoverConfig::default();
    tokentracer_bridge::apply_host_env_defaults(&mut cfg);
    let result = tokentracer_bridge::discover_paths(&cfg);
    serde_json::to_value(result).map_err(|e| IpcError {
        error: e.to_string(),
        source: "bridge",
    })
}

/// Stub — import_run writes ImportMeta via ledger only (OPEN-UI-3).
#[tauri::command]
pub fn import_run() -> Result<Value, IpcError> {
    Err(IpcError::stub(
        "import_run stub — host must call ledger record_import after bridge import (橋樑)",
    ))
}

#[tauri::command]
pub fn get_panel_mode(state: State<'_, PanelState>) -> String {
    state.mode.lock().unwrap().as_str().to_string()
}

#[tauri::command]
pub fn set_panel_mode(
    app: tauri::AppHandle,
    state: State<'_, PanelState>,
    mode: String,
) -> Result<(), String> {
    let next = PanelMode::parse(&mode).ok_or_else(|| format!("invalid mode: {mode}"))?;
    crate::panel::apply_mode(&app, &state, next).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn host_meta() -> Value {
    let paths = spend_cli::SpendPaths::resolve();
    let platform = std::env::consts::OS;
    let platform_note = match platform {
        "macos" => "macOS menu-bar status item (Tauri tray-icon / NSStatusItem equivalent) + dropdown mini-panel (AC-F7′ / AC-F10). Collapsed/Expanded IA shared with Win; not full-page web.",
        "windows" => "Windows tray + bottom-right mini-panel (AC-F7 / F7′).",
        _ => "Host shell: tray/menu-bar mini-panel; Linux GUI not an AC v1.2 install target.",
    };
    json!({
        "host": "tokentracer-host",
        "shell_host": "v0.1",
        "platform": platform,
        "platform_note": platform_note,
        "ledger_root": paths.root,
        "fixtures": {
            "ranged": paths.ranged,
            "pool": paths.pool,
            "model": paths.model,
            "import_state": paths.import_state,
            "spending_align_state": paths.spending_align_state,
        }
    })
}

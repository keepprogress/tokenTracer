//! tokenTracer Tauri 2 host — Windows system tray + bottom-right mini-panel.
//!
//! Package: `tokentracer-host` · Path: `apps/tokenTracer-host`
//! Depends on workspace crate `tokentracer-bridge` as an **in-process lib**
//! (`discover` → `discover_paths`). Spend IPC still shells out to the `spend` CLI
//! (same contract as Vite `/api/ipc`) until ledger is linked.

mod commands;
mod panel;
mod spend_cli;
mod tray;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(panel::PanelState::default())
        .invoke_handler(tauri::generate_handler![
            commands::spend_total,
            commands::spend_series,
            commands::spend_by_model,
            commands::spend_by_pool,
            commands::import_status,
            commands::discover,
            commands::import_run,
            commands::get_panel_mode,
            commands::set_panel_mode,
            commands::host_meta,
        ])
        .setup(|app| {
            if let Ok(win) = panel::main_window(app.handle()) {
                let _ = panel::position_bottom_right(&win);
                // Start default: tray resident + Collapsed visible (AC-F7 / SHELL-HOST).
                let _ = win.show();
            }

            tray::install(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Close → hide to tray (resident), do not quit.
                api.prevent_close();
                let _ = window.hide();
                if let Some(state) = window.try_state::<panel::PanelState>() {
                    *state.mode.lock().unwrap() = panel::PanelMode::Hidden;
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tokenTracer host");
}

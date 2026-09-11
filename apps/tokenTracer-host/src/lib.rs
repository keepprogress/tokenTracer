//! tokenTracer Tauri 2 host — Windows tray + macOS menu-bar mini-panel.
//!
//! Package: `tokentracer-host` · Path: `apps/tokenTracer-host`
//! Depends on workspace crate `tokentracer-bridge` as an **in-process lib**
//! (`discover` → `discover_paths`). Spend IPC still shells out to the `spend` CLI
//! (same contract as Vite `/api/ipc`) until ledger is linked.
//!
//! ## Platform chrome
//! - **Windows (AC-F7 / F7′)**: system tray + bottom-right mini-panel.
//! - **macOS (AC-F7′ / AC-F10)**: menu-bar status item (Tauri tray-icon /
//!   NSStatusItem equivalent) + dropdown mini-panel. Same Collapsed/Expanded IA
//!   as Windows; not a full-page web primary UI.

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
            commands::spending_align,
            commands::discover,
            commands::import_run,
            commands::get_panel_mode,
            commands::set_panel_mode,
            commands::host_meta,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                // Menu-bar accessory: no Dock icon (NSApplicationActivationPolicyAccessory).
                app.handle()
                    .set_activation_policy(tauri::ActivationPolicy::Accessory)?;
                if let Some(state) = app.try_state::<panel::PanelState>() {
                    *state.mode.lock().unwrap() = panel::PanelMode::Hidden;
                }
                if let Ok(win) = panel::main_window(app.handle()) {
                    let _ = win.hide();
                }
            }

            #[cfg(not(target_os = "macos"))]
            {
                // Start default: tray resident + Collapsed visible (AC-F7 / SHELL-HOST).
                if let Ok(win) = panel::main_window(app.handle()) {
                    let _ = panel::position_bottom_right(&win);
                    let _ = win.show();
                }
            }

            tray::install(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                // Close → hide to tray / menu-bar (resident), do not quit.
                api.prevent_close();
                let _ = window.hide();
                if let Some(state) = window.try_state::<panel::PanelState>() {
                    *state.mode.lock().unwrap() = panel::PanelMode::Hidden;
                }
            }
            #[cfg(target_os = "macos")]
            tauri::WindowEvent::Focused(false) => {
                // Menu-bar pattern: lose focus → dismiss panel (icon remains).
                if let Some(state) = window.try_state::<panel::PanelState>() {
                    let app = window.app_handle();
                    let _ = panel::dismiss_to_menu_bar(app, &state);
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tokenTracer host");
}

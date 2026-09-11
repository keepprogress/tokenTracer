//! Windows system tray: left-click toggles Collapsed↔Expanded; right-click menu.
//!
//! Behaviors (must match README):
//! - Left-click: toggle Collapsed ↔ Expanded (show panel if Hidden)
//! - Right-click menu: Show panel / Refresh / Import / Quit
//! - macOS menu bar: out of scope / UNTESTED

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use crate::panel::{self, PanelState};

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let show_i = MenuItem::with_id(app, "show", "Show panel", true, None::<&str>)?;
    let refresh_i = MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)?;
    let import_i = MenuItem::with_id(app, "import", "Import…", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_i, &refresh_i, &import_i, &sep, &quit_i])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .expect("default window icon required for tray");

    let _tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("tokenTracer")
        .menu(&menu)
        // Left-click handled by on_tray_icon_event; menu on right-click only.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(state) = app.try_state::<PanelState>() {
                    let _ = panel::show_panel(app, &state);
                }
            }
            "refresh" => {
                let _ = app.emit("panel-refresh", ());
            }
            "import" => {
                // UI listens and may invoke `import_run` (still stub at host).
                let _ = app.emit("panel-import-stub", ());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(state) = app.try_state::<PanelState>() {
                    let _ = panel::toggle_panel(app, &state);
                }
            }
        })
        .build(app)?;

    Ok(())
}

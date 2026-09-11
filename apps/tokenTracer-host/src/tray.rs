//! System tray / macOS menu-bar status item.
//!
//! Platform chrome (must match README):
//! - **Windows**: left-click toggles Collapsed↔Expanded; right-click menu
//!   (Show panel / Refresh / Import… / Quit).
//! - **macOS**: Tauri `tray-icon` = menu-bar / NSStatusItem equivalent.
//!   Click icon → show Expanded mini-panel (or dismiss if already open);
//!   context menu: Show panel / Refresh / Import… / Quit.
//!
//! Not a full-page browser window — panel is the decorated-less mini webview.

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
        .expect("default window icon required for tray / menu-bar");

    let builder = TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("tokenTracer")
        .menu(&menu)
        // Left-click handled by on_tray_icon_event; menu on right-click / Ctrl-click.
        .show_menu_on_left_click(false);

    // macOS: treat as template image so the status item follows light/dark menu bar.
    #[cfg(target_os = "macos")]
    let builder = builder.icon_as_template(true);

    let _tray = builder
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
                rect,
                ..
            } = &event
            {
                let app = tray.app_handle();
                if let Some(state) = app.try_state::<PanelState>() {
                    #[cfg(target_os = "macos")]
                    panel::remember_tray_rect(&state, rect);
                    #[cfg(not(target_os = "macos"))]
                    let _ = rect;
                    let _ = panel::toggle_panel(app, &state);
                }
            }
        })
        .build(app)?;

    Ok(())
}

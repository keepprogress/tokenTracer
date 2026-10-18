//! System tray / macOS menu-bar status item.
//!
//! Platform chrome (must match README):
//! - **Windows**: left-click toggles Collapsed↔Expanded; right-click menu
//!   (Show panel / Refresh / Import… / Quit).
//! - **macOS**: Tauri `tray-icon` = menu-bar / NSStatusItem equivalent.
//!   Click icon → show Expanded mini-panel (or dismiss if already open);
//!   right-click / Ctrl-click → context menu: Show panel / Refresh / Import… / Quit.
//!
//! ## Why the macOS menu is NOT attached at build time
//! `TrayIconBuilder::menu(..)` makes tray-icon 0.24.2 call `NSStatusItem.setMenu`. AppKit
//! then pops that menu on a plain **left** click, and `show_menu_on_left_click(false)` does
//! not reliably suppress it (observed on a real Mac), so the panel never opens. On macOS the
//! icon is therefore built **without** a menu; for a right-click / Ctrl-click we attach the
//! menu, pop it up, and detach it again (see [`popup_menu_macos`]). Windows/Linux keep the
//! menu attached at build time (left-click is already handled there).
//!
//! Not a full-page browser window — panel is the decorated-less mini webview.

use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use crate::panel::{self, PanelState};
#[cfg(target_os = "macos")]
use tokentracer_tray_guard::{classify_click, ClickIntent, TrayButton, UpAction};

/// Single tray / status-item id (created only in [`install`]).
pub const TRAY_ID: &str = "main";

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let show_i = MenuItem::with_id(app, "show", "Show panel", true, None::<&str>)?;
    let refresh_i = MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)?;
    let import_i = MenuItem::with_id(app, "import", "Import…", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_i, &refresh_i, &import_i, &sep, &quit_i])?;

    // Tray / menu-bar glyph. NOTE: this is the ONLY tray-icon creation site. `tauri.conf.json`
    // must NOT declare `app.trayIcon` — Tauri would build a second status item with the same
    // id (AC-F10 B1: two menu-bar icons).
    //
    // * macOS: `icons/tray.png` = monochrome BLACK glyph on a TRANSPARENT background (44x44,
    //   RGBA, alpha-only information) installed as a *template* image, so AppKit tints it for
    //   light/dark menu bars. (An opaque colour image as template renders as a solid square.)
    //   Regenerate with `scripts/gen-tray-template-icon.py`.
    // * Windows / Linux: `icons/tray-color.png` = opaque coloured icon. A black template glyph
    //   is nearly invisible on a dark taskbar, so it is NOT used there.
    #[cfg(target_os = "macos")]
    let icon = Image::from_bytes(include_bytes!("../icons/tray.png"))?;
    #[cfg(not(target_os = "macos"))]
    let icon = Image::from_bytes(include_bytes!("../icons/tray-color.png"))?;

    let builder = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("tokenTracer")
        // Left-click handled by on_tray_icon_event; menu on right-click / Ctrl-click.
        .show_menu_on_left_click(false);

    // Windows / Linux: attach the menu at build time (right-click shows it; unchanged).
    #[cfg(not(target_os = "macos"))]
    let builder = builder.menu(&menu);

    // macOS: NO menu at build time (see module docs); popped up on demand from the click handler.
    #[cfg(target_os = "macos")]
    let menu_for_popup = menu.clone();

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
        .on_tray_icon_event(move |tray, event| {
            let app = tray.app_handle();

            // macOS: classify the click, then decide panel-vs-focus-loss on Down and act
            // on Up (AC-F10 B2). Menu clicks never touch the panel toggle.
            #[cfg(target_os = "macos")]
            if let TrayIconEvent::Click {
                button,
                button_state,
                rect,
                ..
            } = &event
            {
                let Some(state) = app.try_state::<PanelState>() else {
                    return;
                };
                let tb = match button {
                    MouseButton::Left => TrayButton::Left,
                    MouseButton::Right => TrayButton::Right,
                    _ => TrayButton::Middle,
                };
                let now = std::time::Instant::now();
                match (classify_click(tb, ctrl_held_macos()), button_state) {
                    // Right / Ctrl-left press → context menu (never the panel).
                    (ClickIntent::Menu, MouseButtonState::Down) => {
                        tt_dbg!("tray menu-click Down (right/ctrl) mode={}", state.mode.lock().unwrap().as_str());
                        state.click_guard.lock().unwrap().on_menu_click();
                        popup_menu_macos(app, tray, &menu_for_popup);
                    }
                    (ClickIntent::Menu, MouseButtonState::Up) => {}
                    (ClickIntent::Panel, MouseButtonState::Down) => {
                        panel::remember_tray_rect(&state, rect);
                        let mode = *state.mode.lock().unwrap();
                        let expanded = mode == panel::PanelMode::Expanded;
                        let mut g = state.click_guard.lock().unwrap();
                        g.on_down(expanded, now);
                        tt_dbg!("tray Down: mode={} expanded={} guard={:?}", mode.as_str(), expanded, *g);
                    }
                    (ClickIntent::Panel, MouseButtonState::Up) => {
                        panel::remember_tray_rect(&state, rect);
                        let mode = *state.mode.lock().unwrap();
                        let action = state.click_guard.lock().unwrap().on_up(now);
                        tt_dbg!("tray Up: mode={} -> {:?}", mode.as_str(), action);
                        match action {
                            UpAction::Toggle => {
                                let _ = panel::toggle_panel(app, &state);
                            }
                            UpAction::DismissOnly => {
                                let _ = panel::dismiss_to_menu_bar(app, &state);
                            }
                            // Up half of a Ctrl-left menu press (Ctrl already released).
                            UpAction::Ignore => {}
                        }
                    }
                    (ClickIntent::Ignore, _) => {}
                }
                return;
            }

            // Windows / Linux: left-click Up toggles Collapsed↔Expanded (unchanged).
            #[cfg(not(target_os = "macos"))]
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = &event
            {
                if let Some(state) = app.try_state::<PanelState>() {
                    let _ = panel::toggle_panel(app, &state);
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// macOS: is the Control key down right now? (`+[NSEvent modifierFlags]`.)
///
/// tauri's `TrayIconEvent::Click` carries no modifier flags, so ask AppKit directly.
/// (AppKit may also deliver Ctrl-click as a right-click; both paths end in the menu.)
#[cfg(target_os = "macos")]
fn ctrl_held_macos() -> bool {
    use objc2_app_kit::{NSEvent, NSEventModifierFlags};
    NSEvent::modifierFlags_class().contains(NSEventModifierFlags::Control)
}

/// macOS: attach the menu, pop it up, detach it again.
///
/// Deferred through a one-shot thread → `run_on_main_thread`, i.e. to the *next* main-loop
/// turn. tray-icon runs its own post-callback bookkeeping in the same `mouseDown:`
/// (it highlights the button when no menu is attached); popping from inside the callback
/// would let that highlight land *after* the menu closes and leave the icon stuck
/// highlighted. Once on the main thread tauri's `set_menu` / `with_inner_tray_icon`
/// execute inline (no deadlock). `show_menu` = `NSStatusBarButton.performClick`, which
/// blocks while the menu is tracked, so `set_menu(None)` runs after it is dismissed.
/// Menu item actions are delivered through muda's global `MenuEvent` handler, which does
/// not depend on the menu being attached, so `on_menu_event` still fires.
#[cfg(target_os = "macos")]
fn popup_menu_macos(app: &AppHandle, tray: &tauri::tray::TrayIcon, menu: &Menu<tauri::Wry>) {
    let app = app.clone();
    let tray = tray.clone();
    let menu = menu.clone();
    std::thread::spawn(move || {
        let _ = app.run_on_main_thread(move || {
            if tray.set_menu(Some(menu)).is_ok() {
                let _ = tray.with_inner_tray_icon(|t| t.show_menu());
                let _ = tray.set_menu(None::<Menu<tauri::Wry>>);
            }
        });
    });
}

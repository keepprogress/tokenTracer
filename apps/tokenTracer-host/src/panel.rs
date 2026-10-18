//! Mini-panel window geometry + Collapsed ↔ Expanded mode.
//!
//! Platform chrome (AC-F7′ / AC-F10 · SHELL-HOST v0.1):
//! - **Windows**: bottom-right of work area (near notification area).
//! - **macOS**: menu-bar adjacent (below status item when known; else top-right).
//!   Tauri `tray-icon` on macOS is the menu-bar / NSStatusItem equivalent.

use std::sync::Mutex;

use anyhow::{Context, Result};
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Size, WebviewWindow,
};
#[cfg(target_os = "macos")]
use tauri::Position;
#[cfg(target_os = "macos")]
use tokentracer_tray_guard::TrayClickGuard;

pub const WIN_LABEL: &str = "main";

/// Collapsed strip (AC-F7.1).
pub const COLLAPSED_W: f64 = 360.0;
pub const COLLAPSED_H: f64 = 56.0;
/// Expanded panel (AC-F7.2).
pub const EXPANDED_W: f64 = 360.0;
pub const EXPANDED_H: f64 = 560.0;
/// Margin from work-area / menu-bar edges.
pub const EDGE_MARGIN: i32 = 12;
/// Gap below macOS menu-bar status item.
#[cfg(target_os = "macos")]
pub const MENU_BAR_GAP: i32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelMode {
    Collapsed,
    Expanded,
    /// Icon-only: panel hidden; tray / menu-bar icon remains.
    Hidden,
}

impl PanelMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "collapsed" => Some(Self::Collapsed),
            "expanded" => Some(Self::Expanded),
            "hidden" => Some(Self::Hidden),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Collapsed => "collapsed",
            Self::Expanded => "expanded",
            Self::Hidden => "hidden",
        }
    }

    /// Windows tray left-click: Collapsed|Hidden → Expanded; Expanded → Collapsed.
    pub fn toggle_collapsed_expanded(self) -> Self {
        match self {
            Self::Collapsed | Self::Hidden => Self::Expanded,
            Self::Expanded => Self::Collapsed,
        }
    }

    /// macOS menu-bar click: show Expanded from Hidden/Collapsed; dismiss Expanded → Hidden.
    #[cfg(target_os = "macos")]
    pub fn toggle_menu_bar(self) -> Self {
        match self {
            Self::Hidden | Self::Collapsed => Self::Expanded,
            Self::Expanded => Self::Hidden,
        }
    }
}

/// Last known tray / status-item rect (physical px), used to anchor the macOS panel.
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy)]
pub struct TrayAnchor {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub struct PanelState {
    pub mode: Mutex<PanelMode>,
    #[cfg(target_os = "macos")]
    pub tray_anchor: Mutex<Option<TrayAnchor>>,
    /// AC-F10 B2: arbitrates tray click vs. focus-loss dismissal (macOS only; Windows
    /// keeps plain Collapsed↔Expanded toggle).
    #[cfg(target_os = "macos")]
    pub click_guard: Mutex<TrayClickGuard>,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            // Windows setup shows Collapsed; macOS setup overrides to Hidden (icon-only).
            mode: Mutex::new(PanelMode::Collapsed),
            #[cfg(target_os = "macos")]
            tray_anchor: Mutex::new(None),
            #[cfg(target_os = "macos")]
            click_guard: Mutex::new(TrayClickGuard::new()),
        }
    }
}

pub fn main_window(app: &AppHandle) -> Result<WebviewWindow> {
    app.get_webview_window(WIN_LABEL)
        .context("main webview window missing")
}

#[cfg(target_os = "macos")]
pub fn remember_tray_rect(state: &PanelState, rect: &tauri::Rect) {
    let (x, y) = match rect.position {
        Position::Physical(p) => (p.x as i32, p.y as i32),
        Position::Logical(p) => {
            // Tray events are typically physical; logical fallback assumes 1×.
            (p.x as i32, p.y as i32)
        }
    };
    let (width, height) = match rect.size {
        Size::Physical(s) => (s.width as u32, s.height as u32),
        Size::Logical(s) => (s.width as u32, s.height as u32),
    };
    *state.tray_anchor.lock().unwrap() = Some(TrayAnchor {
        x,
        y,
        width,
        height,
    });
}

fn primary_monitor(win: &WebviewWindow) -> Option<tauri::Monitor> {
    win.current_monitor()
        .ok()
        .flatten()
        .or_else(|| win.primary_monitor().ok().flatten())
        .or_else(|| win.available_monitors().ok().into_iter().flatten().next())
}

/// Anchor the panel to the bottom-right of the primary monitor work area (Windows).
pub fn position_bottom_right(win: &WebviewWindow) -> Result<()> {
    let Some(monitor) = primary_monitor(win) else {
        return Ok(());
    };

    let scale = monitor.scale_factor();
    let work = monitor.work_area();
    let size = win.outer_size().unwrap_or_else(|_| {
        PhysicalSize::new(
            (COLLAPSED_W * scale) as u32,
            (COLLAPSED_H * scale) as u32,
        )
    });

    let x = work.position.x + work.size.width as i32 - size.width as i32 - EDGE_MARGIN;
    let y = work.position.y + work.size.height as i32 - size.height as i32 - EDGE_MARGIN;
    win.set_position(PhysicalPosition::new(x, y))
        .context("set_position bottom-right")?;
    Ok(())
}

/// Place panel below the menu-bar status item (macOS), or top-right of work area.
#[cfg(target_os = "macos")]
pub fn position_near_menu_bar(win: &WebviewWindow, anchor: Option<TrayAnchor>) -> Result<()> {
    let size = win.outer_size().unwrap_or_else(|_| {
        let scale = primary_monitor(win)
            .map(|m| m.scale_factor())
            .unwrap_or(1.0);
        PhysicalSize::new(
            (EXPANDED_W * scale) as u32,
            (EXPANDED_H * scale) as u32,
        )
    });

    if let Some(a) = anchor {
        // Right-align under status item; clamp to work area when known.
        let mut x = a.x + a.width as i32 - size.width as i32;
        let mut y = a.y + a.height as i32 + MENU_BAR_GAP;
        if let Some(monitor) = primary_monitor(win) {
            let work = monitor.work_area();
            let min_x = work.position.x + EDGE_MARGIN;
            let max_x = work.position.x + work.size.width as i32 - size.width as i32 - EDGE_MARGIN;
            let min_y = work.position.y + EDGE_MARGIN;
            let max_y = work.position.y + work.size.height as i32 - size.height as i32 - EDGE_MARGIN;
            x = x.clamp(min_x.min(max_x), max_x.max(min_x));
            y = y.clamp(min_y.min(max_y), max_y.max(min_y));
        }
        win.set_position(PhysicalPosition::new(x, y))
            .context("set_position near menu-bar anchor")?;
        return Ok(());
    }

    // Fallback: top-right of work area (typical menu-bar app corner).
    let Some(monitor) = primary_monitor(win) else {
        return Ok(());
    };
    let work = monitor.work_area();
    let x = work.position.x + work.size.width as i32 - size.width as i32 - EDGE_MARGIN;
    let y = work.position.y + EDGE_MARGIN;
    win.set_position(PhysicalPosition::new(x, y))
        .context("set_position menu-bar top-right fallback")?;
    Ok(())
}

/// Platform-aware panel placement.
pub fn position_panel(win: &WebviewWindow, state: &PanelState) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let anchor = *state.tray_anchor.lock().unwrap();
        position_near_menu_bar(win, anchor)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = state;
        position_bottom_right(win)
    }
}

pub fn apply_mode(app: &AppHandle, state: &PanelState, mode: PanelMode) -> Result<()> {
    let win = main_window(app)?;
    let prev = {
        let mut m = state.mode.lock().unwrap();
        let prev = *m;
        *m = mode;
        prev
    };
    // macOS: new panel state = new generation. Must happen BEFORE show()/set_focus() so a
    // Focused(true)/(false) produced by them is attributed to the new state, and records whether
    // the native window already had focus (e.g. Collapsed → Expanded via the in-panel button).
    #[cfg(target_os = "macos")]
    if prev != mode {
        let focused_now = win.is_focused().unwrap_or(false);
        let mut g = state.click_guard.lock().unwrap();
        g.on_mode_change(mode != PanelMode::Hidden, focused_now);
        tt_dbg!(
            "apply_mode {} -> {} gen={} window_focused_before={}",
            prev.as_str(),
            mode.as_str(),
            g.generation(),
            focused_now
        );
    }
    #[cfg(not(target_os = "macos"))]
    tt_dbg!("apply_mode {} -> {}", prev.as_str(), mode.as_str());
    match mode {
        PanelMode::Hidden => {
            let _ = win.hide();
        }
        PanelMode::Collapsed => {
            win.set_size(Size::Logical(LogicalSize::new(COLLAPSED_W, COLLAPSED_H)))?;
            let _ = win.show();
            let _ = position_panel(&win, state);
        }
        PanelMode::Expanded => {
            win.set_size(Size::Logical(LogicalSize::new(EXPANDED_W, EXPANDED_H)))?;
            let _ = win.show();
            let _ = position_panel(&win, state);
            let _ = win.set_focus();
        }
    }
    let _ = app.emit("panel-set-mode", mode.as_str());
    Ok(())
}

pub fn toggle_panel(app: &AppHandle, state: &PanelState) -> Result<()> {
    let next = {
        let cur = *state.mode.lock().unwrap();
        #[cfg(target_os = "macos")]
        {
            cur.toggle_menu_bar()
        }
        #[cfg(not(target_os = "macos"))]
        {
            cur.toggle_collapsed_expanded()
        }
    };
    apply_mode(app, state, next)
}

pub fn show_panel(app: &AppHandle, state: &PanelState) -> Result<()> {
    let cur = *state.mode.lock().unwrap();
    let next = if cur == PanelMode::Hidden {
        // macOS menu-bar: Show → Expanded; Windows: Show → Collapsed strip.
        #[cfg(target_os = "macos")]
        {
            PanelMode::Expanded
        }
        #[cfg(not(target_os = "macos"))]
        {
            PanelMode::Collapsed
        }
    } else {
        cur
    };
    apply_mode(app, state, next)?;
    if let Ok(win) = main_window(app) {
        let _ = win.show();
        let _ = win.set_focus();
    }
    Ok(())
}

/// macOS: dismiss panel back to menu-bar icon (focus loss / outside).
#[cfg(target_os = "macos")]
pub fn dismiss_to_menu_bar(app: &AppHandle, state: &PanelState) -> Result<()> {
    let cur = *state.mode.lock().unwrap();
    if cur == PanelMode::Hidden {
        return Ok(());
    }
    apply_mode(app, state, PanelMode::Hidden)
}

/// macOS: `Focused(true)` — the current panel generation has had focus.
#[cfg(target_os = "macos")]
pub fn note_focus_gained(state: &PanelState) {
    let mut g = state.click_guard.lock().unwrap();
    g.on_focus_gained();
    tt_dbg!(
        "Focused(true): mode={} gen={}",
        state.mode.lock().unwrap().as_str(),
        g.generation()
    );
}

/// macOS: `Focused(false)` — decide whether the panel really lost focus.
///
/// R6: a blur can be delivered late (after the tray Up already showed Expanded, or as a
/// duplicate/flicker); the old handler hid whatever the *current* mode was. Now
/// (`tokentracer_tray_guard`, state-based — **no** "ignore blurs for N ms" window, so a genuine
/// click-outside still collapses promptly):
/// 1. ignore if the current panel generation never had focus, or the window is focused now;
/// 2. otherwise defer one main-loop turn ([`resolve_focus_loss`]) and re-check.
#[cfg(target_os = "macos")]
pub fn on_focus_loss(app: &AppHandle, state: &PanelState) -> Result<()> {
    use tokentracer_tray_guard::BlurVerdict;
    let cur = *state.mode.lock().unwrap();
    if cur == PanelMode::Hidden {
        tt_dbg!("Focused(false): mode=hidden -> no-op");
        return Ok(());
    }
    let win = main_window(app)?;
    let focused_now = win.is_focused().unwrap_or(false);
    let verdict = state.click_guard.lock().unwrap().on_blur_event(focused_now);
    tt_dbg!(
        "Focused(false): mode={} window_focused_now={} -> {:?} guard={:?}",
        cur.as_str(),
        focused_now,
        verdict,
        *state.click_guard.lock().unwrap()
    );
    if let BlurVerdict::Defer { generation } = verdict {
        // One-shot thread → run_on_main_thread = the *next* main-loop turn (inline calls from
        // the main thread would not defer).
        let app2 = app.clone();
        std::thread::spawn(move || {
            let app3 = app2.clone();
            let _ = app2.run_on_main_thread(move || {
                if let Some(state) = app3.try_state::<PanelState>() {
                    let _ = resolve_focus_loss(&app3, &state, generation);
                }
            });
        });
    }
    Ok(())
}

/// Deferred half of [`on_focus_loss`]: re-read mode + actual focus, then dismiss or ignore.
#[cfg(target_os = "macos")]
fn resolve_focus_loss(app: &AppHandle, state: &PanelState, generation: u64) -> Result<()> {
    use tokentracer_tray_guard::BlurResolution;
    let cur = *state.mode.lock().unwrap();
    let focused_now = main_window(app)
        .ok()
        .and_then(|w| w.is_focused().ok())
        .unwrap_or(false);
    let res = state.click_guard.lock().unwrap().resolve_blur(
        generation,
        focused_now,
        cur == PanelMode::Expanded,
        std::time::Instant::now(),
    );
    tt_dbg!(
        "blur re-check: gen_at_blur={} mode={} window_focused_now={} -> {:?}",
        generation,
        cur.as_str(),
        focused_now,
        res
    );
    match res {
        BlurResolution::Ignore(_) => Ok(()),
        BlurResolution::Dismiss | BlurResolution::DismissAndArm => {
            if cur == PanelMode::Hidden {
                return Ok(());
            }
            apply_mode(app, state, PanelMode::Hidden)
        }
    }
}

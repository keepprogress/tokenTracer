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
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            // Windows setup shows Collapsed; macOS setup overrides to Hidden (icon-only).
            mode: Mutex::new(PanelMode::Collapsed),
            #[cfg(target_os = "macos")]
            tray_anchor: Mutex::new(None),
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
    {
        *state.mode.lock().unwrap() = mode;
    }
    let win = main_window(app)?;
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

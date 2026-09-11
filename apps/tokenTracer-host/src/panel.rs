//! Mini-panel window geometry + Collapsed ↔ Expanded mode.

use std::sync::Mutex;

use anyhow::{Context, Result};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, Size, WebviewWindow};

pub const WIN_LABEL: &str = "main";

/// Collapsed strip (AC-F7.1).
pub const COLLAPSED_W: f64 = 360.0;
pub const COLLAPSED_H: f64 = 56.0;
/// Expanded panel (AC-F7.2).
pub const EXPANDED_W: f64 = 360.0;
pub const EXPANDED_H: f64 = 560.0;
/// Margin from work-area edges (near notification area).
pub const EDGE_MARGIN: i32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelMode {
    Collapsed,
    Expanded,
    /// Icon-only: panel hidden; tray remains.
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

    pub fn toggle_collapsed_expanded(self) -> Self {
        match self {
            Self::Collapsed | Self::Hidden => Self::Expanded,
            Self::Expanded => Self::Collapsed,
        }
    }
}

pub struct PanelState {
    pub mode: Mutex<PanelMode>,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            mode: Mutex::new(PanelMode::Collapsed),
        }
    }
}

pub fn main_window(app: &AppHandle) -> Result<WebviewWindow> {
    app.get_webview_window(WIN_LABEL)
        .context("main webview window missing")
}

/// Anchor the panel to the bottom-right of the primary monitor work area.
pub fn position_bottom_right(win: &WebviewWindow) -> Result<()> {
    let Some(monitor) = win.current_monitor().ok().flatten().or_else(|| {
        win.primary_monitor()
            .ok()
            .flatten()
            .or_else(|| win.available_monitors().ok().into_iter().flatten().next())
    }) else {
        return Ok(());
    };

    let scale = monitor.scale_factor();
    let work = monitor.work_area();
    let size = win.outer_size().unwrap_or_else(|_| {
        tauri::PhysicalSize::new(
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
            let _ = position_bottom_right(&win);
        }
        PanelMode::Expanded => {
            win.set_size(Size::Logical(LogicalSize::new(EXPANDED_W, EXPANDED_H)))?;
            let _ = win.show();
            let _ = position_bottom_right(&win);
            let _ = win.set_focus();
        }
    }
    let _ = app.emit("panel-set-mode", mode.as_str());
    Ok(())
}

pub fn toggle_panel(app: &AppHandle, state: &PanelState) -> Result<()> {
    let next = {
        let cur = *state.mode.lock().unwrap();
        cur.toggle_collapsed_expanded()
    };
    apply_mode(app, state, next)
}

pub fn show_panel(app: &AppHandle, state: &PanelState) -> Result<()> {
    let cur = *state.mode.lock().unwrap();
    let next = if cur == PanelMode::Hidden {
        PanelMode::Collapsed
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

//! Tray / menu-bar click arbitration (AC-F10 · B2).
//!
//! # Problem (macOS menu-bar panel)
//!
//! With the mini-panel open (Expanded), a click on the status-item icon produces
//! **two** events in quick succession:
//!
//! 1. the panel window loses focus (`WindowEvent::Focused(false)`) and the host
//!    collapses it to *Hidden*;
//! 2. the tray icon delivers `Click { Down }` then `Click { Up }`, and the host's
//!    toggle flips *Hidden* straight back to *Expanded*.
//!
//! Net effect: the panel never closes when you click the icon again.
//!
//! # Fix
//!
//! A click that **dismissed** the panel (focus loss within [`SWALLOW_WINDOW`] of the
//! click, or panel already Expanded when the button went down) must not re-open it.
//! [`TrayClickGuard`] is a tiny state machine over timestamps — no Tauri types — so it
//! is verifiable without a GUI. The host owns one guard (macOS only) and consults it:
//!
//! * `on_focus_lost(now)` — when the panel is dismissed because it lost focus;
//! * `on_down(panel_was_expanded, now)` — tray `Click { Left, Down }`;
//! * `on_up(now)` — tray `Click { Left, Up }` → [`UpAction`].
//!
//! Windows keeps its own behavior (left-click toggles Collapsed↔Expanded) and never
//! constructs or consults this guard.

use std::time::{Duration, Instant};

/// A tray click within this long after a focus-loss dismissal is the click that
/// caused the dismissal and must not re-open the panel (~300 ms per AC-F10 B2).
pub const SWALLOW_WINDOW: Duration = Duration::from_millis(300);

/// What the host should do on tray `Click { Left, Up }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpAction {
    /// Normal click: toggle the panel (Hidden/Collapsed → Expanded, Expanded → Hidden).
    Toggle,
    /// The click belonged to a dismissal: make sure the panel is hidden, never re-open.
    DismissOnly,
}

/// `true` when `now` falls inside the swallow window that started at `dismissed_at`.
///
/// Pure function over timestamps. A `now` earlier than `dismissed_at` (clock quirk /
/// reordered events) counts as "within the window" (saturating difference of zero).
pub fn within_swallow_window(dismissed_at: Option<Instant>, now: Instant) -> bool {
    match dismissed_at {
        Some(t) => now.saturating_duration_since(t) <= SWALLOW_WINDOW,
        None => false,
    }
}

/// Click-vs-focus-loss arbitration state (see crate docs).
#[derive(Debug, Default, Clone, Copy)]
pub struct TrayClickGuard {
    last_focus_dismiss: Option<Instant>,
    /// Set on Down when the click is part of a dismissal; consumed on Up.
    swallow_up: bool,
}

impl TrayClickGuard {
    pub const fn new() -> Self {
        Self {
            last_focus_dismiss: None,
            swallow_up: false,
        }
    }

    /// Record that the panel was just dismissed because the window lost focus.
    pub fn on_focus_lost(&mut self, now: Instant) {
        self.last_focus_dismiss = Some(now);
    }

    /// Tray button went down. `panel_was_expanded` = panel mode at that instant.
    ///
    /// The click is a dismissal if the panel is still Expanded (focus loss has not been
    /// processed yet) **or** focus loss already dismissed it within [`SWALLOW_WINDOW`].
    pub fn on_down(&mut self, panel_was_expanded: bool, now: Instant) {
        self.swallow_up =
            panel_was_expanded || within_swallow_window(self.last_focus_dismiss, now);
    }

    /// Tray button released: decide what to do. Consumes the Down decision.
    ///
    /// If no Down was seen (`swallow_up == false`) the timestamp rule still applies, so a
    /// lone Up right after a focus-loss dismissal is also swallowed.
    pub fn on_up(&mut self, now: Instant) -> UpAction {
        let swallow = self.swallow_up || within_swallow_window(self.last_focus_dismiss, now);
        self.swallow_up = false;
        if swallow {
            // One dismissal swallows exactly one click.
            self.last_focus_dismiss = None;
            UpAction::DismissOnly
        } else {
            UpAction::Toggle
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t0() -> Instant {
        Instant::now()
    }
    fn ms(base: Instant, n: u64) -> Instant {
        base + Duration::from_millis(n)
    }

    #[test]
    fn window_none_is_never_within() {
        assert!(!within_swallow_window(None, t0()));
    }

    #[test]
    fn window_boundaries() {
        let b = t0();
        assert!(within_swallow_window(Some(b), b));
        assert!(within_swallow_window(Some(b), ms(b, 299)));
        assert!(within_swallow_window(Some(b), ms(b, 300)));
        assert!(!within_swallow_window(Some(b), ms(b, 301)));
        assert!(!within_swallow_window(Some(b), ms(b, 2000)));
    }

    #[test]
    fn window_now_before_dismiss_counts_as_within() {
        let b = t0();
        assert!(within_swallow_window(Some(ms(b, 50)), b));
    }

    #[test]
    fn fresh_guard_click_toggles() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_down(false, b);
        assert_eq!(g.on_up(ms(b, 80)), UpAction::Toggle);
    }

    /// The B2 bug: focus loss first, then Down/Up → must NOT re-open.
    #[test]
    fn focus_loss_then_click_is_swallowed() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_focus_lost(b);
        g.on_down(false, ms(b, 5)); // panel already Hidden by focus loss
        assert_eq!(g.on_up(ms(b, 90)), UpAction::DismissOnly);
    }

    /// Down arrives before the focus-loss handler ran: panel still Expanded.
    #[test]
    fn down_while_expanded_then_focus_loss_is_swallowed() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_down(true, b);
        g.on_focus_lost(ms(b, 2));
        assert_eq!(g.on_up(ms(b, 120)), UpAction::DismissOnly);
    }

    /// Expanded at Down but no focus loss ever arrives: still dismiss, never toggle open.
    #[test]
    fn down_while_expanded_without_focus_loss_dismisses() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_down(true, b);
        assert_eq!(g.on_up(ms(b, 100)), UpAction::DismissOnly);
    }

    /// A slow press (Up long after Down) still closes, because Down decided.
    #[test]
    fn slow_click_decided_on_down() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_focus_lost(b);
        g.on_down(false, ms(b, 10));
        assert_eq!(g.on_up(ms(b, 1500)), UpAction::DismissOnly);
    }

    /// Lone Up (no Down delivered) right after focus loss is swallowed by timestamp.
    #[test]
    fn lone_up_within_window_swallowed() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_focus_lost(b);
        assert_eq!(g.on_up(ms(b, 120)), UpAction::DismissOnly);
    }

    /// Focus lost long ago (user clicked elsewhere), then icon click → opens normally.
    #[test]
    fn stale_focus_loss_does_not_block_open() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_focus_lost(b);
        let later = ms(b, 10_000);
        g.on_down(false, later);
        assert_eq!(g.on_up(ms(later, 70)), UpAction::Toggle);
    }

    /// One dismissal swallows one click; the next click opens the panel again.
    #[test]
    fn swallow_is_one_shot() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_focus_lost(b);
        g.on_down(false, ms(b, 5));
        assert_eq!(g.on_up(ms(b, 60)), UpAction::DismissOnly);
        // Immediately click again (inside what would be the 300 ms window): opens.
        g.on_down(false, ms(b, 150));
        assert_eq!(g.on_up(ms(b, 200)), UpAction::Toggle);
    }

    /// Closed → click opens → click again closes (full B2 round trip, simulated).
    #[test]
    fn open_then_click_again_closes() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        // 1) click on a hidden panel opens it
        g.on_down(false, b);
        assert_eq!(g.on_up(ms(b, 60)), UpAction::Toggle);
        // 2) panel Expanded; user clicks icon again 3 s later: focus loss, Down, Up
        let c = ms(b, 3000);
        g.on_focus_lost(c);
        g.on_down(false, ms(c, 4));
        assert_eq!(g.on_up(ms(c, 70)), UpAction::DismissOnly);
    }
}

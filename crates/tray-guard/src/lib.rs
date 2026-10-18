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
//! # Menu-popup clicks (AC-F10 · macOS left-click-pops-menu fix)
//!
//! On macOS the context menu must **not** be attached to the status item at build
//! time (tray-icon 0.24.2 calls `NSStatusItem.setMenu`, after which a plain left click
//! pops the menu instead of opening the panel). The host attaches it only for a
//! right-click / Ctrl-click. [`classify_click`] is the pure decision "is this click a
//! menu popup, a panel toggle, or ignorable?"; [`TrayClickGuard::on_menu_click`] makes
//! sure the Up half of a Ctrl-left-click (when Ctrl may already be released) is
//! swallowed and never toggles the panel.
//!
//! Windows keeps its own behavior (left-click toggles Collapsed↔Expanded) and never
//! constructs or consults this guard.

use std::time::{Duration, Instant};

/// A tray click within this long after a focus-loss dismissal is the click that
/// caused the dismissal and must not re-open the panel (~300 ms per AC-F10 B2).
pub const SWALLOW_WINDOW: Duration = Duration::from_millis(300);

/// Mouse button of a tray click, decoupled from tauri's type so this crate stays GUI-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayButton {
    Left,
    Right,
    Middle,
}

/// What a tray click means on macOS (see crate docs, "Menu-popup clicks").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickIntent {
    /// Open / toggle the mini-panel (plain left click).
    Panel,
    /// Pop the context menu (right click, or Ctrl + left click).
    Menu,
    /// Nothing to do (e.g. middle click).
    Ignore,
}

/// Pure classification of a tray click. `ctrl_held` = Control key down at event time.
///
/// * Right, or Left with Ctrl → [`ClickIntent::Menu`] (macOS convention: Ctrl-click == right-click)
/// * Left without Ctrl → [`ClickIntent::Panel`]
/// * anything else → [`ClickIntent::Ignore`]
pub fn classify_click(button: TrayButton, ctrl_held: bool) -> ClickIntent {
    match button {
        TrayButton::Right => ClickIntent::Menu,
        TrayButton::Left if ctrl_held => ClickIntent::Menu,
        TrayButton::Left => ClickIntent::Panel,
        TrayButton::Middle => ClickIntent::Ignore,
    }
}

/// What the host should do on tray `Click { Left, Up }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpAction {
    /// The click was a menu popup (e.g. Ctrl-left-click whose Ctrl was released before
    /// Up): do nothing — in particular never toggle the panel.
    Ignore,
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
    /// Set when a menu popup started on this press; the Left Up of that press is ignored.
    menu_pending: bool,
}

impl TrayClickGuard {
    pub const fn new() -> Self {
        Self {
            last_focus_dismiss: None,
            swallow_up: false,
            menu_pending: false,
        }
    }

    /// A click was classified as [`ClickIntent::Menu`] (right / Ctrl-left). The matching
    /// Left Up, if any, must not toggle the panel. Cleared by the next [`Self::on_down`].
    pub fn on_menu_click(&mut self) {
        self.menu_pending = true;
        self.swallow_up = false;
        // A blur caused by this right/Ctrl click belongs to the menu press, not to a later click.
        self.last_focus_dismiss = None;
    }

    /// Record that the panel was just dismissed because the window lost focus.
    pub fn on_focus_lost(&mut self, now: Instant) {
        self.last_focus_dismiss = Some(now);
    }

    /// Window blur (`Focused(false)`) with the panel in some mode. Records the dismissal
    /// **only if the panel was Expanded**; returns whether it was recorded.
    ///
    /// A blur while Collapsed (small strip) must not arm the swallow window: the next tray
    /// click is then an ordinary Hidden → Expanded open, not "the click that caused the
    /// dismissal". (Host still hides the panel on any blur; only the *guard* differs.)
    pub fn on_blur(&mut self, panel_was_expanded: bool, now: Instant) -> bool {
        if panel_was_expanded {
            self.on_focus_lost(now);
        }
        panel_was_expanded
    }

    /// Tray button went down. `panel_was_expanded` = panel mode at that instant.
    ///
    /// The click is a dismissal if the panel is still Expanded (focus loss has not been
    /// processed yet) **or** focus loss already dismissed it within [`SWALLOW_WINDOW`].
    pub fn on_down(&mut self, panel_was_expanded: bool, now: Instant) {
        self.menu_pending = false;
        self.swallow_up =
            panel_was_expanded || within_swallow_window(self.last_focus_dismiss, now);
    }

    /// Tray button released: decide what to do. Consumes the Down decision.
    ///
    /// If no Down was seen (`swallow_up == false`) the timestamp rule still applies, so a
    /// lone Up right after a focus-loss dismissal is also swallowed.
    pub fn on_up(&mut self, now: Instant) -> UpAction {
        if self.menu_pending {
            self.menu_pending = false;
            self.swallow_up = false;
            return UpAction::Ignore;
        }
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

    // ---- menu-popup classification (macOS left-click-pops-menu fix) ----

    #[test]
    fn classify_plain_left_is_panel() {
        assert_eq!(classify_click(TrayButton::Left, false), ClickIntent::Panel);
    }

    #[test]
    fn classify_right_is_menu_regardless_of_ctrl() {
        assert_eq!(classify_click(TrayButton::Right, false), ClickIntent::Menu);
        assert_eq!(classify_click(TrayButton::Right, true), ClickIntent::Menu);
    }

    #[test]
    fn classify_ctrl_left_is_menu() {
        assert_eq!(classify_click(TrayButton::Left, true), ClickIntent::Menu);
    }

    #[test]
    fn classify_middle_is_ignored() {
        assert_eq!(classify_click(TrayButton::Middle, false), ClickIntent::Ignore);
        assert_eq!(classify_click(TrayButton::Middle, true), ClickIntent::Ignore);
    }

    /// Ctrl-left Down starts a menu popup; Ctrl released before Up → Up (now "plain left")
    /// must NOT toggle the panel.
    #[test]
    fn ctrl_left_up_after_menu_is_ignored() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_menu_click();
        assert_eq!(g.on_up(ms(b, 400)), UpAction::Ignore);
    }

    /// Menu popup consumed once: the next ordinary click toggles again.
    #[test]
    fn menu_ignore_is_one_shot() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_menu_click();
        assert_eq!(g.on_up(ms(b, 100)), UpAction::Ignore);
        g.on_down(false, ms(b, 2000));
        assert_eq!(g.on_up(ms(b, 2060)), UpAction::Toggle);
    }

    /// Right-click popup has no Left Up; the stale flag is cleared by the next Left Down.
    #[test]
    fn stale_menu_flag_cleared_by_next_down() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_menu_click(); // right-click: no Left Up ever arrives
        g.on_down(false, ms(b, 5000));
        assert_eq!(g.on_up(ms(b, 5060)), UpAction::Toggle);
    }

    /// A menu click clears a pending dismissal swallow so it cannot leak into later clicks.
    #[test]
    fn menu_click_clears_pending_swallow() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_down(true, b);
        g.on_menu_click();
        assert_eq!(g.on_up(ms(b, 50)), UpAction::Ignore);
        g.on_down(false, ms(b, 9000));
        assert_eq!(g.on_up(ms(b, 9050)), UpAction::Toggle);
    }

    // ---- blur-while-Collapsed must not swallow the next click (review blocker) ----

    /// Collapsed blur, then tray click within 300 ms → normal toggle (open), not DismissOnly.
    #[test]
    fn collapsed_blur_then_click_toggles() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        assert!(!g.on_blur(false, b));
        g.on_down(false, ms(b, 5)); // panel Hidden after the blur
        assert_eq!(g.on_up(ms(b, 60)), UpAction::Toggle);
    }

    /// Same, but only an Up is delivered (no Down).
    #[test]
    fn collapsed_blur_then_lone_up_toggles() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_blur(false, b);
        assert_eq!(g.on_up(ms(b, 40)), UpAction::Toggle);
    }

    /// Expanded blur, then click within 300 ms → DismissOnly (B2 behavior preserved).
    #[test]
    fn expanded_blur_then_click_dismisses() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        assert!(g.on_blur(true, b));
        g.on_down(false, ms(b, 5));
        assert_eq!(g.on_up(ms(b, 60)), UpAction::DismissOnly);
    }

    /// Down before blur (panel Expanded at Down), blur Expanded afterwards.
    #[test]
    fn down_then_expanded_blur_dismisses() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_down(true, b);
        g.on_blur(true, ms(b, 3));
        assert_eq!(g.on_up(ms(b, 90)), UpAction::DismissOnly);
    }

    /// Down on a Collapsed panel, then blur(Collapsed) → still a normal toggle.
    #[test]
    fn down_collapsed_then_collapsed_blur_toggles() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_down(false, b);
        g.on_blur(false, ms(b, 3));
        assert_eq!(g.on_up(ms(b, 90)), UpAction::Toggle);
    }

    /// Stale Expanded blur (> 300 ms) does not affect a later click.
    #[test]
    fn stale_expanded_blur_via_on_blur_toggles() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_blur(true, b);
        g.on_down(false, ms(b, 301));
        assert_eq!(g.on_up(ms(b, 350)), UpAction::Toggle);
    }

    /// A Collapsed blur after an old Expanded blur does not refresh the window.
    #[test]
    fn collapsed_blur_does_not_refresh_old_expanded_blur() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_blur(true, b);
        g.on_blur(false, ms(b, 1000)); // panel re-opened as Collapsed, blurred
        g.on_down(false, ms(b, 1100));
        assert_eq!(g.on_up(ms(b, 1150)), UpAction::Toggle);
    }

    /// One Expanded blur swallows exactly one click; the next opens.
    #[test]
    fn one_expanded_blur_swallows_exactly_one_click() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_blur(true, b);
        g.on_down(false, ms(b, 4));
        assert_eq!(g.on_up(ms(b, 50)), UpAction::DismissOnly);
        g.on_down(false, ms(b, 120));
        assert_eq!(g.on_up(ms(b, 160)), UpAction::Toggle);
    }

    /// Menu path unaffected by blur kind: right/Ctrl-click after an Expanded blur ignores the
    /// Up and the *following* ordinary click is not swallowed.
    #[test]
    fn menu_path_unaffected_by_blur() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_blur(true, b); // panel was open; right-click blurs it
        g.on_menu_click();
        assert_eq!(g.on_up(ms(b, 40)), UpAction::Ignore);
        // menu_pending consumed; the blur window was consumed too (one-shot)
        g.on_down(false, ms(b, 100));
        assert_eq!(g.on_up(ms(b, 140)), UpAction::Toggle);
        // Collapsed blur + Ctrl-click: still just a menu, panel untouched.
        g.on_blur(false, ms(b, 500));
        g.on_menu_click();
        assert_eq!(g.on_up(ms(b, 520)), UpAction::Ignore);
    }
}

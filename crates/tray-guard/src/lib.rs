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
//! # Stale blurs (AC-F10 · R6 focus race)
//!
//! A `Focused(false)` is only a *dismissal* if it ends a focus the **current** panel state
//! actually had. On a real Mac a blur can be delivered late — after the tray click already
//! switched the panel to Expanded — and a handler that just looks at the *current* mode then
//! hides the freshly shown panel. Instead of a time window ("ignore blurs for N ms after
//! showing"), which would also swallow a genuine click-outside, the host asks the pure guard,
//! using **state only**:
//!
//! 1. every visible/hidden **mode change** bumps a *generation* and records whether the window
//!    was already focused at that moment ([`TrayClickGuard::on_mode_change`]); a `Focused(true)`
//!    marks the generation as having had focus ([`TrayClickGuard::on_focus_gained`]);
//! 2. on `Focused(false)` ([`TrayClickGuard::on_blur_event`]): if this generation never had focus
//!    → stale, ignore; if the window reports itself focused right now → stale, ignore; otherwise
//!    **defer one main-loop turn** and re-decide;
//! 3. on the deferred turn ([`TrayClickGuard::resolve_blur`]): generation changed → superseded,
//!    ignore; window focused again → ignore; still unfocused → dismiss (arming the swallow window
//!    only for an Expanded panel, and not for a blur that belongs to a menu popup).
//!
//! A genuine click-outside on a panel that had focus therefore collapses within one loop turn.
//!
//! Windows keeps its own behavior (left-click toggles Collapsed↔Expanded) and never
//! constructs or consults this guard.

use std::time::{Duration, Instant};

/// A tray click within this long after a focus-loss dismissal is the click that
/// caused the dismissal and must not re-open the panel (~300 ms per AC-F10 B2).
pub const SWALLOW_WINDOW: Duration = Duration::from_millis(300);

/// Why a `Focused(false)` was not treated as a dismissal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoreReason {
    /// The current panel state never had focus (blur belongs to an older state).
    NeverFocusedInThisState,
    /// The window reports itself focused when the blur is handled (focus regained / flicker).
    WindowStillFocused,
    /// A mode change happened between the blur and its deferred re-check.
    Superseded,
}

/// First-stage verdict for a `Focused(false)` (see [`TrayClickGuard::on_blur_event`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlurVerdict {
    Ignore(IgnoreReason),
    /// Re-check on the next main-loop turn, passing this generation to `resolve_blur`.
    Defer { generation: u64 },
}

/// Final verdict after the deferred re-check (see [`TrayClickGuard::resolve_blur`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlurResolution {
    Ignore(IgnoreReason),
    /// Hide the panel; do not arm the swallow window (Collapsed, or blur of a menu popup).
    Dismiss,
    /// Hide the panel and arm the swallow window (Expanded dismissal).
    DismissAndArm,
}

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
    /// A menu popup is (or just was) up: an Expanded blur in this session belongs to the menu
    /// press and must not arm the swallow window. Cleared by the next Down or by the panel
    /// becoming visible again.
    menu_session: bool,
    /// Bumped on every panel mode change.
    generation: u64,
    /// Did the *current* generation have window focus (at the transition, or via Focused(true))?
    focus_gained: bool,
}

impl TrayClickGuard {
    pub const fn new() -> Self {
        Self {
            last_focus_dismiss: None,
            swallow_up: false,
            menu_pending: false,
            menu_session: false,
            generation: 0,
            focus_gained: false,
        }
    }

    /// Current panel-state generation (for logging / tests).
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// The panel mode changed. Call **before** showing/focusing the window.
    /// `now_visible`: new mode is Collapsed/Expanded; `window_already_focused`: the native
    /// window was focused at this moment (e.g. Collapsed → Expanded by the in-panel button).
    pub fn on_mode_change(&mut self, now_visible: bool, window_already_focused: bool) {
        self.generation = self.generation.wrapping_add(1);
        self.focus_gained = now_visible && window_already_focused;
        if now_visible {
            self.menu_session = false;
        }
    }

    /// `Focused(true)` arrived: the current generation has had focus.
    pub fn on_focus_gained(&mut self) {
        self.focus_gained = true;
    }

    /// Stage 1 for `Focused(false)`: cheap, synchronous checks. `window_focused_now` is the
    /// window's actual focused state when the event is handled.
    pub fn on_blur_event(&mut self, window_focused_now: bool) -> BlurVerdict {
        if !self.focus_gained {
            return BlurVerdict::Ignore(IgnoreReason::NeverFocusedInThisState);
        }
        if window_focused_now {
            return BlurVerdict::Ignore(IgnoreReason::WindowStillFocused);
        }
        BlurVerdict::Defer {
            generation: self.generation,
        }
    }

    /// Stage 2, one main-loop turn later. `generation` is the value returned by
    /// [`Self::on_blur_event`]; `panel_was_expanded` is the mode *now*.
    pub fn resolve_blur(
        &mut self,
        generation: u64,
        window_focused_now: bool,
        panel_was_expanded: bool,
        now: Instant,
    ) -> BlurResolution {
        if generation != self.generation {
            return BlurResolution::Ignore(IgnoreReason::Superseded);
        }
        if window_focused_now {
            return BlurResolution::Ignore(IgnoreReason::WindowStillFocused);
        }
        if panel_was_expanded && !self.menu_session {
            self.on_focus_lost(now);
            BlurResolution::DismissAndArm
        } else {
            BlurResolution::Dismiss
        }
    }

    /// A click was classified as [`ClickIntent::Menu`] (right / Ctrl-left). The matching
    /// Left Up, if any, must not toggle the panel. Cleared by the next [`Self::on_down`].
    pub fn on_menu_click(&mut self) {
        self.menu_pending = true;
        self.menu_session = true;
        self.swallow_up = false;
        // A blur caused by this right/Ctrl click belongs to the menu press, not to a later click.
        self.last_focus_dismiss = None;
    }

    /// Record that the panel was just dismissed because the window lost focus.
    pub fn on_focus_lost(&mut self, now: Instant) {
        self.last_focus_dismiss = Some(now);
    }

    /// **Legacy** (pre-R6) single-step blur API, kept for its tests; the host now uses
    /// [`Self::on_blur_event`] + [`Self::resolve_blur`].
    ///
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
        self.menu_session = false;
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

    // ---- R6: stale-blur / focus-race model (state-based, no time window for ignoring) ----

    /// Helper: panel hidden -> shown Expanded by a tray click (not focused yet).
    fn show_expanded_from_hidden(g: &mut TrayClickGuard) {
        g.on_mode_change(true, false);
    }

    /// Stage 1+2 for a blur whose window is `focused1` at event time and `focused2` one turn later.
    fn blur(
        g: &mut TrayClickGuard,
        expanded: bool,
        focused1: bool,
        focused2: bool,
        now: Instant,
    ) -> BlurResolution {
        match g.on_blur_event(focused1) {
            BlurVerdict::Ignore(r) => BlurResolution::Ignore(r),
            BlurVerdict::Defer { generation } => g.resolve_blur(generation, focused2, expanded, now),
        }
    }

    /// THE R6 sequence: Collapsed (focused) -> blur#1 (delivered, deferred) -> click Up shows
    /// Expanded BEFORE the deferred re-check -> re-check sees a newer generation -> ignored.
    #[test]
    fn r6_blur_deferred_then_click_expands_first_is_superseded() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_mode_change(true, true); // Collapsed, window focused
        let BlurVerdict::Defer { generation } = g.on_blur_event(false) else {
            panic!("expected defer")
        };
        // icon Down/Up arrive before the deferred turn runs
        g.on_down(false, ms(b, 150));
        assert_eq!(g.on_up(ms(b, 180)), UpAction::Toggle);
        show_expanded_from_hidden(&mut g); // Collapsed -> Expanded, window not focused yet
        assert_eq!(
            g.resolve_blur(generation, false, true, ms(b, 181)),
            BlurResolution::Ignore(IgnoreReason::Superseded)
        );
    }

    /// Late blur arrives AFTER the Up already showed Expanded (state: never focused): ignored.
    #[test]
    fn r6_late_blur_after_up_never_focused_is_ignored() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_mode_change(true, true); // Collapsed focused
        g.on_mode_change(false, false); // Collapsed blur#1 handled -> Hidden
        g.on_down(false, ms(b, 200));
        assert_eq!(g.on_up(ms(b, 230)), UpAction::Toggle);
        show_expanded_from_hidden(&mut g);
        // delayed duplicate blur from the old state:
        assert_eq!(
            g.on_blur_event(false),
            BlurVerdict::Ignore(IgnoreReason::NeverFocusedInThisState)
        );
    }

    /// Same, but focus already arrived (Focused(true)) and the blur is a flicker: window focused.
    #[test]
    fn r6_transient_blur_with_window_focused_is_ignored() {
        let mut g = TrayClickGuard::new();
        show_expanded_from_hidden(&mut g);
        g.on_focus_gained();
        assert_eq!(
            g.on_blur_event(true),
            BlurVerdict::Ignore(IgnoreReason::WindowStillFocused)
        );
    }

    /// Flicker that recovers by the deferred turn: unfocused at event, focused again at re-check.
    #[test]
    fn r6_flicker_regained_by_recheck_is_ignored() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        show_expanded_from_hidden(&mut g);
        g.on_focus_gained();
        assert_eq!(
            blur(&mut g, true, false, true, b),
            BlurResolution::Ignore(IgnoreReason::WindowStillFocused)
        );
        // and nothing was armed: an ordinary click toggles
        g.on_down(false, ms(b, 50));
        assert_eq!(g.on_up(ms(b, 90)), UpAction::Toggle);
    }

    /// Truly unfocused after show (had focus) -> must collapse (and arm).
    #[test]
    fn r6_genuine_blur_after_focus_collapses_and_arms() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        show_expanded_from_hidden(&mut g);
        g.on_focus_gained();
        assert_eq!(blur(&mut g, true, false, false, b), BlurResolution::DismissAndArm);
        g.on_down(false, ms(b, 100));
        assert_eq!(g.on_up(ms(b, 140)), UpAction::DismissOnly);
    }

    /// Click-outside right after expanding (no time window): as soon as the panel had focus, a
    /// blur 20 ms later collapses.
    #[test]
    fn r6_click_outside_immediately_after_focus_collapses() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        show_expanded_from_hidden(&mut g);
        g.on_focus_gained();
        assert_eq!(blur(&mut g, true, false, false, ms(b, 20)), BlurResolution::DismissAndArm);
    }

    /// Expanded reached from a focused Collapsed panel (in-panel button): no Focused(true) will
    /// come, but the transition recorded focus -> a click-outside still collapses.
    #[test]
    fn r6_collapsed_to_expanded_keeps_focus_state() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_mode_change(true, true); // Collapsed, focused
        g.on_mode_change(true, true); // -> Expanded via UI button, window focused
        assert_eq!(blur(&mut g, true, false, false, b), BlurResolution::DismissAndArm);
    }

    /// Collapsed blur while focused: dismiss without arming (23650c7 preserved).
    #[test]
    fn r6_collapsed_blur_dismisses_without_arming() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_mode_change(true, true);
        assert_eq!(blur(&mut g, false, false, false, b), BlurResolution::Dismiss);
        g.on_mode_change(false, false);
        g.on_down(false, ms(b, 30));
        assert_eq!(g.on_up(ms(b, 60)), UpAction::Toggle);
    }

    /// Double blur: first resolves (Dismiss), second arrives for the hidden state -> the host
    /// returns early for Hidden, and if it did call in, the generation check ignores it.
    #[test]
    fn r6_double_blur_second_is_superseded() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_mode_change(true, true);
        let BlurVerdict::Defer { generation: g1 } = g.on_blur_event(false) else { panic!() };
        let BlurVerdict::Defer { generation: g2 } = g.on_blur_event(false) else { panic!() };
        assert_eq!(g1, g2);
        assert_eq!(g.resolve_blur(g1, false, false, b), BlurResolution::Dismiss);
        g.on_mode_change(false, false); // host hid the panel
        assert_eq!(
            g.resolve_blur(g2, false, false, ms(b, 1)),
            BlurResolution::Ignore(IgnoreReason::Superseded)
        );
    }

    /// Deferred re-check after the user re-opened the panel in between: blur of the older
    /// Expanded must not collapse the newer Expanded.
    #[test]
    fn r6_older_expanded_blur_cannot_collapse_newer_expanded() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        show_expanded_from_hidden(&mut g);
        g.on_focus_gained();
        let BlurVerdict::Defer { generation } = g.on_blur_event(false) else { panic!() };
        g.on_mode_change(false, false); // hidden
        show_expanded_from_hidden(&mut g); // new Expanded, newer generation
        assert_eq!(
            g.resolve_blur(generation, false, true, b),
            BlurResolution::Ignore(IgnoreReason::Superseded)
        );
    }

    /// B2 regression: Expanded panel with focus, icon click while open => blur + Down/Up => closes.
    #[test]
    fn r6_close_click_on_open_panel_still_closes() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        show_expanded_from_hidden(&mut g);
        g.on_focus_gained();
        g.on_down(true, b); // Down sees Expanded
        assert_eq!(blur(&mut g, true, false, false, ms(b, 3)), BlurResolution::DismissAndArm);
        assert_eq!(g.on_up(ms(b, 60)), UpAction::DismissOnly);
    }

    // ---- base issue: menu click vs Expanded blur ORDER ----

    /// Forward order (already handled): blur first, then menu click clears the arm.
    #[test]
    fn menu_after_expanded_blur_clears_arm() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        show_expanded_from_hidden(&mut g);
        g.on_focus_gained();
        assert_eq!(blur(&mut g, true, false, false, b), BlurResolution::DismissAndArm);
        g.on_menu_click();
        assert_eq!(g.on_up(ms(b, 20)), UpAction::Ignore);
        g.on_down(false, ms(b, 100));
        assert_eq!(g.on_up(ms(b, 140)), UpAction::Toggle);
    }

    /// REVERSE order (the bug): menu click first, Expanded blur resolves later, then an ordinary
    /// icon click within 300 ms must NOT be swallowed.
    #[test]
    fn menu_then_expanded_blur_then_click_within_300ms_is_not_swallowed() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        show_expanded_from_hidden(&mut g);
        g.on_focus_gained();
        g.on_menu_click(); // right-click on the open panel
        assert_eq!(blur(&mut g, true, false, false, ms(b, 10)), BlurResolution::Dismiss);
        g.on_mode_change(false, false);
        g.on_down(false, ms(b, 120));
        assert_eq!(g.on_up(ms(b, 160)), UpAction::Toggle);
    }

    /// Menu session ends when the panel becomes visible again: a later real dismissal arms again.
    #[test]
    fn menu_session_ends_when_panel_shown_again() {
        let mut g = TrayClickGuard::new();
        let b = t0();
        g.on_menu_click();
        show_expanded_from_hidden(&mut g); // "Show panel" from the menu
        g.on_focus_gained();
        assert_eq!(blur(&mut g, true, false, false, b), BlurResolution::DismissAndArm);
    }
}

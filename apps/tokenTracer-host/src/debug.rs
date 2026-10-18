//! Opt-in tray / panel decision log (`TT_TRAY_DEBUG`), OFF by default.
//!
//! * unset / `0` / empty → no logging, no cost beyond one cached env lookup.
//! * `1` / `true`        → lines to **stderr**.
//! * anything else       → treated as a **file path**, lines appended (use this when the
//!   app is started with `open`, which discards stderr).
//!
//! Line format: `[tt-tray mono=<ms since process start, 3 decimals> unix=<epoch ms>] message`.
//! `mono` is monotonic (use it for ordering/deltas); `unix` is wall-clock epoch milliseconds so
//! lines can be matched to a driver script's `Date.now()`.

use std::io::Write;
use std::sync::OnceLock;
use std::time::Instant;

fn target() -> &'static Option<String> {
    static T: OnceLock<Option<String>> = OnceLock::new();
    T.get_or_init(|| match std::env::var("TT_TRAY_DEBUG") {
        Ok(v) if !v.is_empty() && v != "0" => Some(v),
        _ => None,
    })
}

fn epoch() -> &'static Instant {
    static E: OnceLock<Instant> = OnceLock::new();
    E.get_or_init(Instant::now)
}

/// Is tray debug logging enabled? (Also pins the monotonic epoch on first call.)
pub fn enabled() -> bool {
    let on = target().is_some();
    if on {
        let _ = epoch();
    }
    on
}

/// Monotonic milliseconds since the first log call (fractional).
pub fn mono_ms() -> f64 {
    epoch().elapsed().as_secs_f64() * 1000.0
}

/// Epoch milliseconds (wall clock).
pub fn unix_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[doc(hidden)]
pub fn write_line(msg: &str) {
    let Some(t) = target() else { return };
    let line = format!(
        "[tt-tray mono={:.3} unix={}] {}\n",
        mono_ms(),
        unix_ms(),
        msg
    );
    if t == "1" || t.eq_ignore_ascii_case("true") {
        let _ = std::io::stderr().write_all(line.as_bytes());
    } else if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(t) {
        let _ = f.write_all(line.as_bytes());
    }
}

/// `tt_dbg!("fmt", args…)` — formats only when `TT_TRAY_DEBUG` is enabled.
macro_rules! tt_dbg {
    ($($arg:tt)*) => {
        if $crate::debug::enabled() {
            $crate::debug::write_line(&format!($($arg)*));
        }
    };
}

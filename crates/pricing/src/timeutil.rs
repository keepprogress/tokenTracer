//! Day-boundary helpers (pure; no filesystem).
//!
//! Default local zone for tokenTracer fixtures / Taipei users: Asia/Taipei (UTC+8).
//! Without chrono-tz we hardcode a small IANA → FixedOffset map.

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};
use thiserror::Error;

use crate::models::DayBoundary;

/// Default IANA label when day_boundary = local.
pub const DEFAULT_LOCAL_TZ: &str = "Asia/Taipei";

#[derive(Debug, Error)]
pub enum TimeError {
    #[error("unrecognized timezone '{0}' (supported: UTC, Asia/Taipei, or ±HH:MM offset)")]
    UnknownTimezone(String),
    #[error("invalid timestamp '{0}': {1}")]
    BadTimestamp(String, String),
    #[error("invalid calendar date '{0}'")]
    BadDate(String),
}

/// Resolve timezone label to a FixedOffset.
pub fn resolve_offset(day_boundary: DayBoundary, timezone: Option<&str>) -> Result<FixedOffset, TimeError> {
    match day_boundary {
        DayBoundary::Utc => Ok(FixedOffset::east_opt(0).unwrap()),
        DayBoundary::Local => {
            let label = timezone.unwrap_or(DEFAULT_LOCAL_TZ);
            offset_for_label(label)
        }
    }
}

pub fn timezone_label(day_boundary: DayBoundary, timezone: Option<&str>) -> String {
    match day_boundary {
        DayBoundary::Utc => "UTC".to_string(),
        DayBoundary::Local => timezone.unwrap_or(DEFAULT_LOCAL_TZ).to_string(),
    }
}

fn offset_for_label(label: &str) -> Result<FixedOffset, TimeError> {
    let l = label.trim();
    if l.eq_ignore_ascii_case("UTC") || l.eq_ignore_ascii_case("Z") {
        return Ok(FixedOffset::east_opt(0).unwrap());
    }
    if l.eq_ignore_ascii_case("Asia/Taipei") || l.eq_ignore_ascii_case("CST") {
        return Ok(FixedOffset::east_opt(8 * 3600).unwrap());
    }
    // ±HH:MM or +0800
    if let Some(off) = parse_offset_str(l) {
        return Ok(off);
    }
    Err(TimeError::UnknownTimezone(label.to_string()))
}

fn parse_offset_str(s: &str) -> Option<FixedOffset> {
    let s = s.strip_prefix("UTC").unwrap_or(s);
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let sign = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let rest = &s[1..];
    let (hh, mm) = if rest.len() == 5 && rest.as_bytes()[2] == b':' {
        (rest[0..2].parse::<i32>().ok()?, rest[3..5].parse::<i32>().ok()?)
    } else if rest.len() == 4 {
        (rest[0..2].parse::<i32>().ok()?, rest[2..4].parse::<i32>().ok()?)
    } else if rest.len() == 2 {
        (rest.parse::<i32>().ok()?, 0)
    } else {
        return None;
    };
    FixedOffset::east_opt(sign * (hh * 3600 + mm * 60))
}

pub fn parse_event_ts(ts: &str) -> Result<DateTime<Utc>, TimeError> {
    DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            // Tolerate missing fractional seconds variants chrono might miss via from_str
            ts.parse::<DateTime<Utc>>()
        })
        .map_err(|e| TimeError::BadTimestamp(ts.to_string(), e.to_string()))
}

/// Calendar date YYYY-MM-DD in the given offset for an instant.
pub fn calendar_date_in(offset: FixedOffset, instant: DateTime<Utc>) -> NaiveDate {
    instant.with_timezone(&offset).date_naive()
}

/// Half-open UTC window [day_start, next_day) for a calendar date in `offset`.
pub fn day_window_utc(offset: FixedOffset, date: NaiveDate) -> (DateTime<Utc>, DateTime<Utc>) {
    let start_local = offset
        .from_local_datetime(&date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()))
        .single()
        .expect("midnight exists");
    let end_local = start_local + Duration::days(1);
    (start_local.with_timezone(&Utc), end_local.with_timezone(&Utc))
}

pub fn parse_ymd(s: &str) -> Result<NaiveDate, TimeError> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| TimeError::BadDate(s.to_string()))
}

pub fn format_ymd(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

/// "Now" as UTC — injectable for tests via optional override in callers.
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

pub fn today_date(offset: FixedOffset, now: DateTime<Utc>) -> NaiveDate {
    calendar_date_in(offset, now)
}

/// Inclusive date span of `n` calendar days ending at `end` (inclusive).
pub fn rolling_n_days_end(end: NaiveDate, n: u32) -> NaiveDate {
    end - Duration::days((n.saturating_sub(1)) as i64)
}

pub fn dates_inclusive(start: NaiveDate, end: NaiveDate) -> Vec<NaiveDate> {
    let mut out = Vec::new();
    if end < start {
        return out;
    }
    let mut d = start;
    loop {
        out.push(d);
        if d == end {
            break;
        }
        d += Duration::days(1);
    }
    out
}

/// Year/month helpers kept for clarity in callers.
pub fn ymd_parts(d: NaiveDate) -> (i32, u32, u32) {
    (d.year(), d.month(), d.day())
}

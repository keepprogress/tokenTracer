//! Pure range filtering for UsageEvents (OPEN-UI-1).

use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::models::{DayBoundary, RangeKind, SpendRange, UsageEvent};
use crate::timeutil::{
    calendar_date_in, day_window_utc, parse_event_ts, resolve_offset, rolling_n_days_end,
    timezone_label, today_date, TimeError,
};

#[derive(Debug, Clone)]
pub struct RangeFilterOpts<'a> {
    pub kind: RangeKind,
    pub day_boundary: DayBoundary,
    /// IANA / offset label when day_boundary = local. Default Asia/Taipei.
    pub timezone: Option<&'a str>,
    /// Injected "now" for deterministic tests.
    pub now: DateTime<Utc>,
    /// Optional custom inclusive calendar dates (boundary TZ) when kind = Custom.
    pub custom_from: Option<NaiveDate>,
    pub custom_to: Option<NaiveDate>,
}

impl<'a> RangeFilterOpts<'a> {
    pub fn all(now: DateTime<Utc>) -> Self {
        Self {
            kind: RangeKind::All,
            day_boundary: DayBoundary::Local,
            timezone: None,
            now,
            custom_from: None,
            custom_to: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedWindow {
    pub kind: RangeKind,
    /// Inclusive UTC start (or None = unbounded for All).
    pub start_utc: Option<DateTime<Utc>>,
    /// Exclusive UTC end (or None = unbounded for All).
    pub end_utc: Option<DateTime<Utc>>,
    pub day_boundary: DayBoundary,
    pub timezone: String,
    /// Inclusive calendar start YYYY-MM-DD (when bounded).
    pub cal_start: Option<NaiveDate>,
    /// Inclusive calendar end YYYY-MM-DD (when bounded).
    pub cal_end: Option<NaiveDate>,
}

impl ResolvedWindow {
    pub fn to_spend_range(&self) -> SpendRange {
        SpendRange {
            kind: self.kind.clone(),
            start: self
                .start_utc
                .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
            end: self
                .end_utc
                .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
        }
    }

    pub fn contains_instant(&self, ts: DateTime<Utc>) -> bool {
        if let Some(start) = self.start_utc {
            if ts < start {
                return false;
            }
        }
        if let Some(end) = self.end_utc {
            if ts >= end {
                return false;
            }
        }
        true
    }
}

pub fn resolve_window(opts: &RangeFilterOpts<'_>) -> Result<ResolvedWindow, TimeError> {
    let offset = resolve_offset(opts.day_boundary, opts.timezone)?;
    let tz_label = timezone_label(opts.day_boundary, opts.timezone);
    let today = today_date(offset, opts.now);

    let (kind, cal_start, cal_end) = match opts.kind {
        RangeKind::All => (RangeKind::All, None, None),
        RangeKind::Today => (RangeKind::Today, Some(today), Some(today)),
        RangeKind::Days7 => {
            let start = rolling_n_days_end(today, 7);
            (RangeKind::Days7, Some(start), Some(today))
        }
        RangeKind::Days30 => {
            let start = rolling_n_days_end(today, 30);
            (RangeKind::Days30, Some(start), Some(today))
        }
        RangeKind::Days90 => {
            let start = rolling_n_days_end(today, 90);
            (RangeKind::Days90, Some(start), Some(today))
        }
        RangeKind::Custom => {
            let from = opts.custom_from.ok_or_else(|| {
                TimeError::BadDate("custom range missing --from".into())
            })?;
            let to = opts.custom_to.unwrap_or(today);
            (RangeKind::Custom, Some(from), Some(to))
        }
    };

    let (start_utc, end_utc) = match (cal_start, cal_end) {
        (Some(s), Some(e)) => {
            let (start, _) = day_window_utc(offset, s);
            let (_, end_excl) = day_window_utc(offset, e);
            (Some(start), Some(end_excl))
        }
        _ => (None, None),
    };

    Ok(ResolvedWindow {
        kind,
        start_utc,
        end_utc,
        day_boundary: opts.day_boundary,
        timezone: tz_label,
        cal_start,
        cal_end,
    })
}

/// Filter events whose `ts` falls in the resolved half-open window.
/// Pure: no I/O. Returns owned Vec (lifetime-independent).
pub fn filter_events_by_range(
    events: &[UsageEvent],
    opts: &RangeFilterOpts<'_>,
) -> Result<(Vec<UsageEvent>, ResolvedWindow), TimeError> {
    let window = resolve_window(opts)?;
    if matches!(window.kind, RangeKind::All) && window.start_utc.is_none() {
        return Ok((events.to_vec(), window));
    }
    let mut out = Vec::new();
    for ev in events {
        match parse_event_ts(&ev.ts) {
            Ok(ts) if window.contains_instant(ts) => out.push(ev.clone()),
            Ok(_) => {}
            Err(_) => {
                // Skip unparseable timestamps rather than fail the whole batch.
            }
        }
    }
    Ok((out, window))
}

/// Bucket an event into a calendar date in the given boundary, if ts parses.
pub fn event_calendar_date(
    event: &UsageEvent,
    day_boundary: DayBoundary,
    timezone: Option<&str>,
) -> Result<Option<NaiveDate>, TimeError> {
    let offset = resolve_offset(day_boundary, timezone)?;
    let ts = match parse_event_ts(&event.ts) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };
    Ok(Some(calendar_date_in(offset, ts)))
}

/// Convenience: N-day lookback ending today as half-open UTC span length hint.
pub fn approx_span_days(kind: &RangeKind) -> Option<u32> {
    match kind {
        RangeKind::Today => Some(1),
        RangeKind::Days7 => Some(7),
        RangeKind::Days30 => Some(30),
        RangeKind::Days90 => Some(90),
        RangeKind::All | RangeKind::Custom => None,
    }
}

/// Extend end exclusive by zero — kept for API clarity.
pub fn exclusive_end_after_inclusive_day(
    offset: chrono::FixedOffset,
    inclusive_end: NaiveDate,
) -> DateTime<Utc> {
    let (_, end) = day_window_utc(offset, inclusive_end);
    end
}

pub fn shift_days(d: NaiveDate, delta: i64) -> NaiveDate {
    d + Duration::days(delta)
}

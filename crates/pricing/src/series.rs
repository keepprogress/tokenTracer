//! Daily spend series (OPEN-UI-2). Pure over events + price table.

use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

use crate::models::*;
use crate::price_table::PriceTable;
use crate::pricing::{event_amount_usd_parts, price, round_money};
use crate::range::{filter_events_by_range, resolve_window, RangeFilterOpts};
use crate::timeutil::{dates_inclusive, format_ymd, parse_ymd, TimeError};

#[derive(Debug, Clone)]
pub struct SeriesOpts<'a> {
    pub day_boundary: DayBoundary,
    pub timezone: Option<&'a str>,
    pub now: DateTime<Utc>,
    /// Prefers --from/--to when both set (kind=custom).
    pub range_kind: RangeKind,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
}

/// Build a continuous daily series; empty days get amount 0.
pub fn daily_spend_series(
    events: &[UsageEvent],
    price_table: &PriceTable,
    fx: Option<&FxSnapshot>,
    opts: &SeriesOpts<'_>,
) -> Result<DailySpendSeries, TimeError> {
    let (kind, custom_from, custom_to) = if opts.from.is_some() {
        let from = parse_ymd(opts.from.unwrap())?;
        let to = match opts.to {
            Some(t) => parse_ymd(t)?,
            None => {
                let w = resolve_window(&RangeFilterOpts {
                    kind: RangeKind::Today,
                    day_boundary: opts.day_boundary,
                    timezone: opts.timezone,
                    now: opts.now,
                    custom_from: None,
                    custom_to: None,
                })?;
                w.cal_end.unwrap()
            }
        };
        (RangeKind::Custom, Some(from), Some(to))
    } else {
        (opts.range_kind.clone(), None, None)
    };

    // For `all`, bound by earliest event calendar day → today.
    let filter_opts = if matches!(kind, RangeKind::All) {
        let offset = crate::timeutil::resolve_offset(opts.day_boundary, opts.timezone)?;
        let today = crate::timeutil::today_date(offset, opts.now);
        let mut earliest = today;
        for ev in events {
            if let Ok(ts) = crate::timeutil::parse_event_ts(&ev.ts) {
                let d = crate::timeutil::calendar_date_in(offset, ts);
                if d < earliest {
                    earliest = d;
                }
            }
        }
        RangeFilterOpts {
            kind: RangeKind::Custom,
            day_boundary: opts.day_boundary,
            timezone: opts.timezone,
            now: opts.now,
            custom_from: Some(earliest),
            custom_to: Some(today),
        }
    } else {
        RangeFilterOpts {
            kind: kind.clone(),
            day_boundary: opts.day_boundary,
            timezone: opts.timezone,
            now: opts.now,
            custom_from,
            custom_to,
        }
    };

    let (filtered, window) = filter_events_by_range(events, &filter_opts)?;
    let cal_start = window.cal_start.expect("series window bounded");
    let cal_end = window.cal_end.expect("series window bounded");

    // Sum per calendar day via per-event amounts (same formula as price()).
    let mut by_day: BTreeMap<String, f64> = BTreeMap::new();
    for d in dates_inclusive(cal_start, cal_end) {
        by_day.insert(format_ymd(d), 0.0);
    }

    let offset = crate::timeutil::resolve_offset(opts.day_boundary, opts.timezone)?;
    let mut vendor = 0u64;
    let mut notional = 0u64;
    for ev in &filtered {
        let Ok(ts) = crate::timeutil::parse_event_ts(&ev.ts) else {
            continue;
        };
        let date = format_ymd(crate::timeutil::calendar_date_in(offset, ts));
        let (amt, used_vendor, _) = event_amount_usd_parts(ev, price_table);
        if used_vendor {
            vendor += 1;
        } else {
            notional += 1;
        }
        *by_day.entry(date).or_insert(0.0) += amt;
    }

    // Apply FX scale if TWD.
    let summary_probe = price(&filtered, price_table, fx);
    let scale = if matches!(summary_probe.currency, Currency::Twd) {
        fx.map(|s| s.rate).unwrap_or(1.0)
    } else {
        1.0
    };

    let points: Vec<DailySpendPoint> = by_day
        .into_iter()
        .map(|(date, amount)| DailySpendPoint {
            date,
            amount: round_money(amount * scale),
        })
        .collect();

    let cost_nature = if vendor > 0 && notional > 0 {
        CostNature::Mixed
    } else if vendor > 0 {
        CostNature::VendorReported
    } else {
        CostNature::NotionalApiEstimate
    };

    // Preserve requested kind label for all/7d/… even when internally custom-bounded.
    let out_kind = if opts.from.is_some() {
        RangeKind::Custom
    } else {
        opts.range_kind.clone()
    };

    Ok(DailySpendSeries {
        currency: summary_probe.currency,
        grain: "day".into(),
        day_boundary: opts.day_boundary,
        timezone: window.timezone,
        range: SeriesRange {
            kind: out_kind,
            start: format_ymd(cal_start),
            end: format_ymd(cal_end),
        },
        points,
        price_table_version: price_table.version.clone(),
        cost_nature,
        computed_at: summary_probe.computed_at,
        meta: None,
    })
}

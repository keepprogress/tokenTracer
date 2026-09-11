//! tokenTracer pricing core.
//!
//! Pure pricing is isolated from I/O: parsers read bytes/strings; `price(...)`
//! never touches the filesystem. Range filtering and daily series are also pure.
//! `import_meta` and `import` intentionally perform file I/O (state + discover).
//! AC v1.4 official Admin parse/reconcile is pure JSON (fixture-first).

pub mod cursor_official;
pub mod import;
pub mod import_meta;
pub mod models;
pub mod parsers;
pub mod price_table;
pub mod pricing;
pub mod range;
pub mod series;
pub mod timeutil;

pub use cursor_official::{
    gate_undocumented_dashboard, parse_admin_filtered_usage_events, parse_teams_spend,
    reconcile_charged_cents, reconcile_charged_cents_with_tol, require_admin_api_key,
    sum_charged_cents, AdminApiClient, CursorOfficialError, EnvAdminApiClient,
    CODE_MISSING_KEY, SOURCE_TAG as CURSOR_OFFICIAL_SOURCE_TAG,
};
pub use import::{
    import_from_discover, import_from_discover_result, load_discover_result, DiscoverError,
    DiscoverResult, DiscoverSource, FileListResult, FromDiscoverOpts, FromDiscoverResult,
    ImportError, ImportReport, scope_event_id,
};
pub use import_meta::{read_import_meta, record_import, write_import_meta};
pub use models::*;
pub use price_table::{PriceTable, PriceTableError};
pub use pricing::{
    event_amount_usd_parts, event_notional_usd, is_cursor_local_enrichment_event,
    is_cursor_official_admin_event, price, price_with_range, resolve_usage_pool, round_money,
};
pub use range::{filter_events_by_range, resolve_window, RangeFilterOpts, ResolvedWindow};
pub use series::{daily_spend_series, SeriesOpts};

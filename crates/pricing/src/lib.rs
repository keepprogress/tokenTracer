//! tokenTracer pricing core.
//!
//! Pure pricing is isolated from I/O: parsers read bytes/strings; `price(...)`
//! never touches the filesystem. Range filtering and daily series are also pure.
//! `import_meta` and `import` intentionally perform file I/O (state + discover).

pub mod import;
pub mod import_meta;
pub mod models;
pub mod parsers;
pub mod price_table;
pub mod pricing;
pub mod range;
pub mod series;
pub mod timeutil;

pub use import::{
    import_from_discover, import_from_discover_result, load_discover_result, DiscoverError,
    DiscoverResult, DiscoverSource, FileListResult, FromDiscoverOpts, FromDiscoverResult,
    ImportError, ImportReport,
};
pub use import_meta::{read_import_meta, record_import, write_import_meta};
pub use models::*;
pub use price_table::{PriceTable, PriceTableError};
pub use pricing::{
    event_amount_usd_parts, event_notional_usd, price, price_with_range, resolve_usage_pool,
    round_money,
};
pub use range::{filter_events_by_range, resolve_window, RangeFilterOpts, ResolvedWindow};
pub use series::{daily_spend_series, SeriesOpts};

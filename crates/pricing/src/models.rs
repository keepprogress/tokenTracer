use serde::{Deserialize, Serialize};

/// Agent identifiers aligned with AC §5.1 / AC v1.1a.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentId {
    Codex,
    ClaudeCode,
    Cursor,
    GrokBuild,
    Cline,
    Opencode,
    GithubCopilot,
    Other,
}

impl AgentId {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentId::Codex => "codex",
            AgentId::ClaudeCode => "claude_code",
            AgentId::Cursor => "cursor",
            AgentId::GrokBuild => "grok_build",
            AgentId::Cline => "cline",
            AgentId::Opencode => "opencode",
            AgentId::GithubCopilot => "github_copilot",
            AgentId::Other => "other",
        }
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Normalized usage event (AC §5.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageEvent {
    pub id: String,
    pub agent: AgentId,
    pub source: String,
    /// ISO-8601 UTC timestamp.
    pub ts: String,
    pub model: Option<String>,
    /// Cursor usage pool (AC v1.3a). Not a model id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_pool: Option<UsagePool>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_write_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_cost_usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    Usd,
    Twd,
}

impl Currency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Currency::Usd => "USD",
            Currency::Twd => "TWD",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSpendStatus {
    Ok,
    Partial,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSpend {
    pub agent: AgentId,
    pub amount: f64,
    pub status: AgentSpendStatus,
}

/// Cursor / agent usage pool (AC v1.3a). Not a model name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsagePool {
    CursorModels,
    OtherModels,
    Unknown,
}

impl UsagePool {
    pub fn as_str(self) -> &'static str {
        match self {
            UsagePool::CursorModels => "cursor_models",
            UsagePool::OtherModels => "other_models",
            UsagePool::Unknown => "unknown",
        }
    }

    /// Community PARTIAL map from Cursor AggregatedUsage `tier` (EC-cursor-other-models-v1).
    /// `1 → other_models`, `2 → cursor_models`. Not an official enum.
    pub fn from_cursor_tier(tier: i64) -> Self {
        match tier {
            1 => UsagePool::OtherModels,
            2 => UsagePool::CursorModels,
            _ => UsagePool::Unknown,
        }
    }
}

impl std::fmt::Display for UsagePool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Sentinel model id when Cursor event has no concrete model (NOT an Other-pool name).
pub const CURSOR_UNKNOWN_MODEL: &str = "cursor:unknown";

/// Forbidden literal model strings (must never be treated as model or pool authority).
pub const FORBIDDEN_OTHER_MODEL_LITERALS: &[&str] = &["Other", "Other model", "other", "other model"];

/// Per-(agent, model) spend row (AC v1.3 ∪ v1.3a / BY-MODEL v0.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelSpend {
    /// Concrete model id only (e.g. `composer-1.5`). Missing Cursor model → `cursor:unknown`.
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_pool: Option<UsagePool>,
    /// `null` when unpriced (N16 / OPEN-P4).
    #[serde(default)]
    pub amount: Option<f64>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    /// false = unpriced; excluded from `total`.
    pub priced: bool,
    /// Price-table key after `alias_of` resolution; null if unmatched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_table_row: Option<String>,
}

/// Cursor dual-pool rollup (AC-F13′).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsagePoolSpend {
    pub agent: AgentId,
    pub pool: UsagePool,
    #[serde(default)]
    pub amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percent_used: Option<f64>,
}

/// Unpriced token buckets (optional diagnostics).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnpricedTokens {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Product pricing mode (AC-F12). Locked to notional API list prices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PricingMode {
    NotionalApiEstimate,
}

impl PricingMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PricingMode::NotionalApiEstimate => "notional_api_estimate",
        }
    }
}

pub const NOTIONAL_DISCLAIMER: &str =
    "Notional estimate from public API list prices (price_table_version). Not a subscription invoice, credit-card charge, or vendor bill. Subscription plans (Claude Pro/Max, ChatGPT Plus/Pro, Cursor) are not invoices.";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FxSnapshot {
    pub pair: String,
    pub rate: f64,
    pub as_of: String,
    pub source: String,
}

/// Range kinds for SpendSummary + series (OPEN-UI-1/2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RangeKind {
    All,
    Today,
    #[serde(rename = "7d")]
    Days7,
    #[serde(rename = "30d")]
    Days30,
    #[serde(rename = "90d")]
    Days90,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpendRange {
    pub kind: RangeKind,
    /// Inclusive window start as ISO-8601 UTC instant (filled for today / N-day).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// Exclusive window end as ISO-8601 UTC instant for half-open [start, end).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

/// Day boundary mode (OPEN-UI-1). Default = local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DayBoundary {
    #[default]
    Local,
    Utc,
}

impl DayBoundary {
    pub fn as_str(self) -> &'static str {
        match self {
            DayBoundary::Local => "local",
            DayBoundary::Utc => "utc",
        }
    }
}

/// Nature of the computed cost (AC-F6 / commander EXTRA).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostNature {
    /// Local tokens × public API price table (subscription ≠ invoice).
    NotionalApiEstimate,
    /// Vendor-reported dollar amount (e.g. Cursor Admin API / CSV).
    VendorReported,
    /// Mix of natures in one summary.
    Mixed,
}

impl CostNature {
    pub fn as_str(&self) -> &'static str {
        match self {
            CostNature::NotionalApiEstimate => "notional_api_estimate",
            CostNature::VendorReported => "vendor_reported",
            CostNature::Mixed => "mixed",
        }
    }
}

/// Spend query result (AC §5.2 + EXTRA fields).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpendSummary {
    pub currency: Currency,
    /// Sum of priced rows only; unpriced excluded (AC v1.3 N16).
    pub total: f64,
    pub by_agent: Vec<AgentSpend>,
    /// Per-(agent, model) breakdown; concrete model ids only (AC v1.3a).
    pub by_model: Vec<ModelSpend>,
    /// Cursor dual-pool rollup (AC-F13′); omit/empty when no Cursor events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub by_usage_pool: Vec<UsagePoolSpend>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unpriced_tokens: Option<Vec<UnpricedTokens>>,
    pub price_table_version: String,
    /// AC-F12: always notional_api_estimate for this product path.
    pub pricing_mode: PricingMode,
    /// Must state notional ≠ invoice / ≠ credit-card bill.
    pub disclaimer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fx_snapshot: Option<FxSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<SpendRange>,
    pub computed_at: String,
    /// Finer diagnostic: notional / vendor_reported / mixed (kept alongside pricing_mode).
    pub cost_nature: CostNature,
    pub price_table_source: String,
    /// Optional diagnostics (unknown models skipped, warnings, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// One day in a trend series (OPEN-UI-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailySpendPoint {
    /// YYYY-MM-DD in the series timezone / day boundary.
    pub date: String,
    pub amount: f64,
}

/// Calendar-day spend series (OPEN-UI-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailySpendSeries {
    pub currency: Currency,
    /// v0 only supports `"day"`.
    pub grain: String,
    pub day_boundary: DayBoundary,
    /// IANA-ish label, e.g. `"Asia/Taipei"` or `"UTC"`.
    pub timezone: String,
    pub range: SeriesRange,
    pub points: Vec<DailySpendPoint>,
    pub price_table_version: String,
    pub cost_nature: CostNature,
    pub computed_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesRange {
    pub kind: RangeKind,
    /// Inclusive calendar date YYYY-MM-DD (boundary timezone).
    pub start: String,
    /// Inclusive calendar date YYYY-MM-DD (boundary timezone).
    pub end: String,
}

/// Import pipeline freshness (OPEN-UI-3) — separate from SpendSummary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportMeta {
    pub last_imported_at: Option<String>,
    pub last_import_source_ids: Vec<String>,
    pub parse_error_count: u64,
    pub events_upserted: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_path: Option<String>,
    pub schema_version: String,
}

impl ImportMeta {
    pub const SCHEMA_VERSION: &'static str = "import-meta/v0";

    pub fn empty() -> Self {
        Self {
            last_imported_at: None,
            last_import_source_ids: Vec::new(),
            parse_error_count: 0,
            events_upserted: 0,
            ledger_path: None,
            schema_version: Self::SCHEMA_VERSION.to_string(),
        }
    }
}

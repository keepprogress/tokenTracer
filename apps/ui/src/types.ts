/** Ledger / shell-host shapes consumed by the mini-panel (UI does not price). */

export type Currency = "USD" | "TWD";
export type AgentId =
  | "codex"
  | "claude_code"
  | "cursor"
  | "grok_build"
  | "cline"
  | "opencode"
  | "github_copilot"
  | "other";

export type AgentStatus = "ok" | "partial" | "unsupported";
export type PanelRangeKind = "all" | "7d" | "30d" | "90d";
export type RangeKind = "today" | PanelRangeKind;
export type PricingMode = "notional_api_estimate";
export type UsagePool = "cursor_models" | "other_models" | "unknown";

export interface ByModelEntry {
  model: string;
  agent: AgentId | null;
  amount: number | null;
  input_tokens: number;
  output_tokens: number;
  priced: boolean;
  /** Cursor usage pool when known; null/omit for non-Cursor agents. */
  usage_pool?: UsagePool | null;
  price_table_row?: string | null;
}

export interface ByUsagePoolEntry {
  agent: "cursor";
  pool: UsagePool;
  amount: number | null;
  percent_used?: number | null;
}

export interface SpendSummary {
  currency: Currency;
  total: number | null;
  pricing_mode: PricingMode | string;
  price_table_version: string;
  price_table_source: string;
  by_agent: { agent: AgentId; amount: number; status: AgentStatus }[];
  by_model: ByModelEntry[];
  by_usage_pool?: ByUsagePoolEntry[];
  unpriced_tokens?: { model: string; input_tokens: number; output_tokens: number }[];
  fx_snapshot?: { pair: string; rate: number; as_of: string; source: string };
  range?: { kind: RangeKind; start?: string; end?: string };
  computed_at: string;
  disclaimer: string;
  /** @deprecated prefer pricing_mode */
  cost_nature?: string;
}

export interface DailySpendSeries {
  currency: Currency;
  grain: "day";
  day_boundary: "local" | "utc";
  timezone: string;
  range: { kind: string; start: string; end: string };
  points: { date: string; amount: number }[];
  price_table_version: string;
  cost_nature: string;
  computed_at: string;
}

export interface ImportMeta {
  last_imported_at: string | null;
  last_import_source_ids: string[];
  parse_error_count: number;
  events_upserted: number;
  ledger_path?: string;
  schema_version: "import-meta/v0";
}

export interface AgentRowVM {
  agent: AgentId;
  display_name: string;
  amount: number;
  share: number | null;
  status: AgentStatus;
}

export interface ModelRowVM {
  model: string;
  display_name: string;
  agent: AgentId | null;
  usage_pool: UsagePool | null;
  amount: number | null;
  input_tokens: number;
  output_tokens: number;
  priced: boolean;
  share: number | null;
  is_unknown: boolean;
}

export interface PoolGroupVM {
  agent: "cursor";
  pool: UsagePool;
  display_name: string;
  amount: number | null;
  percent_used: number | null;
  models: ModelRowVM[];
}

export interface DailyPointVM {
  date: string;
  amount: number;
}

export interface WarningVM {
  agent: AgentId;
  level: "partial";
  text: string;
}

export interface MiniPanelVM {
  currency: Currency;
  today_total: number | null;
  range_total: number | null;
  range: { kind: PanelRangeKind; start?: string; end?: string };
  by_agent: AgentRowVM[];
  by_model: ModelRowVM[] | null;
  by_usage_pool: PoolGroupVM[] | null;
  trend: DailyPointVM[] | null;
  last_imported_at: string | null;
  partial_warnings: WarningVM[];
  pricing_mode: string;
  price_table_version: string;
  price_table_source: string;
  disclaimer: string;
  computed_at: string;
  fx_snapshot?: SpendSummary["fx_snapshot"];
  load_state: "idle" | "loading" | "error";
  error?: { code: string; message: string };
}

/** Fallback when SpendSummary.disclaimer is absent (EN/ZH-safe). */
export const DEFAULT_DISCLAIMER =
  "Notional API estimate — not an invoice; ≠ credit-card bill. / 名義估價，非帳單／≠信用卡帳單。";

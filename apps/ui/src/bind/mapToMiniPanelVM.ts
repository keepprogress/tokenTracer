/**
 * Pure SpendSummary / series / ImportMeta → MiniPanelVM mapping.
 * No pricing. No side effects.
 * UI-BIND v0.4-draft / BY-MODEL v0.2 — filter sentinel models; group by usage_pool;
 * SpendingAlign is a separate payload (never f(by_usage_pool.amount) → pct).
 */
import type {
  AgentId,
  AgentRowVM,
  DailySpendSeries,
  ImportMeta,
  MiniPanelVM,
  ModelRowVM,
  OfficialAdminReconcile,
  OfficialAdminReconcileVM,
  PanelRangeKind,
  PoolGroupVM,
  SpendingAlign,
  SpendingAlignSourceMode,
  SpendingAlignVM,
  SpendSummary,
  UsagePool,
  WarningVM,
} from "../types";
import { DEFAULT_DISCLAIMER, F17_PARTIAL_REASON } from "../types";

const DISPLAY_NAMES: Record<string, string> = {
  cursor: "Cursor",
  claude_code: "Claude Code",
  codex: "Codex",
  grok_build: "Grok Build",
  cline: "Cline",
  opencode: "OpenCode",
  github_copilot: "GitHub Copilot",
  other: "Other",
};

/** Migration-only sentinels — never shown as product model rows. */
const OTHER_SENTINELS = new Set(["cursor:other", "__other__"]);

const POOL_DISPLAY: Record<UsagePool, string> = {
  cursor_models: "Cursor Models",
  other_models: "Other Models",
  unknown: "池未知／待 API",
};

export function displayNameFor(agent: AgentId | string): string {
  return DISPLAY_NAMES[agent] ?? String(agent).replace(/_/g, " ");
}

/** Concrete model id display — never invent "Other model". */
export function modelDisplayName(model: string): string {
  if (model === "cursor:unknown") return "Unknown model";
  return model;
}

function partialWarningText(agent: AgentId): string {
  if (agent === "cursor") {
    return "Cursor：本機 tokenCount 不可靠；計價依 API／估算路徑（EC-cursor-v1）";
  }
  if (agent === "claude_code" || agent === "codex") {
    return `${displayNameFor(agent)}：USD 可能為 notional API 估價，非帳單原件`;
  }
  return `${displayNameFor(agent)}：資料不完整（partial）`;
}

export function mapByAgent(
  by_agent: SpendSummary["by_agent"],
): { rows: AgentRowVM[]; warnings: WarningVM[] } {
  const usable = by_agent.filter((a) => a.status === "ok" || a.status === "partial");
  const denom = usable.reduce((s, a) => s + a.amount, 0);

  const rows: AgentRowVM[] = [...by_agent]
    .filter((a) => a.status !== "unsupported")
    .sort((a, b) => b.amount - a.amount)
    .map((a) => ({
      agent: a.agent,
      display_name: displayNameFor(a.agent),
      amount: a.amount,
      share: denom > 0 && (a.status === "ok" || a.status === "partial")
        ? a.amount / denom
        : null,
      status: a.status,
    }));

  const warnings: WarningVM[] = by_agent
    .filter((a) => a.status === "partial")
    .map((a) => ({
      agent: a.agent,
      level: "partial" as const,
      text: partialWarningText(a.agent),
    }));

  return { rows, warnings };
}

/**
 * Map by_model → ModelRowVM[].
 * Drops cursor:other / __other__ sentinels (migration only; never product rows).
 */
export function mapByModel(
  by_model: SpendSummary["by_model"] | undefined | null,
): ModelRowVM[] | null {
  if (!by_model || by_model.length === 0) return null;

  const filtered = by_model.filter((m) => !OTHER_SENTINELS.has(m.model));
  if (filtered.length === 0) return null;

  const pricedAmounts = filtered.filter((m) => m.priced && m.amount != null);
  const denom = pricedAmounts.reduce((s, m) => s + (m.amount as number), 0);

  const rows: ModelRowVM[] = [...filtered]
    .sort((a, b) => (b.amount ?? -1) - (a.amount ?? -1))
    .map((m) => {
      const is_unknown = m.model === "cursor:unknown";
      const share =
        m.priced && m.amount != null && denom > 0 ? m.amount / denom : null;
      return {
        model: m.model,
        display_name: modelDisplayName(m.model),
        agent: m.agent,
        usage_pool: m.usage_pool ?? null,
        amount: m.amount,
        input_tokens: m.input_tokens,
        output_tokens: m.output_tokens,
        priced: m.priced,
        share,
        is_unknown,
      };
    });

  return rows;
}

/** Group by_usage_pool + attach matching by_model rows (display only; no reprice). */
export function mapByUsagePool(
  by_usage_pool: SpendSummary["by_usage_pool"] | undefined | null,
  modelRows: ModelRowVM[] | null,
): PoolGroupVM[] | null {
  if (!by_usage_pool || by_usage_pool.length === 0) return null;

  const order: UsagePool[] = ["cursor_models", "other_models", "unknown"];
  const sorted = [...by_usage_pool].sort(
    (a, b) => order.indexOf(a.pool) - order.indexOf(b.pool),
  );

  return sorted.map((p) => ({
    agent: "cursor" as const,
    pool: p.pool,
    display_name: POOL_DISPLAY[p.pool],
    amount: p.amount,
    percent_used: p.percent_used ?? null,
    models: (modelRows ?? []).filter((m) => m.usage_pool === p.pool),
  }));
}


const SOURCE_MODES = new Set<SpendingAlignSourceMode>([
  "none",
  "manual_p1",
  "official_admin",
  "undocumented_opt_in",
]);

/**
 * Map SpendingAlign wire → SpendingAlignVM.
 * Pass-through only — MUST NOT accept notional totals / by_usage_pool amounts.
 */
export function mapSpendingAlign(
  raw: SpendingAlign | null | undefined,
): SpendingAlignVM | null {
  if (!raw) return null;
  const mode = SOURCE_MODES.has(raw.source_mode) ? raw.source_mode : "none";
  const partial = Boolean(raw.partial);
  let partial_reason = raw.partial_reason ?? null;
  if (partial && !partial_reason?.trim()) {
    partial_reason = F17_PARTIAL_REASON;
  }
  // Personal / manual path must keep F17 「無公開個人 usage API」 visible when partial.
  if (
    partial &&
    (mode === "manual_p1" || mode === "none") &&
    partial_reason &&
    !partial_reason.includes("無公開個人 usage API")
  ) {
    partial_reason = `${partial_reason} · 無公開個人 usage API`;
  }
  return {
    cursor_models_pct: raw.cursor_models_pct ?? null,
    other_models_pct: raw.other_models_pct ?? null,
    reset_label: raw.reset_label ?? null,
    on_demand: raw.on_demand ?? null,
    source_mode: mode,
    partial,
    partial_reason,
    grok_bot_week_note: raw.grok_bot_week_note ?? null,
    is_example: Boolean(raw._example),
    example_label: raw._label ?? null,
  };
}

/** Optional F14 reconcile VM — display only; never write cents into spending_align %. */
export function mapOfficialAdminReconcile(
  raw: OfficialAdminReconcile | null | undefined,
): OfficialAdminReconcileVM | null {
  if (!raw) return null;
  return {
    events_charged_cents_sum: raw.events_charged_cents_sum,
    spend_overall_cents: raw.spend_overall_cents,
    delta_cents: raw.delta_cents,
    within_tol: raw.within_tol,
    price_period_start: raw.price_period_start ?? null,
    computed_at: raw.computed_at,
  };
}

export interface MapInput {
  today: SpendSummary | null;
  rangeSummary: SpendSummary;
  series: DailySpendSeries | null;
  importMeta: ImportMeta | null;
  panelRange: PanelRangeKind;
  /** Independent B-surface payload — never derived from rangeSummary notional $. */
  spending_align?: SpendingAlign | null;
  official_admin_reconcile?: OfficialAdminReconcile | null;
  load_state?: MiniPanelVM["load_state"];
  error?: MiniPanelVM["error"];
}

/** Pure mapper — fixtures / live IPC payloads in, view-model out. */
export function mapToMiniPanelVM(input: MapInput): MiniPanelVM {
  const { rows, warnings } = mapByAgent(input.rangeSummary.by_agent);
  const by_model = mapByModel(input.rangeSummary.by_model);
  const by_usage_pool = mapByUsagePool(
    input.rangeSummary.by_usage_pool,
    by_model,
  );
  const range = input.rangeSummary.range;
  const pricing_mode =
    input.rangeSummary.pricing_mode ??
    input.rangeSummary.cost_nature ??
    "notional_api_estimate";
  // Hard rule: spending_align comes only from its own payload — never from notional $.
  const spending_align = mapSpendingAlign(input.spending_align);
  const official_admin_reconcile = mapOfficialAdminReconcile(
    input.official_admin_reconcile,
  );

  return {
    currency: input.rangeSummary.currency,
    today_total: input.today ? input.today.total : null,
    range_total: input.rangeSummary.total,
    range: {
      kind: input.panelRange,
      start: range?.start,
      end: range?.end,
    },
    by_agent: rows,
    by_model,
    by_usage_pool,
    trend: input.series
      ? input.series.points.map((p) => ({ date: p.date, amount: p.amount }))
      : null,
    last_imported_at: input.importMeta?.last_imported_at ?? null,
    partial_warnings: warnings,
    pricing_mode,
    price_table_version: input.rangeSummary.price_table_version,
    price_table_source: input.rangeSummary.price_table_source ?? "",
    disclaimer: input.rangeSummary.disclaimer?.trim()
      ? input.rangeSummary.disclaimer
      : DEFAULT_DISCLAIMER,
    computed_at: input.rangeSummary.computed_at,
    fx_snapshot: input.rangeSummary.fx_snapshot,
    load_state: input.load_state ?? "idle",
    error: input.error,
    spending_align,
    official_admin_reconcile,
  };
}

/**
 * Mock IPC surface — same command names as shell-host-contract-v0.
 * Spend totals come from real ledger CLI JSON (refresh-from-ledger.sh).
 * Real host will swap these for Tauri `invoke`.
 *
 * Range filtering awaits CLI `--range`; until then every range.kind returns
 * the same real by-pool SpendSummary with only `range.kind` retagged.
 * No invented / scaled prices.
 */
import type { Currency, DailySpendSeries, ImportMeta, PanelRangeKind, RangeKind, SpendSummary } from "./types";

import ledgerByModel from "./mock/ledger-by-model.json";
import ledgerByPool from "./mock/ledger-by-pool.json";
import spendToday from "./mock/spend-total-today.json";
import spendAll from "./mock/spend-total-all.json";
import spend7d from "./mock/spend-total-7d.json";
import spend30d from "./mock/spend-total-30d.json";
import spend90d from "./mock/spend-total-90d.json";
import seriesAll from "./mock/spend-series-all.json";
import series7d from "./mock/spend-series-7d.json";
import series30d from "./mock/spend-series-30d.json";
import series90d from "./mock/spend-series-90d.json";
import importStatusFixture from "./mock/import-status.json";

/** Real ledger by-pool summary (cursor-pools fixture) retagged per range.kind. */
const TOTALS: Record<RangeKind, SpendSummary> = {
  today: spendToday as SpendSummary,
  all: spendAll as SpendSummary,
  "7d": spend7d as SpendSummary,
  "30d": spend30d as SpendSummary,
  "90d": spend90d as SpendSummary,
};

const SERIES: Record<PanelRangeKind, DailySpendSeries> = {
  all: seriesAll as DailySpendSeries,
  "7d": series7d as DailySpendSeries,
  "30d": series30d as DailySpendSeries,
  "90d": series90d as DailySpendSeries,
};

function delay(ms = 40): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

function cloneSummary(base: SpendSummary, currency: Currency): SpendSummary {
  return {
    ...base,
    currency,
    by_agent: base.by_agent.map((a) => ({ ...a })),
    by_model: (base.by_model ?? []).map((m) => ({ ...m })),
    by_usage_pool: (base.by_usage_pool ?? []).map((p) => ({ ...p })),
    range: base.range ? { ...base.range } : undefined,
  };
}

/** Aligns CLI: `spend total --range … --currency …` (fixtures from ledger CLI). */
export async function spend_total(
  range: RangeKind,
  currency: Currency = "USD",
): Promise<SpendSummary> {
  await delay();
  const base = TOTALS[range];
  if (!base) throw new Error(`unknown range: ${range}`);
  return cloneSummary(base, currency);
}

/**
 * Real ledger `spend by-model` JSON (fixtures/ac-v1.3/by-model.json).
 * Multi-agent sample; may lack dual Cursor pools.
 */
export async function spend_by_model(currency: Currency = "USD"): Promise<SpendSummary> {
  await delay();
  return cloneSummary(ledgerByModel as SpendSummary, currency);
}

/**
 * Real ledger `spend by-pool` JSON (fixtures/ac-v1.3a/cursor-pools.json).
 * Primary all-summary / dual-pool fixture used by spend_total ranges.
 */
export async function spend_by_pool(currency: Currency = "USD"): Promise<SpendSummary> {
  await delay();
  return cloneSummary(ledgerByPool as SpendSummary, currency);
}

/** Aligns CLI: `spend series --grain day --range …` (still mock series until CLI series lands). */
export async function spend_series(
  grain: "day",
  range: PanelRangeKind,
  currency: Currency = "USD",
): Promise<DailySpendSeries> {
  await delay();
  if (grain !== "day") throw new Error("v0 grain is day only");
  const base = SERIES[range];
  if (!base) throw new Error(`unknown series range: ${range}`);
  return {
    ...base,
    currency,
    points: base.points.map((p) => ({ ...p })),
  };
}

/** Aligns CLI: `import status --json` */
export async function import_status(): Promise<ImportMeta> {
  await delay();
  return { ...(importStatusFixture as ImportMeta) };
}

export const mockApi = {
  spend_total,
  spend_by_model,
  spend_by_pool,
  spend_series,
  import_status,
};

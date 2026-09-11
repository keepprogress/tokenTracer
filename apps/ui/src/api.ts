/**
 * Shell-host IPC surface (SHELL-HOST v0).
 *
 * Primary path (dev / vite preview): fetch `/api/ipc/*` which shells out to the
 * real `spend` CLI via `scripts/spend-dev-bridge.mjs` — no invented prices.
 * Fallback: bundled `src/mock/*.json` from `refresh-from-ledger.sh` when CLI /
 * bridge is unavailable (static build, Tauri-not-ready, FORCE_FIXTURES).
 *
 * Command names match shell-host-contract-v0; Tauri host will swap to `invoke`.
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

export type DataSource = "cli" | "fixture";

let lastSource: DataSource = "fixture";

/** Last successful IPC source (cli = live spend bridge, fixture = static mock). */
export function dataSource(): DataSource {
  return lastSource;
}

/** Per-range SpendSummary from real `spend by-pool --range <kind>` (fixture fallback). */
const TOTALS: Record<RangeKind, SpendSummary> = {
  today: spendToday as SpendSummary,
  all: spendAll as SpendSummary,
  "7d": spend7d as SpendSummary,
  "30d": spend30d as SpendSummary,
  "90d": spend90d as SpendSummary,
};

/** Per-range series from real `spend series --grain day --range <kind>`. */
const SERIES: Record<PanelRangeKind, DailySpendSeries> = {
  all: seriesAll as DailySpendSeries,
  "7d": series7d as DailySpendSeries,
  "30d": series30d as DailySpendSeries,
  "90d": series90d as DailySpendSeries,
};

const IPC_BASE = "/api/ipc";

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

async function tryLiveJson<T>(
  command: string,
  query: Record<string, string>,
): Promise<T | null> {
  // File:// / Tauri without host: relative /api/ipc is meaningless.
  if (typeof window !== "undefined" && window.location?.protocol === "file:") {
    return null;
  }
  try {
    const qs = new URLSearchParams(query).toString();
    const url = qs ? `${IPC_BASE}/${command}?${qs}` : `${IPC_BASE}/${command}`;
    const res = await fetch(url, { headers: { Accept: "application/json" } });
    if (!res.ok) return null;
    const data = (await res.json()) as T;
    lastSource = "cli";
    return data;
  } catch {
    return null;
  }
}

/** Aligns CLI: `spend by-pool --range … --currency …` */
export async function spend_total(
  range: RangeKind,
  currency: Currency = "USD",
): Promise<SpendSummary> {
  const live = await tryLiveJson<SpendSummary>("spend_total", { range, currency });
  if (live) return live;
  lastSource = "fixture";
  const base = TOTALS[range];
  if (!base) throw new Error(`unknown range: ${range}`);
  return cloneSummary(base, currency);
}

/**
 * Real ledger `spend by-model` JSON.
 * Multi-agent sample; may lack dual Cursor pools.
 */
export async function spend_by_model(
  currency: Currency = "USD",
  range: RangeKind = "all",
): Promise<SpendSummary> {
  const live = await tryLiveJson<SpendSummary>("spend_by_model", { currency, range });
  if (live) return live;
  lastSource = "fixture";
  return cloneSummary(ledgerByModel as SpendSummary, currency);
}

/**
 * Real ledger `spend by-pool` JSON (dual-pool / ranged).
 */
export async function spend_by_pool(
  currency: Currency = "USD",
  range: RangeKind = "all",
): Promise<SpendSummary> {
  const live = await tryLiveJson<SpendSummary>("spend_by_pool", { currency, range });
  if (live) return live;
  lastSource = "fixture";
  if (range === "all") return cloneSummary(ledgerByPool as SpendSummary, currency);
  const base = TOTALS[range];
  if (!base) throw new Error(`unknown range: ${range}`);
  return cloneSummary(base, currency);
}

/** Aligns CLI: `spend series --grain day --range …` */
export async function spend_series(
  grain: "day",
  range: PanelRangeKind,
  currency: Currency = "USD",
): Promise<DailySpendSeries> {
  if (grain !== "day") throw new Error("v0 grain is day only");
  const live = await tryLiveJson<DailySpendSeries>("spend_series", {
    grain,
    range,
    currency,
  });
  if (live) return live;
  lastSource = "fixture";
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
  const live = await tryLiveJson<ImportMeta>("import_status", {});
  if (live) return live;
  lastSource = "fixture";
  return { ...(importStatusFixture as ImportMeta) };
}

export const mockApi = {
  spend_total,
  spend_by_model,
  spend_by_pool,
  spend_series,
  import_status,
  dataSource,
};

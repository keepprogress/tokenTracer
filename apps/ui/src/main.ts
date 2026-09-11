import "./style.css";
import { spend_total, spend_series, import_status } from "./api";
import { mapToMiniPanelVM } from "./bind/mapToMiniPanelVM";
import type { MiniPanelVM, PanelRangeKind, SpendSummary } from "./types";
import { DEFAULT_DISCLAIMER } from "./types";

type PanelMode = "collapsed" | "expanded";

const RANGES: PanelRangeKind[] = ["all", "7d", "30d", "90d"];
const TZ = "Asia/Taipei";

let mode: PanelMode = "collapsed";
let range: PanelRangeKind = "all";
let vm: MiniPanelVM | null = null;
let loading = false;

const app = document.querySelector<HTMLDivElement>("#app")!;

function fmtUsd(n: number | null | undefined): string {
  if (n == null || Number.isNaN(n)) return "—";
  return `$${n.toFixed(2)}`;
}

function fmtPct(n: number | null | undefined): string {
  if (n == null || Number.isNaN(n)) return "";
  return `${n.toFixed(1)}% used`;
}

/** Display last_imported_at in Asia/Taipei — never use computed_at here. */
function fmtTaipei(iso: string | null): string {
  if (!iso) return "尚未匯入";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "尚未匯入";
  const parts = new Intl.DateTimeFormat("zh-TW", {
    timeZone: TZ,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).formatToParts(d);
  const get = (t: string) => parts.find((p) => p.type === t)?.value ?? "";
  return `${get("year")}-${get("month")}-${get("day")} ${get("hour")}:${get("minute")} (Asia/Taipei)`;
}

function emptySummary(): SpendSummary {
  return {
    currency: "USD",
    total: 0,
    pricing_mode: "notional_api_estimate",
    price_table_version: "—",
    price_table_source: "",
    by_agent: [],
    by_model: [],
    by_usage_pool: [],
    computed_at: new Date().toISOString(),
    disclaimer: DEFAULT_DISCLAIMER,
  };
}

async function loadAll(nextRange: PanelRangeKind = range): Promise<void> {
  loading = true;
  range = nextRange;
  render();
  try {
    const [today, rangeSummary, series, meta] = await Promise.all([
      spend_total("today", "USD"),
      spend_total(nextRange, "USD"),
      spend_series("day", nextRange, "USD"),
      import_status(),
    ]);
    vm = mapToMiniPanelVM({
      today,
      rangeSummary,
      series,
      importMeta: meta,
      panelRange: nextRange,
      load_state: "idle",
    });
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    vm = mapToMiniPanelVM({
      today: null,
      rangeSummary: emptySummary(),
      series: null,
      importMeta: null,
      panelRange: nextRange,
      load_state: "error",
      error: { code: "mock_load", message },
    });
  } finally {
    loading = false;
    render();
  }
}

function renderCollapsed(): string {
  const today = vm?.today_total ?? null;
  const total = vm?.range_total ?? null;
  const partialDot =
    (vm?.partial_warnings.length ?? 0) > 0
      ? `<span title="PARTIAL" style="color:var(--warn)">⚠</span>`
      : "";
  const notion =
    vm?.pricing_mode === "notional_api_estimate"
      ? `<span class="notional-chip" title="${escapeHtml(vm.disclaimer)}">notional</span>`
      : "";
  return `
    <div class="panel collapsed" role="button" tabindex="0" data-action="expand" aria-label="Expand tokenTracer panel">
      <div class="collapsed-row">
        <div class="primary-nums">
          <span><span class="label">Today</span><span class="value">${fmtUsd(today)}</span></span>
          <span><span class="label">Total</span><span class="value">${fmtUsd(total)}</span> USD</span>
          ${notion}
          ${partialDot}
        </div>
        <span class="chevron">▸ details</span>
      </div>
    </div>
  `;
}

function renderTrend(v: MiniPanelVM): string {
  const pts = v.trend ?? [];
  if (!pts.length) {
    return `<p class="section-title">Daily trend</p><div class="trend"><span style="color:var(--muted)">—</span></div>`;
  }
  const max = Math.max(...pts.map((p) => p.amount), 0);
  const bars = pts
    .map((p) => {
      const zero = p.amount === 0;
      const h = zero ? 2 : max > 0 ? Math.max(4, Math.round((p.amount / max) * 48)) : 2;
      return `<div class="trend-bar${zero ? " zero" : ""}" style="height:${h}px" title="${p.date}: ${fmtUsd(p.amount)}"></div>`;
    })
    .join("");
  return `
    <p class="section-title">Daily trend (${v.range.kind} · includes $0 days)</p>
    <div class="trend" aria-label="Daily spend bars">${bars}</div>
  `;
}

function renderAgents(v: MiniPanelVM): string {
  if (!v.by_agent.length) {
    return `<p class="section-title">By agent</p><p style="color:var(--muted)">尚無匯入；點 ↻ 或見 README</p>`;
  }
  const rows = v.by_agent
    .map((a) => {
      const pct = a.share == null ? "—" : `${Math.round(a.share * 100)}%`;
      const w = a.share == null ? 0 : Math.round(a.share * 100);
      return `
        <li class="agent-row">
          <span class="name" title="${a.agent}">${a.display_name}</span>
          <span class="amt">${fmtUsd(a.amount)}</span>
          <div class="bar-track"><div class="bar-fill" style="width:${w}%"></div></div>
          <span class="pct">${pct}</span>
          <span class="status-pill ${a.status}">${a.status}</span>
        </li>`;
    })
    .join("");
  return `<p class="section-title">By agent</p><ul class="agent-list">${rows}</ul>`;
}

function renderModelRow(m: {
  model: string;
  display_name: string;
  amount: number | null;
  share: number | null;
  priced: boolean;
}): string {
  const pct = m.share == null ? "—" : `${Math.round(m.share * 100)}%`;
  const w = m.share == null ? 0 : Math.round(m.share * 100);
  const unpriced = m.priced ? "" : `<span class="status-pill partial">unpriced</span>`;
  return `
    <li class="model-row">
      <span class="name" title="${escapeHtml(m.model)}">${escapeHtml(m.display_name)}</span>
      <span class="amt">${fmtUsd(m.amount)}</span>
      <div class="bar-track"><div class="bar-fill" style="width:${w}%"></div></div>
      <span class="pct">${pct}</span>
      ${unpriced}
    </li>`;
}

function renderPools(v: MiniPanelVM): string {
  const pools = v.by_usage_pool;
  if (pools && pools.length) {
    const blocks = pools
      .map((p) => {
        const poolBadge =
          p.pool === "other_models"
            ? `<span class="pool-badge other" title="usage pool — not a model id">◇ pool</span>`
            : p.pool === "unknown"
              ? `<span class="pool-badge unknown">unknown</span>`
              : `<span class="pool-badge cursor">pool</span>`;
        const pct = p.percent_used != null ? ` · ${fmtPct(p.percent_used)}` : "";
        const models =
          p.models.length > 0
            ? `<ul class="model-list">${p.models.map(renderModelRow).join("")}</ul>`
            : `<p class="pool-empty">No models in this pool (example data)</p>`;
        return `
          <div class="pool-group" data-pool="${p.pool}">
            <div class="pool-head">
              <span class="pool-name">${escapeHtml(p.display_name)}</span>
              ${poolBadge}
              <span class="pool-amt">${fmtUsd(p.amount)}${pct}</span>
            </div>
            ${models}
          </div>`;
      })
      .join("");
    return `
      <p class="section-title">By model <span class="hint">(Cursor usage pools)</span></p>
      <div class="pool-stack">${blocks}</div>`;
  }

  // Non-Cursor / no pool: flat by_model
  if (v.by_model && v.by_model.length) {
    return `
      <p class="section-title">By model</p>
      <ul class="model-list">${v.by_model.map(renderModelRow).join("")}</ul>`;
  }

  return `
    <p class="section-title">By model</p>
    <p class="pool-placeholder">Cursor usage pools (Other Models vs Cursor Models) — waiting for ledger data</p>`;
}

function renderExpanded(): string {
  const v = vm;
  const chips = RANGES.map(
    (r) =>
      `<button type="button" class="chip${r === range ? " active" : ""}" data-range="${r}">${r === "all" ? "All" : r}</button>`,
  ).join("");

  const warns =
    v?.partial_warnings
      .map((w) => `<div class="warn-banner" role="status">⚠ PARTIAL: ${escapeHtml(w.text)}</div>`)
      .join("") ?? "";

  const err =
    v?.load_state === "error" && v.error
      ? `<div class="error-banner">${escapeHtml(v.error.message)}</div>`
      : "";

  const disclaimer = escapeHtml(v?.disclaimer ?? DEFAULT_DISCLAIMER);
  const pricing = escapeHtml(v?.pricing_mode ?? "notional_api_estimate");

  return `
    <div class="panel expanded">
      <div class="header">
        <h1>tokenTracer</h1>
        <div class="header-actions">
          <button type="button" class="icon-btn" data-action="refresh" title="Refresh" ${loading ? "disabled" : ""}>↻</button>
          <button type="button" class="icon-btn" data-action="collapse" title="Collapse">–</button>
        </div>
      </div>
      <div class="body">
        ${err}
        <div class="range-chips" role="tablist" aria-label="Range">${chips}</div>
        <div class="disclaimer" role="note">${disclaimer}</div>
        <div class="summary-grid">
          <span class="k">Today</span>
          <span class="v${loading ? " loading-dot" : ""}">${fmtUsd(v?.today_total)} USD</span>
          <span class="k">Total (${range})</span>
          <span class="v">${fmtUsd(v?.range_total)} USD</span>
          <span class="k">Pricing</span>
          <span class="v notional-label">${pricing}</span>
        </div>
        ${v ? renderAgents(v) : ""}
        ${v ? renderPools(v) : ""}
        ${v ? renderTrend(v) : ""}
        <div class="meta">
          <div><strong style="color:var(--text)">Last import</strong> · ${fmtTaipei(v?.last_imported_at ?? null)}</div>
          ${warns}
          <div>price_table ${escapeHtml(v?.price_table_version ?? "—")} · computed_at ${escapeHtml(v?.computed_at ?? "—")} (not import time)</div>
          <div class="footer-note">Ledger CLI fixtures (notional API estimate). UI does not price.</div>
        </div>
      </div>
    </div>
  `;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function render(): void {
  app.innerHTML = mode === "collapsed" ? renderCollapsed() : renderExpanded();
}

app.addEventListener("click", (ev) => {
  const t = ev.target as HTMLElement;
  const actionEl = t.closest<HTMLElement>("[data-action]");
  const rangeEl = t.closest<HTMLElement>("[data-range]");

  if (rangeEl?.dataset.range) {
    const next = rangeEl.dataset.range as PanelRangeKind;
    if (RANGES.includes(next)) void loadAll(next);
    return;
  }

  const action = actionEl?.dataset.action ?? (t.closest(".collapsed") ? "expand" : null);
  if (action === "expand") {
    mode = "expanded";
    render();
  } else if (action === "collapse") {
    mode = "collapsed";
    render();
  } else if (action === "refresh") {
    void loadAll(range);
  }
});

app.addEventListener("keydown", (ev) => {
  if (ev.key === "Escape" && mode === "expanded") {
    mode = "collapsed";
    render();
  }
  if ((ev.key === "Enter" || ev.key === " ") && (ev.target as HTMLElement).closest(".collapsed")) {
    ev.preventDefault();
    mode = "expanded";
    render();
  }
});

void loadAll("all");

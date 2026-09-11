import "./style.css";
import {
  spend_total,
  spend_by_pool,
  spend_series,
  import_status,
  spending_align,
  official_admin_reconcile,
  dataSource,
  isTauriHost,
} from "./api";
import { mapToMiniPanelVM } from "./bind/mapToMiniPanelVM";
import type { MiniPanelVM, PanelRangeKind, SpendSummary } from "./types";
import { DEFAULT_DISCLAIMER, SPENDING_COMPARE_DISCLAIMER } from "./types";

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

function fmtAlignPct(n: number | null | undefined): string {
  if (n == null || Number.isNaN(n)) return "—";
  return `${Math.round(n)}%`;
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
    // A (notional): today via spend_total; Expanded range via spend_by_pool so
    // by_usage_pool / by_model land for AC-F7.2. B (spending_align) is a separate
    // payload — never derived from notional totals.
    const [today, rangeSummary, series, meta, align, reconcile] =
      await Promise.all([
        spend_total("today", "USD"),
        spend_by_pool("USD", nextRange),
        spend_series("day", nextRange, "USD"),
        import_status(),
        spending_align(),
        official_admin_reconcile(),
      ]);
    vm = mapToMiniPanelVM({
      today,
      rangeSummary,
      series,
      importMeta: meta,
      panelRange: nextRange,
      spending_align: align,
      official_admin_reconcile: reconcile,
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
      spending_align: null,
      official_admin_reconcile: null,
      load_state: "error",
      error: { code: "ipc_load", message },
    });
  } finally {
    loading = false;
    render();
  }
}

function renderCollapsed(): string {
  const today = vm?.today_total ?? null;
  const total = vm?.range_total ?? null;
  const agentPartial =
    (vm?.partial_warnings.length ?? 0) > 0
      ? `<span title="PARTIAL" style="color:var(--warn)">⚠</span>`
      : "";
  const alignPartial =
    vm?.spending_align?.partial === true
      ? `<span class="partial-chip" title="${escapeHtml(vm.spending_align.partial_reason ?? "PARTIAL")}">PARTIAL</span>`
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
          ${agentPartial}
          ${alignPartial}
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
        const models =
          p.models.length > 0
            ? `<ul class="model-list">${p.models.map(renderModelRow).join("")}</ul>`
            : `<p class="pool-empty">No models in this pool (example data)</p>`;
        return `
          <div class="pool-group" data-pool="${p.pool}">
            <div class="pool-head">
              <span class="pool-name">${escapeHtml(p.display_name)}</span>
              ${poolBadge}
              <span class="pool-amt">${fmtUsd(p.amount)}</span>
            </div>
            ${models}
          </div>`;
      })
      .join("");
    return `
      <p class="section-title">By model <span class="hint">(Cursor usage_pool $ — notional)</span></p>
      <div class="pool-stack">${blocks}</div>`;
  }

  if (v.by_model && v.by_model.length) {
    return `
      <p class="section-title">By model</p>
      <ul class="model-list">${v.by_model.map(renderModelRow).join("")}</ul>`;
  }

  return `
    <p class="section-title">By model</p>
    <p class="pool-placeholder">Cursor usage pools (Other Models vs Cursor Models) — waiting for ledger data</p>`;
}

function renderSpendingAlign(v: MiniPanelVM): string {
  const sa = v.spending_align;
  if (!sa || sa.source_mode === "none") {
    return `
      <section class="surface-b" aria-label="B. Spending align">
        <div class="surface-label">B. Spending align (Cursor) — 對照層／非本機加總</div>
        <p class="align-empty">尚未對齊 Spending；可手動輸入／截圖核對（P1）。source: none</p>
        <p class="align-disclaimer">${escapeHtml(SPENDING_COMPARE_DISCLAIMER)}</p>
      </section>`;
  }

  const example =
    sa.is_example
      ? `<div class="example-banner" role="note">EXAMPLE DATA · ${escapeHtml(sa.example_label ?? "fixture")}</div>`
      : "";

  const undoc =
    sa.source_mode === "undocumented_opt_in"
      ? `<div class="undoc-banner" role="status">UNSUPPORTED／undocumented／易碎 — 非 official_admin（F16）</div>`
      : "";

  const partialBanner =
    sa.partial
      ? `<div class="warn-banner align-partial" role="status">⚠ ${escapeHtml(sa.partial_reason ?? "PARTIAL")}</div>`
      : "";

  const grok =
    sa.grok_bot_week_note?.trim()
      ? `<div class="grok-note">${escapeHtml(sa.grok_bot_week_note)}</div>`
      : "";

  return `
    <section class="surface-b" aria-label="B. Spending align">
      <div class="surface-label">B. Spending align (Cursor) — 對照層／非本機加總</div>
      ${example}
      ${undoc}
      ${partialBanner}
      <div class="align-meta">source: <code>${escapeHtml(sa.source_mode)}</code></div>
      <div class="align-grid">
        <span class="k">Cursor Models</span>
        <span class="v align-pct">${fmtAlignPct(sa.cursor_models_pct)} <span class="hint">(align target)</span></span>
        <span class="k">Other Models</span>
        <span class="v align-pct">${fmtAlignPct(sa.other_models_pct)} <span class="hint">(align target)</span></span>
        <span class="k">Reset</span>
        <span class="v">${escapeHtml(sa.reset_label ?? "—")}</span>
        <span class="k">On-demand</span>
        <span class="v">${escapeHtml(sa.on_demand ?? "—")}</span>
      </div>
      <p class="align-disclaimer">${escapeHtml(SPENDING_COMPARE_DISCLAIMER)}</p>
      ${grok}
    </section>`;
}

function renderOfficialReconcile(v: MiniPanelVM): string {
  const r = v.official_admin_reconcile;
  if (!r) return "";
  const tol = r.within_tol ? "within tol" : "OUT OF TOL";
  return `
    <section class="surface-f14" aria-label="F14 Official Admin reconcile">
      <div class="surface-label">F14 Official Admin reconcile (stub) — ≠ B pool %</div>
      <div class="align-grid">
        <span class="k">Σ chargedCents</span>
        <span class="v">${r.events_charged_cents_sum}</span>
        <span class="k">spend overall</span>
        <span class="v">${r.spend_overall_cents}</span>
        <span class="k">delta</span>
        <span class="v">${r.delta_cents} · ${tol}</span>
      </div>
      <p class="align-disclaimer">Cents must not drive SpendingAlign pool % (OPEN-BIND-S1).</p>
    </section>`;
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

  const f15 =
    `<div class="f15-note" role="note">A 區僅 local notional — 不得單獨當 Cursor 帳單／USD 權威；請對照 B 區 Spending 或 Team Admin（F15）</div>`;

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

        <section class="surface-a" aria-label="A. Local notional">
          <div class="surface-label">A. Local notional</div>
          ${f15}
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
        </section>

        ${v ? renderSpendingAlign(v) : ""}
        ${v ? renderOfficialReconcile(v) : ""}

        <div class="meta">
          <div><strong style="color:var(--text)">Last import</strong> · ${fmtTaipei(v?.last_imported_at ?? null)}</div>
          ${warns}
          <div>price_table ${escapeHtml(v?.price_table_version ?? "—")} · computed_at ${escapeHtml(v?.computed_at ?? "—")} (not import time)</div>
          <div class="footer-note">${
            dataSource() === "tauri"
              ? "Tauri host invoke → spend CLI (notional API estimate). UI does not price. A≠B payloads."
              : dataSource() === "cli"
                ? "Live spend CLI via /api/ipc (notional API estimate). UI does not price. B surface = separate spending_align payload. · debug only"
                : "Fixtures fallback (A notional + B spending-align EXAMPLE). UI does not price."
          }</div>
        </div>
      </div>
    </div>
  `;
}


type TauriListen = (
  event: string,
  handler: (event: { payload: unknown }) => void,
) => Promise<() => void>;

function getTauriListen(): TauriListen | null {
  if (typeof window === "undefined") return null;
  const g = window as unknown as {
    __TAURI__?: { event?: { listen?: TauriListen } };
  };
  return g.__TAURI__?.event?.listen?.bind(g.__TAURI__.event) ?? null;
}

function getTauriInvoke():
  | ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>)
  | null {
  if (typeof window === "undefined") return null;
  const g = window as unknown as {
    __TAURI__?: { core?: { invoke?: (c: string, a?: Record<string, unknown>) => Promise<unknown> } };
  };
  return g.__TAURI__?.core?.invoke?.bind(g.__TAURI__.core) ?? null;
}

/** Drive Collapsed/Expanded and notify host to resize window when embedded. */
async function setMode(next: PanelMode, notifyHost = true): Promise<void> {
  mode = next;
  render();
  if (!notifyHost || !isTauriHost()) return;
  const invoke = getTauriInvoke();
  if (!invoke) return;
  try {
    await invoke("set_panel_mode", { mode: next });
  } catch {
    /* host may already own the resize */
  }
}

async function wireHostEvents(): Promise<void> {
  if (!isTauriHost()) return;
  document.documentElement.classList.add("host-tauri");
  document.documentElement.dataset.host = "tauri";

  const invoke = getTauriInvoke();
  if (invoke) {
    try {
      const m = String(await invoke("get_panel_mode"));
      if (m === "collapsed" || m === "expanded") {
        mode = m;
        render();
      }
    } catch {
      /* ignore — host may not be ready */
    }
  }

  const listen = getTauriListen();
  if (!listen) return;
  await listen("panel-set-mode", (e) => {
    const raw = String(e.payload ?? "");
    if (raw === "collapsed" || raw === "expanded") {
      void setMode(raw, false);
    } else if (raw === "hidden") {
      /* window hidden by host; keep last UI mode */
    }
  });
  await listen("panel-refresh", () => {
    void loadAll(range);
  });
  await listen("panel-import-stub", () => {
    const inv = getTauriInvoke();
    if (!inv) {
      console.info("[tokenTracer] Import… stub (no invoke)");
      return;
    }
    void inv("import_run")
      .then(() => loadAll(range))
      .catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : String(err);
        console.info("[tokenTracer] import_run stub:", msg);
      });
  });
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
    void setMode("expanded");
  } else if (action === "collapse") {
    void setMode("collapsed");
  } else if (action === "refresh") {
    void loadAll(range);
  }
});

app.addEventListener("keydown", (ev) => {
  if (ev.key === "Escape" && mode === "expanded") {
    void setMode("collapsed");
  }
  if ((ev.key === "Enter" || ev.key === " ") && (ev.target as HTMLElement).closest(".collapsed")) {
    ev.preventDefault();
    void setMode("expanded");
  }
});

void wireHostEvents();
void loadAll("all");

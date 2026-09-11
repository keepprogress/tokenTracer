# tokenTracer mini-panel UI

Vite + TypeScript vanilla shell for the tokenTracer **mini-panel**（儀表）.  
**UI does not price** — it consumes `SpendSummary` / series / `ImportMeta` JSON.

## Data priority（SHELL-HOST v0.1）

1. **Tauri host** (`apps/tokenTracer-host`): `window.__TAURI__.core.invoke(...)` when embedded  
2. **Dev / `vite preview`**: live `spend` CLI via `/api/ipc/*` (`scripts/spend-dev-bridge.mjs`) — **debug only**（N5；非主驗收面）  
3. **Fallback**: bundled `src/mock/*.json` from `refresh-from-ledger.sh`

Contracts: UI-IA v0.4-draft, UI-BIND v0.4-draft (AC v1.4 F12/F15/F17–F19), SHELL-HOST v0.1.  
Design refs: `design/ui/mini-panel-ia-v0.4-draft.md`, `design/ui/spend-summary-binding-v0.4-draft.md`,
`design/ledger/open-bind-spending-align-v0.md`.

## AC-F7 / F7′ UI surface

| 態 | 內容 |
|----|------|
| **Collapsed** | Today USD + Total USD（至少一項主數字）+ notional chip + PARTIAL 點 |
| **Expanded** | **A** notional：Range chips、today／total、by_agent、by_model／usage pools、daily trend；**B** Spending align：池％／reset／on-demand（獨立 payload）；last import（Asia/Taipei）、PARTIAL、notional disclaimer |
| **Tray-driven** | 聽 `panel-set-mode`／`panel-refresh`／`panel-import-stub`；UI `setMode` → `set_panel_mode` 讓宿主改窗尺寸 |

數字來源：`spend_total`(today) + `spend_by_pool`(range) + `spend_series` + `import_status` + `spending_align`（B）＋ optional `official_admin_reconcile` — 與 CLI／宿主 IPC 同名，不發明計價；A≠B。

## Windows 托盤行為（宿主寫死；UI 對齊）

詳見 [`../tokenTracer-host/README.md`](../tokenTracer-host/README.md)：

| 操作 | 行為 |
|------|------|
| **左鍵** | Collapsed ↔ Expanded（UI 收 `panel-set-mode`） |
| **右鍵** | Show panel／Refresh（→ `panel-refresh`）／Import…（stub）／Quit |
| 啟動 | 托盤常駐 + Collapsed；面板錨右下 |

macOS 選單列：**UNTESTED / later**。

## Preview / build

```bash
cd /workspace/tokenTracer-main
cargo build -p pricing --bin spend
cd apps/ui
npm install
npm run dev          # http://127.0.0.1:5173/  — debug only
npm run build && npm run preview
```

Force fixtures: `TOKENTRACER_FORCE_FIXTURES=1 npm run dev`

Host embed（Windows）:

```powershell
npm --prefix apps/ui run build
cd apps\tokenTracer-host
cargo tauri dev
```

## IPC surface (`src/api.ts`)

| Command | Params | Notes |
|---------|--------|-------|
| `spend_total` | `range`, `currency` | today + fallback totals |
| `spend_by_pool` | `currency`, `range` | Expanded range（pools + models） |
| `spend_series` | `grain`, `range`, `currency` | daily trend |
| `spend_by_model` | `currency`, `range?` | available; Expanded uses pool path |
| `import_status` | — | last import |

| `spending_align` | — | B-surface OPEN-BIND；CLI if present else EXAMPLE fixture |
| `official_admin_reconcile` | — | optional F14 stub（≠ B pool %） |

`dataSource()` → `"tauri"` | `"cli"` | `"fixture"`。Footer 反映目前路徑（A≠B payloads）。

## Env（bridge / host）

同 `spend-dev-bridge.mjs`／宿主：`TOKENTRACER_LEDGER_ROOT`、`TOKENTRACER_SPEND_BIN`、`TOKENTRACER_EVENTS_*`、`TOKENTRACER_IMPORT_STATE`、`TOKENTRACER_FORCE_FIXTURES`。

Mapping: `src/bind/mapToMiniPanelVM.ts` (no sentinel `"Other"` model rows — usage_pool groups only).

## Refresh static fixtures from ledger CLI

Still useful for offline / Tauri-embed fallback:

```bash
cd /workspace/tokenTracer-main/apps/ui && ./scripts/refresh-from-ledger.sh
# or: npm run refresh-fixtures
```

Writes:

| File | Source |
|------|--------|
| `src/mock/ledger-by-model.json` | by-model (multi-agent sample) |
| `src/mock/ledger-by-pool.json` | by-pool on `cursor-pools.json` (dual pools) |
| `src/mock/spend-total-{all,today,7d,30d,90d}.json` | `by-pool --range <kind>` on `cursor-pools-ranged.json` |
| `src/mock/spend-series-{all,7d,30d,90d}.json` | `series --grain day --range <kind>` |

## Env (bridge)

| Var | Purpose |
|-----|---------|
| `LEDGER_ROOT` / `TOKENTRACER_LEDGER_ROOT` | Cargo workspace root (default: repo root) |
| `TOKENTRACER_SPEND_BIN` | Explicit path to `spend` binary |
| `TOKENTRACER_EVENTS_RANGED` | Override ranged events fixture |
| `TOKENTRACER_EVENTS_POOL` | Override dual-pool fixture |
| `TOKENTRACER_EVENTS_MODEL` | Override by-model fixture |
| `TOKENTRACER_IMPORT_STATE` | Import-meta state path |
| `TOKENTRACER_FORCE_FIXTURES=1` | Bridge returns 503 → UI fixtures |
| `TOKENTRACER_SPENDING_ALIGN` | OPEN-BIND SpendingAlign fixture path |
| `TOKENTRACER_SPENDING_ALIGN_STATE` | Optional `.token-tracer/spending-align.json` |
| `TOKENTRACER_ADMIN_EVENTS` / `TOKENTRACER_ADMIN_SPEND` | Optional F14 reconcile stub fixtures |

## Features

- Collapsed ↔ Expanded toggle (click row / – / Esc)
- Today USD + range total USD from live CLI (or fixtures)
- Range chips: All / 7d / 30d / 90d (re-fetches IPC; no local reprice)
- By-agent bars + `ok` / `partial` status
- By-model + Cursor usage pools (`cursor_models` / `other_models`) — no sentinel “Other model” rows
- Daily trend bars from CLI series `points`
- `last_imported_at` shown in **Asia/Taipei** (never `computed_at`)
- Cursor **PARTIAL** warning banner
- Footer / disclaimer from ledger `disclaimer` + `pricing_mode`


## A vs B surfaces (AC v1.4)

| Surface | Meaning | Authority |
|---------|---------|-----------|
| **A. Local notional** | `SpendSummary` $ / by_agent / by_model / by_usage_pool $ / trend | Local ledger `notional_api_estimate` |
| **B. Spending align** | Cursor Models % / Other Models % / reset / on-demand | Spending UI / manual P1 / future opt-in — **not** local token sums |

Hard rules:

- Local notional totals / `by_usage_pool.amount` **MUST NOT** drive or overwrite B `%` (N25 / F17).
- Notional ≠ subscription quota / Spending % (F12).
- No fake **Other** / **Other model** product row (F19).
- Personal path without Admin: PARTIAL + 「無公開個人 usage API」 (F17).
- `undocumented_opt_in` default **off**; if shown, label UNSUPPORTED (F16).
- Optional F14 Admin reconcile is a **separate** stub — cents never become B pool %.

### Ban list (UI)

| Ban | Why |
|-----|-----|
| Drive Spending % from L1 bubble / `tokenCount` / notional $ | F15 / N25 |
| Invent literal Other model row | F19 |
| Merge A $ and B % into one “spend %” bar | G14 / F17 |
| Claim personal public usage API exists | F17 BLOCK |
| Enable undocumented by default / call it `official_admin` | F16 |
| Invent Grok Bot week % endpoint / machine sync numbers | F18 |

B-surface fixture: `src/mock/spending-align-manual-p1.json` (EXAMPLE · `manual_p1` · 65% / 100% · PARTIAL).

Until ledger ships `spend spending-align --json`, the Vite bridge serves that fixture (probe CLI; fallback — **does not fail build**).

## IPC additions (v1.4)

| Command | Behavior |
|---------|----------|
| `spending_align` | Tauri invoke / `/api/ipc/spending_align` → CLI if present, else OPEN-BIND fixture |
| `official_admin_reconcile` | Optional F14 stub when `TOKENTRACER_ADMIN_EVENTS` + `TOKENTRACER_ADMIN_SPEND` set; else 404 (B mock still works) |

## Tray notes (host; not this webview)

- Win left-click: toggle Collapsed ↔ Expanded  
- Win right-click: refresh / import / quit  
- macOS: menu-bar icon opens dropdown panel  

Tauri host (`apps/tokenTracer-host`) already prefers `invoke(...)`; Vite `/api/ipc` remains debug-only.

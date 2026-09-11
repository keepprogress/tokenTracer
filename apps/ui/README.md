# tokenTracer mini-panel UI

Vite + TypeScript vanilla shell for the tokenTracer **mini-panel**（儀表）.  
**UI does not price** — it consumes `SpendSummary` / series / `ImportMeta` JSON.

## Data priority（SHELL-HOST v0.1）

1. **Tauri host** (`apps/tokenTracer-host`): `window.__TAURI__.core.invoke(...)` when embedded  
2. **Dev / `vite preview`**: live `spend` CLI via `/api/ipc/*` (`scripts/spend-dev-bridge.mjs`) — **debug only**（N5；非主驗收面）  
3. **Fallback**: bundled `src/mock/*.json` from `refresh-from-ledger.sh`

Contracts: UI-IA v0.3, UI-BIND v0.3, SHELL-HOST v0.1.

## AC-F7 / F7′ UI surface

| 態 | 內容 |
|----|------|
| **Collapsed** | Today USD + Total USD（至少一項主數字）+ notional chip + PARTIAL 點 |
| **Expanded** | Range chips（All／7d／30d／90d）、today／total、by_agent、by_model／usage pools、daily trend、last import（Asia/Taipei）、PARTIAL warn、notional disclaimer |
| **Tray-driven** | 聽 `panel-set-mode`／`panel-refresh`／`panel-import-stub`；UI `setMode` → `set_panel_mode` 讓宿主改窗尺寸 |

數字來源：`spend_total`(today) + `spend_by_pool`(range) + `spend_series` + `import_status` — 與 CLI／宿主 IPC 同名，不發明計價。

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

`dataSource()` → `"tauri"` \| `"cli"` \| `"fixture"`。Footer 反映目前路徑。

## Env（bridge / host）

同 `spend-dev-bridge.mjs`：`TOKENTRACER_LEDGER_ROOT`、`TOKENTRACER_SPEND_BIN`、`TOKENTRACER_EVENTS_*`、`TOKENTRACER_IMPORT_STATE`、`TOKENTRACER_FORCE_FIXTURES`。

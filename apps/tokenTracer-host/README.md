# tokentracer-host（Tauri 2）

Windows **系統托盤 + 右下角迷你面板**宿主（AC-F7 / F7′ · SHELL-HOST v0.1）。

| 欄位 | 值 |
|------|-----|
| 路徑 | `apps/tokenTracer-host` |
| Rust package | `tokentracer-host` |
| 前端 | `apps/ui`（release 嵌 `dist`；`tauri dev` 用 Vite `127.0.0.1:5173`） |
| Bridge | dep `tokentracer-bridge`（`apps/bridge`）**in-process lib**；`discover` → `discover_paths`；`import_run` stub |
| 帳本 | 以 spawn `spend` CLI 實作 IPC（與 Vite `/api/ipc` 同源，不發明計價） |

> macOS 選單列：**UNTESTED / later**（本任務不驗）。

---

## Windows 托盤行為（寫死）

| 操作 | 行為 |
|------|------|
| **左鍵** | 切換 **Collapsed ↔ Expanded**（若 Hidden 則展開／顯示面板） |
| **右鍵選單** | **Show panel**／**Refresh**／**Import…**（optional stub）／**Quit** |
| 關閉視窗 (X) | **隱藏到托盤**（常駐），不結束行程；結束請用 Quit |
| 啟動預設 | 托盤常駐 + **Collapsed** 可見於螢幕右下（通知區附近）；可後續加「僅圖示」 |

面板錨點：主螢幕 **work area 右下**（`EDGE_MARGIN=12`）。全頁 localhost web **不是**主 UI；Vite `npm run dev` / `preview` 僅 **debug only**。

---

## IPC（`invoke`）

| Command | 實作 |
|---------|------|
| `spend_total` / `spend_series` / `spend_by_model` / `spend_by_pool` / `import_status` | spawn `spend` CLI |
| `spending_align` | B surface: `spend spending-align --json [--state …]` (UI fixture fallback on Err) |
| `discover` | **in-process** `tokentracer_bridge::discover_paths` |
| `import_run` | stub（帳本 record_import 後續） |
| `get_panel_mode` / `set_panel_mode` / `host_meta` | 殼控制 |

Events → UI：`panel-set-mode`、`panel-refresh`、`panel-import-stub`。

Env（同 `spend-dev-bridge.mjs`）：`TOKENTRACER_LEDGER_ROOT`、`TOKENTRACER_SPEND_BIN`、`TOKENTRACER_EVENTS_*`、`TOKENTRACER_IMPORT_STATE`。

---

## Windows 建置／執行

前置：Rust stable、[WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)、Node 18+。

```powershell
# 在 repo root
cargo build -p pricing --bin spend
npm --prefix apps/ui ci
npm --prefix apps/ui run build

# 安裝 Tauri CLI（一次）
cargo install tauri-cli --version "^2" --locked

# Dev（Vite + tray host）
cargo tauri dev --config apps/tokenTracer-host/tauri.conf.json
# 或在 apps/tokenTracer-host：
cd apps/tokenTracer-host
cargo tauri dev

# Release bundle
cargo tauri build
```

亦可：

```powershell
cargo run -p tokentracer-host
```

（需先 `npm --prefix apps/ui run build`，或設 `devUrl` 開發流程。）

---

## Blockers

- Linux agent box **rustc 1.85** may be too old for current Tauri 2 deps (**rustc ≥ 1.88** preferred on Win build host).
- Full tray verify needs **WebView2** + **tauri-cli ^2** on Windows (NB-T3261).
- Do **not** claim AC-F7′ PASS until Win tray smoke.

## Linux box 限制

本機（Linux CI／agent box）**無法**完整驗 Windows tray／錨點：

- 可嘗試 `cargo check -p tokentracer-host`（可能缺 `webkit2gtk`／tray 系統庫）
- **不要**用假 PASS 截圖代替 Win 托盤驗收
- 錨點／L-R click 實機請橋樑 Win host assist

---

## 與橋樑分工

| 儀表（本 crate + `apps/ui`） | 橋樑 |
|-----------------------------|------|
| UI 狀態機、invoke 消費、tray 選單骨架、webview、上表四條 README | 環境／路徑協助（WebView2、spend.exe）；**不代跑**托盤四條；結果以驗收官為準 |

---

## 驗收官獨立重跑（PR #8 · AC-F7′ 四條）

> **主驗收面＝本 Tauri 宿主**，不是 Vite `npm run dev`／`preview`（那只是 debug）。  
> 實作群／橋樑不代跑、不自 PASS；結果以**驗收官**在 **NB-T3261** 重跑為準。  
> 證據建議目錄：`tokenTracer-evidence/.../pr8-win-tray-20260911/`。

| # | 要驗 | How（Win） | 對照 |
|---|------|------------|------|
| **①** | **真 Tauri 托盤**（禁僅 Vite） | `cargo tauri build` 或 `cargo run -p tokentracer-host` 後，系統托盤出現圖示；進程＝`tokentracer-host`／bundle，**不是**瀏覽器開 `127.0.0.1:5173` | AC-F7.3／F7′ |
| **②** | **收合／展開／開面板** | 左鍵：Collapsed ↔ Expanded；右鍵 **Show panel** 可開；X＝藏托盤不退出；面板在螢幕**右下** | AC-F7.1／F7.2 |
| **③** | **數字 vs `spend` CLI（F6）** | 面板同一 `range`／`currency` 與 `spend by-pool`（或 `total`）／`series` JSON 對容差；footer／`host_meta` 應顯示 CLI 源，非 fixture | AC-F6／F7.4 |
| **④** | **F12 notional** | 可見 `pricing_mode=notional_api_estimate`（或等價）＋ disclaimer 文案；金額為 notional，非帳單 | AC-F12／F13′ |

**驗收前置（NB-T3261）**：rustc ≥ 1.88、WebView2、`tauri-cli ^2`、已建 `spend.exe`（`cargo build -p pricing --bin spend --release`）、可選設 `TOKENTRACER_SPEND_BIN`／fixture 路徑。

**明確非目標**：CI 綠 ≠ F7′ PASS；macOS menu bar（F10）＝HOLD；`import_run` 仍 stub。

# tokentracer-host（Tauri 2）

**Windows 系統托盤 + 右下角迷你面板**（AC-F7 / F7′）與 **macOS 選單列迷你面板**（AC-F7′ / AC-F10 · SHELL-HOST v0.1）。

| 欄位 | 值 |
|------|-----|
| 路徑 | `apps/tokenTracer-host` |
| Rust package | `tokentracer-host` |
| 前端 | `apps/ui`（release 嵌 `dist`；`tauri dev` 用 Vite `127.0.0.1:5173`） |
| Bridge | dep `tokentracer-bridge`（`apps/bridge`）**in-process lib**；`discover` → `discover_paths`（macOS 路徑沿用 **AC v1.2b**）；`import_run` stub |
| 帳本 | 以 spawn `spend` CLI 實作 IPC（與 Vite `/api/ipc` 同源，不發明計價） |

> **平台 chrome：** Tauri 2 `tray-icon` on macOS = **menu bar / NSStatusItem** equivalent（not a separate AppKit status-item crate）. Same Collapsed／Expanded IA as Win; **not** a full-page web primary UI (N5).

---

## Windows 托盤行為（寫死）

| 操作 | 行為 |
|------|------|
| **左鍵** | 切換 **Collapsed ↔ Expanded**（若 Hidden 則展開／顯示面板） |
| **右鍵選單** | **Show panel**／**Refresh**／**Import…**（optional stub）／**Quit** |
| 關閉視窗 (X) | **隱藏到托盤**（常駐），不結束行程；結束請用 Quit |
| 啟動預設 | 托盤常駐 + **Collapsed** 可見於螢幕右下（通知區附近） |

面板錨點：主螢幕 **work area 右下**（`EDGE_MARGIN=12`）。

---

## macOS 選單列行為（寫死 · AC-F10／F7′）

| 操作 | 行為 |
|------|------|
| **點選單列圖示** | 顯示 **Expanded** 迷你面板（對齊 IA）；若已開則 **收合為僅圖示**（Hidden） |
| **失焦／點外面** | 面板關閉，回到選單列圖示（常駐） |
| **右鍵／Ctrl-點選單** | **Show panel**／**Refresh**／**Import…**／**Quit** |
| 關閉視窗 | 隱藏到選單列（常駐），不結束；結束請用 **Quit** |
| 啟動預設 | **僅選單列圖示**（Hidden）；無 Dock 圖示（`ActivationPolicy::Accessory`） |
| Collapsed | 自 Expanded 點收合／Esc 時顯示縮圖條（錨在選單列附近／右上）；再點圖示可回 Expanded 或 Hidden |

資訊架構與 Windows 相同：Collapsed 主數字（Today／Total USD）；Expanded 含 range、by_agent、by_model／usage_pool、trend、last import、PARTIAL／notional、`spending_align`（B 面）。空資料／PARTIAL **必須可診斷**（文案／banner；不得靜默空白 0）。

面板錨點：優先 **status item 下方**（記住上次 tray rect）；否則主螢幕 work area **右上**。

### macOS 安裝（AC-F10）

- 形態：**`.dmg` 拖曳安裝**（`cargo tauri build` → `bundle/dmg/`）
- 最低：**macOS 13 Ventura+**；**Apple Silicon 必過**；Intel = **best-effort**（不擋 PASS）
- **Full Disk Access 非預設必要**（AC v1.2b）：一般讀 `~/Library/Application Support/Cursor`、`~/.cursor`、`~/.claude/projects`、`~/.codex/sessions` 即可。若權限不足 → UI／CLI 須給**可照做步驟**，不得靜默 0 資料。
- 路徑矩陣：**AC v1.2b**（無變更）；CONFIRMED\* = 路徑／格式已支援、**無實機 dump 不得虛報 live-verify**。

### 權限失敗時（可照做）

1. 確認 agent 目錄存在（見 v1.2b 路徑：Cursor `Application Support` **與** `~/.cursor`；Claude `~/.claude/projects`；Codex `~/.codex/sessions`）。  
2. 若 macOS 擋讀：系統設定 → **隱私權與安全性** → 視需要開 **檔案與檔案夾**／疑難時才考慮 **完整磁碟取用**（FDA **非**硬安裝前置）。  
3. Bridge CLI／discover 應出現 **`TT-F10-FDA`**（或 `TT-F10-001`）與可照做 `next_step`，**不得**靜默 0 資料。  
4. 重開 tokenTracer → 點 ↻ Refresh；空態應顯示「尚無匯入…」或 PARTIAL banner，而非空白。  
5. CLI 對照：`spend total|by-pool|series …` 與面板同 range／currency。

> **`.dmg`：** `cargo tauri build` on a **Mac** → `bundle/dmg/*.dmg` → 拖曳至 Applications。Linux agent box **無法**產出真實 `.dmg`。Apple Silicon 必過；Intel best-effort（不擋 PASS）。Menu-bar 行為表由**儀表**維護；橋樑不自 PASS 實機選單列。

---

## IPC（`invoke`）

| Command | 實作 |
|---------|------|
| `spend_total` / `spend_series` / `spend_by_model` / `spend_by_pool` / `import_status` | spawn `spend` CLI |
| `spending_align` | B surface: `spend spending-align --json [--state …]` (UI fixture fallback on Err) |
| `discover` | **in-process** `tokentracer_bridge::discover_paths`（含 macOS 路徑） |
| `import_run` | stub（帳本 record_import 後續） |
| `get_panel_mode` / `set_panel_mode` / `host_meta` | 殼控制；`host_meta.platform` / `platform_note` 標 Win vs macOS |

Events → UI：`panel-set-mode`、`panel-refresh`、`panel-import-stub`。

Env（同 `spend-dev-bridge.mjs`）：`TOKENTRACER_LEDGER_ROOT`、`TOKENTRACER_SPEND_BIN`、`TOKENTRACER_EVENTS_*`、`TOKENTRACER_IMPORT_STATE`。

---

## Windows 建置／執行

前置：Rust stable（≥ 1.88 建議）、[WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)、Node 18+。

```powershell
# 在 repo root
cargo build -p pricing --bin spend
npm --prefix apps/ui ci
npm --prefix apps/ui run build

cargo install tauri-cli --version "^2" --locked

cd apps/tokenTracer-host
cargo tauri dev
# Release:
cargo tauri build   # nsis / msi
```

---

## macOS 建置／執行（實機 · Apple Silicon 優先）

前置：**Xcode**（含 CLT）、Rust stable、Node 18+、macOS 13+。

```bash
# repo root
xcode-select --install   # if needed
rustup target add aarch64-apple-darwin   # Apple Silicon
# Intel best-effort:
# rustup target add x86_64-apple-darwin

cargo build -p pricing --bin spend
npm --prefix apps/ui ci && npm --prefix apps/ui run build
cargo install tauri-cli --version "^2" --locked

cd apps/tokenTracer-host
cargo tauri dev
# .dmg:
cargo tauri build
# → target/release/bundle/dmg/*.dmg  （拖曳至 Applications）
```

驗收（給驗收官／橋樑，**不自 PASS**）：

1. `.dmg` 可裝可開（Apple Silicon）  
2. 選單列圖示 → Expanded 迷你面板；再點／失焦收合；Quit 結束  
3. 數字 vs `spend` CLI（F6 容差）；F12 notional disclaimer  
4. 空／PARTIAL 可診斷（非靜默空白）

---

## Blockers / Linux box 限制

- Linux agent box **無法**完整驗 Windows tray 或 **macOS 選單列**（無 AppKit／選單列）。
- 可嘗試 `cargo check -p tokentracer-host`（可能缺 `webkit2gtk`／tray 系統庫）。
- **不要**用假 PASS 截圖代替 Win／macOS 實機驗收。
- `cfg(target_os = "macos")` 路徑在 Linux 上只做編譯可見性／文件核對；實機行為由 Mac host／驗收官跑。

---

## 與橋樑分工

| 儀表（本 crate + `apps/ui`） | 橋樑 |
|-----------------------------|------|
| UI 狀態機、invoke、tray／選單列骨架、webview、README 行為表 | Win／Mac 環境協助（WebView2、Xcode、`.dmg` 實機）；**不代跑**驗收四條；結果以驗收官為準 |
| macOS 路徑 | 沿用 **v1.2b**（無本任務變更） |

---

## 驗收官獨立重跑

> **主驗收面＝本 Tauri 宿主**，不是 Vite `npm run dev`／`preview`（那只是 debug）。  
> 實作群／橋樑不代跑、不自 PASS。

### Win（AC-F7′）— 見既有四條（托盤／收合展開／CLI／notional）

### macOS（AC-F10 + F7′）

| # | 要驗 | How（Mac） | 對照 |
|---|------|------------|------|
| **①** | **選單列圖示**（禁僅瀏覽器） | `cargo tauri build`／`dev` 後選單列出現圖示；無 Dock 常駐；進程＝host bundle | AC-F10／F7′ |
| **②** | **迷你面板** | 點圖示 → Expanded；再點／失焦 → 僅圖示；選單 Quit；IA 對齊 Collapsed／Expanded | AC-F7′ |
| **③** | **數字 vs CLI** | 同 range／currency 對 `spend` JSON；空／PARTIAL 有文案 | F6／F7′／v1.2b |
| **④** | **`.dmg` + 權限** | dmg 拖曳安裝；權限失敗有步驟；FDA 非硬前置 | AC-F10 |

**明確非目標：** CI 綠 ≠ F10／F7′ PASS；`import_run` 仍 stub；Intel = best-effort。

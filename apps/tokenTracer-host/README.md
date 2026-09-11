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
| UI 狀態機、invoke 消費、tray 選單骨架、webview | Win 錨點／tray L/R 實機驗收、import glue、spend.exe 路徑 |

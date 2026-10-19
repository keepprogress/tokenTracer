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
| **選單列圖示** | **只有 1 個** status item（僅 `tray.rs` 建立；`tauri.conf.json` 不得再宣告 `app.trayIcon`）。macOS 圖示＝`icons/tray.png`（44×44 RGBA、**黑色字形＋透明背景**的 template image，系統依淺／深色選單列自動著色；由 `scripts/gen-tray-template-icon.py` 產生）；Windows／Linux 用彩色 `icons/tray-color.png`（黑色字形在深色工作列幾乎看不見，故不共用） |
| **點選單列圖示** | 顯示 **Expanded** 迷你面板（對齊 IA）；若已開，**設計上**收合為僅圖示（Hidden）。點圖示本身造成的失焦，會依 Down 當下的狀態（面板已 Expanded）／緊接前一次 Expanded 失焦（≤300 ms）判定為「同一次關閉」而不重開 |
| **失焦／點外面** | 面板**通常**關閉並回到選單列圖示（常駐）。只有「這個面板狀態確實取得過焦點、且稍後（下一個 main-loop 回合）視窗實際仍未取得焦點」的失焦才算關閉；較晚到的舊失焦、重複失焦、焦點很快回來的閃動會被忽略（見下「失焦判定」）。**Collapsed** 失焦會隱藏面板，但不影響下一次點擊（仍可 Hidden→Expanded）；**Expanded** 失焦才會讓緊接（≤300 ms）的圖示點擊視為「同一次關閉」。macOS 專屬；Windows 不使用 |
| **右鍵／Ctrl-點選單** | **Show panel**／**Refresh**／**Import…**／**Quit**。macOS 的選單**刻意不在建立圖示時掛上**（tray-icon 0.24.2 於 build 時 `setMenu`，實機上左鍵會直接彈出選單而打不開面板）；僅在右鍵／Ctrl+左鍵時暫時掛上→彈出→卸下，因此**預期**左鍵只切換面板（實機已驗證 d5abd29 左鍵 windows 0→1→0；未涵蓋所有 macOS 版本）。Windows／Linux 仍於建立時掛選單 |
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
npm --prefix apps/ui ci          # 必須先裝 UI 依賴（build hook 只跑 `npm run build`，不自動 ci）
cargo install tauri-cli --version "^2" --locked

cd apps/tokenTracer-host
cargo tauri dev
# .dmg（照這條原指令即可，無需本機覆寫）:
cargo tauri build
# → <repo>/target/release/bundle/dmg/tokenTracer_0.1.0_aarch64.dmg  （Intel: …_x64.dmg；拖曳至 Applications）
# 證據用 sha256（驗收／Mac 管理回報必附）:
shasum -a 256 ../../target/release/bundle/dmg/*.dmg
```

**建置 hook（B3）：** `tauri.conf.json` 的 `beforeBuildCommand`／`beforeDevCommand` 為
`{ "script": "npm run build|dev", "cwd": "../ui" }`。tauri-cli 在跑 hook 前會先 chdir 到 tauri 目錄
（`apps/tokenTracer-host`），所以 `../ui` ＝ `apps/ui`，與你從哪個目錄呼叫無關（**不要**再用
`npm --prefix ../ui`：hook 預設 cwd 是 `apps/`，會解到 `<repo>/ui` 而找不到）。

### 失焦判定（macOS，R6）

實機曾見「Collapsed → 失焦 → 100–250 ms 後點圖示」約 16%（5/32）最後沒有視窗。程式碼讀法（**未在 Mac 上證實**）：
較晚到的 `Focused(false)` 在 Up 已把面板切成 Expanded **之後**才處理，舊程式用「目前模式」把剛展開的面板又收回。
現行設計**不用**「展開後 N ms 內忽略失焦」（那會讓點外面不收合），改為純狀態判定（`crates/tray-guard`）：

1. 每次面板模式變更＝新的 *generation*，並記錄當下視窗是否已有焦點；`Focused(true)` 到達則標記此 generation 取得過焦點。
2. `Focused(false)`：此 generation 沒取得過焦點 → 忽略（舊狀態的失焦）；視窗當下 `is_focused()` ＝ true → 忽略；否則延後到**下一個 main-loop 回合**重判。
3. 重判：generation 已變 → 忽略；視窗又有焦點 → 忽略；仍無焦點 → 關閉（Expanded 才會啟動「同一次點擊不重開」的 300 ms 窗）。

取捨：面板若從未取得焦點（例如啟用失敗），就不會因失焦自動關閉，需再點圖示或用選單。

### 除錯：`TT_TRAY_DEBUG`（預設關閉）

啟動時設 `TT_TRAY_DEBUG`：`1` ＝ 寫 stderr；其他值 ＝ 當檔案路徑（附加寫入）。`open` 會丟棄 stderr，建議：

```bash
rm -f /tmp/tt-tray.log
open --env TT_TRAY_DEBUG=/tmp/tt-tray.log /Applications/tokenTracer.app
# 或直接跑二進位： TT_TRAY_DEBUG=1 /Applications/tokenTracer.app/Contents/MacOS/tokentracer-host
tail -f /tmp/tt-tray.log
```

每行 `[tt-tray mono=<ms 單調時鐘，自首次記錄起> unix=<epoch ms>] …`：`mono` 用來看順序／間隔，`unix` 可對齊腳本的 `Date.now()`。
會記錄：`apply_mode a -> b gen=… window_focused_before=…`、`Focused(true)`、
`Focused(false): mode=… window_focused_now=… -> Ignore(原因)|Defer{generation}`、`blur re-check: … -> Ignore(原因)|Dismiss|DismissAndArm`、`tray Down/Up: mode=… -> Toggle|DismissOnly|Ignore`。

重跑情境 c 時每次試驗要看：
- 成功：`Focused(false) … -> Defer` → `blur re-check … Dismiss`（Collapsed）→ `apply_mode collapsed -> hidden` → `tray Down: mode=hidden` → `tray Up … Toggle` → `apply_mode hidden -> expanded` →（`Focused(true)`）。
- 若 blur 晚到：`Focused(false)` 出現在 `apply_mode … -> expanded` **之後** → 應看到 `Ignore(NeverFocusedInThisState)`／`Ignore(Superseded)`，視窗維持展開。
- 仍失敗：找 `apply_mode expanded -> hidden` 前一行——若是 `blur re-check … DismissAndArm`，記下 `window_focused_now`、`gen` 與前後 `Focused(true)`；若前面沒有 blur 行，是別的路徑（UI invoke／CloseRequested）；若整個試驗沒有 `tray Down/Up` 行，點擊根本沒到 App（驅動／OS）。

### 無人值守／遠端 session 建 `.dmg`（D2）

`bundle_dmg.sh` 會用 Finder AppleScript 排視窗／圖示位置；在 **SSH／無人值守／無登入 GUI session** 下該步驟會
**逾時卡住**。設 `CI=true` 略過（dmg 仍會產出，只是 Finder 視窗版面為預設）：

```bash
cd apps/tokenTracer-host
CI=true cargo tauri build
shasum -a 256 ../../target/release/bundle/dmg/*.dmg
```

### 首次開啟：Gatekeeper 擋下時（D1）

`tokenTracer.app` 目前是 **ad-hoc 簽署、未 notarize**（無 Apple Developer ID）。從 `.dmg` 拖到
`/Applications` 後第一次開啟，macOS Gatekeeper 可能顯示「無法打開，因為無法驗證開發者」。任選其一：

1. **右鍵（或 Control-點）`tokenTracer.app` → 打開 → 在對話框再按「打開」**（只需做一次）。  
   若仍被擋：系統設定 → **隱私權與安全性** → 捲到底「仍要打開 “tokenTracer”」。
2. 或移除隔離屬性後再開：
   ```bash
   xattr -dr com.apple.quarantine /Applications/tokenTracer.app
   open /Applications/tokenTracer.app
   ```

這**不是** Full Disk Access（FDA）：FDA 仍只是權限讀取失敗時的疑難排解，**不是安裝前置**（見上「權限失敗時」）。

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

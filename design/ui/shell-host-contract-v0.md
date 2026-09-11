# 宿主／IPC 契約 v0（儀表 × 橋樑鎖定）

| 欄位 | 值 |
|------|-----|
| 版本 | **SHELL-HOST v0.1**（+ by-model／by-pool IPC） |
| 日期 | 2026-09-11 |
| 作者 | 儀表（依橋樑 2026-09-11 定稿） |
| 狀態 | **已鎖定** — 可開 mock 面板殼 |
| 前端 | `/workspace/tokenTracer-ui` |
| 對照 | UI-IA v0、UI-BIND v0.1、LEDGER-QUERY v0 |

---

## 1. 宿主

| 項 | 決策 |
|----|------|
| 框架 | **Tauri 2（Rust）** |
| Windows | 右下角迷你面板＋系統托盤；錨點＝工作列右下／通知區附近 |
| macOS | 選單列圖示＋下拉面板 |
| 進程 | **單進程**：discovery／權限／托盤殼 = Tauri 宿主內 **`tokentracer-bridge` Rust lib crate**；**不是**常駐 bridge 子進程 |
| CLI | 另提供 `tokentracer-bridge discover|…` 除錯／無 UI |
| 帳本 | 獨立 CLI／lib；宿主 `Command` 呼叫或後續 link |
| 啟動預設 | 托盤／選單列常駐 + **Collapsed 可見**（可設「僅圖示」） |

### Tray 行為（README 須寫死）
- Win 左鍵：切換 Collapsed↔Expanded（或開面板）
- Win 右鍵選單：重新整理／匯入／結束
- macOS：點選單列圖示開下拉面板（對齊 IA）

---

## 2. Repo 佈局（暫）

```text
/workspace/tokenTracer-bridge   # 橋樑：lib + CLI bin
/workspace/tokenTracer-ui       # 儀表：webview 前端（本契約消費者）
# 日後併 keepprogress/tokenTracer workspace
```

---

## 3. IPC（宿主 → webview；先 mock 後接真）

| 命令 | 參數 | 回傳 | 對齊 CLI／契約 |
|------|------|------|----------------|
| `spend_total` | `range`, `currency` | `SpendSummary` | `spend total --range … --currency …` |
| `spend_series` | `grain`, `range`, `currency` | `DailySpendSeries` | `spend series --grain day …` |
| `import_status` | — | `ImportMeta` | `import status --json` |
| `spend_by_model` | `currency`（+ range） | `by_model` 視圖／SpendSummary 子集 | `spend by-model` |
| `spend_by_pool` | `currency` | `by_usage_pool` | `spend by-pool` |
| `discover` | — | 橋樑發現結果 | 預留 |
| `import_run` | … | 含更新後 `ImportMeta` | 預留；觸發匯入 |

`range`：`today` \| `all` \| `7d` \| `30d` \| `90d`（panel switch 不含 today；today 另呼一次）。  
`grain`：v0 僅 `"day"`。

Mock 階段：前端 `src/mock/` 實作同名函式；真接線時改呼叫 Tauri `invoke`。

---

## 4. 交接

| 角色 | 下一步 |
|------|--------|
| 儀表 | mock 面板殼 Collapsed／Expanded；fixture 對核 |
| 橋樑 | Tauri 宿主殼＋tray／錨點；露出上表 invoke |
| 帳本 | bin 落地後宿主 Command／link |

---

**SHELL-HOST v0 — 已與橋樑鎖定**

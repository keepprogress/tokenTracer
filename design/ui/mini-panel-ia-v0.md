# tokenTracer 迷你面板 — 資訊架構／狀態線框 v0

| 欄位 | 值 |
|------|-----|
| 版本 | **UI-IA v0.3**（AC v1.3a；OPEN-UI-4 定案） |
| 日期 | 2026-09-11 |
| 作者 | 儀表 |
| 對照規格 | AC v1.1／v1.2／v1.1a／**v1.3（F11–F13）** |
| 狀態 | 先交付；可核對；**不發明計價** |
| 姊妹契約 | [`spend-summary-binding-v0.md`](./spend-summary-binding-v0.md) |

---

## 1. 目標與邊界

**做：** Windows 右下角縮圖／可展開＋可托盤（AC-F7）；macOS 選單列迷你面板（AC-F10；可先 UI 殼）；資訊架構與狀態對齊，數字綁帳本 `SpendSummary`／CLI。

**不做：** 全頁 web 主 UI（N5）；自算單價／匯率；虛報未支援 agent；最終放行。

**顯示名（v1.1a 必做）：** Cursor、Claude Code、Codex。其餘路線圖 agent 僅能以 `partial`／`unsupported` 出現，不得當已支援。

---

## 2. 平台殼（與橋樑同棧）

| 平台 | 殼位 | 交互慣例 | 備註 |
|------|------|----------|------|
| Windows | 右下角迷你面板；可選系統托盤 | 收合＝縮圖條；展開＝浮出面板；托盤左／右鍵開面板或選單（行為寫 README） | 主驗收面 AC-F7 |
| macOS | 選單列圖示 + 下拉迷你面板 | 點圖示展開／再點或失焦收合 | AC-F7′／AC-F10；可先殼 |

技術建議（不鎖死）：Rust＋Tauri（或橋樑選定之殼）。儀表負責面板視圖與狀態機；殼／托盤／權限歸橋樑。

---

## 3. 面板狀態機

```text
                    tray_click / hotkey / edge_click
   [Hidden] --------------------------------------> [Collapsed]
                                                         |
                                           expand_click  |  collapse_click / Esc / outside
                                                         v
                                                    [Expanded]
                                                         |
                              range_change / refresh / import_done
                                                         v
                                              (同態重繪；資料層更新)
```

| 狀態 | 可見內容 | 觸發進出 |
|------|----------|----------|
| **Hidden** | 僅托盤／選單列圖示（若有） | 關閉面板；啟動預設可進 Collapsed 或僅圖示（橋樑定 README） |
| **Collapsed** | 主數字摘要（見 §4.1） | 預設可見態（Win 右下角縮圖） |
| **Expanded** | 明細（見 §4.2） | 自 Collapsed 展開 |

錯誤／空資料不另開狀態：在 Collapsed／Expanded 內用 banner／佔位（§4.3）。

---

## 4. 資訊架構

### 4.1 收合態（Collapsed）— AC-F7.1

至少一項主數字 + 幣別：

```text
┌─ Win 右下角縮圖 ─────────────────┐
│  Today  $X.XX   ·  Total  $Y.YY  │  ← 主列；幣別 USD 為準
│  ▸  details                      │  ← 展開；可省略字改 chevron
└──────────────────────────────────┘
```

| 區塊 ID | 內容 | 資料來源 | 必／選 |
|---------|------|----------|--------|
| `primary.today` | 今日花費 | 見 binding：`SpendQuery(range=today)` → 對齊 CLI | **建議必顯**（AC-F7：「今日與／或歷史」；本 IA 採雙顯，窄寬時可只留 `primary.total`） |
| `primary.total` | 歷史總花費（目前區間；預設 `all`） | `SpendSummary.total` | **必顯其一**（與 today 至少一項） |
| `primary.currency` | `USD`（TWD 可選次標） | `SpendSummary.currency` | 隨主數字 |
| `affordance.expand` | 展開控制 | UI | 必 |

窄寬優先序：`total` > `today` > expand。

### 4.2 展開態（Expanded）— AC-F7.2／F7′

```text
┌─ Mini panel (≈320–360px) ──────────────────────┐
│ tokenTracer                          [↻] [–]   │
│ Range: ( All | 7d | 30d | 90d )                 │
│ ※ amounts notional (API estimate) unless noted  │
│─────────────────────────────────────────────────│
│ Today          $X.XX USD                        │
│ Total (range)  $Y.YY USD                        │
│─────────────────────────────────────────────────│
│ By agent                                        │
│  Claude Code   $a.aa   ████████░░  nn%   ok     │
│  Codex         $b.bb   ██████░░░░  nn%   ok     │
│  Cursor        $c.cc   ████░░░░░░  nn%   partial│
│─────────────────────────────────────────────────│
│ By model (Cursor pools)                         │
│  ▾ Cursor Models              $…                │
│      grok-4.6    $…   ████████░░  nn%            │
│      composer-…  $…   ██████░░░░  nn%            │
│  ▾ Other Models               $…   ◇ pool        │
│      claude-…    $…   ████░░░░░░  nn%            │
│      gpt-…       $…   ██░░░░░░░░  nn%            │
│  （禁止假列名「Other」／「Other model」當 model） │
│─────────────────────────────────────────────────│
│ Daily trend (spark / bars)                      │
│  ▁▂▃▅▄▆▇ …                                      │
│─────────────────────────────────────────────────│
│ Last import  2026-09-11 14:02 (local)           │
│ ⚠ PARTIAL: Cursor — 計價依 API／估算（見證據卡）│
│ price_table  v… · cost_nature notional · …      │
└─────────────────────────────────────────────────┘
```

| 區塊 ID | 內容 | 對照 AC | 資料 |
|---------|------|---------|------|
| `header.title` | 產品名 | — | 常數 `tokenTracer` |
| `header.refresh` | 重新整理／觸發匯入 | N4 可手動 | 橋樑／帳本指令 |
| `header.collapse` | 收合 | F7 | UI |
| `range.switch` | 全部／7／30／90 | O8、F7.2 | `SpendSummary.range.kind` |
| `summary.today` | 今日 USD | F7.2 | binding today 查詢 |
| `summary.total` | 區間總花費 USD | F7.2 | `SpendSummary.total` |
| `by_agent.list` | 各 agent 金額／佔比／status | F5、F7.2、v1.1a | `SpendSummary.by_agent` |
| `by_model.list` | 各 **具體模型** 金額／佔比 | AC v1.3 F11 | `SpendSummary.by_model`（真實 model id） |
| `by_pool.groups` | Cursor **用量池**分組：Cursor Models vs Other Models，其下再列 model | 指揮官更正＋EC-cursor-other-models-v1 | `by_usage_pool`／`spend by-pool` |
| `meta.notional` | 標 notional／估價性質 | F6、v1.1a | `cost_nature` 等；禁止當帳單原件 |
| `trend.daily` | 按日趨勢 | F7.2 | binding `DailySpendSeries`（帳本提供；非 UI 自算） |
| `meta.last_import` | 最後匯入時間 | F7.2 | binding `ImportMeta` |
| `meta.partial_warn` | PARTIAL 警告 | F7.2、v1.1a | 由 `by_agent[].status`＋證據文案組裝 |
| `meta.provenance` | `price_table_version`、`computed_at`；（可選）fx | F6 | `SpendSummary` 欄位 |

**佔比：** UI 只做展示用 `amount / sum(ok∪partial amounts)`；加總核對以帳本 CLI 為準（AC-F5／F6 容差）。禁止 UI 重算單價。

#### 4.2.1 各模型花費＋用量池（展開態）

| 規則 | 說明 |
|------|------|
| 範圍 | **Expanded 必顯**；Collapsed 不塞 |
| by_model | 具體 model id 列（AC-F11）；UI 不自加總事件 |
| **usage_pool（Cursor）** | 分組 **Cursor Models** vs **Other Models**（池標籤），其下再列具體 model 花費 |
| **禁止** | 虛構 model 列名「Other」／「Other model」（EC：Other 是池，不是 model id） |
| 池標記 | Other Models 組可用 `◇ pool` 徽章與 Cursor Models 組視覺區分 |
| Notional | `pricing_mode`／`disclaimer` 必顯（AC-F12）；≠ 帳單 |

資料來自 `by_usage_pool`＋`by_model`；**不**加假 Other 模型列；unknown 池標「池未知／待 API」。

### 4.3 空／錯／PARTIAL 呈現

| 情況 | Collapsed | Expanded |
|------|-----------|----------|
| 尚無資料 | `Total  —` | 空態文案：「尚無匯入；點 ↻ 或見 README」 |
| 匯入／計算中 | 主數字旁 subtle spinner | 區塊級 skeleton；不清空舊數字直到新 `SpendSummary` 到達 |
| PARTIAL agent | 可選小點／⚠ | `by_agent` 列 status + 底部 banner（Cursor：禁止把本機 `tokenCount=0` 當 billed；估價須標明） |
| unsupported | 不進主列 | 可摺疊「未支援」或省略 |
| CLI／帳本錯誤 | `!` 徽章 | 可診斷碼＋下一步（橋樑契約） |

### 4.4 托盤／選單列（若實作則必驗）

| 平台 | 最小行為 |
|------|----------|
| Win 托盤 | 圖示存在；左鍵切換 Collapsed／Expanded（或開面板）；右鍵選單至少：顯示面板、重新整理、退出（文案進 README） |
| macOS 選單列 | 圖示；點開＝Expanded 等價內容；可含「重新整理」 |

---

## 5. 畫面驗收對照清單（給自己／驗收官預對）

- [ ] Collapsed 至少一主數字 + USD  
- [ ] Expanded：total、by_agent、**by_model**、Cursor **usage_pool 雙池**（其下具體 model）、daily trend、range、last import、PARTIAL／notional；**無**假 Other 模型列  
- [ ] Range 切換只打帳本查詢，不改價表  
- [ ] 數字與 `spend …` CLI 同容差（≤ $0.01 或 0.1%）  
- [ ] 非全頁 web 為主驗收面  
- [ ] macOS 選單列 IA 與上表對齊（殼可先）  
- [ ] v1.1a：Cursor 列為 partial；不虛報路線圖 agent  

---

## 6. 下一步

1. ~~與橋樑鎖殼~~ → SHELL-HOST v0 已鎖；mock 殼在 `/workspace/tokenTracer-ui`。  
2. OPEN-UI-4 已回填；UI-BIND v0.3 定案。  
3. mock：雙池＋具體 model＋disclaimer。  
4. 接真 CLI／IPC 後交適配驗／驗收官。

---

**UI-IA v0.3 — 儀表定案（AC v1.3a）**

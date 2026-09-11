# tokenTracer 迷你面板 — 資訊架構／狀態線框 v0.4-draft

| 欄位 | 值 |
|------|-----|
| 版本 | **UI-IA v0.4-draft**（AC **v1.4-cursor-official** F17–F19；繼承 v0.3） |
| 日期 | 2026-09-11 |
| 作者 | 儀表 |
| 對照規格 | AC v1.1／v1.2／v1.1a／v1.3a（F11–F13′／F12）／**AC v1.4-cursor-official（F14–F19）** |
| 狀態 | **draft**（帳本 OPEN-BIND-S1…S4 ✅ → `design/ledger/open-bind-spending-align-v0.md`） — 待帳本 `spending_align` 欄位與指揮官核對；**不發明 API／不自 PASS** |
| 姊妹契約 | [`spend-summary-binding-v0.4-draft.md`](./spend-summary-binding-v0.4-draft.md) |
| 前版 | [`mini-panel-ia-v0.md`](./mini-panel-ia-v0.md)（v0.3 定案） |

---

## 1. 目標與邊界

**做：** 繼承 v0.3 Collapsed／Expanded＋F7′ 托盤／選單列殼；在 Expanded **永久拆成兩表面**：

| 表面 | 含義 | 權威 |
|------|------|------|
| **A. Local notional** | 既有 `SpendSummary` $／`by_model`／`by_usage_pool` $；F12 disclaimer | 本機帳本 notional_api_estimate |
| **B. Spending align (Cursor)** | Cursor Models %／Other Models %／reset／on-demand 為 **對齊目標**（P1） | Spending UI／手動／未來 opt-in；**非**本機 token 加總 |

**不做：** 全頁 web 主 UI（N5）；自算單價／匯率；用本機 token 冒充 Spending 兩池％（N25）；虛構 Other model 列（F19）；宣稱個人有公開 usage API（F17 BLOCK）；undocumented 預設開（F16）；最終 PASS。

**顯示名（v1.1a）：** Cursor、Claude Code、Codex。其餘路線圖 agent 僅 `partial`／`unsupported`。

---

## 2. 平台殼（繼承 v0.3）

| 平台 | 殼位 | 備註 |
|------|------|------|
| Windows | 右下角迷你面板；可選系統托盤 | 主驗收面 AC-F7／F7′ |
| macOS | 選單列圖示 + 下拉 | AC-F7′／F10；可先殼 |

狀態機（Hidden → Collapsed → Expanded）**不變**；本版只擴 Expanded 內容分區。

---

## 3. 面板狀態機（繼承）

```text
                    tray_click / hotkey / edge_click
   [Hidden] --------------------------------------> [Collapsed]
                                                         |
                                           expand_click  |  collapse_click / Esc / outside
                                                         v
                                                    [Expanded]
                                                         |
                              range_change / refresh / import_done / spending_align_update
                                                         v
                                              (同態重繪；資料層更新)
```

錯誤／空資料／PARTIAL（含 F17）不另開狀態：在 Collapsed／Expanded 內用 banner／佔位。

---

## 4. 資訊架構

### 4.1 收合態（Collapsed）— AC-F7.1（繼承 v0.3）

主數字仍為 **A. Local notional**（Today／Total $）。**不**在 Collapsed 塞 Spending %（避免誤讀為帳單／額度）。

```text
┌─ Win 右下角縮圖 ─────────────────┐
│  Today  $X.XX   ·  Total  $Y.YY  │  ← notional；非 Spending ％
│  ▸  details                      │
└──────────────────────────────────┘
```

可選：若 `spending_align.partial=true`，Collapsed 僅小點／⚠，細節進 Expanded。

### 4.2 展開態（Expanded）— 雙表面永久分離

```text
┌─ Mini panel (≈320–380px) ──────────────────────────┐
│ tokenTracer                              [↻] [–]   │
│ Range: ( All | 7d | 30d | 90d )                     │
│ ※ A amounts = notional (API estimate); ≠ 訂閱額度   │  ← F12
│─────────────────────────────────────────────────────│
│ ▌ A. Local notional                                 │
│ Today          $X.XX USD                            │
│ Total (range)  $Y.YY USD                            │
│─────────────────────────────────────────────────────│
│ By agent                                            │
│  Claude Code   $a.aa   ████████░░  nn%   ok         │
│  Codex         $b.bb   ██████░░░░  nn%   ok         │
│  Cursor        $c.cc   ████░░░░░░  nn%   partial    │
│─────────────────────────────────────────────────────│
│ By model (Cursor usage_pool $ — notional)           │
│  ▾ Cursor Models              $…                    │
│      grok-4.6    $…   ████████░░  nn%                │
│      composer-…  $…   ██████░░░░  nn%                │
│  ▾ Other Models               $…   ◇ pool           │
│      claude-…    $…   ████░░░░░░  nn%                │
│      gpt-…       $…   ██░░░░░░░░  nn%                │
│  （禁止假列名「Other」／「Other model」當 model — F19）│
│─────────────────────────────────────────────────────│
│ Daily trend (spark / bars)                          │
│  ▁▂▃▅▄▆▇ …                                          │
│─────────────────────────────────────────────────────│
│ ▌ B. Spending align (Cursor) — 對照層／非本機加總    │
│  source: manual_p1 | official_admin | … | none      │
│  ⚠ PARTIAL · 無公開個人 usage API          ← F17   │
│  Cursor Models   ~~65%~~   (align target)           │
│  Other Models    ~~100%~~  (align target)           │
│  Reset           ≈ 9/12                             │
│  On-demand       Disabled                           │
│  ※ % 來自 Spending／手動／Admin；≠ A 區 token $     │
│  （可選）Spending 有 Grok Bot 週列 · 機器同步=HOLD  │  ← F18
│─────────────────────────────────────────────────────│
│ Last import  2026-09-11 14:02 (local)               │
│ ⚠ PARTIAL: Cursor — 計價依 API／估算（見證據卡）    │
│ price_table  v… · cost_nature notional · …          │
└─────────────────────────────────────────────────────┘
```

| 區塊 ID | 內容 | 對照 AC | 資料 |
|---------|------|---------|------|
| （繼承）`header.*`／`range.switch`／`summary.*`／`by_agent`／`by_model`／`by_pool`／`trend`／`meta.*` | 同 v0.3 | F7／F11–F13′／F12 | `SpendSummary`／series／ImportMeta |
| `section.local_notional` | 視覺分區標籤「A. Local notional」 | G14、F12 | UI 常數 |
| `section.spending_align` | 視覺分區標籤「B. Spending align」 | G14、F17–F19 | UI 常數 |
| `spending.source_mode` | `none`／`manual_p1`／`official_admin`／`undocumented_opt_in` | F14／F16／F17 | `SpendingAlignVM.source_mode` |
| `spending.partial_banner` | PARTIAL＋「無公開個人 usage API」 | **F17** | `partial`＋`partial_reason` |
| `spending.cursor_models_pct` | Cursor Models %（對齊目標） | **F17** | `cursor_models_pct`；**禁止**用 A 區 $ 推算 |
| `spending.other_models_pct` | Other Models %（對齊目標） | **F17** | `other_models_pct`；同上 |
| `spending.reset_label` | 月 reset 文案 | F17 | `reset_label` |
| `spending.on_demand` | on-demand 狀態文案 | F17 | `on_demand` |
| `spending.grok_bot_note` | 可選「Spending 有 Grok Bot 週列」；機器=HOLD | **F18** | `grok_bot_week_note` |
| `spending.compare_disclaimer` | 「對照層；≠ 本機 notional／假 Other model」 | **F19**、N25 | UI 常數 |

#### 4.2.1 表面分離硬規則（本版鎖定）

| 規則 | 說明 |
|------|------|
| **永遠拆分** | A／B 兩區視覺與資料源分離；不得合併成單一「花費％」條 |
| **A 不驅動 B** | Local notional totals／`by_usage_pool` $ **不得**寫入或推算 `spending_align` % |
| **B 不回寫假 model** | Spending 兩池％＝**comparison layer only**；禁止雙寫為 `by_model` 假「Other」列（**F19**） |
| **F17 個人 Ultra** | 無 Admin key → 必顯 PARTIAL＋「無公開個人 usage API」；池％＝手動／截圖路徑（`manual_p1`）或未來 `undocumented_opt_in`；**不發明 API** |
| **F18** | 可選註「Spending 有 Grok Bot 週列」；機器同步用量％＝**HOLD** |
| **F12 回歸** | A 區 notional disclaimer 仍須；notional ≠ 訂閱額度／Spending ％ |
| **F15** | 僅本機源時不得輸出唯一權威 USD 冒充帳單；指向 B 區／需 Admin |

#### 4.2.2 `source_mode` 呈現

| `source_mode` | UI 行為 |
|---------------|---------|
| `none` | B 區佔位：「尚未對齊 Spending；可手動輸入／截圖核對（P1）」 |
| `manual_p1` | 顯示使用者／fixture 對齊目標％；標 PARTIAL（個人） |
| `official_admin` | Team／Enterprise Admin（F14）；可標 official；仍與 A 區分欄 |
| `undocumented_opt_in` | 每次標 **UNSUPPORTED／undocumented／易碎**；不得自稱 `official_admin`（F16） |

### 4.3 空／錯／PARTIAL（擴充 F17）

| 情況 | Collapsed | Expanded |
|------|-----------|----------|
| （繼承 v0.3）無資料／loading／agent partial／error | 同 v0.3 | 同 v0.3 |
| 個人 Ultra、無 Admin（F17） | 可選 ⚠ | B 區必顯 PARTIAL＋「無公開個人 usage API」；A 區仍可有 notional $ |
| `spending_align=null`／`source_mode=none` | 無 | B 區空態＋手動路徑提示 |
| Grok Bot 機器源未知（F18） | — | 僅 optional note；不顯示臆測％ |

### 4.4 托盤／選單列（繼承 v0.3 F7′）

行為同 v0.3；托盤不單獨顯示 Spending ％為主數字。

---

## 5. 畫面驗收對照清單（視覺預對 — **非 PASS**）

> 最終 PASS **僅驗收官**；本清單供自測／設計核對。

- [ ] Collapsed：至少一主數字 + USD（A notional）；不把 Spending % 當主列
- [ ] Expanded：**A／B 兩區清楚分離**（標題或分隔線可見）
- [ ] A：total、by_agent、by_model、Cursor usage_pool $、daily trend、range、last import、F12 notional；**無**假 Other 模型列（F19／F13′）
- [ ] B：Cursor Models %／Other Models %／reset／on-demand 為對齊目標；文案標對照層
- [ ] F17：個人路徑必顯 PARTIAL＋「無公開個人 usage API」
- [ ] F18：若顯示 Grok Bot 註記，機器同步標 HOLD／不臆測 endpoint
- [ ] F19：Spending ％不雙寫成假 Other model 列
- [ ] N25／綁定：A 區 $ **不**驅動 B 區 %
- [ ] F15：僅 local 時不冒充帳單權威 USD
- [ ] F16：undocumented 預設關；opt-in 必標 UNSUPPORTED
- [ ] Range 切換只打帳本查詢，不改價表；數字對 CLI 容差仍屬 A 區
- [ ] 非全頁 web 為主驗收面；macOS IA 對齊（殼可先）

---

## 6. OPEN／下一步（draft）

| ID | 內容 | 擁有者 |
|----|------|--------|
| OPEN-UI-S1 | 帳本 `spending_align` schema：Team Admin（F14）vs 個人 PARTIAL（F17）欄位定稿 | 帳本＋儀表 |
| OPEN-UI-S2 | 手動／截圖核對 UX（輸入％ vs 貼圖） | 儀表／指揮官 |
| OPEN-UI-S3 | Collapsed 是否永遠不露 B％（本 draft 建議不露） | 指揮官 |
| OPEN-C6…C9 | 見 AC v1.4（不擋本 draft） | 規格 |

1. 帳本回填 `SpendingAlign`／query → 升正式 v0.4（去 draft）。  
2. mock：雙表面＋F17 banner＋禁假 Other。  
3. 與 PR #8／殼並行；本文件僅設計稿。

---

**UI-IA v0.4-draft — 儀表草案（AC v1.4-cursor-official F17–F19）**  
**狀態：draft · 待帳本欄位／指揮官 · 不自 PASS**

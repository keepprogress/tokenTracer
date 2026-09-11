# 迷你面板 ↔ SpendSummary／SpendingAlign 綁定契約 v0.4-draft

| 欄位 | 值 |
|------|-----|
| 版本 | **UI-BIND v0.4-draft**（AC **v1.4-cursor-official** F17–F19；繼承 v0.3） |
| 日期 | 2026-09-11 |
| 作者 | 儀表 |
| 對照 | AC v1.1 §5.2；AC-F6／F7；v1.1a；v1.3a（F11–F13′／F12）；**v1.4 F14–F19** |
| 狀態 | **draft** — 待帳本 `spending_align` 欄位定稿；**UI 不計價／不自 PASS** |
| 姊妹 | [`mini-panel-ia-v0.4-draft.md`](./mini-panel-ia-v0.4-draft.md) |
| 前版 | [`spend-summary-binding-v0.md`](./spend-summary-binding-v0.md)（v0.3 定案） |

---

## 1. 規範來源（唯讀）

繼承 v0.3 `SpendSummary` 形狀（currency／total／by_agent／by_model／by_usage_pool／disclaimer／…）。本版 **新增對照層**，不改 notional 計價語意：

```text
// 帳本待定（OPEN — 見 §7）；儀表先鎖 VM 消費面
SpendingAlign? {
  cursor_models_pct: number | null     // 0–100；對齊目標
  other_models_pct: number | null
  reset_label: string | null           // 例 "Resets 9/12" / "≈ 9/12"
  on_demand: string | null             // 例 "Disabled" / "Enabled · $…"
  source_mode: "none" | "manual_p1" | "official_admin" | "undocumented_opt_in"
  partial: bool
  partial_reason: string | null        // 個人：須能表達「無公開個人 usage API」
  grok_bot_week_note: string | null    // 可選；機器同步 = HOLD
}
```

**硬約束：** `SpendSummary` 的 local notional totals／`by_usage_pool` 金額 **不得** 驅動或覆寫 `SpendingAlign` 的％欄位（AC N25／G14／F17）。

---

## 2. 視圖模型（UI 層，純映射）

繼承 v0.3 `MiniPanelVM` 全部欄位；**擴充：**

```text
MiniPanelVM {
  // ——— 繼承 v0.3 ———
  currency: "USD" | "TWD"
  today_total: number | null
  range_total: number
  range: { kind: "all"|"7d"|"30d"|"90d", start?: string, end?: string }
  by_agent: AgentRowVM[]
  by_model: ModelRowVM[] | null
  by_usage_pool: PoolGroupVM[] | null
  trend: DailyPointVM[] | null
  last_imported_at: string | null
  partial_warnings: WarningVM[]
  cost_nature: "notional_api_estimate" | "vendor_reported" | "mixed" | null
  price_table_version: string
  computed_at: string
  fx_snapshot?: …
  load_state: "idle"|"loading"|"error"
  error?: { code: string, message: string }

  // ——— v0.4-draft 新增 ———
  spending_align: SpendingAlignVM | null
}

SpendingAlignVM {
  cursor_models_pct: number | null       // 對齊目標％；非 notional 推算
  other_models_pct: number | null
  reset_label: string | null
  on_demand: string | null
  source_mode: "none" | "manual_p1" | "official_admin" | "undocumented_opt_in"
  partial: bool
  partial_reason: string | null
  grok_bot_week_note: string | null      // optional；機器同步 HOLD（F18）
}

// AgentRowVM / ModelRowVM / PoolGroupVM / DailyPointVM / WarningVM — 同 v0.3
```

### 2.1 映射表

| UI 區塊 ID | 來源 | 規則 |
|------------|------|------|
| （繼承）`summary.*`／`by_agent`／`by_model`／`by_usage_pool`／`meta.notional`／… | `SpendSummary` 等 | 同 v0.3；**僅 A 表面** |
| `spending.source_mode` | `spending_align.source_mode` | 缺省／null VM → 視為 UI 顯示 `none` 佔位 |
| `spending.cursor_models_pct` | `cursor_models_pct` | 原樣展示；**禁止** `f(by_usage_pool.amount)` |
| `spending.other_models_pct` | `other_models_pct` | 同上 |
| `spending.reset_label` | `reset_label` | 原樣 |
| `spending.on_demand` | `on_demand` | 原樣 |
| `spending.partial_banner` | `partial`＋`partial_reason` | `partial=true` 必顯；個人路徑文案見 §2.3 |
| `spending.grok_bot_note` | `grok_bot_week_note` | 有則顯；無則省略；不臆測％ |

### 2.2 固定顯示名（繼承 v0.3）

| AgentId | display_name |
|---------|--------------|
| `cursor` | Cursor |
| `claude_code` | Claude Code |
| `codex` | Codex |

### 2.3 PARTIAL／對齊文案（擴充）

| 情境 | AC | 預設文案（可調，語意不可弱化） |
|------|-----|--------------------------------|
| Cursor agent notional | F12／v1.1a | （繼承）`Cursor：本機 tokenCount 不可靠；計價依 API／估算路徑（EC-cursor-v1）` |
| 個人 Ultra、無公開 API | **F17** | `PARTIAL：無公開個人 usage API；兩池％為 Spending 對齊目標（手動／截圖），非本機 token 加總` |
| 僅 local enrichment | **F15** | `PARTIAL：本機源不得單獨當 Cursor 帳單／USD 權威；請對照 Spending 或 Team Admin` |
| undocumented opt-in | **F16** | `UNSUPPORTED／undocumented／易碎 — 非 official_admin` |
| Grok Bot 週列 | **F18** | `Spending 有 Grok Bot 週列；機器同步＝HOLD`（僅 note，不填假％） |

---

## 3. 繼承查詢（v0.3 — 不變）

OPEN-UI-1…4（today／series／ImportMeta／by_model＋by_usage_pool）維持 v0.3。刷新時序對 A 區仍並行：

```text
spend total / today / by-model / by-pool / series / import status
```

### 3.1 新增：Spending align 載入（draft）

```text
UI refresh | spending_manual_save | admin_import_done
    →（與 A 區並行、獨立 payload）
         spend spending-align --json   # 名稱待帳本；或獨立 ImportMeta 旁路
    → spending_align ← SpendingAlignVM | null
    → 映射時：若誤把 notional total 寫入 pct → BIND 違規（測試應抓）
```

**明確：local notional totals must not drive `spending_align` %。**

---

## 4. UI chips／copy → AC ID 對照

| UI 元素 | Copy／行為摘要 | AC ID |
|---------|----------------|-------|
| A 區 notional disclaimer | amounts notional；≠ 訂閱額度 | **F12** |
| A 區／僅本機時反帳單 banner | 不得唯一權威 USD 冒充帳單；指向 Spending／Admin | **F15** |
| B 區 PARTIAL＋「無公開個人 usage API」 | 個人 Ultra 對齊目標路徑 | **F17** |
| B 區兩池％／reset／on-demand | 對齊目標（P1），非本機加總 | **F17** |
| B 區 Grok Bot 週列註（可選） | UI 可註；機器同步 HOLD | **F18** |
| 無假 Other model 列；％僅對照層 | 不雙寫冒充 model | **F19** |
| usage_pool 組標題下具體 model $ | 事件層池標籤（繼承） | F13′／**F19** |
| undocumented 標籤 | 預設關；opt-in 標 UNSUPPORTED | F16 |
| Team Admin 對帳路徑（若有） | official_admin | F14 |

---

## 5. 禁止清單（Ban list）

| 禁止 | 理由 |
|------|------|
| 把 L1 bubble／`tokenCount` 當帳單或 Spending ％ | F15／N22／C-block |
| 發明／顯示假 **Other** model 列（字面 `Other`／`Other model`／主路徑 `cursor:other`） | **F19**／F13′ |
| 只顯示 notional $ 卻標成 **Spending %**（或單一合併條） | N25／G14／**F17** |
| 用 `by_usage_pool.amount`／token 加總推算 Cursor／Other Models % | N25；綁定硬約束 |
| 個人路徑宣稱有公開官方 usage API | F17 BLOCK／OPEN-C1 |
| undocumented 預設開啟或自稱 `official_admin` | F16 |
| 臆測 Grok Bot 週％ endpoint／機器同步數字 | **F18** |
| UI 硬編碼單價、匯率、臆造金額 | 繼承 v0.3 |

---

## 6. 數字格式（A 區繼承）

| 規則 | 值 |
|------|-----|
| 主幣別 | USD；`$` + 兩位小數 |
| 容差 | ≤ $0.01 或 ≤ 0.1%（AC-F6）— **僅** notional／Admin chargedCents 對核 |
| B 區 % | 人手／截圖一致即可；機器 opt-in 建議 ±1 pp（AC OPEN-C3）— 不由 UI 自算 |

---

## 7. OPEN（帳本 — 擋正式 v0.4 去 draft）

| ID | 內容 |
|----|------|
| **OPEN-BIND-S1** | `SpendingAlign`（或等價）帳本 schema／CLI：`Team Admin (F14)` 是否併入同一物件，或 `official_admin` 另附 `charged_cents_reconcile` |
| **OPEN-BIND-S2** | 個人 PARTIAL（F17）：`manual_p1` 儲存位置（ledger 旁路 vs settings）；截圖核對是否只留 UI 態 |
| **OPEN-BIND-S3** | `source_mode` 枚舉是否與規格 `official_admin`／`local_enrichment`／`undocumented_dashboard` 1:1 對表（本 draft VM 用 `manual_p1`＋`undocumented_opt_in` 對齊產品語意） |
| **OPEN-BIND-S4** | `grok_bot_week_note` 是否升級為結構化欄（HOLD 至 OPEN-C9） |

不擋本 draft 交付；實作 mock 可先硬編碼 `SpendingAlignVM` fixture（標 example data）。

---

## 8. Fixture 自測（儀表 — 非 PASS）

1. Mock A：`SpendSummary`＋雙池 $ → A 區正常；B＝`null`／`none` 佔位。  
2. Mock B：`manual_p1`＋65／100／reset／on-demand＋`partial=true`＋F17 reason → B 區對齊；A 數字不變。  
3. 反例：若適配層用 pool $ 填 pct → **失敗**（ban）。  
4. 反例：`by_model` 出現字面 Other → **失敗**（F19）。  
5. F18：僅 note、無週％數字。  

---

## 9. 與角色交界

| 角色 | 期待 |
|------|------|
| 帳本 | 回填 OPEN-BIND-S1…S3；不把 notional 當 Spending % |
| 橋樑 | IPC 透傳兩路 payload；不合併計算 |
| 儀表 | IA＋本綁定 draft；mock 雙表面；不計價 |
| 驗收官 | F12／F15／F17–F19 視覺＋對 CLI；最終 PASS 僅驗收官 |

---

## 10. 交接摘要

| 項目 | 狀態 |
|------|------|
| IA 雙表面 | **draft** `mini-panel-ia-v0.4-draft.md` |
| MiniPanelVM.`spending_align` | **本文件 draft** |
| Local ↛ Spending % | **鎖定** |
| 帳本 schema | **OPEN** |
| 產品碼／PASS | **無**（設計稿 only） |

---

**UI-BIND v0.4-draft — 儀表草案（AC v1.4-cursor-official）**  
**狀態：draft · 待帳本欄位／指揮官 · 不自 PASS**

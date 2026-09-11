# 迷你面板 ↔ SpendSummary 綁定契約 v0.3

| 欄位 | 值 |
|------|-----|
| 版本 | **UI-BIND v0.3**（已定案：AC v1.3∪**v1.3a**；OPEN-UI-4 帳本已回填） |
| 日期 | 2026-09-11 |
| 作者 | 儀表 |
| 對照 | AC v1.1 §5.2 `SpendSummary`；AC-F6／F7；AC v1.1a |
| 帳本延伸 | LEDGER-QUERY v0；[`open-ui-4-by-model-v0.md`](../ledger/open-ui-4-by-model-v0.md)；[`by-model-v0.md`](../ledger/by-model-v0.md)（v0.2）；AC v1.3a |
| 原則 | **UI 不計價**；只消費帳本結果；金額標 **notional**（除非帳本標 vendor_reported） |
| 姊妹 | [`mini-panel-ia-v0.md`](./mini-panel-ia-v0.md) |

---

## 1. 規範來源（唯讀，不改欄位語意）

AC v1.3 §3.2（繼承 v1.1 + `today` range）權威形狀：

```text
SpendSummary {
  currency: "USD" | "TWD"
  total: number | null
  pricing_mode: "notional_api_estimate"
  price_table_version: string
  price_table_source: string
  by_agent: { agent, amount, status }[]
  by_model: {
    model: string                 // 含哨兵 cursor:other / cursor:unknown
    agent: AgentId | null
    amount: number | null
    input_tokens: int
    output_tokens: int
    priced: bool
  }[]
  unpriced_tokens?: { model, input_tokens, output_tokens }[]
  fx_snapshot?: ...
  range?: { kind: "today"|"all"|"7d"|"30d"|"90d", start?: string, end?: string }
  computed_at: string
  disclaimer: string              // 須含 notional／非帳單語意
}
```

`AgentId`（§5.1）：`codex|claude_code|cursor|grok_build|cline|opencode|github_copilot|other`

儀表 **不得** 增刪計價語意；延伸查詢以 LEDGER-QUERY v0 為準，不在 UI 內推算金額。

---

## 2. 視圖模型（UI 層，純映射）

```text
MiniPanelVM {
  currency: "USD" | "TWD"          // ← SpendSummary.currency；預設顯示 USD
  today_total: number | null       // ← spend total --range today → SpendSummary.total
  range_total: number              // ← spend total --range <panel> → SpendSummary.total
  range: { kind: "all"|"7d"|"30d"|"90d", start?: string, end?: string }
  by_agent: AgentRowVM[]
  by_model: ModelRowVM[] | null
  by_usage_pool: PoolGroupVM[] | null  // ← SpendSummary.by_usage_pool
  trend: DailyPointVM[] | null     // ← spend series → DailySpendSeries.points
  last_imported_at: string | null  // ← import status → ImportMeta（≠ computed_at）
  partial_warnings: WarningVM[]
  cost_nature: "notional_api_estimate" | "vendor_reported" | "mixed" | null
  price_table_version: string
  computed_at: string
  fx_snapshot?: …
  load_state: "idle"|"loading"|"error"
  error?: { code: string, message: string }
}

AgentRowVM {
  agent: AgentId
  display_name: string
  amount: number
  share: number | null             // UI 展示用：amount / Σ(ok∪partial)；分母 0 → null
  status: "ok"|"partial"|"unsupported"
}

ModelRowVM {
  model: string                    // 具體 model id（modelIntent 等）；禁止假 id "Other"
  display_name: string             // 通常 = model
  agent: AgentId | null
  usage_pool: "cursor_models" | "other_models" | "unknown" | null
  amount: number | null
  input_tokens: number
  output_tokens: number
  priced: boolean
  share: number | null
}

PoolGroupVM {
  agent: "cursor"
  pool: "cursor_models" | "other_models" | "unknown"
  display_name: string             // "Cursor Models" | "Other Models" | "池未知／待 API"
  amount: number | null
  percent_used: number | null
  models: ModelRowVM[]             // 由 by_model 依 usage_pool 歸組；非 UI 自計價
}

DailyPointVM { date: string /* YYYY-MM-DD */, amount: number }

WarningVM { agent: AgentId, level: "partial", text: string }
```

### 2.1 映射表（SpendSummary → 面板區塊）

| UI 區塊 ID | SpendSummary 欄位 | 規則 |
|------------|-------------------|------|
| `summary.total` / `primary.total` | `total` + `currency` | 直接顯示；格式見 §5 |
| `summary.today` / `primary.today` | today 查詢之 `total` | 見 §3.1 |
| `range.switch` | `range.kind` ∈ all／7d／30d／90d | 切換＝重新查詢，不本地過濾 |
| `by_agent.list` | `by_agent[]` | 排序：amount 降序；`unsupported` 可折疊 |
| `by_model.list` | `by_model[]` | 具體 model 列；**禁止**假列 `Other` |
| `by_usage_pool` | `by_usage_pool[]`／`spend by-pool` | 池標籤 Cursor Models／Other Models；其下接 by_model |
| `meta.notional` | `cost_nature` | 預設視為 notional；標在 summary 區 |
| `meta.provenance` | `price_table_version`, `computed_at`, `fx_snapshot?` | 原樣展示 |
| `meta.partial_warn` | `by_agent[].status` | status=`partial` → 警告列 |

### 2.2 固定顯示名（v1.1a）

| AgentId | display_name |
|---------|--------------|
| `cursor` | Cursor |
| `claude_code` | Claude Code |
| `codex` | Codex |
| 其餘 | 與 AgentId 可讀化；路線圖預設不主列 |

### 2.3 PARTIAL 文案（來源 v1.1a 硬約束；非計價）

| AgentId | 預設 `WarningVM.text` |
|---------|----------------------|
| `cursor` | `Cursor：本機 tokenCount 不可靠；計價依 API／估算路徑（EC-cursor-v1）` |
| `claude_code` / `codex` | 若 status=partial：`USD 可能為 notional API 估價，非帳單原件` |
| 其他 partial | `{display_name}：資料不完整（partial）` |

文案可調，但 **不得** 暗示已完整支援或隱瞞估價性質。

---

## 3. 延伸查詢（**已回填** ← LEDGER-QUERY v0）

權威細節以帳本 [`spend-query-extensions-v0.md`](../ledger/spend-query-extensions-v0.md) 為準。以下為儀表消費摘要；CLI 實作隨 pricing crate 落地前可用 schema mock。

### 3.1 OPEN-UI-1 — 今日花費 ✅ 已回填

- **決策：** 選項 A — `range.kind` 含 `"today"`；日界預設 **local**（可 `--day-boundary local|utc`）；窗 `[day_start, day_start+1d)`。
- **CLI：**
  ```bash
  spend total --currency USD --range today
  # 可選別名：spend today --currency USD
  ```
- **UI 綁定：** `today_total ← SpendSummary.total`（該次 today 查詢）；`range.start`／`end` 為 UTC 瞬間，對核用。
- **Collapsed：** 可顯 today + total；CLI 未就緒前 mock 同 schema。

### 3.2 OPEN-UI-2 — 按日趨勢 ✅ 已回填

帳本 schema（UI 只取 `points` 繪圖）：

```text
DailySpendSeries {
  currency: "USD" | "TWD"
  grain: "day"
  day_boundary: "local" | "utc"
  timezone: string                 // 例 "Asia/Taipei"
  range: {
    kind: "7d" | "30d" | "90d" | "all" | "custom"
    start: string                  // YYYY-MM-DD inclusive
    end: string                    // YYYY-MM-DD inclusive
  }
  points: { date: string, amount: number }[]   // 空日 amount=0，連續
  price_table_version: string
  cost_nature: "notional_api_estimate" | "vendor_reported" | "mixed"
  computed_at: string
}
```

- **CLI：**
  ```bash
  spend series --grain day --range 30d --currency USD
  spend series --grain day --from 2026-09-01 --to 2026-09-11 --currency USD
  ```
- **UI 綁定：** `trend ← points[]` → `DailyPointVM[]`；**禁止** UI 自加總 `UsageEvent`。
- 面板 range 為 `all` 時：series 可用同 range，或 Expanded 預設先要 `30d` spark（實作時二選一寫進 README；預設跟隨 panel range）。

### 3.3 OPEN-UI-3 — 最後匯入時間 ✅ 已回填

- **決策：** **不**塞進 `SpendSummary`；獨立 `ImportMeta`（勿把 `computed_at` 當匯入時間）。

```text
ImportMeta {
  last_imported_at: string | null
  last_import_source_ids: string[]
  parse_error_count: number
  events_upserted: number
  ledger_path?: string
  schema_version: "import-meta/v0"
}
```

- **CLI：** `import status --json`（或 `import run … --json` 回應附帶）。
- **UI 綁定：** `last_imported_at` 轉使用者時區顯示；`null` →「尚未匯入」。


### 3.4 OPEN-UI-4 — by_model + by_usage_pool ✅ 帳本已回填（定案）

權威：`design/ledger/open-ui-4-by-model-v0.md` + `design/ledger/by-model-v0.md`（v0.2）＋ **AC v1.3a**。

| UI | 帳本來源 |
|----|----------|
| 按 model 列 | `SpendSummary.by_model` 或 `spend by-model --currency USD` |
| Cursor 雙池 | `SpendSummary.by_usage_pool` 或 `spend by-pool` |
| Other Models 文案 | `pool=other_models`（**池名**，不是 model） |
| notional 免責 | `pricing_mode` + `disclaimer`（必顯） |

**UI 硬規則（AC-F13′）：**
1. `by_model[].model` 只顯示具體 id；**禁止**字面 `Other`／`Other model`／主路徑 `cursor:other` 假列。
2. Expanded：雙池標題「Cursor Models」／「Other Models」可並陳；池下再列該 `usage_pool` 的 model。
3. `pool=unknown` →「池未知／待 API」＋可附 PARTIAL，不得假造 Other 字串 model。
4. 歸組：`by_usage_pool` 提供池合計；model 子列 = filter `by_model` where `usage_pool==pool`（展示歸組，不重算金額）。
5. 歷史若仍見 `cursor:other`：映射層丟棄／不當產品列（遷移責任在帳本）。

**CLI（mock 同名）：**
```bash
spend total --currency USD --range <kind>
spend by-model --currency USD
spend by-pool --currency USD
```

---

## 4. 查詢／重新整理時序

對齊帳本建議並行呼叫：

```text
UI range_change | refresh_click
    →（可選）import run …
    → 並行：
         spend total --currency USD --range <panel_range>   → range_total / by_agent / by_model / by_usage_pool
         spend total --currency USD --range today           → today_total
         spend by-model --currency USD                      →（若 total 未帶齊可另呼）
         spend by-pool --currency USD                       → by_usage_pool
         spend series --grain day --range <panel|30d> --currency USD → trend
         import status --json                               → last_imported_at
    → 映射 MiniPanelVM → 重繪（舊數字保留至新 payload）
```

**一致性：** 同一次 refresh 的 `range_total` 與 `by_agent` 必須來自**同一** `SpendSummary`。today／series 若 `price_table_version` 不同 → 警告「價表版本不一致，請再整理」。

---

## 5. 數字格式與對齊 CLI

| 規則 | 值 |
|------|-----|
| 主幣別 | USD |
| 顯示 | `$` + 兩位小數（不二次捨入；跟帳本輸出） |
| 容差 | ≤ $0.01 abs **或** ≤ 0.1% rel（AC-F6） |
| 對核 | 面板 vs `spend total --currency USD --range …`；trend Σ vs 同 range total |
| TWD | 僅 `currency=TWD` 且具 `fx_snapshot`；標 pair／rate／as_of／source |

**禁止：** UI 硬編碼單價、匯率、或臆造金額。

---

## 6. Fixture 對核（儀表自測）

1. Mock `SpendSummary`（含 `--range today`）→ `today_total`／`range_total`／`by_agent`。  
2. Mock `DailySpendSeries` → trend 連續點（含 0 日）。  
3. Mock `ImportMeta` → last import 文案；`computed_at` 不得誤顯為匯入時間。  
4. Cursor `status=partial` → 必現警告。  

未有正式 fixture 前：sample JSON 標 **example data**。

---

## 7. 與角色交界

| 角色 | 本契約期待 |
|------|------------|
| 帳本 | LEDGER-QUERY v0 已凍結；CLI／fixture 落地中 |
| 橋樑 | 殼、托盤、選單列、權限、IPC 呼叫上列 CLI |
| 儀表 | IA＋本綁定；可先 schema mock；不計價 |
| 驗收官 | AC-F7／F7′；數字對 CLI |

---

## 8. 交接摘要

| 項目 | 狀態 |
|------|------|
| IA／狀態線框 | **已交** `mini-panel-ia-v0.md` |
| SpendSummary 綁定 | **本文件 v0.1** |
| OPEN-UI-1 today | ✅ 已回填（LEDGER-QUERY v0） |
| OPEN-UI-2 daily series | ✅ 已回填 |
| OPEN-UI-3 last import | ✅ 已回填 |
| OPEN-UI-4 by_model／by_usage_pool | ✅ 帳本已回填；本契約 **v0.3 定案** |
| 實作 UI 殼 | mock 接雙池＋具體 model＋disclaimer |

---

**UI-BIND v0.3 — 儀表定案（AC v1.3a／OPEN-UI-4）**

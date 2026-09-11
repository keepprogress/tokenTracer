# 帳本查詢延伸 — OPEN-UI-1…3 回填（v0）

| 欄位 | 值 |
|------|-----|
| 版本 | **LEDGER-QUERY v0** |
| 日期 | 2026-09-11 |
| 作者 | 帳本 |
| 對照 | `design/ui/spend-summary-binding-v0.md` §3；AC v1.1 §5.2／F6／F7 |
| 狀態 | **帳本提案已凍結給儀表消費**（CLI 名可微調，語意不改） |

原則：UI 不計價；下列皆由帳本純函數／CLI 產出，容差同 AC-F6（≤ $0.01 abs）。

---

## OPEN-UI-1 — 今日花費

### 決策
採用 **選項 A**：擴充 `SpendSummary.range.kind`，並提供等價 CLI。

```text
range.kind: "today" | "all" | "7d" | "30d" | "90d"
```

### 日界規則
| 項 | 值 |
|----|-----|
| 預設 | **local calendar day**（使用者時區；tokenTracer 預設 `Asia/Taipei`／系統本地，可配置） |
| 可配置 | `--day-boundary local\|utc`（環境／設定檔同名） |
| `today` 視窗 | `[day_start, day_start+1d)` 半開；`day_start` = 該時區當日 00:00:00 |
| 事件過濾 | `UsageEvent.ts` 解析為 UTC 瞬間後，落入上窗才計入 |
| `range.start` / `range.end` | ISO-8601 UTC 瞬間（輸出必填，便於對核） |

### CLI
```bash
spend total --currency USD --range today
# 等價別名（可選）：
spend today --currency USD
```

回傳仍為完整 `SpendSummary`：
- `range.kind = "today"`
- `total` = 今日 notional／vendor 加總（同價表）
- `by_agent` 僅含今日事件
- `cost_nature` / `price_table_version` / `computed_at` 同既有契約

Collapsed 面板：`today_total ← SpendSummary.total`（本次 today 查詢）。

---

## OPEN-UI-2 — 按日趨勢系列

### Schema
```text
DailySpendSeries {
  currency: "USD" | "TWD"
  grain: "day"                    // v0 僅 day；預留 week/month
  day_boundary: "local" | "utc"   // 同 OPEN-UI-1 預設 local
  timezone: string                // IANA，例 "Asia/Taipei"；utc 時為 "UTC"
  range: {
    kind: "7d" | "30d" | "90d" | "all" | "custom"
    start: string                 // inclusive calendar date YYYY-MM-DD（boundary 時區）
    end: string                   // inclusive calendar date YYYY-MM-DD
  }
  points: {
    date: string                  // YYYY-MM-DD
    amount: number                // 該日加總；無事件日仍輸出 0（連續序列）
  }[]
  price_table_version: string
  cost_nature: "notional_api_estimate" | "vendor_reported" | "mixed"
  computed_at: string             // ISO-8601 UTC
}
```

### 規則
- `points` 由帳本依 `UsageEvent.ts` 桶入日後 `price` 加總；**禁止 UI 自加總事件**。
- `7d`／`30d`／`90d`：以查詢當下 local（或 utc）日為 `end`，往回含端共 N 日。
- `all`：自最早事件日 → 今日（可截斷說明於 meta；v0 實作可設合理上限並在輸出標 `truncated: true` 若觸頂——預設不截斷 fixture）。
- 空日必須出現且 `amount: 0`（儀表可直接繪圖）。
- `Σ points.amount` 與同 `range`＋同價表的 `spend total --range …` 之 `total` 容差內一致（`all`／自訂亦然）。

### CLI
```bash
spend series --grain day --range 30d --currency USD
spend series --grain day --from 2026-09-01 --to 2026-09-11 --currency USD
# --from/--to 為 boundary 時區之日曆日；與 --range 互斥（同時給則 --from/--to 勝出，kind=custom）
```

stdout：單一 JSON `DailySpendSeries`。

---

## OPEN-UI-3 — `last_imported_at`

### 決策
**不**塞進 `SpendSummary`（避免與 `computed_at` 混淆）。獨立 `ImportMeta`，由匯入管線寫入帳本狀態檔。

```text
ImportMeta {
  last_imported_at: string | null     // ISO-8601 UTC；從未匯入則 null
  last_import_source_ids: string[]    // DiscoverSource.id 列表（可空）
  parse_error_count: number           // 最近一次匯入
  events_upserted: number             // 最近一次
  ledger_path?: string                // 可選，除錯用
  schema_version: "import-meta/v0"
}
```

### 語意
| 欄位 | 意義 |
|------|------|
| `last_imported_at` | 匯入管線**成功寫入**正規化事件庫的牆鐘時間（UTC） |
| `computed_at` | 某次 `price`／查詢運算時間；**每次查詢都會變**，≠ 匯入時間 |

### CLI
```bash
import status --json
# 或附在：
import run … --json   # 回應含 ImportMeta + 本次計數
```

儀表：`last_imported_at` 轉使用者時區顯示；`null` →「尚未匯入」。

---

## 建議 refresh 並行呼叫（給儀表／橋樑）

```text
spend total --currency USD --range <panel_range>   → range_total / by_agent
spend total --currency USD --range today           → today_total
spend series --grain day --range <panel_range|30d> --currency USD → trend
import status --json                               → last_imported_at
```

同一 refresh 內：`range_total` 與 `by_agent` 必須同一次 `SpendSummary`。若 today／series 的 `price_table_version` 不同 → UI 警告（綁定契約 §4）。

---

## Fixture（帳本交付中）

- `fixtures/ac-v1/`（指揮官路徑）＋對齊 AC-F4 之 `fixtures/ac-v1.1/` 鏡像
- 將含：today／series 小樣本與期望 JSON（與 `cargo test` 對核）

---

**LEDGER-QUERY v0 — 帳本回填 OPEN-UI-1…3**

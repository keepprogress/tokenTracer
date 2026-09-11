# OPEN-BIND-S1…S4 回填 — SpendingAlign（帳本 → 儀表）

| 欄位 | 值 |
|------|-----|
| 版本 | **OPEN-BIND spending-align v0** |
| 日期 | 2026-09-11 |
| 作者 | 帳本 |
| 對照 | UI-BIND v0.4-draft §7；AC v1.4-cursor-official F14–F19 |
| 狀態 | **帳本定稿給儀表消費**（CLI 名可微調） |

---

## OPEN-BIND-S1 — schema／CLI

### `SpendingAlign`（對照層；≠ UsageEvent／≠ notional）

```text
SpendingAlign {
  schema_version: "spending-align/v0"
  cursor_models_pct: number | null      // 0–100
  other_models_pct: number | null
  reset_label: string | null
  on_demand: string | null
  source_mode: "none" | "manual_p1" | "official_admin" | "undocumented_opt_in"
  partial: bool
  partial_reason: string | null
  grok_bot_week_note: string | null     // F18；機器％ HOLD
  computed_at: string                   // ISO-8601 UTC
}
```

### `OfficialAdminReconcile`（F14；**另物件**，不塞進 SpendingAlign ％）

```text
OfficialAdminReconcile {
  schema_version: "official-admin-reconcile/v0"
  events_charged_cents_sum: number
  spend_overall_cents: number
  delta_cents: number
  within_tol: bool                      // abs(delta)/100 ≤ $0.01
  price_period_start?: string
  computed_at: string
}
```

**CLI：**
```bash
spend spending-align --json [--state <path>]     # 讀／印 SpendingAlign
spend spending-align set --json <file> [--state <path>]  # manual_p1 寫入（見 S2）
spend cursor official-reconcile --events … --spend …   # F14 fixture／檔案對帳
```

預設 `--state` = `.token-tracer/spending-align.json`（與 ImportMeta 分離）。缺檔讀取 → `source_mode=none` 佔位（pct 全 `null`、`partial=true`、reason 說明尚無對照資料／F17）；**禁止**從 notional `SpendSummary`／`by_usage_pool.amount` 推算％。`set` 只接受 `source_mode=manual_p1`，校驗 schema 後原子寫入並重蓋 `computed_at`。

Team Admin 對帳結果走 `OfficialAdminReconcile`；**不**把 Σ chargedCents 寫進 `cursor_models_pct`。`local_enrichment` **不得**出現在 `SpendingAlign.source_mode`。憑證契約（`TOKENTRACER_CURSOR_ADMIN_API_KEY`／`TT-C14-*`）與此％對照層無關。

---

## OPEN-BIND-S2 — 個人 PARTIAL（F17）儲存

| 項 | 決策 |
|----|------|
| `manual_p1` 儲存 | ledger 旁路狀態檔：`.token-tracer/spending-align.json`（與 ImportMeta 分離） |
| 截圖核對 | **只留 UI 態／檔案路徑註記**；帳本不存圖、不 OCR |
| 機器％ | 個人路徑預設 **null**；不要求先給機器％（F17／OPEN-C1） |

---

## OPEN-BIND-S3 — `source_mode` 對表

| UI／SpendingAlign.source_mode | 規格來源模式 | 說明 |
|-------------------------------|--------------|------|
| `none` | （無對照資料） | 佔位 |
| `manual_p1` | C-align-target（P1） | 人手／截圖對齊目標 |
| `official_admin` | **`official_admin`** | Team Admin；％通常仍來自 Spending／另源，對帳走 Reconcile |
| `undocumented_opt_in` | **`undocumented_dashboard`** | 僅顯式 opt-in；輸出須標 UNSUPPORTED |
| （無獨立 enum 值） | `local_enrichment` | **不**出現在 SpendingAlign.source_mode；L1 只影響 A 區 PARTIAL（F15） |

---

## OPEN-BIND-S4 — Grok Bot

`grok_bot_week_note: string | null` **維持字串**至 OPEN-C9；不升級結構化週％欄。無機器％。

---

## 硬約束（給儀表測試）

1. **禁止** `f(by_usage_pool.amount)` → pct  
2. **禁止** notional `SpendSummary.total` 驅動 SpendingAlign  
3. A／B 兩路 payload 分開；橋樑只透傳

---

**OPEN-BIND spending-align v0 — 帳本回填**

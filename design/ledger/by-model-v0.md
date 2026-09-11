# by_model + usage_pool — 設計短文（對齊 AC v1.3 ∪ v1.3a）

| 欄位 | 值 |
|------|-----|
| 版本 | **BY-MODEL v0.2**（取代 v0 之 `cursor:other` 主路徑） |
| 日期 | 2026-09-11 |
| 作者 | 帳本 |
| 規格 | 已核准 **AC v1.3** + **AC v1.3a** |
| 證據 | `EC-cursor-other-models-v1`（parent `EC-cursor-v1`） |
| 狀態 | **設計凍結 → 實作** |

---

## 1. 鎖定語意

| 概念 | 是 | 不是 |
|------|----|------|
| `by_model[].model` | 具體 model id（`grok-4.6`、`composer-1.5`、`claude-4.6-opus-high-thinking`…） | 字面 `Other`／`Other model`／主路徑 `cursor:other` |
| `usage_pool` | Cursor 用量池：`cursor_models`｜`other_models`｜`unknown` | model 名稱 |
| notional | 官方公開 API 價目 → `pricing_mode=notional_api_estimate` | 信用卡／訂閱發票 |

**廢止：** v1.3 主路徑哨兵 `model=cursor:other`。歷史若出現 → 遷移為具體 model（若有）+ `usage_pool=other_models`；model 缺失用 `null` 或 `cursor:unknown`，**pool 仍可區分**。

**禁止：** bubble `tokenCount` 當帳單；字串匹配 `"Other"` 當池權威。

---

## 2. 資料契約

### UsageEvent（增）
```text
model: string | null
usage_pool?: "cursor_models" | "other_models" | "unknown"   // Cursor 宜填
meta.cursor_tier?: number   // 若來自 API；PARTIAL 對照
```

### usage_pool 推導（優先序）
1. `tier`：社群 PARTIAL — `1 → other_models`，`2 → cursor_models`（版本化對照表）
2. 週期 `apiPercentUsed`／`autoPercentUsed` → 僅 `by_usage_pool[].percent_used` 總覽
3. 備援：官方 Cursor Models 名單 vs 其餘；Auto 依**實際 model id** 歸池
4. 本機無訊號 → `unknown`（UI：「池未知／待 API」）

### SpendSummary（v1.3a）
```text
SpendSummary {
  currency, total,                    // total = Σ priced only；unpriced 不計入
  pricing_mode: "notional_api_estimate"
  price_table_version, price_table_source
  disclaimer                          // notional ≠ invoice
  by_agent: { agent, amount, status }[]
  by_model: {
    model: string                     // 具體 id；缺則 "cursor:unknown" 或跳過列策略見下
    agent: AgentId | null
    usage_pool?: "cursor_models"|"other_models"|"unknown"
    amount: number | null             // unpriced → null
    input_tokens: int
    output_tokens: int
    priced: bool
    price_table_row?: string | null
  }[]
  by_usage_pool?: {
    agent: "cursor"
    pool: "cursor_models"|"other_models"|"unknown"
    amount: number | null
    percent_used?: number | null
  }[]
  unpriced_tokens?: { model, input_tokens, output_tokens }[]
  range?, fx_snapshot?, computed_at
}
```

缺 model 的 Cursor 事件：可匯出 `model="cursor:unknown"` **僅作 unknown 標籤**（非 Other 池名）；`usage_pool` 仍獨立。

---

## 3. CLI（OPEN-UI-4／AC-F11）

```bash
spend total --currency USD --events …           # JSON 含 by_model、pricing_mode、disclaimer
spend by-model --currency USD --events …        # 同摘要或精簡 by_model 表；必含 usage_pool 欄
spend by-pool --currency USD --events …         # 輸出 by_usage_pool（Cursor）
```

不變量：`Σ by_model.amount where priced ≈ total`（≤$0.01）；`Σ by_agent` 同。

---

## 4. Fixture

- `fixtures/ac-v1.3/by-model.json` — ≥2 具名 model + ≥1 `usage_pool=other_models`（**無** `model=cursor:other`）
- `fixtures/ac-v1.3a/cursor-pools.json` — tier／pool 雙池樣本
- 更新既有 `fixtures/ac-v1/` 期望含 `pricing_mode`／`by_model`

---

## 5. 實作步驟（crate）

1. `UsageEvent.usage_pool` + enum  
2. `ModelSpend`／`UsagePoolSpend` + `SpendSummary` 欄位（`pricing_mode`、`disclaimer`；`cost_nature` 可對齊或改名為 pricing_mode）  
3. `price()` 聚合 by_model（含 tokens）與 by_usage_pool  
4. CLI subcommands `by-model`／`by-pool`  
5. Fixture + tests；禁 `"Other"` 字串匹配測試  
6. Cursor stub：保留具體 model；無 tier → pool=unknown  

---

## 6. 給儀表（OPEN-UI-4）

可定案 UI-BIND：
- `by_model` ← `spend by-model`／`SpendSummary.by_model`（具體 id + optional `usage_pool`）
- 雙池 ← `SpendSummary.by_usage_pool`／`spend by-pool`
- 文案：「Cursor Models」／「Other Models」對 `cursor_models`／`other_models`；**不是** model 名叫 Other
- `disclaimer`／`pricing_mode` 必顯

---

**BY-MODEL v0.2 — 對齊 AC v1.3a**

---

## 附錄：命名隔離（指揮官 2026-09-11）

| 符號 | 層級 | 意義 |
|------|------|------|
| `usage_pool=other_models` | **池** | Cursor「Other Models」方案／tier 池 |
| `usage_pool=cursor_models` | **池** | Cursor「Cursor Models」池 |
| `model` 缺欄 | **model** | 用 `null` 或哨兵 **`cursor:unknown`**（AC v1.3a）；**不要**用 `__other__` 以免與 other_models 池混淆 |
| 字面 `"Other"` | 禁止 | 不作 model、不作池權威 |

UI／JSON 必須同時能分開：某列 `model=grok-4.6` + `usage_pool=other_models` 完全合法且語意清晰。

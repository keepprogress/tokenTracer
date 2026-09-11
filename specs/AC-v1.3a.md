> **已核准：AC v1.3a**  
> - 核准日：2026-09-11  
> - 核准基準：指揮官代核  
> - 生效：廢止主路徑 `model=cursor:other`；改 `usage_pool`（`other_models`／`cursor_models`）  
> - 同一內容未變前勿再退回待核

# AC v1.3a — Cursor「Other Models」＝usage pool（修正 AC-F13／哨兵）

| 欄位 | 值 |
|------|-----|
| 版本 | **AC v1.3a** |
| 狀態 | **已核准** |
| 類型 | 語意修正小補丁（繼承已核准 **AC v1.3**；不重開 by_model／價目大條款） |
| 日期 | 2026-09-11 |
| 作者 | 規格 |
| 依據 | `evidence/EC-cursor-other-models-v1.md`（parent `EC-cursor-v1`） |
| 觸發 | 指揮官：Other Models 是 usage pool／tier，不是 model 字串 |

---

## 0. Diff 要點（相對已核准 AC v1.3）

1. **撤銷／修正**將 Cursor「Other」建模成 `model = cursor:other` 的主路徑。  
2. 新增維度 **`usage_pool`**（或等價欄位）：至少 `cursor_models`｜`other_models`｜`unknown`。  
3. **`by_model` 仍只用具體 model id**（如 `modelIntent`／`modelConfig.modelName`），不得用字面 `"Other"`／`"Other model"`。  
4. UI／CLI 須能同時展示：按 model 列 **與** Cursor 雙池（Cursor Models vs Other Models）區分。  
5. 重寫 **AC-F13 → AC-F13′**；AC-F11／F12 保留，但 fixture 語意對齊本補丁。

---

## 1. 語意（鎖定）

| 概念 | 是什麼 | 不是什麼 |
|------|--------|----------|
| **Other Models** | Cursor 帳單／dashboard 的 **第二用量池**（API／tier 訊號） | 名叫 `Other` 的 model id |
| **Cursor Models** | 第一方用量池（Grok／Composer 等） | agent 名稱本身 |
| **model id** | 每筆用量的具體模型（`modelIntent` 等） | 池名 |

### 推導 `usage_pool`（優先序，寫進帳本）
1. **優先**：來源若有 `tier` → 對照表（探針：社群 **`tier==1` → `other_models`，`tier==2` → `cursor_models`；標 PARTIAL 待官方 enum）寫入 `usage_pool`。  
2. **次佳**：週期級 `apiPercentUsed`／`autoPercentUsed` 僅作 **池總覽**，不代替 per-event model。  
3. **備援**：官方「Cursor Models」名單 vs 其餘 → `other_models`；Auto 路由依 **實際 model id** 歸池。  
4. **禁止**：`model == "Other"`／`"Other model"`／大小寫變體字串匹配作為池或 model 權威。

---

## 2. 契約修正

### 2.1 UsageEvent
```text
UsageEvent {
  ...
  model: string | null     // 具體模型 id；禁止寫入字面 "Other" 充當池
  usage_pool?: "cursor_models" | "other_models" | "unknown"  // Cursor 事件宜填
  ...
}
```
- 廢止 v1.3 主路徑哨兵 **`cursor:other` 作為 model id**。  
- 若歷史 fixture 曾用 `cursor:other`：遷移為「具體 model（若有）+ `usage_pool=other_models`」；僅當 model 缺失時 `model=null` 或 `cursor:unknown`，**pool 仍要可區分**。

### 2.2 SpendSummary 增修
```text
SpendSummary {
  ...
  by_model: { model, agent?, usage_pool?, amount, input_tokens, output_tokens, priced }[]
  by_usage_pool?: {   // Cursor 必備可測；其他 agent 可省略
    agent: "cursor"
    pool: "cursor_models" | "other_models" | "unknown"
    amount: number | null
    percent_used?: number | null  // 若有 apiPercentUsed/autoPercentUsed
  }[]
  ...
}
```

---

## 3. AC-F13′（取代 v1.3 AC-F13）

### AC-F13′ Cursor 雙池可區分（資料＋UI）
- **Given** fixture／樣本含 Cursor 雙池訊號（依 `EC-cursor-other-models-v1`：`tier` 與／或 percent 欄；且 `modelIntent` 為具體 id）  
- **When** 匯入後執行 `spend by-model` 與 `spend by-pool`（或 `SpendSummary.by_usage_pool`；名稱可調，須入契約）並打開迷你面板展開態  
- **Then**  
  1. **by_model** 列的 `model` 皆為具體 id（例：`composer-1.5`、`claude-4.6-opus-high-thinking`），**不得**出現字面 `Other`／`Other model` 充當 model  
  2. 存在可區分的 **`usage_pool=other_models`**（與 `cursor_models`）維度——在 `by_usage_pool` 與／或 by_model 列的 `usage_pool` 欄  
  3. UI 展開態顯示 **「Other Models」池**（或等價文案）與 **「Cursor Models」池**，兩者可並陳；池下可再列具體 model  
  4. **禁止**僅靠匹配字面 `"Other"` 實作上述區分  
  5. 本機 DB 無池標籤時：允許 pool=`unknown` 並在 UI 標「池未知／待 API」；**不得**假造 Other 字串 model  

### AC-F11／F12 對齊註記
- AC-F11 fixture 改為：≥2 具名 model，且至少一筆帶 `usage_pool=other_models`（非 `model=cursor:other`）。  
- AC-F12 notional／免責不變。

---

## 4. 非目標（追加）

N18. 不把 `tier` 社群對照當成已官方 enum 文件（標 PARTIAL；對照表可版本化）。  
N19. 不以週期 `apiPercentUsed` 反推每筆 model 花費（池總覽 ≠ per-model 權威）。

---

## 5. 驗收指令

```bash
verify-ac-v1.3a --fixture fixtures/ac-v1.3a/cursor-pools.json
# 期望：by_model 無字面 Other；存在 usage_pool=other_models；
# UI／JSON 可區分 Cursor Models vs Other Models 池
```

Checklist：
- [ ] AC-F13′ 通過  
- [ ] 無字面 `"Other"` model 匹配實作（code review／測試）  
- [ ] `EC-cursor-other-models-v1` 已引用於證據對照  
- [ ] 規格標籤含已核准 AC v1.3a  

---

## 6. 交接

| 項目 | 狀態 |
|------|------|
| 已核准 | AC v1.3、**AC v1.3a** |
| 本補丁 | **AC v1.3a — 已核准** |
| 影響 | 帳本事件／SpendSummary、儀表 by_pool UI、Cursor 匯入（橋樑／帳本） |
| 證據 | `EC-cursor-other-models-v1` |

---

**AC v1.3a — 已核准**

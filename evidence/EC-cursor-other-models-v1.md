# Evidence Appendix: Cursor 「Other Models」如何出現

| Field | Value |
|-------|-------|
| Evidence ID | `EC-cursor-other-models-v1` |
| Parent | `EC-cursor-v1` |
| Date | 2026-09-11 (Asia/Taipei) |
| Goal | 讓 UI／帳本穩定區分 **Cursor Models** vs **Other Models** 兩池 |
| Overall | **CONFIRMED**（官方語意 + API 欄位）；**PARTIAL**（`tier` 數值對照依社群實測）；本機 DB **無**「Other model」字串 |

---

## 1. 結論（先讀）

「Other Models」**不是**本機 / CSV 裡某個模型名叫做 `Other` / `Other model`。

它是 Cursor 帳單上的 **第二個用量池（usage pool）**：

| UI 文案（官方） | 語意 | 主要 API 訊號 |
|-----------------|------|----------------|
| **Cursor Models** | 第一方：Grok / Composer 等「generous」池 | `planUsage.autoPercentUsed`；聚合列 `tier == 2`（社群） |
| **Other Models** | 第三方模型，按該模型 API 價扣池 | `planUsage.apiPercentUsed`；聚合列 `tier == 1`（社群） |

每筆用量仍帶 **具體模型 id**（`modelIntent` / CSV `Model` / 本機 `modelConfig.modelName`）。池別靠 **`tier` 或官方模型名單／百分比欄位** 推，**不要**用字串相等 `"Other"`。

---

## 2. 官方定義 — CONFIRMED

來源：[cursor.com/docs/models-and-pricing](https://cursor.com/docs/models-and-pricing)

- 兩池：`Cursor Models`（Grok 4.6/4.5、Composer 2.5 等）與 `Other Models`（選特定第三方模型時，依 API 價扣池）。
- 兩池都顯示在 editor settings 與 usage dashboard。
- Auto 依路由到的實際模型計價；第三方另可能有 Cursor Token Rate（Teams/Enterprise）。

---

## 3. Dashboard API — CONFIRMED（欄位）／PARTIAL（tier 語意）

### 3.1 `GetCurrentPeriodUsage`（池百分比／文案）

Forum 實例（[cursor.com forum](https://forum.cursor.com/t/update-the-docs-and-dashboard-to-tell-us-exactly-whatbwe-ae-paying-for/152538/6)）＋ OpenUsage／CursorUsage：

| 欄位 | 對應池 | 可信度 |
|------|--------|--------|
| `individualUsage.plan.autoPercentUsed` | **Cursor Models** 池已用 % | **CONFIRMED**（OpenUsage + CursorUsage README） |
| `individualUsage.plan.apiPercentUsed` | **Other Models** 池已用 % | **CONFIRMED** |
| `individualUsage.plan.totalPercentUsed` | 合併 % | CONFIRMED |
| `autoModelSelectedDisplayMessage` | Cursor Models 文案（例：「You've used N% of your included total usage」） | CONFIRMED（forum sample） |
| `namedModelSelectedDisplayMessage` | Other／API 池文案（例：「You've used N% of your included API usage」） | CONFIRMED（forum sample） |
| `planUsage.totalSpend` / `includedSpend` / `limit`（cents） | 週期花費 | CONFIRMED（OpenUsage） |

### 3.2 `GetAggregatedUsageEvents`（逐模型 + tier）

樣本（forum，同帖）：

```json
{
  "aggregations": [
    {
      "modelIntent": "composer-1.5",
      "inputTokens": "3200018",
      "outputTokens": "497699",
      "cacheReadTokens": "90338912",
      "totalCents": 2782.59541,
      "tier": 2
    },
    {
      "modelIntent": "claude-4.6-opus-high-thinking",
      "inputTokens": "66",
      "outputTokens": "6074",
      "cacheWriteTokens": "120467",
      "cacheReadTokens": "1952379",
      "totalCents": 188.128825,
      "tier": 1
    }
  ],
  "totalCostCents": 2970.7242349999997
}
```

| 欄位 | 含義 | 可信度 |
|------|------|--------|
| `aggregations[].modelIntent` | 模型 id（**不是** `"Other"`） | **CONFIRMED** |
| `aggregations[].tier` | 池別：社群實測 **`2` = Cursor Models，`1` = Other Models** | **PARTIAL**（CursorUsage：sum(tier)≈`totalSpend`；非官方 enum 文件） |
| token / `totalCents` | 字串 token + 美分 | CONFIRMED |

**穩定區分建議（帳本／UI）：**

1. **優先**：`tier`（若 API 有給）→ `usage_pool = other_models | cursor_models`。
2. **次佳**：`GetCurrentPeriodUsage` 的 `apiPercentUsed` / `autoPercentUsed` 做池總覽（無 per-event）。
3. **備援**：用官方「Cursor Models」名單（Grok／Composer 系）vs 其餘 → Other；Auto 路由到第三方時依 **實際 `modelIntent`** 歸 Other（官方：第三方扣 Other／Token Rate）。
4. **禁止**：`model == "Other"` / `"Other model"` 字串匹配（本機與公開樣本皆無此 model id）。

---

## 4. CSV 匯出 — PARTIAL

EC-cursor-v1／agent-walker：`Model` 欄為具體模型名；`Cost` / token 欄。

| 問題 | 狀態 |
|------|------|
| CSV 是否有 `Other Models` 欄或 `Model=Other` | **UNKNOWN**（未抓到含該字樣之公開 CSV 列） |
| 實務 | 用 `Model` 字串 + 官方第一方名單／價目表推池；或優先走 Aggregated API 的 `tier` |

---

## 5. 本機 `state.vscdb` — CONFIRMED（無「Other」字串）

NB-T3261 只讀抽樣（既有 EC-cursor-v1 probe）：

| 觀察 | 結果 | 可信度 |
|------|------|--------|
| `composerData.modelConfig.modelName` | 例：`grok-4.6`、`cursor-grok-4.6-xhigh-fast`、`default` | CONFIRMED |
| 值為 `"Other"` / `"Other model"` | **未見** | CONFIRMED（本機） |
| ItemTable 存雙池百分比 | **未見**穩定 key（用量在 API） | PARTIAL |
| `usageData` on composer | 社群文件有 `usageData.default.{costInCents,amount}` — **非**池標籤 | PARTIAL |

本機適合作 **model 標籤 enrichment**，**不適合作 Other Models 池權威來源**。

---

## 6. 對 UsageEvent / UI 建議欄位

```text
UsageEvent.meta.cursor_usage_pool?: "cursor_models" | "other_models" | "unknown"
UsageEvent.model: string   // modelIntent / Model / modelConfig.modelName
```

| UI 顯示 | 資料來源 |
|---------|----------|
| 雙池 %（Cursor / Other） | `autoPercentUsed` / `apiPercentUsed` |
| 雙池 $ | Σ `aggregations` where `tier==2` / `tier==1`（PARTIAL 對照） |
| 分模型明細 | `modelIntent` + tokens + `totalCents` |
| PARTIAL 警告 | 僅本機、無 `tier`、或 `tier` 語意未再驗證時 |

---

## 7. BLOCK / UNKNOWN

| ID | 內容 |
|----|------|
| BLOCK | 把本機 `modelName`/`bubble` 字串 `"Other"` 當池（不存在／不可靠） |
| BLOCK | 無 `tier` 時硬編碼臆測所有 Auto = Cursor Models（官方：Auto 可路由第三方 → Other） |
| UNKNOWN | `tier` 是否永遠只有 1/2；未來是否改名／加池 |
| UNKNOWN | CSV 是否新增池欄 |
| UNKNOWN | Admin `filtered-usage-events` 是否含等價 `tier`（待樣本件） |

---

## Sources

- https://cursor.com/docs/models-and-pricing  
- https://openusage.sh/docs/providers/cursor/  
- https://forum.cursor.com/t/update-the-docs-and-dashboard-to-tell-us-exactly-whatbwe-ae-paying-for/152538/6  
- https://github.com/litianyi-007/cusor-usage (README + RESEARCH)  
- https://github.com/shadeov/cursor-costs-raycast `.cursor/rules/cursor-api.mdc`  
- Parent: `EC-cursor-v1` + NB-T3261 local probe  


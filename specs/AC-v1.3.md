> **已核准：AC v1.3**  
> - 核准日：2026-09-11  
> - 核准基準：指揮官代核（人類明示：按 model／官方 API notional／Cursor Other 可區分）  
> - v1.2c 仍留給實機 macOS 升格  
> - 同一內容未變前勿再退回待核
> - 語意修正見 **[AC v1.3a](./AC-v1.3a.md)**（Other Models＝usage pool；**已核准**）  


# AC v1.3 — 按 model 彙總／官方價目 notional／Cursor Other 可區分

| 欄位 | 值 |
|------|-----|
| 版本 | **AC v1.3** |
| 狀態 | **已核准** |
| 類型 | 功能補丁（繼承已核准 AC v1.1∪v1.2∪v1.1a∪v1.2a∪v1.2b；不重開平台矩陣） |
| 日期 | 2026-09-11 |
| 作者 | 規格 |
| 觸發 | 人類經指揮官：by_model、官方 API 價目 notional、Cursor Other 可區分 |

---

## 0. Diff 要點（相對已核准棧）

1. 花費須可 **按 model** 彙總／查詢（既有 by_agent 保留）。  
2. 計價權威鎖定：各模型 **官方公開 token API 價目** → `notional_api_estimate`；訂閱用戶同樣估；UI／CLI **必須**標 notional／非帳單。  
3. Cursor 來源的 **Other model**（及非具名／聚合桶）在資料與 UI **可區分**，不得只併進含糊「Cursor」一桶結束。  
4. 新增可測 AC-F11…F13；擴充 `SpendSummary`／CLI；明確非目標（≠ 信用卡帳單）。

---

## 1. 增修目標

G9. 使用者能按 **model** 查看用量與 notional 花費（含區間／與 by_agent 交叉）。  
G10. 所有 notional 花費可追溯到 **價表版本** + 官方公開 API 價目來源。  
G11. Cursor 的 Other／未知模型桶在帳本事件、查詢結果、迷你面板上皆可辨識。

---

## 2. 非目標（本補丁特別強調）

N13. **不宣稱** notional 總額等於信用卡／訂閱帳單／發票實付。  
N14. 不實作「從卡片帳單／銀行 CSV 對帳」。  
N15. 不以 Cursor UI bubble `tokenCount` 當 billed usage（繼承 v1.1a）。  
N16. 不要求官方價目未公開的模型「發明」單價；無價時標 `unpriced`，金額可為 null／單列，不得靜默塞進別的 model。  
N17. 本補丁不改平台安裝／矩陣必做名單（仍為 Cursor／Claude Code／Codex）。

---

## 3. 契約增修

### 3.1 UsageEvent（增修約束）
- `model: string | null` **必須保留來源語意**：  
  - 具名模型 → 正規化後的穩定 id（帳本定義對照表，如 `gpt-4.1`）。  
  - Cursor（或他源）的聚合／非具名桶 → **不得**寫成與「Cursor 產品」混淆的空桶；須使用約定哨兵，至少包含：  
    - `cursor:other` — 來源標為 Other model／Other  
    - `cursor:unknown` — 完全無模型欄  
    - （可選）`cursor:auto` 等若證據卡另有穩定桶名，須進對照表，不得折成 `cursor:other` 以外的「消失」  
- `agent` 仍為 `cursor`／`claude_code`／`codex`…；**Other 是 model 維度，不是把 agent 改名。**

### 3.2 SpendSummary（擴充）
```text
SpendSummary {
  currency: "USD" | "TWD"
  total: number | null          // 僅加總已定價列；unpriced 不計入或分列見下
  pricing_mode: "notional_api_estimate"
  price_table_version: string
  price_table_source: string    // 官方公開價目出處說明／URL 清單摘要
  by_agent: { agent, amount, status }[]
  by_model: {
    model: string               // 含 cursor:other 等哨兵
    agent: AgentId | null       // 可選；便於 UI 分組
    amount: number | null
    input_tokens: int
    output_tokens: int
    priced: bool                // false = unpriced
  }[]
  unpriced_tokens?: { model: string, input_tokens: int, output_tokens: int }[]
  fx_snapshot?: ...
  range?: ...
  computed_at: string
  disclaimer: string            // 固定或可本地化；須含 notional／非帳單語意
}
```

### 3.3 價表
- 權威：各模型 **官方公開 token API 價目**（input／output／cache 若公開則分列）。  
- 交付物：`price_table_version` + 機器可讀價表 + 來源註記。  
- 訂閱用戶：**同一** notional_api_estimate，不另造「訂閱均攤」公式（除非未來新 AC）。

---

## 4. 可測 Acceptance Criteria

### AC-F11 按 model 彙總（CLI）
- **Given** fixture `fixtures/ac-v1.3/by-model.json`（含 ≥2 具名 model + 至少一筆 `cursor:other`）  
- **When** 執行 `spend by-model --currency USD`（名稱可調，須入 CLI 契約對照）  
- **Then**  
  - 輸出每個 model 列的 tokens 與 notional amount（或 `unpriced`）  
  - 各 **priced** model 的 amount 之和 = `SpendSummary.total`（AC-F6 容差）  
  - `cursor:other` **獨立成列**，不得只出現在 by_agent 的 Cursor 合計中而 by_model 消失  

### AC-F12 官方價目 notional 標註
- **Given** 任意 `spend total`／`spend by-model`／`spend by-agent` 成功輸出  
- **Then**  
  - 含 `pricing_mode=notional_api_estimate`（或等價欄位）與 `price_table_version`  
  - stdout／JSON／迷你面板可見 **notional** 與 **非帳單／≠ invoice** 類免責（`disclaimer` 或 UI 固定文案）  
  - 更換價表版本後，同 fixture 重算結果變更可重現（報告記錄舊／新 version）  

### AC-F13 Cursor Other（資料＋UI）可區分
- **Given** 匯入含 Cursor Other／非具名桶之樣本（證據路徑依 `EC-cursor-v1`；fixture 可合成）  
- **When** 匯入後查 by_model 與打開迷你面板展開態  
- **Then**  
  - 帳本事件 `model` ∈ {`cursor:other`／對照表登記之桶}，**禁止**寫 `null` 後在 UI 顯示成普通「Cursor」而無子列  
  - UI 展開態在 Cursor 分組下（或扁平 by_model 列表中）顯示可讀標籤，例如 **「Other model」**，與具名模型並列  
  - by_agent 的 Cursor 合計可包含 Other 的 notional，但 **不得**因此省略 by_model 的 Other 列  

### 繼承回歸
- AC-F4…F6、F7′／F9、v1.1a 禁止 bubble token 當帳單 — 仍須通過；本補丁不得放寬。

---

## 5. 驗收指令大綱

```bash
verify-ac-v1.3 --fixture fixtures/ac-v1.3/by-model.json
# 期望：exit 0；by-model 含 cursor:other；disclaimer／price_table_version 存在；
# priced 列加總 = total（容差內）
```

Checklist 增量：
- [ ] AC-F11 by_model CLI  
- [ ] AC-F12 notional／價表版本／免責  
- [ ] AC-F13 Cursor Other 資料＋UI  
- [ ] 未宣稱等於信用卡帳單（文案抽樣）  
- [ ] 規格版本標籤含已核准 AC v1.3  

---

## 6. OPEN（不猜）

| ID | 問題 | 建議默認（代核可採） |
|----|------|----------------------|
| OPEN-P3 | 官方價目彙整表維護頻率／自動化？ | 手動版本釘選 + README 更新流程；CI 只校驗 fixture |
| OPEN-P4 | 無公開 API 價之模型？ | `unpriced` 列；不計入 total |
| OPEN-P5 | Cursor 除 Other 外的桶名清單？ | 以 `EC-cursor-v1` 為準建對照表；未知 → `cursor:unknown` |

---

## 7. 交接摘要

| 項目 | 狀態 |
|------|------|
| 已核准棧 | v1.1、v1.2、v1.1a、v1.2a、v1.2b、**v1.3** |
| 本補丁 | **AC v1.3 — 已核准** |
| 影響角色 | 帳本（by_model／價表）、儀表（UI 標籤／disclaimer）、橋樑（匯入保留 model 哨兵） |
| 不可自稱 PASS | 規格不自 PASS；驗收官對已核版本放行 |

---

**AC v1.3 — 已核准**

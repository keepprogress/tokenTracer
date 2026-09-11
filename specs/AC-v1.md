> **已取代：** 本文件被 **[AC v1.1](./AC-v1.1.md)** 取代（2026-09-11）。請勿再依 AC v1 核准或實作。狀態：歷史稿。

# AC v1 — Token Spend Tracker（開源 token 用量／花費統計）

| 欄位 | 值 |
|------|-----|
| 版本 | **AC v1** |
| 狀態 | **已取代（見 AC v1.1）** |
| 日期 | 2026-09-11 |
| 作者 | 規格 |
| 產品暫名 | Token Spend Tracker（OPEN：正式名稱） |

---

## 1. 目標（Goals）

G1. 在 **Windows** 可安裝並啟動，能統計本機 + **WSL2** 上 coding agent 的 token／用量／花費。  
G2. 支援歷史彙總：**歷史總花費**、可依日／區間查詢。  
G3. 花費以 **USD** 為主計價單位；需標明單價與匯率來源（若有換算）。  
G4. 至少涵蓋下列 agent（Win 與／或 WSL2，依探針證據地圖）：  
    Codex、Claude Code、Cursor、Grok Build、Cline、OpenCode、GitHub Copilot。  
G5. 數字可重現：同一 fixture → 同一計算步驟 → 同一輸出（供帳本／復現／驗收官對照）。  
G6. 開源可釋出（授權見 OPEN）。

---

## 2. 非目標（Non-goals）— AC v1 不做

N1. 不上雲同步、不帳號登入、不收集遙測。  
N2. 不做多使用者／團隊分享儀表。  
N3. 不做自動付費／訂閱管理、不代為呼叫付費 API 扣款。  
N4. 不保證即時串流用量（可接受手動重新整理或短輪詢；OPEN：目標延遲）。  
N5. 不在 AC v1 做美化優先的設計系統；UI 以「可核對數字」為先。  
N6. 不臆測未知資料來源：探針標 `UNKNOWN`／`BLOCK` 的 agent，規格標 **PARTIAL**，不得假裝已支援。

---

## 3. 角色與交接契約

| 角色 | 職責 | 放行權 |
|------|------|--------|
| 探針 | 產出各 agent「資料地圖」證據卡 | 無 |
| 規格 | AC／契約／驗收指令／版本標籤 | 無最終 PASS；僅提版本待人核 |
| 帳本 | 計價核心、模型、fixture、CLI／API | 依已核准 AC；不自 PASS |
| 橋樑 | Win 安裝、WSL 發現／讀取、權限 | 依已核准 AC；不自 PASS |
| 儀表 | 本機儀表板畫面 | 依已核准 AC；不自 PASS |
| 適配驗／復現 | 預驗、環境復現 | 報告；不最終放行 |
| 驗收官 | 對已核准 AC 做最終 PASS／BLOCK | **唯一最終閘門** |
| 人類（getzch） | 核准 AC 版本 | 規格生效條件 |

**交接規則：** 僅當本文件標「已核准：AC vN」後，才可交接實作群開工。未核准 = **HOLD**。

---

## 4. 功能 Acceptance Criteria（可測）

### AC-F1 安裝與啟動（Windows）
- **Given** 乾淨 Windows 環境（版本見 OPEN）  
- **When** 依安裝文件執行安裝  
- **Then** 應用可啟動；失敗時錯誤訊息含可診斷碼／下一步（橋樑契約）

### AC-F2 WSL2 發現
- **Given** 已安裝 WSL2 且至少一個目標 distro  
- **When** 執行「發現」  
- **Then** 列出可讀的 agent 來源（路徑或來源 ID）；無權限時明確報錯，不靜默略過

### AC-F3 用量匯入
- **Given** 探針已確認之來源路徑／格式（證據卡 ID 對應）  
- **When** 執行匯入／重新整理  
- **Then** 寫入帳本正規化事件；無法解析的檔案計入 `parse_error` 計數，不中斷整批（除非 `--fail-fast`）

### AC-F4 歷史總花費（USD）
- **Given** fixture 集 `fixtures/ac-v1/total-spend.json`（帳本交付）  
- **When** 執行 `spend total --currency USD`（指令名可調整，但須寫進 CLI 契約）  
- **Then** 輸出總額與 fixture 期望值一致（容差見 AC-F6）

### AC-F5 分 agent 佔比
- **Given** 同上 fixture  
- **When** 查「各 agent 花費」  
- **Then** 各 agent USD 小計之和 = 總花費（容差內）；未支援 agent 顯示 `unsupported`／`unknown`，不計入已支援總和除非標明

### AC-F6 計價可重現
- 單價表版本號、匯率快照（若用）、計算步驟寫入報告  
- 金額比較：**絕對容差 ≤ $0.01** 或相對 ≤ 0.1%（取較寬者須在驗收報告註明選用哪條）  
- **禁止** 無來源的硬編碼「看起來合理」數字

### AC-F7 儀表板（本機）
畫面至少含（文案可中英，數字格式跟規格）：
1. 今日花費（USD）  
2. 歷史總花費（USD）  
3. 各 agent 佔比  
4. 簡單趨勢（至少按日；OPEN：預設區間）  
5. 資料新鮮度／最後匯入時間  
6. 部分支援警告（若有 PARTIAL agent）

### AC-F8 開源可建置
- README 含：安裝、WSL 權限、如何跑 fixture 驗收  
- 授權檔存在（OPEN： SPDX）  
- CI 或本機一鍵：`test`／`verify-ac-v1` 可跑（至少 fixture 層）

---

## 5. 資料／CLI 契約（草案）

### 5.1 正規化用量事件（帳本）
```text
UsageEvent {
  id: string              // 穩定去重鍵
  agent: AgentId          // codex|claude_code|cursor|grok_build|cline|opencode|github_copilot|other
  source: string          // 證據卡來源 ID
  ts: string              // ISO-8601 UTC
  model: string | null
  input_tokens: int >= 0
  output_tokens: int >= 0
  cache_read_tokens?: int >= 0
  cache_write_tokens?: int >= 0
  raw_cost_usd?: number | null   // 來源若已給成本
  meta?: object
}
```

### 5.2 花費查詢結果
```text
SpendSummary {
  currency: "USD"
  total: number
  by_agent: { agent: AgentId, amount: number, status: "ok"|"partial"|"unsupported" }[]
  price_table_version: string
  fx_snapshot?: { pair: string, rate: number, as_of: string, source: string }
  computed_at: string
}
```

### 5.3 驗收指令（建議，實作可改名但須對照表）
```bash
# 匯入 fixture 並核對總花費
verify-ac-v1 --fixture fixtures/ac-v1/total-spend.json
# 期望：exit 0；stdout 含 PASS 與 total USD
```

---

## 6. Agent 支援矩陣（AC v1）

| Agent | Win | WSL2 | AC v1 狀態 |
|-------|-----|------|------------|
| Codex | TBD | TBD | 待探針證據 → 再鎖 |
| Claude Code | TBD | TBD | 待探針證據 → 再鎖 |
| Cursor | TBD | TBD | 待探針證據 → 再鎖 |
| Grok Build | TBD | TBD | 待探針證據 → 再鎖 |
| Cline | TBD | TBD | 待探針證據 → 再鎖 |
| OpenCode | TBD | TBD | 待探針證據 → 再鎖 |
| GitHub Copilot | TBD | TBD | 待探針證據 → 再鎖 |

**規則：** 矩陣未由探針填證前，實作不得宣稱「已支援」。AC v1 可採 **分批**：先鎖「已有證據」的子集為 v1.0 scope，其餘進 v1.1+（需你核准切法）。

---

## 7. 驗收 Checklist（給驗收官）

- [ ] AC-F1 安裝啟動通過  
- [ ] AC-F2 WSL 發現通過  
- [ ] AC-F3 匯入＋錯誤可診斷  
- [ ] AC-F4 fixture 總花費一致  
- [ ] AC-F5 分 agent 加總一致  
- [ ] AC-F6 單價／匯率來源標明且可重跑  
- [ ] AC-F7 儀表六項可見且數字與 CLI 一致  
- [ ] AC-F8 README／授權／verify 指令  
- [ ] 支援矩陣與實際行為一致（無虛報支援）  
- [ ] 規格版本標籤 = 已核准版本  

**最終 PASS 僅驗收官；規格不自 PASS。**

---

## 8. OPEN／HOLD（缺資訊，不猜）

| ID | 問題 | 為何擋 |
|----|------|--------|
| O1 | 產品正式名稱與 repo（GitHub `keepprogress/…` 或新庫？） | 文件／CI／開源釋出 |
| O2 | AC v1 要「七個全做」還是「先 2–3 個有證據的」？ | 支援矩陣與時程 |
| O3 | 單價來源權威（官方價目靜態表／使用者自備／混合）？ | 帳本契約 |
| O4 | 是否只要 USD，或 UI 另顯示 TWD（匯率來源）？ | AC-F4／F7 |
| O5 | UI 形態：本機 web（localhost）／系統托盤／兩者？ | 儀表＋橋樑 |
| O6 | 目標 Windows 版本與必備 WSL 版本？ | AC-F1 |
| O7 | 開源授權（MIT／Apache-2.0／其他）？ | AC-F8 |
| O8 | 歷史深度預設（全部 vs 最近 N 天）？ | 儀表預設 |

**目前：HOLD 實作交接。** 探針可並行做資料地圖；帳本／橋樑／儀表等 **AC v1 核准**（或你核准「帶 OPEN 的 v1 初稿 + 明確默認」）。

---

## 9. 建議默認（僅在你明示「用默認」後才寫死進核准版）

若你回「用建議默認」，規格會把下列寫進 **AC v1.0 已核准**：
- O2：分批 — v1.0 先做探針已證實的前 3 個 agent，其餘列 PARTIAL 路線圖  
- O3：內建官方公開價目表（可覆寫）＋版本號  
- O4：USD 為準；TWD 可選顯示，匯率來源標明  
- O5：本機 `localhost` web UI  
- O6：Windows 10 22H2+／11；WSL2  
- O7：MIT  
- O8：預設「全部歷史」，UI 可切 7／30／90 天  

O1（名稱／repo）仍須你指定，無法默認。

---

## 交接摘要

| 項目 | 狀態 |
|------|------|
| 已核准版本 | **無** |
| 待你核准 | **AC v1（本文件）** |
| 可並行 | 探針資料地圖 |
| 不可開始 | 帳本／橋樑／儀表產品碼（待核准） |

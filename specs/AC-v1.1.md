> **已核准：AC v1.1**  
> - 核准日：2026-09-11  
> - 核准基準：指揮官依人類鎖定項（tokenTracer、右下角迷你 UI、Rust、O2–O8 默認）代核  
> - OPEN-R1（建庫 `keepprogress/tokenTracer`）可後補，不擋探針  
> - 實作（帳本／橋樑／儀表）仍等探針交前 3 證據後由指揮官再派  
> - macOS 範圍見 **[AC v1.2](./AC-v1.2.md)**（**已核准** 2026-09-11）
> - 矩陣已由 **[AC v1.1a](./AC-v1.1a.md)** 填空並**已核准**（Cursor／Claude Code／Codex；2026-09-11）  

# AC v1.1 — tokenTracer（開源 token 用量／花費統計）

| 欄位 | 值 |
|------|-----|
| 版本 | **AC v1.1** |
| 狀態 | **已核准** |
| 取代 | AC v1（見 `AC-v1.md`） |
| 日期 | 2026-09-11 |
| 作者 | 規格 |
| 產品正式名稱 | **tokenTracer** |
| Repo（暫定） | `keepprogress/tokenTracer`（若帳號／可見性需再確認，標 OPEN-R1） |
| 預設實作技術 | **Rust**；效能為硬需求 |
| 授權 | **MIT** |

---

## 0. 相對 AC v1 的變更摘要（diff 要點）

1. **O1 鎖定**：正式名稱 `tokenTracer`；repo 暫定 `keepprogress/tokenTracer`。  
2. **O5 鎖定**：主 UI = Windows **右下角縮圖／迷你面板**（可展開明細）；可有系統托盤；**不以全頁 web 為主**（localhost 僅次要／除錯）。  
3. **技術鎖定**：Rust 預設；效能硬需求；形式化驗證（關鍵計價純函數）為**可選加強、不擋 v1**；資料模型須方便日後驗證。  
4. **其餘 OPEN 採建議默認**：O2 分批前 3 有證據；O3 官方價目可覆寫；O4 USD 為準、可選 TWD；O6 Win10 22H2+/11 + WSL2；O7 MIT；O8 全部歷史可切 7／30／90。  
5. **AC-F7 重寫**：迷你面板／托盤驗收項（取代全頁儀表為主）。  
6. **新增 AC-F9**：效能可測上限（fixture 匯入／`spend total`）。  
7. 上一輪核准 widget **視為略過**；本版為新待核准稿。

---

## 1. 目標（Goals）

G1. 在 **Windows 10 22H2+／11** 可安裝並啟動；能統計本機 + **WSL2** 上 coding agent 的 token／用量／花費。  
G2. 支援歷史彙總：**歷史總花費**、可依日／區間查詢（預設全部歷史；UI 可切 7／30／90 天）。  
G3. 花費以 **USD** 為主計價；可選顯示 TWD（須標匯率來源與快照）。  
G4. AC v1.1 **scope**：探針已證實的前 **3** 個 agent 為必須支援；其餘 4 個列 PARTIAL／路線圖，不得虛報。目標全集仍為：Codex、Claude Code、Cursor、Grok Build、Cline、OpenCode、GitHub Copilot。  
G5. 數字可重現：同一 fixture → 同一計算步驟 → 同一輸出。  
G6. 開源（MIT）可釋出；預設 **Rust** 實作，效能達 AC-F9。  
G7. 主介面為右下角迷你面板（見 AC-F7）；計價核心資料模型利於日後形式化驗證（見 §5.4）。

---

## 2. 非目標（Non-goals）— AC v1.1 不做

N1. 不上雲同步、不帳號登入、不收集遙測。  
N2. 不做多使用者／團隊分享儀表。  
N3. 不做自動付費／訂閱管理、不代為呼叫付費 API 扣款。  
N4. 不保證即時串流用量（可接受手動重新整理或短輪詢）。  
N5. **不以全頁 localhost web 為主產品 UI**（可保留除錯用次要端點，但不作為 AC-F7 主驗收面）。  
N6. 不臆測未知資料來源：探針標 `UNKNOWN`／`BLOCK` 的 agent 標 **PARTIAL**，不得假裝已支援。  
N7. **不要求** AC v1.1 完成形式化驗證證明；僅要求模型可驗證友好（§5.4）。形式化驗證為可選加強。  
N8. v1.1 不強制一次做滿七個 agent（見 G4 分批）。

---

## 3. 角色與交接契約

| 角色 | 職責 | 放行權 |
|------|------|--------|
| 探針 | 各 agent 資料地圖證據卡；排出「前 3 有證據」清單 | 無 |
| 規格 | AC／契約／驗收指令／版本標籤 | 無最終 PASS；僅提版本待人核 |
| 帳本 | Rust 計價核心、模型、fixture、CLI／API | 依已核准 AC；不自 PASS |
| 橋樑 | Win 安裝、托盤／迷你面板殼、WSL 發現／讀取、權限 | 依已核准 AC；不自 PASS |
| 儀表 | 迷你面板／展開明細 UI（非全頁主 UI） | 依已核准 AC；不自 PASS |
| 適配驗／復現 | 預驗、環境復現 | 報告；不最終放行 |
| 驗收官 | 對已核准 AC 做最終 PASS／BLOCK | **唯一最終閘門** |
| 人類（getzch） | 核准 AC 版本 | 規格生效條件 |

**交接規則：** 僅當本文件標「已核准：AC v1.1」後，才可交接實作群開工產品碼。未核准 = **HOLD**。探針可並行。

---

## 4. 功能 Acceptance Criteria（可測）

### AC-F1 安裝與啟動（Windows）
- **Given** 乾淨 Windows 10 22H2+ 或 Windows 11  
- **When** 依安裝文件執行安裝  
- **Then** tokenTracer 可啟動；失敗時錯誤訊息含可診斷碼／下一步（橋樑契約）

### AC-F2 WSL2 發現
- **Given** 已安裝 WSL2 且至少一個目標 distro  
- **When** 執行「發現」  
- **Then** 列出可讀的 agent 來源（路徑或來源 ID）；無權限時明確報錯，不靜默略過

### AC-F3 用量匯入
- **Given** 探針已確認之來源路徑／格式（證據卡 ID 對應）；且屬於 v1.1 必做前 3 agent 之一  
- **When** 執行匯入／重新整理  
- **Then** 寫入帳本正規化事件；無法解析的檔案計入 `parse_error` 計數，不中斷整批（除非 `--fail-fast`）

### AC-F4 歷史總花費（USD）
- **Given** fixture 集 `fixtures/ac-v1.1/total-spend.json`（帳本交付）  
- **When** 執行 `spend total --currency USD`（指令名可調整，但須寫進 CLI 契約）  
- **Then** 輸出總額與 fixture 期望值一致（容差見 AC-F6）

### AC-F5 分 agent 佔比
- **Given** 同上 fixture  
- **When** 查「各 agent 花費」  
- **Then** 各 agent USD 小計之和 = 總花費（容差內）；未支援／未納入 v1.1 必做集的 agent 顯示 `unsupported`／`partial`／`unknown`，不虛報為已支援

### AC-F6 計價可重現
- 單價表版本號、匯率快照（若用）、計算步驟寫入報告  
- 金額比較：**絕對容差 ≤ $0.01** 或相對 ≤ 0.1%（取用哪條須在驗收報告註明）  
- 單價來源：內建**官方公開價目表**（可覆寫）＋ `price_table_version`  
- **禁止** 無來源的硬編碼「看起來合理」數字

### AC-F7 迷你面板／托盤（主 UI）— *取代 v1 全頁儀表為主*
縮圖／迷你面板（Windows 右下角區域）至少滿足：
1. **收合態**：可見今日花費（USD）與／或歷史總花費摘要（至少一項主數字 + 幣別）  
2. **展開態**：歷史總花費（USD）、各 agent 佔比、簡單按日趨勢、區間切換（全部／7／30／90）、資料新鮮度／最後匯入時間、PARTIAL 警告（若有）  
3. **托盤**（可選但若實作則必驗）：圖示存在；左鍵／右鍵可打開迷你面板或選單（行為寫進 README）  
4. 數字與 CLI／帳本查詢一致（同 AC-F6 容差）  
5. **非目標確認**：全頁 web **不是**主驗收面；若存在 localhost 除錯頁，文件須標「debug only」

### AC-F8 開源可建置
- README 含：安裝、WSL 權限、迷你面板操作、如何跑 fixture 驗收  
- `LICENSE` = MIT  
- 本機一鍵：`test`／`verify-ac-v1.1` 可跑（至少 fixture 層）  
- Repo 目標：`keepprogress/tokenTracer`（見 OPEN-R1）

### AC-F9 效能（硬需求，可測）
- **Given** fixture `fixtures/ac-v1.1/perf-medium.json`（建議規模：≥ 10_000 筆 `UsageEvent`，帳本交付時鎖定實際 N 與機器基線說明）  
- **When** 在文件標明的參考環境執行：  
  - `import`（或等價匯入）該 fixture  
  - 隨後 `spend total --currency USD`  
- **Then**  
  - 匯入牆鐘時間 **≤ 5s**  
  - `spend total` 牆鐘時間 **≤ 500ms**  
  - 峰值額外記憶體（相對進程基線）**≤ 256 MiB**（量測方式寫進驗收報告）  
- 未達標 = **BLOCK**（效能為硬需求，不可僅「已知問題」放行）  
- 參考環境與量測指令必須寫進 README／驗收指令，以便復現官重跑

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
  raw_cost_usd?: number | null
  meta?: object
}
```

### 5.2 花費查詢結果
```text
SpendSummary {
  currency: "USD" | "TWD"
  total: number
  by_agent: { agent: AgentId, amount: number, status: "ok"|"partial"|"unsupported" }[]
  price_table_version: string
  fx_snapshot?: { pair: string, rate: number, as_of: string, source: string }
  range?: { kind: "all"|"7d"|"30d"|"90d", start?: string, end?: string }
  computed_at: string
}
```

### 5.3 驗收指令（建議）
```bash
verify-ac-v1.1 --fixture fixtures/ac-v1.1/total-spend.json
verify-ac-v1.1-perf --fixture fixtures/ac-v1.1/perf-medium.json
# 期望：exit 0；stdout 含 PASS、total USD、以及 perf 各項實測值
```

### 5.4 可驗證友好（形式化驗證非阻擋，但模型要準備好）
- 計價核心應能抽成**純函數**介面，例如：  
  `price(events, price_table, fx?) -> SpendSummary`  
  無隱藏全域可變狀態、無隱式 I/O。  
- I/O（讀檔、WSL、UI）與純計價分離。  
- AC v1.1 **不要求**交付形式化證明；日後可選加強時不得被迫改破資料模型。

---

## 6. Agent 支援矩陣（AC v1.1）

| Agent | Win | WSL2 | AC v1.1 狀態 |
|-------|-----|------|--------------|
| （探針排序第 1 有證據） | TBD | TBD | **必做**（名稱由探針鎖定後填入） |
| （探針排序第 2 有證據） | TBD | TBD | **必做** |
| （探針排序第 3 有證據） | TBD | TBD | **必做** |
| 其餘目標 agent | TBD | TBD | **PARTIAL／路線圖**（不得虛報） |

目標全集：Codex、Claude Code、Cursor、Grok Build、Cline、OpenCode、GitHub Copilot。  
**規則：** 探針交付「前 3 有證據」名單後，規格出 **AC v1.1a** 補丁只填矩陣名稱與證據卡 ID（不改其他條款，除非人類另核）。

---

## 7. 驗收 Checklist（給驗收官）

- [ ] AC-F1 安裝啟動通過  
- [ ] AC-F2 WSL 發現通過  
- [ ] AC-F3 匯入＋錯誤可診斷  
- [ ] AC-F4 fixture 總花費一致  
- [ ] AC-F5 分 agent 加總一致、無虛報支援  
- [ ] AC-F6 單價／匯率來源標明且可重跑  
- [ ] AC-F7 迷你面板收合／展開／（若有）托盤；非全頁主 UI  
- [ ] AC-F8 README／MIT／verify 指令  
- [ ] AC-F9 效能上限達標且可復現量測  
- [ ] 支援矩陣與實際行為一致  
- [ ] 規格版本標籤 = 已核准版本（AC v1.1）  
- [ ] 計價核心符合 §5.4 純函數分離（抽樣 code review／結構檢查即可，非形式化證明）

**最終 PASS 僅驗收官；規格不自 PASS。**

---

## 8. 已鎖定決策 vs 仍 OPEN

### 已鎖定（人類／指揮官轉達）
| ID | 決策 |
|----|------|
| O1 | 名稱 **tokenTracer**；repo 暫定 `keepprogress/tokenTracer` |
| O2 | 分批：v1.1 必做探針已證實前 3；其餘 PARTIAL |
| O3 | 內建官方公開價目表（可覆寫）＋版本號 |
| O4 | USD 為準；TWD 可選顯示，匯率來源標明 |
| O5 | 主 UI = 右下角縮圖／迷你面板＋可托盤；localhost web 僅次要／除錯 |
| O6 | Windows 10 22H2+／11；WSL2 |
| O7 | MIT |
| O8 | 預設全部歷史；UI 可切 7／30／90 天 |
| Tech | Rust 預設；效能硬需求（AC-F9）；形式化驗證可選不擋 v1 |

### 仍 OPEN／HOLD（不猜）
| ID | 問題 | 為何 |
|----|------|------|
| OPEN-R1 | `keepprogress/tokenTracer` 是否已建庫／可見性／是否改名空間 | 釋出與 CI |
| OPEN-P1 | 探針「前 3 有證據」確切名單與證據卡 ID | 鎖死矩陣必做列 |
| OPEN-P2 | `perf-medium` 最終 N（事件數）與官方參考機器規格文案 | AC-F9 復現基線（帳本可提案，人類或指揮官確認） |

**目前：HOLD 實作交接（產品碼）。** 探針可並行；帳本可準備 fixture 草案但不得當已核准開工。

---

## 9. 交接摘要

| 項目 | 狀態 |
|------|------|
| 已核准版本 | **AC v1.1（本文件）** |
| 待核准 | 見後續版本（如 AC v1.2） |
| 已廢止待核 | AC v1（被本版取代） |
| 可並行 | 探針資料地圖（排出前 3） |
| 產品碼開工 | 仍等探針前 3 證據後由指揮官派工（AC v1.1 範圍：Win＋WSL2） |

---

**AC v1.1 — 已核准**

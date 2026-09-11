# AC v1.4-cursor-official — Cursor 官方用量／Spending 對齊

| 欄位 | 值 |
|------|-----|
| 版本 | **AC v1.4-cursor-official** |
| 狀態 | **已核准**（人類 2026-09-11；可實作；規格不自 PASS） |
| 日期 | 2026-09-11 |
| 作者 | 規格 |
| 證據 | `EC-cursor-official-api-v1`、`EC-cursor-spending-align-v1`（S1–G1）；`EC-open-c6-c9-v1`（C7 已填）；人類 Ultra 截圖 `screenshots/cursor-spending-ultra-2026-09-11.png` |
| 繼承 | 已核 AC（含 v1.3a `usage_pool`／F12 notional）；**不取代**本機匯入；本版加官方／Spending **對照層** |

---

## 核准記錄

| 項目 | 值 |
|------|-----|
| 核准 | **人類核准**（經指揮官轉達 2026-09-11） |
| 實作門檻 | 已開：帳本／儀表／橋樑可依本版開工 |
| 最終 PASS | **僅驗收官**；規格不自 PASS |

---

## 0. Diff 要點

1. 引入來源模式：`official_admin`／`local_enrichment`／`undocumented_dashboard`。  
2. Team／Enterprise：`official_admin`＝`filtered-usage-events`＋`/teams/spend`。  
3. `local_enrichment`：**禁止**單獨當 Cursor USD／帳單／兩池％。  
4. `undocumented_dashboard`：**預設關閉**。  
5. 個人 Ultra：兩池％＋Grok Bot＝**UI 證據／PARTIAL**；**無**公開個人 usage API 則標明。  
6. 非目標：bubble `tokenCount` 充帳單；未文件化 RPC 不當唯一正式源。

---

## 1. 目標

G12. 有 **Team Admin API key** 時，能拉官方事件／spend 並可測對帳（Σ`chargedCents` ↔ `/teams/spend`）。  
G13. 個人／Ultra：UI／CLI 能表達 Spending **對齊目標**（兩池％、reset、on-demand），且標 **PARTIAL／無公開 usage API**。  
G14. 永遠分清：**官方／Spending 面** vs **本機 notional_api_estimate 面**。  
G15.（可選／HOLD）Grok Bot 週％——UI 可註「儀表有此列」；機器同步待證據。

---

## 2. 非目標

N20–N22.（繼承）notional≠實付；禁 bubble 充帳單；未文件 RPC 非唯一正式源。  
N25. 不用本機 token 加總冒充 Spending 兩池％（截圖 65%／100%）。  
N26. Cloud Agents／SDK／Analytics DAU **不是** Cursor USD／Spending 權威。  
N27. `daily-usage-data`（S3）**不是**精準 billable token 首選（官方明文）。  
N28. 不把 `68272` 類同樹雙列誤當已證實雙 distro（與 F3 矩陣分開）。

---

## 3. 來源模式（鎖定）

| Mode | 含義 | 預設 |
|------|------|------|
| **`official_admin`** | Enterprise／Team Admin key → S1 `POST /teams/filtered-usage-events`（必）＋ S2 `POST /teams/spend` | 有 key 時**啟用** |
| **`local_enrichment`** | 本機 DB／bubble／context（L1） | 可開；**禁**單獨出 Cursor USD／帳單／池％ |
| **`undocumented_dashboard`** | U1/U2（`GetCurrentPeriodUsage` 等） | **預設關**；opt-in 須標 UNSUPPORTED／易碎 |

**BLOCK：** 宣稱個人 Pro／Ultra 有公開官方 usage API key；僅 L1 宣稱帳單準確。

---

## 4. 可測 Acceptance Criteria

### AC-F14 `official_admin`（Team／Enterprise）
- **Given** 有效 Team Admin API key（Basic `-u KEY:`）  
- **When** 匯入／查詢官方用量（S1）  
- **Then** 事件含 `model`、`tokenUsage.*`、`chargedCents`（等 docs 欄位）  
- **And** 同週期 Σ`chargedCents` 與 S2 `/teams/spend` 對齊（容差：**建議** ≤$0.01 或報告註明）  
- **And** 無 key／403 Enterprise required → 明確錯誤，不靜默

### AC-F15 `local_enrichment` 不得單獨當 USD／帳單
- **Given** 僅本機 Cursor 源（無 `official_admin`、未開 undocumented）  
- **When** 顯示 Cursor 花費／用量  
- **Then** **不得**輸出唯一權威 USD 總額冒充帳單  
- **And** UI／CLI 標 **PARTIAL**＋指向 Spending／需 Admin  
- **And** bubble `{0,0}` **禁止**當真實用量（BLOCK 若產品如此宣稱）

### AC-F16 `undocumented_dashboard` 預設關
- **Given** 預設設定  
- **Then** 不呼叫 U1/U2；設定檔／旗標預設 `off`  
- **When** 使用者顯式 opt-in  
- **Then** 每次輸出標 **UNSUPPORTED／undocumented／易碎**；仍不得自稱 `official_admin`

### AC-F17 個人 Ultra Spending 對齊（UI／PARTIAL）
- **Given** 個人 Ultra（無 Admin key）＋人類／fixture 截圖類證據（P1）  
- **Then** 產品說明或面板寫明：**無公開個人 usage API**  
- **And** 對齊目標＝**Cursor Models %**、**Other Models %**、reset、on-demand（P1／P2）；**不是**本機 token 加總  
- **And** 可提供手動對照／截圖核對路徑；機器自動讀兩池％＝僅 AC-F16 opt-in 或後續 AC  
- **Fixture 參照（截圖）：** Cursor Models ≈65%、Other Models＝100%、On-Demand Disabled、月 reset≈9/12  

### AC-F18 Grok Bot 週額
- **Given** 截圖確認 UI 有 Grok Bot 週％（G1）  
- **Then** 規格／UI 可註「Spending 有獨立週列」  
- **And** 機器同步用量％＝**HOLD／UNKNOWN**（公開 Admin **無**用量％ API）直到新證據卡  
- **禁止**臆測 endpoint

### AC-F19 與 `usage_pool`／禁假 Other
- 事件層繼續 AC-F13′（`usage_pool`，禁字面 `"Other"` model）  
- 官方／Spending 兩池％為**對照層**，不與假 Other 模型列混淆

### 回歸
AC-F12 notional disclaimer 仍須；notional **≠** 訂閱額度／Spending ％。

---

## 5. 驗收指令大綱

```bash
# Team（有 Admin key）
verify-ac-v1.4-official-admin --fixture fixtures/ac-v1.4/admin-events.json
# 期望：Σ chargedCents ↔ spend；exit 0

# 個人／無 key
verify-ac-v1.4-personal-partial
# 期望：明確 PARTIAL／無公開 API；未啟 undocumented；無 L1 冒充帳單
```

Checklist：
- [ ] F14 Admin 對帳  
- [ ] F15 local 不單獨 USD  
- [ ] F16 undocumented 預設關  
- [ ] F17 Ultra PARTIAL＋兩池對齊目標  
- [ ] F18 Grok Bot HOLD 機器源  
- [ ] F19 無假 Other 列  
- [ ] 證據卡 ID 引用  

**最終 PASS 僅驗收官；規格不自 PASS。**

---

## 6. OPEN-C1…C5 填死（依探針；個人 Ultra）

| ID | 問題 | 結論（證據） | 規格鎖定 |
|----|------|--------------|----------|
| **OPEN-C1** | 個人 Ultra 精確 API／端點／欄位 | **無**公開個人 usage API。正式機器源僅 **Enterprise／Team Admin**：S1 `POST /teams/filtered-usage-events`、S2 `POST /teams/spend`（Basic Admin key）。個人對齊目標＝**P1 Spending UI** 兩池％＋reset／on-demand（截圖 CONFIRMED） | 個人＝**P1＋PARTIAL**；Admin＝**official_admin**（F14） |
| **OPEN-C2** | 無 Admin key 時允許的替代源 | **允許對齊目標：** P1 UI／截圖人手核。**機器 PARTIAL／opt-in only：** U1 undocumented `GetCurrentPeriodUsage`（`autoPercentUsed`≈Cursor Models％、`apiPercentUsed`≈Other％）、U2 CSV／cookie（非正式）。**BLOCK：** L1 bubble／state.vscdb 當帳單／池％ | 替代源＝P1 必；U1/U2＝**undocumented_dashboard 預設關**（F16）；L1＝F15 |
| **OPEN-C3** | ％對齊容差 | 人手／截圖對核：與 Spending 顯示一致即可。機器 opt-in（U1）建議 **±1 percentage point**（未另裁前） | 寫進 F17／F16 驗收 |
| **OPEN-C4** | Grok Bot 週額必做？ | UI **CONFIRMED**（截圖週％）；公開 Admin **無**用量％ API → 機器同步 **HOLD／UNKNOWN（G1）** | **F18**：可註 UI 有列；機器同步非本版必做 |
| **OPEN-C5** | 與 `usage_pool` 雙寫或只對照？ | 事件層維持 F13′ `usage_pool`；Spending 兩池％＝**對照層**（P1／P2）；禁假 `"Other"` model 列 | **F19**：只對照、不雙寫冒充 |

### OPEN-C6…C9 追蹤（`EC-open-c6-c9-v1`；**不改已核 F14–F19 條款**）

| ID | 狀態 | 規格結論 |
|----|------|----------|
| **OPEN-C7** | **已填（docs CONFIRMED）** | Admin `filtered-usage-events` **無** documented `tier`；**禁止**要求 `tier` 才能 ingest。池標籤用 `model`／`kind`／`cursorTokenFee` 等已文件欄；live 是否暗含 `tier`＝UNKNOWN（不擋） |
| OPEN-C6 | **PARTIAL／HOLD** | Overview＝Enterprise＋403「Enterprise access required」；Teams 定價文矛盾；**無** Teams live 403 樣本 |
| OPEN-C8 | **PARTIAL／HOLD** | 對齊 **池％**（P1）；％↔$ 精確公式 **UNKNOWN**；Ultra Other≈$400＝staff／舊表 PARTIAL，現 live help 無 $ 表 |
| OPEN-C9 | **PARTIAL／HOLD** | UI＋help 週額 CONFIRMED；機器＝undocumented `GetSandUsageStatus`／`get-sand-usage-status`（opt-in／fail-soft）；**非**官方契約；F18 機器同步仍非必做 |
| AC v1.3b | 仍待人類核（獨立） | |

證據：`evidence/EC-open-c6-c9-v1.md`

### 個人 Ultra 來源階（驗收可引用）

| 階 | 來源 | 狀態 |
|----|------|------|
| C-official | Admin S1/S2 | **個人帳不可用** |
| C-align-target | Spending UI 兩池％（P1）＋P2 | **CONFIRMED** 正式對齊目標 |
| C-partial-machine | U1／U2 | **undocumented**；PARTIAL／opt-in |
| C-block | L1 bubble | **BLOCK** 當帳單／池％ |
| C-hold | Grok Bot 週％（G1） | UI CONFIRMED；機器 **PARTIAL／UNDOCUMENTED**（C9；非契約） |

---

## 7. 交接摘要

| 項目 | 狀態 |
|------|------|
| **已核准** | **AC v1.4-cursor-official（本文件）** |
| OPEN-C1…C5 | **已填死**（上表） |
| 交接 | 帳本／儀表／橋樑依 F14–F19；**OPEN-C7 已填**；C6／C8／C9 仍 HOLD |
| 不可 | 自 PASS、undocumented 預設開、L1 冒充帳單 |

---

**AC v1.4-cursor-official — 已核准**（人類 2026-09-11；最終閘門＝驗收官）

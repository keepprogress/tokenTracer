> **已核准：AC v1.2**  
> - 核准日：2026-09-11  
> - 核准基準：人類明示「核准 AC v1.2，OPEN-M 用建議默認」  
> - 已鎖定 OPEN-M1…M4（見 §6）  
> - 繼承已核准 AC v1.1；實作範圍 = v1.1（Win＋WSL2）∪ v1.2（macOS 一等公民＋單機聚合）  
> - OPEN-R1／OPEN-P1 仍可後補；產品碼開工仍等探針前 3 證據後由指揮官派工
> - 必做前 3 見 **[AC v1.2a](./AC-v1.2a.md)**；macOS 欄見 **[AC v1.2b](./AC-v1.2b.md)**（已生效）  

# AC v1.2 — tokenTracer（加 macOS 安裝與聚合）

| 欄位 | 值 |
|------|-----|
| 版本 | **AC v1.2** |
| 狀態 | **已核准** |
| 基底 | 繼承 **已核准 AC v1.1** 全部條款，除非本文件明示修改／新增 |
| 日期 | 2026-09-11 |
| 作者 | 規格 |
| 產品 | **tokenTracer** |
| 觸發 | 人類要求：macOS 也能安裝，提高聚合度 |

---

## 0. Diff 要點（相對已核准 AC v1.1）

1. **新增平台**：macOS 可安裝、可啟動（與 Windows 並列為一等公民安裝目標）。  
2. **提高聚合**：同一帳本彙總 **Windows + WSL2 + macOS** 本機可發現之用量來源（同一 `UsageEvent` 模型；跨機合併見 OPEN-M3）。  
3. **UI**：macOS 主介面為 **選單列（menu bar）迷你面板**（對齊 v1.1 右下角／托盤精神）；不以全頁 web 為主。  
4. **新增／修訂 AC**：AC-F1 → 雙平台安裝；新增 AC-F10（macOS 安裝／選單列）；AC-F2／F3 範圍扩至 macOS 本機路徑（依探針 macOS 證據卡）；矩陣加 macOS 欄。  
5. **效能 AC-F9**：參考環境須分別標 Win 與 macOS 基線（或文件註明「以較慢平台為準」）；數值門檻暫同 v1.1，除非量測後人類改核。  
6. v1.1 其餘鎖定（Rust、USD、MIT、分批前 3、官方價目等）**不變**。

---

## 1. 增修目標

G1′. 在 **Windows 10 22H2+／11** 與 **macOS 13 Ventura+** 皆可安裝並啟動。  
G2′. 單一產品能聚合更多來源：Win 本機、WSL2、macOS 本機上目標 agents 的用量（提高聚合度）。  
G8. macOS 選單列迷你面板提供與 AC-F7 等價的摘要／展開明細能力（平台慣用交互可不同，資訊架構對齊）。

---

## 2. 增修非目標

N9. AC v1.2 **不要求** Linux 裸機 GUI（WSL2 內 agent 資料仍由 Win 橋讀取，屬 v1.1）。  
N10. 不要求 iOS／iPadOS／Android。  
N11. 不預設「一台 Mac 自動拉取另一台 Windows 的 DB」——跨裝置同步為 OPEN-M3；v1.2 預設 **單機聚合**（該機上看得到的來源）。  
N12. 不因 macOS 而放寬 AC-F9；亦不在未核准前猜測改門檻。

---

## 3. 修訂／新增 Acceptance Criteria

### AC-F1′ 安裝與啟動（Windows **與** macOS）
- **Given** 乾淨 Windows 10 22H2+／11，**或** 乾淨 **macOS 13 Ventura+**  
- **When** 依該平台安裝文件安裝  
- **Then** tokenTracer 可啟動；失敗含可診斷碼／下一步  
- 驗收須 **兩平台各至少一條通過紀錄**（可分兩次跑）

### AC-F2′ 來源發現（Win／WSL2／macOS）
- 繼承 AC-F2；另：**Given** macOS 本機存在探針已證之 agent 路徑  
- **When** 執行發現  
- **Then** 列出可讀來源；無權限明確報錯，不靜默略過

### AC-F3′ 用量匯入
- 同 v1.1 AC-F3，來源可含 macOS 證據卡；v1.2 必做 agent 集仍為「探針前 3 有證據」，但證據可來自 Win、WSL2 或 macOS（**同一 agent 至少一平台可讀即算該 agent 進入必做集的合格來源**；矩陣須標明哪一格 ok）

### AC-F7′ 迷你面板（平台適配）
- **Windows**：維持 v1.1 AC-F7（右下角／托盤）  
- **macOS**：選單列圖示 + 迷你面板；收合態主數字、展開態項目與 v1.1 AC-F7 資訊對齊（今日／歷史總花費 USD、佔比、趨勢、區間、新鮮度、PARTIAL 警告）  
- 數字與 CLI 一致

### AC-F10 macOS 打包與權限（新增）
- 提供 macOS 安裝方式：**`.dmg` 拖曳安裝**（寫進 README）  
- 若需「完整磁碟存取」或讀其他 app 容器：首次失敗時 UI／文件給出**可照做的權限步驟**；不得靜默 0 資料  
- Apple Silicon 與 Intel：至少 **Apple Silicon 必過**；Intel **best-effort**（不擋 PASS）

### AC-F4…F6、F8、F9
- 繼承 v1.1；F8 README 須含 macOS 安裝／權限／選單列操作；F9 量測須註平台。

---

## 4. 環境矩陣（v1.2 擴充）

| Agent | Win | WSL2 | macOS | v1.2 狀態 |
|-------|-----|------|-------|-----------|
| 探針前 3 #1 | TBD | TBD | TBD | 必做（至少一格 ok） |
| 探針前 3 #2 | TBD | TBD | TBD | 必做 |
| 探針前 3 #3 | TBD | TBD | TBD | 必做 |
| 其餘目標 agent | TBD | TBD | TBD | PARTIAL／路線圖 |

目標全集仍為：Codex、Claude Code、Cursor、Grok Build、Cline、OpenCode、GitHub Copilot。  
探針須補 **macOS 證據卡**（可與 Win／WSL2 分開）；無證據不得宣稱 macOS 已支援該 agent。

---

## 5. 契約
- `UsageEvent`／`SpendSummary` 同 v1.1；建議 `meta.host_os: "windows"|"wsl2"|"macos"` 利於佔比鑽取（帳本可採，驗收可抽樣）。  
- 驗收指令：`verify-ac-v1.2`（含雙平台 checklist 條目）；或 `verify-ac-v1.1` + `verify-ac-v1.2-macos` 對照表。

---

## 6. 已鎖定（OPEN-M）與仍 OPEN

### 已鎖定（人類核准＋建議默認）
| ID | 決策 |
|----|------|
| OPEN-M1 | 最低 **macOS 13 Ventura+** |
| OPEN-M2 | 安裝形態：**`.dmg` 拖曳安裝** |
| OPEN-M3 | 跨裝置 Win↔Mac 同步：**不做**；v1.2 = **單機聚合** only |
| OPEN-M4 | Intel Mac：**best-effort**，不擋 PASS；Apple Silicon 必過 |

### 仍 OPEN（不擋本版核准）
| ID | 問題 | 狀態 |
|----|------|------|
| OPEN-R1 | 建庫 `keepprogress/tokenTracer` | 可後補，不擋探針 |
| OPEN-P1 | 探針前 3 名單（Win／WSL2／macOS 證據卡） | 等探針；交後可出 v1.2a 只填矩陣 |

**產品碼：** 仍等探針前 3 證據後由指揮官派工（範圍含已核准 v1.1＋v1.2）。

---

## 7. 驗收 Checklist 增量

- [ ] AC-F1′ Win 與 macOS 皆能裝能開  
- [ ] AC-F10 權限失敗可診斷  
- [ ] AC-F7′ macOS 選單列迷你面板資訊對齊  
- [ ] 矩陣 macOS 欄無虛報  
- [ ] README 含 macOS  
- [ ] 規格標籤 = 已核准 AC v1.2  

---

## 8. 交接摘要

| 項目 | 狀態 |
|------|------|
| 已核准版本 | **AC v1.1** ＋ **AC v1.2（本文件）** |
| 待核准 | 無（矩陣填空等 v1.2a） |
| 可並行 | 探針：Win／WSL2／macOS 證據卡（排出前 3） |
| 產品碼開工 | 等探針前 3 證據後由指揮官派工（範圍含 Win＋WSL2＋macOS） |

---

**AC v1.2 — 已核准**

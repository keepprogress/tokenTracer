> **已核准：AC v1.1a**  
> - 核准日：2026-09-11  
> - 核准基準：指揮官代核（依探針 INDEX／人類已接受之前 3；與已派帳本／橋樑／儀表對齊）  
> - 類型：已核准 AC v1.1 之矩陣填空補丁（不重開大範圍）  
> - 依據：`evidence/INDEX-v1.1-first3.md`  
> - 必做：Cursor／Claude Code／Codex（邊界見本文）  
> - 規則：同一矩陣內容未變更前，不再退回「待核准」

# AC v1.1a — 支援矩陣填空補丁（前 3 證據）

| 欄位 | 值 |
|------|-----|
| 版本 | **AC v1.1a** |
| 狀態 | **已核准**（矩陣補丁；不改 v1.1 其他條款） |
| 核准基準 | 繼承已核准 **AC v1.1**；本補丁僅填 OPEN-P1 矩陣（探針 2026-09-11 前 3 證據卡） |
| 日期 | 2026-09-11 |
| 作者 | 規格 |
| 證據索引 | `evidence/INDEX-v1.1-first3.md` |

---

## 0. 範圍

只更新「必做前 3」agent 名稱、平台格、證據卡 ID，以及由此衍生的驗收對照。  
**不修改** AC-F1…F9 門檻、UI、Rust／效能、計價容差、授權等。

---

## 1. 必做前 3（鎖定）

| 優先序 | AgentId | 顯示名 | Evidence Card | 整體 |
|--------|---------|--------|---------------|------|
| 1 | `cursor` | Cursor | `EC-cursor-v1` | **PARTIAL**（路徑 CONFIRMED；本機 token 不可靠，計價依卡內 API／估算路徑） |
| 2 | `claude_code` | Claude Code | `EC-claude-code-v1` | **CONFIRMED**（以 WSL 本機 JSONL 為準） |
| 3 | `codex` | Codex | `EC-codex-v1` | **CONFIRMED**（以 WSL rollout JSONL 為準） |

其餘目標 agent（Grok Build、Cline、OpenCode、GitHub Copilot）仍為 **PARTIAL／路線圖**，不得虛報。

---

## 2. 取代 AC v1.1 §6 矩陣（Win／WSL2）

| Agent | Win | WSL2 | AC v1.1 狀態 | 證據卡 |
|-------|-----|------|--------------|--------|
| Cursor | PARTIAL（路徑 CONFIRMED；計價依賴 API／估算） | PARTIAL（可經 `/mnt/c/Users/.../AppData/Roaming/Cursor/...` 讀） | **必做** | `EC-cursor-v1` |
| Claude Code | PARTIAL（須掃 `%USERPROFILE%\.claude\projects`） | **CONFIRMED** | **必做** | `EC-claude-code-v1` |
| Codex | PARTIAL（須掃 `%USERPROFILE%\.codex\sessions`） | **CONFIRMED** | **必做** | `EC-codex-v1` |
| Grok Build | TBD | TBD | 路線圖 | — |
| Cline | TBD | TBD | 路線圖 | — |
| OpenCode | TBD | TBD | 路線圖 | — |
| GitHub Copilot | TBD | TBD | 路線圖 | — |

### 硬約束（來自證據卡，寫進驗收）
1. Cursor bubble `tokenCount` 本機常為 0 → **禁止**當 billed usage。  
2. Claude／Codex 訂閱場景之 USD 可為 **notional API 估價**，非帳單原件；須在 UI／CLI 標明估價性質（符合 AC-F6 來源標註）。  
3. Win 與 WSL 安裝樹分離 → 橋樑／匯入 **必須雙掃**，不可只掃一邊。

---

## 3. 驗收 Checklist 增量（給驗收官／適配驗）

- [ ] 必做三者皆出現在支援矩陣且行為與上表一致（含 PARTIAL 標示）  
- [ ] 匯入來源 ID 可對應 `EC-cursor-v1`／`EC-claude-code-v1`／`EC-codex-v1`  
- [ ] Cursor 不以本機 `tokenCount=0` 虛報用量  
- [ ] Claude Code：WSL `~/.claude/projects/**/*.jsonl` 可匯入  
- [ ] Codex：WSL `~/.codex/sessions/**/rollout-*.jsonl`（`token_usage_record`）可匯入  
- [ ] Win＋WSL 雙掃有測到或文件說明未覆蓋之格為 PARTIAL  

---

## 4. 交接

| 項目 | 狀態 |
|------|------|
| 已核准基底 | AC v1.1 ＋ AC v1.2 |
| 本補丁 | **AC v1.1a — 已核准**（矩陣填空） |
| macOS 欄 | 見 **AC v1.2a**（同三 agent；macOS 格仍 TBD 等探針 macOS 卡） |
| 可派工 | 已可派／已派：帳本／橋樑／儀表（必做＝Cursor／Claude Code／Codex） |

---

**AC v1.1a — 已核准**

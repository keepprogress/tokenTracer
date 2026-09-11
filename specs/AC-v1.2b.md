> - 後續功能補丁見 **[AC v1.3](./AC-v1.3.md)**（by_model／notional／Cursor Other；待核准）  

# AC v1.2b — macOS 欄填空補丁（前 3）

| 欄位 | 值 |
|------|-----|
| 版本 | **AC v1.2b** |
| 狀態 | **已核准**（矩陣補丁；只填 macOS 欄；不改 v1.2／v1.2a 其他條款） |
| 核准基準 | 繼承已核准 AC v1.2；對齊已核准 AC v1.1a；依據探針 INDEX macOS appendix（2026-09-11） |
| 日期 | 2026-09-11 |
| 作者 | 規格 |
| 證據 | `evidence/INDEX-v1.1-first3.md`（含 macOS）；各 EC 之 macOS appendix |
| 硬證據宿主 | NB-T3261 = Win+WSL2；**無 macOS 實機 dump** |

---

## 0. 範圍

只更新前 3 必做 agent 的 **macOS** 格與相關禁止／標註。Win／WSL2 格以 **AC v1.1a** 為準。

---

## 1. macOS 欄（鎖定）

| Agent | AgentId | macOS | 證據卡 | 註記 |
|-------|---------|-------|--------|------|
| Cursor | `cursor` | **PARTIAL** | `EC-cursor-v1` | 路徑 CONFIRMED（`~/Library/Application Support/Cursor/...`、`~/.cursor/`）；billable 仍非本機可靠；**無實機 dump** |
| Claude Code | `claude_code` | **CONFIRMED\*** | `EC-claude-code-v1` | `~/.claude/projects/**/*.jsonl` 路徑／格式與 Linux 一致；\* = 無實機驗證宿主 |
| Codex | `codex` | **CONFIRMED\*** | `EC-codex-v1` | `~/.codex/sessions/**/rollout-*.jsonl`；`token_usage_record` 跨客戶端；\* = 無 Mac dump，欄位形狀沿用 WSL 樣本 |

\* **無實機抽樣**＝文件／格式 **CONFIRMED**、實機驗證 **UNKNOWN**；**禁止**宣稱「已在某台 Mac 實機驗證」。實機抽樣後可出 v1.2c 升格「實機 CONFIRMED」。

---

## 2. 完整三欄矩陣（v1.1a ∪ v1.2b）

| Agent | Win | WSL2 | macOS | 狀態 | 證據卡 |
|-------|-----|------|-------|------|--------|
| Cursor | PARTIAL | PARTIAL | **PARTIAL** | 必做 | `EC-cursor-v1` |
| Claude Code | PARTIAL | CONFIRMED | **CONFIRMED\*** | 必做 | `EC-claude-code-v1` |
| Codex | PARTIAL | CONFIRMED | **CONFIRMED\*** | 必做 | `EC-codex-v1` |
| 其餘目標 | TBD | TBD | TBD | 路線圖 | — |

---

## 3. macOS 驗收增量／禁止事項

1. 發現路徑至少嘗試：Cursor `Application Support` + `~/.cursor`；Claude `~/.claude/projects`；Codex `~/.codex/sessions`。  
2. **禁止**把 Cursor bubble `tokenCount` 當帳單（三平台相同）。  
3. **禁止**為匯入去讀 Keychain；一般讀上述目錄**不**把 Full Disk Access 當硬安裝前置（**FDA 非預設必要**；疑難排解寫 README）。  
4. UI／CLI 對 CONFIRMED\* 來源須能標「路徑已支援；實機抽樣未做」或等價（不可靜默假裝已 live-verify）。  
5. Apple Silicon 必過安裝／啟動（AC-F10）；讀取上述路徑在無資料時給可診斷空狀態，非崩潰。

---

## 4. 交接

| 項目 | 狀態 |
|------|------|
| 已核准 | AC v1.1、v1.2、**v1.1a**、**v1.2b** |
| 本補丁 | **AC v1.2b — 已核准／已生效**（macOS 欄） |
| 對齊 | AC v1.2a 指向本檔 |
| 後續 | 有 Mac 實機 dump → 可出 v1.2c 升「實機 CONFIRMED」 |

---

**AC v1.2b — 已核准／已生效**

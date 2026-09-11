# tokenTracer — 前 3 家證據卡摘要（AC v1.1a / v1.2a → v1.2b 輸入）

日期：2026-09-11（Asia/Taipei）  
探針：探針  
硬證據宿主：NB-T3261（Windows + WSL2）；**無 macOS 實機**（路徑依公開文件）

| 優先序 | AgentId | Evidence Card ID | Win | WSL2 | macOS | 整體 | 主資料來源 |
|--------|---------|------------------|-----|------|-------|------|------------|
| 1 | `cursor` | `EC-cursor-v1` | PARTIAL | PARTIAL | **PARTIAL**（路徑 CONFIRMED；計價仍 API／估算；無實機） | PARTIAL | macOS: `~/Library/Application Support/Cursor/User/globalStorage/state.vscdb` + `~/.cursor/` |
| 2 | `claude_code` | `EC-claude-code-v1` | PARTIAL | CONFIRMED | **CONFIRMED***（路徑／JSONL；*無實機驗證） | CONFIRMED (WSL) / CONFIRMED* (mac path) | `~/.claude/projects/**/*.jsonl`（三平台同形） |
| 3 | `codex` | `EC-codex-v1` | PARTIAL | CONFIRMED | **CONFIRMED***（路徑／rollout；*欄位形狀沿用 WSL 樣本） | CONFIRMED (WSL) / CONFIRMED* (mac path) | `~/.codex/sessions/**/rollout-*.jsonl` |

\* = 公開文件 CONFIRMED 路徑／格式；本機 macOS dump = UNKNOWN。規格填矩陣時建議 macOS 標 **CONFIRMED（路徑）** 或 **PARTIAL** 並註「待實機」——探針建議：

| Agent | 建議 macOS 格 | 理由 |
|-------|---------------|------|
| Cursor | **PARTIAL** | 路徑 CONFIRMED；billable 仍非本機可靠 |
| Claude Code | **CONFIRMED** | JSONL 路徑／欄位與 Linux 一致（官方 + parsers）；缺實機僅降低「宿主驗證」非格式 |
| Codex | **CONFIRMED** | rollout 路徑與 Linux 一致；`token_usage_record` 為跨客戶端格式（缺 Mac dump 標 PARTIAL 亦可接受） |

更保守填法（全 PARTIAL until live Mac）：亦可。**禁止臆測已在某台 Mac 驗證。**

## 檔案

- `/home/box/agent-data/projects/token-spend-tracker/evidence/INDEX-v1.1-first3.md`（本檔已含 macOS）
- `EC-cursor-v1.md` / `EC-claude-code-v1.md` / `EC-codex-v1.md`（各含 macOS appendix）
- 工作區：`/workspace/tokenTracer-evidence/`

## macOS 權限摘要（tokenTracer 讀取）

- 讀 `~/Library/Application Support/Cursor`、`~/.cursor`、`~/.claude`、`~/.codex`：**一般不需 Full Disk Access**。
- FDA／Files & Folders：主要與 Desktop/Documents/Downloads 或外接碟有關；寫進 README 作疑難排解即可，**不**當硬安裝前置。
- Claude：用量 JSONL 不需 Keychain；勿為匯入去讀 Keychain。
- Cursor：勿用 bubble `tokenCount` 當帳單。

## 仍 UNKNOWN / 下一步

1. 有 Mac 時：對三路徑做與 NB-T3261 同級的只讀抽樣，升級「實機 CONFIRMED」。
2. Win 原生 Claude/Codex `sessions`/`projects` 樹補探（仍 PARTIAL）。
3. 規格可出 **AC v1.2b** 填 macOS 欄。

## Addendum

- `EC-cursor-other-models-v1.md` — Cursor Models vs Other Models 池（API `tier` / `apiPercentUsed`；非本機字串）

# Live probe — NB-T3261 (2026-09-11, Asia/Taipei ~14:20)

Read-only discovery on registered machine `NB-T3261` (Windows + WSL2).

## WSL2 distros
| Name | State | Version |
|------|-------|---------|
| Ubuntu-Work (default) | Running | 2 |
| Ubuntu | Stopped | 2 |

WSL user: `t3261` · Linux home: `/home/t3261`

## Windows native
| Source | Path | Result |
|--------|------|--------|
| Claude Code | `C:\Users\T3261\.claude\projects\**\*.jsonl` | **OK** — 36 jsonl |
| Codex | `C:\Users\T3261\.codex\sessions\**/rollout-*.jsonl` | **OK** — 155 rollout |
| Codex archived | `C:\Users\T3261\.codex\archived_sessions` | absent |
| Cursor | `C:\Users\T3261\AppData\Roaming\Cursor\User\globalStorage\state.vscdb` | **OK** — ~770 MiB (LastWrite ~2026-08-31) |

## WSL2 Ubuntu-Work
| Source | Path | Result |
|--------|------|--------|
| Claude Code | `/home/t3261/.claude/projects/**/*.jsonl` | **OK** — earlier count **1427** jsonl; samples present |
| Codex | `/home/t3261/.codex/sessions/**/rollout-*.jsonl` | **OK** — earlier count **311** rollout; samples present |
| Cursor agent home | `/home/t3261/.cursor/` | **OK** — exists (`ai-tracking`, etc.); primary Cursor UI DB remains on Windows AppData |

## Implication
Dual-scan is mandatory: Win Claude 36 vs WSL 1427; Win Codex 155 vs WSL 311. Skipping either side under-counts.

## Notes
- Do not open Cursor `state.vscdb` as billed usage; discovery only lists path.
- Do not read Claude/Codex credential files.

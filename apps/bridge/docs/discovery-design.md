# Discovery module design — tokenTracer 橋樑

| Field | Value |
|-------|-------|
| Role | 橋樑 (Windows/WSL bridge) |
| Specs | **AC v1.1a ∪ v1.2b** (inherit v1.1 / v1.2) |
| Date | 2026-09-11 (Asia/Taipei) |
| Live probe | [`live-probe-NB-T3261.md`](./live-probe-NB-T3261.md) |
| Contract | [`../CONTRACT-path-list-v0.md`](../CONTRACT-path-list-v0.md) |

---

## 1. Architecture

```text
┌─────────────────────────────────────────────────────────┐
│  Tauri 2 host (single process)                          │
│   ┌─────────────┐  IPC   ┌────────────────────────────┐ │
│   │ mini-panel  │◄──────►│ tokentracer-bridge (lib)   │ │
│   │ (儀表 UI)   │        │  discover_paths()          │ │
│   └─────────────┘        │  + WSL / path probes       │ │
│                          └──────────┬─────────────────┘ │
└─────────────────────────────────────┼───────────────────┘
                                      │ DiscoverResult
                                      ▼
                               帳本 (ledger import)
```

- Bridge is a **Rust library crate** embedded in the host — **not** a long-running subprocess.
- Separate **CLI bin** (`tokentracer-bridge`) for headless / CI / fixture runs.
- Discovery is read-only; no credential files in fixtures or logs.

---

## 2. Dual-scan (mandatory)

AC v1.1a hard constraint: Win and WSL install trees are **separate**. Skipping either side under-counts.

### NB-T3261 live evidence (2026-09-11)

| Agent | Windows | WSL2 Ubuntu-Work |
|-------|---------|------------------|
| Claude Code jsonl | **36** | **1427** |
| Codex rollout | **155** | **311** |
| Cursor state.vscdb | ~**770 MiB** (AppData) | UI DB on Win; `~/.cursor` exists in WSL |

WSL distros: **Ubuntu-Work** (Running, default), **Ubuntu** (Stopped). User `t3261`.

### Access patterns (both documented)

1. UNC: `\\wsl$\<Distro>\home\<user>\.claude\...` (forward-slash form in ids: `//wsl$/...` when used as importer path)
2. Exec: `wsl -d <Distro> -- <cmd>`

WSL `canonical_root` = `wsl:<Distro>:<posix-abs>` (path-list-v0.2); `meta.import_path` defaults to **posix** (same as `meta.posix_path`); `meta.unc_path` for Win open. UNC is never the sole id.

---

## 3. Agents (v1.1a / v1.2b)

| Agent | Win | WSL2 | macOS | Evidence |
|-------|-----|------|-------|----------|
| cursor | PARTIAL | PARTIAL | PARTIAL | EC-cursor-v1 |
| claude_code | PARTIAL | CONFIRMED | CONFIRMED* | EC-claude-code-v1 |
| codex | PARTIAL | CONFIRMED | CONFIRMED* | EC-codex-v1 |

\* = path/format confirmed; **live Mac verification UNKNOWN** — do not claim live Mac dump.

### Paths

- **claude_code:** Win `%USERPROFILE%\.claude\projects/**/*.jsonl` **and** WSL `~/.claude/projects/**/*.jsonl`
- **codex:** Win `%USERPROFILE%\.codex\sessions/**/rollout-*.jsonl` (+ archived) **and** WSL same under `~/.codex`
- **cursor:** Win `%APPDATA%\Cursor\User\globalStorage\state.vscdb` (+ optional workspaceStorage); note `~/.cursor` trees; macOS `~/Library/Application Support/Cursor/User/globalStorage/state.vscdb` **and** `~/.cursor/` (enrichment; not L1 billing)
- **NEVER** treat bubble `tokenCount` as billed usage — Cursor stays `status: partial`

---

## 4. WSL2 discovery algorithm

1. List distros: `wsl.exe -l -v` (Windows) or injectable fixture list (Linux POC).
2. For each distro (prefer Running): resolve Linux home (`/home/<user>` via UNC or `wsl -d … -- printenv HOME`).
3. Probe `.claude` / `.codex` roots; emit sources + TT-F2-\* on failure.
4. Never silent-skip missing paths.

On this Linux box, `DiscoverConfig.wsl_distros` + `tests/fixtures/` simulate dual-scan without real WSL.

---

## 5. Path-list API (summary)

See **CONTRACT-path-list-v0.md**.

- `id = {evidence_card}:{host}:{canonical_root}` (WSL root = `wsl:<Distro>:<posix>`)
- Default: **no** `files[]`; expand via `files --source-id [--limit N]` / `--list-files`
- Limits: discover incidental **32**; `files` default **500**; `--limit 0` unlimited (+ warn >5000); truncated ⇒ `truncated:true` + TT-F2-006
- `host` === `UsageEvent.meta.host_os`
- TT-F2-\* pass through to import report

---

## 6. Diagnostic codes (AC-F1 / F2)

| Code | Meaning | next_step |
|------|---------|-----------|
| TT-F1-001 | Install failed | Re-run installer; check install-dir ACLs; capture installer log |
| TT-F1-002 | Start failed | Confirm binary on PATH; check tray/host logs; try CLI `--help` |
| TT-F1-003 | Missing dependency | Install listed dependency (WebView2 / WSL optional feature), restart |
| TT-F2-001 | WSL not installed | `wsl --install` (or enable features), reboot, rediscover |
| TT-F2-002 | WSL distro list failed | Run `wsl.exe -l -v` manually; repair WSL; ensure `wsl.exe` on PATH |
| TT-F2-003 | Path not found | Verify agent installed on that host (Win≠WSL trees); rediscover |
| TT-F2-004 | Permission denied | Grant read; fix `\\wsl$\` ACL or use `wsl -d` probes |
| TT-F2-005 | Path unreadable | Check locks/corruption; open SQLite read-only; retry |
| TT-F2-006 | File list truncated | Raise `--limit` or `--limit 0`; do not treat list as complete |
| TT-F10-FDA | macOS permission denied (FDA troubleshooting) | System Settings → Privacy & Security → Full Disk Access → enable tokenTracer; FDA troubleshooting only, not required for normal install; re-run discover |
| TT-F10-001 | macOS permission denied (numeric twin) | Same next_step as TT-F10-FDA |

Each `DiscoverError` includes human `next_step`. macOS PermissionDenied maps to **TT-F10-***; Win/WSL keep **TT-F2-004**. See [`macos-f10-packaging-v0.md`](./macos-f10-packaging-v0.md).

---

## 7. Handoff to 帳本

1. Consume `DiscoverResult` JSON (this contract).
2. Import readable `ok`/`partial` sources; attach TT-F2-\* to import report.
3. **ImportMeta** single writer = **帳本** state file (OPEN-UI-3). Host calls ledger `record_import` / IPC after successful `import_run`; bridge/host must **not** rewrite that JSON. Fields: `last_imported_at` (UTC), `last_import_source_ids`, `parse_error_count`, `events_upserted`. Read via `import_status()`.
4. Cursor: do not invent billed tokens from local bubble counts.

---

## 8. Out of scope (this POC)

- Full Tauri UI / tray implementation (scaffold note only — see `host-shell.md`)
- Live macOS path verification
- Parsing / pricing (帳本)
- Claiming product PASS

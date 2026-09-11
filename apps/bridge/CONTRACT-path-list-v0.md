# CONTRACT — Path List API v0 (橋樑 → 帳本)

| Field | Value |
|-------|-------|
| Version | **path-list-v0.2** |
| Date | 2026-09-11 (Asia/Taipei) |
| Specs | AC v1.1a ∪ v1.2b |
| Producer | 橋樑 (`tokentracer-bridge`) |
| Consumer | 帳本 (ledger importer) |
| Status | **Locked with 帳本** (commander rulings v0.2) |

---

## Changelog

| Rev | Notes |
|-----|-------|
| **v0.2** | WSL `canonical_root` = `wsl:<Distro>:<posix-abs>` (multi-distro safe; UNC never sole id). `files --source-id` default **limit 500**; `--limit 0` = unlimited (+ warn if count >5000). Truncated lists MUST set `truncated: true` + **TT-F2-006**. ImportMeta **single writer = 帳本** (OPEN-UI-3). |
| v0.1 | Initial lock: default discover omits `files[]`; dual-scan; `id = card:host:root`. |

---

## 1. Entry points

| Surface | Signature |
|---------|-----------|
| Library | `discover_paths(config) -> DiscoverResult` |
| Library expand | `list_files_for_source(config, source_id, limit) -> FileListResult` |
| CLI | `tokentracer-bridge discover --json [--fixture DIR] [--list-files] [--win-user-profile PATH] [--win-appdata PATH] [--macos-home PATH]` |
| CLI live defaults | Without `--fixture`: Windows → `USERPROFILE`/`APPDATA` for Win roots; macOS → `HOME`. Live WSL resolves `$USER`/`$HOME` per distro into `\\wsl$\<Distro>\home\<user>` (never invents `unknown`). |
| CLI expand | `tokentracer-bridge files --source-id <id> [--limit N] [--fixture DIR]` |

**Default discover:** returns `root_path` + `glob` + `file_count` only.  
**No `files[]` by default.** Optional expand via `--list-files` / `files` subcommand / `DiscoverConfig.list_files = true`.

### files[] limits (v0.2)

| Path | Default cap | Notes |
|------|-------------|-------|
| discover / preview (`--list-files`) | **32** | Incidental listing only |
| `files --source-id` (import expand) | **500** | Product default |
| `--limit 0` | unlimited | Full import expand; if `file_count > 5000` emit **warn** (stderr / `FileListResult.warning`) |
| `--limit N` (N>0) | hard cap N | When truncated: `truncated: true` + diagnostic **TT-F2-006** |

Importers MUST NOT treat a truncated `files[]` as the complete set — use `file_count` and/or re-expand with higher/`0` limit.

---

## 2. JSON schema

```text
DiscoverResult {
  discovered_at: string          // ISO-8601 UTC
  host_os: "windows" | "macos" | "linux"
  sources: DiscoverSource[]
  errors: DiscoverError[]
}

DiscoverSource {
  id: string                     // see §3
  agent: "claude_code" | "codex" | "cursor"
  evidence_card: "EC-claude-code-v1" | "EC-codex-v1" | "EC-cursor-v1"
  host: "windows" | "wsl2" | "macos"   // === UsageEvent.meta.host_os
  root_path: string              // absolute canonical root (§3)
  glob: string                   // relative under probe/import root
  files?: string[]               // ONLY when expand requested
  file_count: number             // u64 — total matches (may exceed files.len)
  truncated?: bool               // set when files requested and capped
  readable: bool
  status: "ok" | "partial" | "unsupported" | "error"
  meta?: { [k: string]: string } // distro, posix_path, unc_path, import_path, …
}

FileListResult {                 // files --source-id / list_files_for_source
  source_id: string
  files: string[]
  file_count: number             // total matches
  truncated: bool                // true ⇒ incomplete list; see errors[]
  errors: DiscoverError[]        // TT-F2-006 when truncated
  warning?: string               // e.g. unlimited expand >5000
}

DiscoverError {
  code: string                   // TT-F1-xxx / TT-F2-xxx
  message: string
  next_step: string              // required — never silent skip
  path?: string
}
```

---

## 3. `source.id` stability (v0.2 — parseable, cross-platform)

**Format:** `{evidence_card}:{host}:{canonical_root}`  
Parse with `splitn(3, ':')` — `canonical_root` may itself contain `:`.

### Native (windows / macos)

- `canonical_root` = absolute path, **forward slashes**, **no trailing slash** (except drive root `C:/`)
- Examples:
  - `EC-claude-code-v1:windows:C:/Users/T3261/.claude`
  - `EC-codex-v1:windows:C:/Users/T3261/.codex/sessions`
  - `EC-cursor-v1:windows:C:/Users/T3261/AppData/Roaming/Cursor/User/globalStorage`
  - `EC-claude-code-v1:macos:/Users/you/.claude`

### WSL2 (host = `wsl2`)

- `canonical_root` = **`wsl:<Distro>:<posix-abs>`** (distro embedded — multi-distro safe)
- **Do NOT** use UNC as the sole id.
- Examples:
  - `EC-claude-code-v1:wsl2:wsl:Ubuntu-Work:/home/t3261/.claude`
  - `EC-codex-v1:wsl2:wsl:Ubuntu-Work:/home/t3261/.codex/sessions`
- Required / recommended meta:
  - `meta.distro` — same Distro string
  - `meta.posix_path` — `/home/<user>/...` only
  - `meta.unc_path` (or `meta.access_unc`) — optional Windows open path `\\wsl$\<Distro>\home\...`
  - `meta.import_path` — probed mount/UNC path used by the bridge host to read files

**Rules:**
- `host` ∈ `windows` \| `wsl2` \| `macos` (same strings as `UsageEvent.meta.host_os`)
- Stable across reruns for the same machine + distro layout
- `id` third segment === `root_path`

---

## 4. Agent roots & globs (v0)

| Agent | host | root (canonical) | glob | status notes |
|-------|------|------------------|------|--------------|
| claude_code | windows | `%USERPROFILE%/.claude` | `projects/**/*.jsonl` | PARTIAL until Win tree confirmed populated |
| claude_code | wsl2 | `wsl:<Distro>:/home/<user>/.claude` | `projects/**/*.jsonl` | CONFIRMED (NB-T3261) |
| claude_code | macos | `~/.claude` | `projects/**/*.jsonl` | CONFIRMED* path; live Mac UNKNOWN |
| codex | windows | `…/.codex/sessions` (+ `…/archived_sessions`) | `**/rollout-*.jsonl` / `rollout-*.jsonl` | PARTIAL Win |
| codex | wsl2 | `wsl:<Distro>:/home/<user>/.codex/sessions` (+ archived) | same | CONFIRMED |
| codex | macos | `~/.codex/sessions` (+ archived) | same | CONFIRMED*; live Mac UNKNOWN |
| cursor | windows | `%APPDATA%/Cursor/User/globalStorage` | `state.vscdb` | **partial** — path OK; not billed from bubble tokenCount |
| cursor | windows | `…/workspaceStorage` | `**/state.vscdb` | optional |
| cursor | macos | `~/Library/Application Support/Cursor/User/globalStorage` | `state.vscdb` | PARTIAL; live Mac UNKNOWN |

**Hard rules:**
1. Win + WSL **dual-scan mandatory** (AC v1.1a). Live probe: Win Claude 36 vs WSL 1427; Win Codex 155 vs WSL 311.
2. Cursor `status` stays **`partial`** even when readable.
3. NEVER treat Cursor bubble `tokenCount` as billed usage.
4. Never read `.credentials.json` / `auth.json` / Cursor auth tokens into fixtures or logs.

---

## 5. Diagnostic codes (pass-through to import report)

| Code | Meaning | next_step (summary) |
|------|---------|---------------------|
| TT-F1-001 | Install failed | Re-run installer; check permissions / log |
| TT-F1-002 | Start failed | Check PATH / tray host logs |
| TT-F1-003 | Missing dependency | Install missing runtime (e.g. WebView2, WSL) |
| TT-F2-001 | WSL not installed | `wsl --install` then rediscover |
| TT-F2-002 | WSL distro list failed | Repair `wsl.exe -l -v` |
| TT-F2-003 | Path not found | Verify agent install on that host |
| TT-F2-004 | Permission denied | Grant read / fix `\\wsl$` ACL |
| TT-F2-005 | Path unreadable | Unlock / open SQLite read-only |
| TT-F2-006 | File list truncated | Raise `--limit` or use `--limit 0`; do not treat list as complete |

**TT-F2-\*** codes from discovery **pass through** into the 帳本 import report. Never silent-skip.

---

## 6. Handoff notes for 帳本

- Importer walks `sources` where `readable && status ∈ {ok, partial}`.
- Expand files only when needed: `files --source-id [--limit N]` or re-discover with `--list-files`.
- Map `DiscoverSource.host` → `UsageEvent.meta.host_os`.
- Map `evidence_card` / `id` → `UsageEvent.source`.
- Cursor: prefer API/CSV/Admin for billable tokens; local DB is discovery/enrichment.

### ImportMeta (OPEN-UI-3) — single writer = 帳本

| Rule | Detail |
|------|--------|
| **Single writer** | **帳本 state file** only (OPEN-UI-3). |
| Host on success | After `import_run` succeeds, host calls ledger `record_import` / `import run …` (or IPC) — does **not** rewrite ImportMeta JSON itself. |
| Bridge | Must **NOT** write ImportMeta. |
| Read | `import status --json` / IPC `import_status()`. |

| Field | Type | Notes |
|-------|------|-------|
| `last_imported_at` | ISO-8601 **UTC** | Display local in UI |
| `last_import_source_ids` | `string[]` | path-list `source.id` values |
| `parse_error_count` | `u64` | from import report |
| `events_upserted` | `u64` | from import report |

---

## 7. Open questions

1. ~~WSL id shape / multi-distro~~ — **Resolved v0.2:** embed `wsl:<Distro>:<posix>` in `canonical_root`.
2. ~~files[] expand cap~~ — **Resolved v0.2:** discover incidental 32; `files` default 500; `--limit 0` unlimited (+ warn >5000).

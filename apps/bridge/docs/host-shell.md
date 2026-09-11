# Host shell outline — Tauri 2 (tray / mini-panel)

| Field | Value |
|-------|-------|
| Specs | AC-F7 / AC-F7′ / AC-F10 · AC v1.1a ∪ v1.2b |
| UI IA | [`mini-panel-ia-v0.md`](/home/box/agent-data/projects/token-spend-tracker/design/ui/mini-panel-ia-v0.md) |
| UI crate (儀表) | `/workspace/tokenTracer-ui` |
| Bridge | this crate as **library** inside host |
| Contract | path-list-v0.2 |

**Do not build full UI here** — scaffold outline only.

---

## 1. Process model (locked)

- **Tauri 2 single-process host**
- `tokentracer-bridge` linked as a **Rust lib** inside the host (**not** a long-running subprocess)
- Separate **CLI bin** for headless discover/import tooling / CI

```text
tokenTracer.exe / tokenTracer.app
  ├─ Tauri shell + tray / menu bar
  ├─ 儀表 webview (mini-panel IA)
  └─ tokentracer_bridge::discover_paths (+ future import glue)
       └─ IPC to 帳本 query / import APIs
```

---

## 2. Platform chrome

| Platform | Shell | Notes |
|----------|-------|-------|
| Windows | System **tray** + **bottom-right** mini-panel | AC-F7 primary surface |
| macOS | **Menu bar** icon + dropdown mini-panel | AC-F7′ / AC-F10; macOS data paths wait for live probe cards — shell scaffold OK |

Align states with 儀表 IA: Hidden ↔ Collapsed ↔ Expanded.

---

## 3. IPC surface (host ↔ 帳本 / bridge)

| Command | Direction | Purpose |
|---------|-----------|---------|
| `discover` | UI → bridge lib | `DiscoverResult` (path-list-v0.2) |
| `import_run` | UI → bridge/ledger | Trigger import for selected source ids |
| `spend_total` | UI → ledger | `SpendSummary` for current range |
| `spend_series` | UI → ledger | Daily trend series for spark/bars |
| `import_status` | UI → ledger | **ImportMeta** read (LEDGER-QUERY OPEN-UI-3) |

### ImportMeta (OPEN-UI-3) — single writer = 帳本

**Locked (path-list-v0.2 / commander):**

| Rule | Detail |
|------|--------|
| **Single writer** | **帳本 state file** only. Bridge/host must **NOT** rewrite that JSON themselves. |
| Host on success | After `import_run` succeeds, host calls ledger `record_import` / `import run …` (or IPC) with the fields below. |
| Read | `import status --json` / IPC `import_status()`. |

| Field | Type | Notes |
|-------|------|-------|
| `last_imported_at` | ISO-8601 **UTC** | Display local in UI |
| `last_import_source_ids` | `string[]` | path-list `source.id` values |
| `parse_error_count` | `u64` | from import report |
| `events_upserted` | `u64` | from import report |

Mini-panel block `meta.last_import` binds this (IA §4.2).

---

## 4. Crate layout (suggested monorepo)

```text
tokenTracer/
  crates/tokentracer-bridge/   # this POC → library + CLI
  crates/tokentracer-ledger/   # 帳本
  apps/tokenTracer-ui/         # 儀表 (/workspace/tokenTracer-ui)
  apps/tokenTracer-host/       # Tauri 2 host (tray/menu bar)
```

Host `Cargo.toml` depends on `tokentracer-bridge` and ledger query crates; UI stays in the webview.

---

## 5. Permissions / diagnostics

- Surface TT-F1-\* on install/start failure in tray balloon / first-run banner.
- Surface TT-F2-\* from discover in Expanded PARTIAL / error banner (never silent empty).
- Surface TT-F2-006 when a capped `files[]` expand truncates (do not treat as complete).
- macOS: do **not** require Full Disk Access as hard install prerequisite (AC v1.2b); document as troubleshooting only.

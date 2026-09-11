# macOS AC-F10 packaging / permissions / paths — design v0

| Field | Value |
|-------|-------|
| Role | 橋樑 (packaging + discover diagnostics) |
| Spec | AC-F10 ∪ AC v1.2b (FDA 非預設必要) |
| Date | 2026-09-11 (Asia/Taipei) |
| Stack | On `ui/macos-menubar-f10` shell (儀表 owns menu-bar UI) |
| Live Mac | **UNKNOWN / BLOCKED** — no Apple Silicon host registered |

---

## 1. Install shape

- **Artifact:** `.dmg` drag-to-Applications (`cargo tauri build` → `target/release/bundle/dmg/*.dmg`).
- **Config:** `apps/tokenTracer-host/tauri.conf.json` already includes `bundle.targets: dmg` + `macOS.minimumSystemVersion: "13.0"` (儀表 shell commit). Bridge does **not** re-claim universal binary.
- **OS floor:** macOS **13 Ventura+**.
- **Arch:** **Apple Silicon required** for PASS; **Intel** (`x86_64-apple-darwin` or Rosetta) = **best-effort**, does not block PASS.
- **Linux agent box:** cannot produce a real `.dmg` (no Apple toolchain). Document only; no fake PASS.

## 2. FDA diagnostic contract

| Rule | Detail |
|------|--------|
| FDA default | **Not** an install prerequisite (AC v1.2b). Normal reads of agent homes should work without FDA. |
| First failure | Must emit a **followable** diagnostic — never silent `0` / empty with no code. |
| Code | `TT-F10-FDA` (primary) / `TT-F10-001` (numeric twin). Win/WSL keep `TT-F2-004`. |
| `next_step` | System Settings → Privacy & Security → Full Disk Access → enable tokenTracer; FDA is troubleshooting only, not required for a normal install. Re-run discover. |
| Entitlements | Do **not** ship FDA-forcing entitlements as hard install gate. |

## 3. Path table (AC v1.2b / probe INDEX)

| Agent | macOS path | Support note |
|-------|------------|--------------|
| Claude Code | `~/.claude/projects/**/*.jsonl` (root `~/.claude`) | CONFIRMED\* path/format; live Mac dump **UNKNOWN** |
| Codex | `~/.codex/sessions/**/rollout-*.jsonl` (+ archived) | CONFIRMED\*; live **UNKNOWN** |
| Cursor IDE DB | `~/Library/Application Support/Cursor/User/globalStorage/state.vscdb` | PARTIAL — path CONFIRMED; **never** L1 billing from bubble/`tokenCount` |
| Cursor agent home | `~/.cursor/` (e.g. `ai-tracking/*.db`) | PARTIAL enrichment; **not** authoritative billed tokens |

\*CONFIRMED = path-supported in bridge; **not** live-verified on a real Mac in this pack.

Empty / missing path → `TT-F2-003` (or empty-file `Partial`) — **diagnosable**, never panic.

## 4. Arch policy

- Apple Silicon (`aarch64-apple-darwin`): required gate for F10 PASS.
- Intel: best-effort; document Rosetta / `x86_64-apple-darwin`; **does not block PASS**.

## 5. Non-goals (this bridge pack)

- Menu-bar mini-panel UI / tray IA — **儀表** (`AC-F7′`); bridge does not implement or self-PASS it.
- Keychain reads.
- Treating Cursor `state.vscdb` / bubble counts as billing.
- Claiming universal binary without configuring it.
- Producing `.dmg` on Linux CI / agent box.
- Live Mac smoke without a registered Apple Silicon machine → see evidence **BLOCKED** note.

## 6. Implementation map

| Deliverable | Location |
|-------------|----------|
| TT-F10 codes | `apps/bridge/src/errors.rs` |
| Host-aware PermissionDenied | `apps/bridge/src/fsutil.rs` (`probe_glob_for_host`) |
| `~/.cursor` on macOS | `scan_macos` → `scan_cursor_agent_home(..., SourceHost::Macos)` |
| `.dmg` target | `tauri.conf.json` (already on shell branch) |
| Evidence (no Mac) | `evidence/pr18-macos-f10-20260911/` |

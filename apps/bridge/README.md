# tokentracer-bridge

Windows/WSL discovery bridge POC for **tokenTracer** (role: 橋樑).

Specs: **AC v1.1a ∪ v1.2b**. Path contract: [`CONTRACT-path-list-v0.md`](./CONTRACT-path-list-v0.md) (**path-list-v0.2**).


## Live Windows / macOS defaults

When `--fixture` is **not** set:

| Host | Flag omitted | Default |
|------|--------------|---------|
| Windows | `--win-user-profile` | `%USERPROFILE%` (`USERPROFILE` env) |
| Windows | `--win-appdata` | `%APPDATA%` Roaming (`APPDATA` env) |
| macOS | `--macos-home` | `$HOME` |

So on NB-T3261 / any Windows box:

```bat
tokentracer-bridge discover --json
```

picks native Win roots automatically and dual-scans WSL (resolving each distro's `$USER`/`$HOME` → `\\wsl$\<Distro>\home\<user>`). Explicit flags still override env defaults.

WSL sources set `meta.import_path` to the **posix** path (e.g. `/home/<user>/.claude`) by default so import inside that distro needs no remap; UNC remains in `meta.unc_path`.

## Build / test (Linux box)

```bash
cd /workspace/tokenTracer-bridge
cargo test
cargo run --example discover_fixtures
cargo run -- discover --fixture tests/fixtures --json
# optional incidental expand (cap 32):
cargo run -- discover --fixture tests/fixtures --list-files --json
# import expand (default --limit 500; 0 = unlimited):
cargo run -- files --source-id 'EC-claude-code-v1:wsl2:wsl:Ubuntu-Work:/home/t3261/.claude' --fixture tests/fixtures
cargo run -- files --source-id 'EC-claude-code-v1:wsl2:wsl:Ubuntu-Work:/home/t3261/.claude' --fixture tests/fixtures --limit 1
cargo run -- files --source-id 'EC-claude-code-v1:wsl2:wsl:Ubuntu-Work:/home/t3261/.claude' --fixture tests/fixtures --limit 0
```

## Docs

- `docs/discovery-design.md` — architecture, dual-scan, error codes
- `docs/host-shell.md` — Tauri 2 tray/menu-bar outline
- `docs/manual-verify-windows.md` / `scripts/manual-verify-windows.md` — NB-T3261 live steps
- `docs/live-probe-NB-T3261.md` — live read-only counts

## Library

Host (Tauri 2) links this crate as a **library** (not a long-running subprocess). CLI bin is separate for headless use.

```rust
use tokentracer_bridge::{discover_paths, config_from_fixture_root, list_files_for_source};
```

## macOS paths & FDA (AC-F10 / v1.2b)

| Path | Role |
|------|------|
| `~/Library/Application Support/Cursor/.../globalStorage/state.vscdb` | Cursor IDE DB (PARTIAL; **not** L1 billing) |
| `~/.cursor/` (e.g. `ai-tracking`) | Cursor agent-home enrichment (PARTIAL) |
| `~/.claude/projects` | Claude Code jsonl |
| `~/.codex/sessions` | Codex rollout jsonl |

- CONFIRMED\* = path-supported in bridge; **live Mac dump UNKNOWN** until a real Mac re-run.
- Empty/missing → diagnosable (`TT-F2-003` / Partial), never crash / silent blank.
- **Full Disk Access is not an install prerequisite.** On PermissionDenied, discover emits **`TT-F10-FDA`** (or `TT-F10-001`) with System Settings steps; Win/WSL keep `TT-F2-004`.
- No Keychain reads.

Design: [`docs/macos-f10-packaging-v0.md`](./docs/macos-f10-packaging-v0.md). Evidence (no Mac): [`../../evidence/pr18-macos-f10-20260911/`](../../evidence/pr18-macos-f10-20260911/).

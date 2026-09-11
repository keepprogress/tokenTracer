# BLOCKED — AC-F10 live Mac smoke (no Apple Silicon host)

| Field | Value |
|-------|-------|
| Date | 2026-09-11 (Asia/Taipei) |
| Role | 橋樑 |
| Branch | `feat/macos-f10-packaging` (stacked on `ui/macos-menubar-f10` @ b7c3205) |
| Verdict | **BLOCKED** — not PASS, not FAIL on live Mac gates |
| Self-PASS | **No** |

## Machine list (registered)

| machineId | label | connected | OS / role |
|-----------|-------|-----------|-----------|
| `e9a51208-fce4-4d9d-98a9-3d9895e24f61` | **NB-T3261** | true | **Windows** (Win tray / WSL host) |

**No macOS / Apple Silicon machine** is registered for this user. Linux agent box (`Debian 13`, `x86_64`) cannot run AppKit menu-bar smoke or produce a signed/usable real `.dmg`.

## What was attempted

1. `ListMachines` → only NB-T3261 (Windows).
2. Bridge unit/integration tests on Linux box: `cargo test -p tokentracer-bridge` (FDA mapping via chmod fixture, `~/.cursor` fake `$HOME`, missing-path diagnostics).
3. Confirmed `tauri.conf.json` already has `dmg` + `minimumSystemVersion: "13.0"` from 儀表 shell commit — **not** re-faked on Linux.
4. **Not** attempted: `cargo tauri build` `.dmg` on Linux; menu-bar click smoke; real System Settings FDA deny path on Mac.

## Packaging / diagnostics delivered without Mac (code + docs)

- TT-F10-FDA / TT-F10-001 + macOS PermissionDenied mapping
- `scan_macos` also probes `~/.cursor`
- Design note + README FDA / `.dmg` / arch policy
- FDA explicitly **not** install prerequisite

## Exact re-run when an Apple Silicon Mac is available

```bash
# 0) Register / connect the Mac in Cursor (ListMachines must show it).
# 1) Sync branch
git fetch && git checkout feat/macos-f10-packaging   # or ui/macos-menubar-f10 + packaging commits

# 2) Build menu-bar host + dmg (on the Mac)
rustup target add aarch64-apple-darwin
cargo build -p pricing --bin spend
npm --prefix apps/ui ci && npm --prefix apps/ui run build
cd apps/tokenTracer-host
cargo install tauri-cli --version "^2" --locked
cargo tauri build
# → target/release/bundle/dmg/*.dmg

# 3) .dmg drag install
# Open the .dmg → drag tokenTracer.app → /Applications → launch from Applications

# 4) Menu-bar smoke (儀表 AC-F7′ / F10 UI — 驗收官; bridge does not self-PASS)
# - Menu-bar icon appears (no Dock accessory)
# - Click → Expanded mini-panel; click/blur → icon only; Quit ends process

# 5) Discover paths (CLI)
cargo run -p tokentracer-bridge -- discover --json
# Expect sources under:
#   ~/.claude , ~/.codex/sessions , ~/Library/Application Support/Cursor/.../globalStorage
#   ~/.cursor/ai-tracking (enrichment; status partial — not L1 billing)

# 6) FDA deny → TT-F10 diagnostic (troubleshooting only)
# - System Settings → Privacy & Security → Full Disk Access → ensure tokenTracer is OFF
# - Optionally restrict a readable agent dir to force PermissionDenied
# - Re-run: cargo run -p tokentracer-bridge -- discover --json
# - Expect error.code TT-F10-FDA (or TT-F10-001) with next_step containing
#   System Settings → Privacy & Security → Full Disk Access …
#   and "not required for a normal install"
# - Enable FDA (or fix ACL) → re-run discover → diagnostic clears / sources readable
# - Confirm empty/missing agent trees emit TT-F2-003 (or Partial), never silent blank crash

# 7) Intel best-effort (optional; does not block PASS)
# rustup target add x86_64-apple-darwin
# cargo tauri build --target x86_64-apple-darwin
```

## Explicit non-claims

- No live Mac dump; CONFIRMED\* remains path-supported only.
- No Keychain reads; no Cursor bubble/`state.vscdb` as billing.
- Linux box `.dmg` = **not** produced; do not treat CI green as F10 PASS.

## Linux box test result (2026-09-11)

`cargo test -p tokentracer-bridge` — **40 passed** (unit + discovery_fixtures), including TT-F10 FDA mapping, missing-path diagnostics, and fake-$HOME `~/.cursor` scan.

This does **not** constitute live Mac / `.dmg` / menu-bar PASS.

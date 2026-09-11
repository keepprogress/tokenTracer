# AC-F7′ host linkage notes — for 儀表 (`apps/tokenTracer-host`)

| Field | Value |
|-------|-------|
| Owner | **儀表** owns Tauri tray UI/host at `apps/tokenTracer-host` (crate `tokentracer-host`) |
| Bridge | this crate `apps/bridge` → package **`tokentracer-bridge`** (lib + CLI bin) |
| Frontend | `apps/ui` (already in monorepo; embeds via Tauri `frontendDist` / Vite) |
| Contract | SHELL-HOST v0 / `design/ui/shell-host-contract-v0.md` |
| Status | **Assist** — host scaffold lives at `apps/tokenTracer-host`; `discover` wired in-process; no competing `apps/desktop` |

---

## 1. Workspace Cargo wiring (suggested)

Root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
  "crates/pricing",
  "apps/bridge",
  "apps/tokenTracer-host",   # 儀表 adds
]
```

`apps/tokenTracer-host/Cargo.toml` (crate name `tokentracer-host`):

```toml
[package]
name = "tokentracer-host"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
tokentracer-bridge = { path = "../bridge" }
# spend CLI stub path — configurable env / tauri.conf; do not hardcode WSL paths
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
# Tauri 2 + tray plugins (versions per Tauri 2 docs on Windows build host):
# tauri = { version = "2", features = ["tray-icon"] }
# tauri-plugin-shell = "2"   # only if spawning spend CLI; prefer lib link later

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

**Process model (locked):** single Tauri process; call `tokentracer_bridge::discover_paths` **in-process**. Do **not** spawn a resident `tokentracer-bridge` subprocess for discovery.

CLI remains available for headless/CI:

```bash
cargo run -p tokentracer-bridge -- discover --json
```

---

## 2. Bridge lib entry points (today)

| Need | Use |
|------|-----|
| Live discover | `tokentracer_bridge::discover_paths(&cfg)` after `apply_host_env_defaults(&mut cfg)` |
| Fixture discover | `discover_fixtures(fixture_root)` / `config_from_fixture_root` |
| Types | `DiscoverResult`, `DiscoverConfig`, `DiscoverSource`, … from `tokentracer_bridge` |
| Errors | `errors::make_error` / catalog TT-F2-\* |

`DiscoverConfig` defaults: set `windows_user_profile`, `wsl_enabled`, etc. via `apply_host_env_defaults` on Win.

ImportMeta **writer** = 帳本 only (see `host-shell.md` §3). Host must not rewrite ledger import-meta JSON itself.

---

## 3. IPC commands to expose (Tauri `invoke`)

Align names with `apps/ui/src/api.ts` / SHELL-HOST v0. Stub OK: shell out to configurable `spend` binary until lib-link.

| Command | Args | Stub → CLI / lib | Notes |
|---------|------|------------------|-------|
| `spend_total` | `range`, `currency` | `spend total --range … --currency …` | UI also uses by-pool shaped totals in live bridge |
| `spend_series` | `grain`=`day`, `range`, `currency` | `spend series --grain day …` | |
| `import_status` | — | `import status --json` / spend import-meta | |
| `spend_by_model` | `currency`, `range` | `spend by-model …` | |
| `spend_by_pool` | `currency`, `range` | `spend by-pool …` | |
| `discover` | — | **`tokentracer_bridge::discover_paths`** (lib) | **wired** in host `commands.rs` |
| `import_run` | TBD | stub → future ledger import; then refresh `import_status` | reserved |

**Spend binary path:** env `TOKENTRACER_SPEND_BIN` or host config key; default search: workspace `target/release/spend.exe` / `spend` next to host exe. Fail with clear error JSON if missing — do not invent prices.

`range`: `today` \| `all` \| `7d` \| `30d` \| `90d`.  
`grain`: v0 `"day"` only.

---

## 4. Win tray behavior checklist (README must pin)

- [ ] **Start:** system tray resident + **Collapsed** mini-panel visible (bottom-right / notification-area anchor)
- [ ] **Left-click tray:** toggle Collapsed ↔ Expanded (show/hide or resize panel)
- [ ] **Right-click tray menu:** Refresh / Import / Quit
  - Refresh → re-invoke spend_* (+ optional discover)
  - Import → `import_run` stub
  - Quit → exit host process
- [ ] Panel: always-on-top optional; no taskbar app button preferred (tool window)
- [ ] Embed `apps/ui` dist (or Vite URL in debug)
- [ ] macOS menu bar: **HOLD / TODO** (do not block Win tray)

---

## 5. Suggested `apps/tokenTracer-host` layout (儀表 creates)

```text
apps/tokenTracer-host/
  Cargo.toml                 # name = tokentracer-host
  tauri.conf.json            # Tauri 2
  capabilities/…
  icons/                     # tray + window
  src/
    main.rs                  # tray + window + invoke handlers
    ipc.rs                   # spend_* / discover / import_* stubs
    spend_cli.rs             # Command wrapper, TOKENTRACER_SPEND_BIN
  README.md                  # Windows run steps
```

Frontend: point `build.frontendDist` at `../ui/dist` (or `../ui` + `beforeDevCommand` / `beforeBuildCommand` for Vite).

---

## 6. Windows run steps (for host README)

Prereqs on NB-T3261 (or any Win build host):

1. **Rust** stable + MSVC toolchain
2. **WebView2** Evergreen Runtime
3. **Node** 20+ (build `apps/ui`)
4. **Tauri CLI 2:** `cargo install tauri-cli --version "^2"` (or `npm i -D @tauri-apps/cli@2`)

```powershell
# from monorepo root
cd apps\ui
npm ci
npm run build

cd ..\..\
cargo build -p pricing --release          # produces spend.exe
cargo build -p tokentracer-bridge --release

cd apps\tokenTracer-host
$env:TOKENTRACER_SPEND_BIN = "..\..\target\release\spend.exe"
cargo tauri dev     # or: cargo run
# release: cargo tauri build
```

Linux box note: full `tauri build` may lack WebView GTK deps — scaffold + `cargo check` of non-UI crates is enough; **real tray verify on Windows**.

---

## 7. Out of scope / blockers (do not block tray)

| Item | Owner |
|------|-------|
| Win-native import smoke | separate agent |
| macOS menu bar | HOLD |
| Competing `apps/desktop` | **do not create** |
| Claiming AC-F7′ PASS | forbidden until Win tray smoke |

**Blockers for 儀表 build:** WebView2 + `tauri-cli` 2 + Rust on Windows; optional Linux CI only checks compile of Rust IPC stubs without tray plugins if gated.

---

## 8. Handoff

Bridge crate is ready as a path dependency. 儀表: create `apps/tokenTracer-host`, wire workspace member, implement tray + IPC stubs per §§3–4. Bridge will not open a second host app.

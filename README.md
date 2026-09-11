# tokenTracer

Open-source **token usage & notional USD spend** tracker for coding agents on **Windows + WSL2 + macOS**.

> Notional estimates from public API list prices — **not** a subscription invoice or credit-card bill.

## Status

Early scaffold (AC v1.1–v1.3a + **v1.2／v1.2b macOS**). Pricing CLI + discovery bridge + mini-panel UI + **Tauri 2 host** (Win tray + **macOS menu bar**).

## Layout

- `crates/pricing` — Rust ledger / `spend` CLI (`total`, `by-model`, `by-pool`, `series`, `spending-align`)
- `apps/bridge` — Win/WSL/macOS discovery (`tokentracer-bridge` lib + CLI); **macOS paths per AC v1.2b** (unchanged)
- `apps/ui` — bottom-right / menu-bar mini panel (Vite); consumes ledger JSON via Tauri `invoke` or `/api/ipc`
- `apps/tokenTracer-host` — **Tauri 2 host** (`tokentracer-host`): Windows **system tray** + macOS **menu bar** mini-panel (AC-F7 / F7′ / **F10**)
- `specs/` — acceptance criteria
- `evidence/` — probe cards
- `fixtures/` — reproducible spend fixtures

> Path lock: host lives at **`apps/tokenTracer-host`** (not `apps/host`). Soft-depends on `tokentracer-bridge`.

## Windows tray（AC-F7）— 行為寫死

| 操作 | 行為 |
|------|------|
| **左鍵** | 切換 Collapsed ↔ Expanded（Hidden 時顯示面板） |
| **右鍵選單** | Show panel / Refresh / Import…（optional） / Quit |
| 關閉視窗 | 隱藏到托盤（常駐），不結束；結束用 Quit |
| 啟動預設 | 托盤常駐 + Collapsed 可見（右下／通知區附近） |

## macOS menu bar（AC-F7′／AC-F10）— 行為寫死

| 操作 | 行為 |
|------|------|
| **點選單列圖示** | 開 Expanded 迷你面板；再點或失焦 → 僅圖示 |
| **選單** | Show panel / Refresh / Import… / **Quit** |
| 啟動預設 | 僅選單列圖示（無 Dock）；`ActivationPolicy::Accessory` |
| 安裝 | **`.dmg` 拖曳安裝**；macOS 13+；Apple Silicon 必過；Intel best-effort |
| 空／PARTIAL | 必須可診斷（文案／banner），不得靜默空白 |

> Tauri 2 `tray-icon` on macOS = **menu-bar / NSStatusItem** equivalent (platform chrome). Same Collapsed／Expanded IA as Win; **not** full-page web (N5).

詳見 [`apps/tokenTracer-host/README.md`](apps/tokenTracer-host/README.md)。

全頁 localhost Vite UI 僅 **debug only**，不是主驗收面（N5）。

## Quick check

```bash
cargo test -p pricing
cargo run -p pricing --bin spend -- total --currency USD --events fixtures/ac-v1/events.json
```

### UI（debug / Vite）

```bash
npm --prefix apps/ui ci
npm --prefix apps/ui run dev    # http://127.0.0.1:5173 — /api/ipc → spend CLI
```

### Windows host（tray + mini-panel）

```powershell
cargo build -p pricing --bin spend
npm --prefix apps/ui ci && npm --prefix apps/ui run build
cd apps/tokenTracer-host
cargo install tauri-cli --version "^2" --locked   # once
cargo tauri dev    # or: cargo run -p tokentracer-host
```

### macOS host（menu bar + mini-panel + `.dmg`）

```bash
# Requires Xcode / CLT on a Mac (Apple Silicon preferred)
cargo build -p pricing --bin spend
npm --prefix apps/ui ci && npm --prefix apps/ui run build
cd apps/tokenTracer-host
cargo install tauri-cli --version "^2" --locked
cargo tauri dev
cargo tauri build   # → bundle/dmg/*.dmg
```

Linux agent box: `cargo check -p tokentracer-host` only as far as system libs allow — **no fake tray / menu-bar PASS screenshots**.

## License

MIT

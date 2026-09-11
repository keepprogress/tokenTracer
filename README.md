# tokenTracer

Open-source **token usage & notional USD spend** tracker for coding agents on **Windows + WSL2 + macOS**.

> Notional estimates from public API list prices — **not** a subscription invoice or credit-card bill.

## Status

Early scaffold (AC v1.1–v1.3a). Pricing CLI + discovery bridge + mini-panel UI + **Tauri 2 Win tray host**.

## Layout

- `crates/pricing` — Rust ledger / `spend` CLI (`total`, `by-model`, `by-pool`, `series`)
- `apps/bridge` — Win/WSL/macOS discovery (`tokentracer-bridge` lib + CLI)
- `apps/ui` — bottom-right / menu-bar mini panel (Vite); consumes ledger JSON via Tauri `invoke` or `/api/ipc`
- `apps/tokenTracer-host` — **Tauri 2 host** (`tokentracer-host`): Windows **system tray** + bottom-right mini-panel (AC-F7 / F7′)
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

詳見 [`apps/tokenTracer-host/README.md`](apps/tokenTracer-host/README.md)。macOS 選單列：**UNTESTED / later**。

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

Linux agent box: `cargo check -p tokentracer-host` only as far as system libs allow — **no fake tray PASS screenshots**.

## License

MIT

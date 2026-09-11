# tokenTracer

Open-source **token usage & notional USD spend** tracker for coding agents on **Windows + WSL2 + macOS**.

> Notional estimates from public API list prices — **not** a subscription invoice or credit-card bill.

## Status

Early scaffold (AC v1.1–v1.3a). Pricing CLI + discovery bridge + mini-panel UI mock.

## Layout

- `crates/pricing` — Rust ledger / `spend` CLI (`total`, `by-model`, `by-pool`, `series`)
- `apps/bridge` — Win/WSL/macOS discovery (`tokentracer-bridge`)
- `apps/ui` — bottom-right / menu-bar mini panel (Vite); consumes ledger JSON
- `specs/` — acceptance criteria
- `evidence/` — probe cards
- `fixtures/` — reproducible spend fixtures

## Quick check

```bash
cargo test -p pricing
cargo run -p pricing --bin spend -- total --currency USD --events fixtures/ac-v1/events.json
```

## License

MIT

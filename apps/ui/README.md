# tokenTracer mini-panel UI

Vite + TypeScript vanilla shell for the tokenTracer **mini-panel** (儀表).  
Consumes **real ledger CLI** `SpendSummary` / series JSON fixtures (via `refresh-from-ledger.sh`) — **UI does not price**.

Contracts: UI-IA v0, UI-BIND v0.3, SHELL-HOST v0.

## Refresh fixtures from ledger CLI

Auto-detects the cargo workspace (this repo at `/workspace/tokenTracer`, or set `LEDGER_ROOT`).

```bash
cd /workspace/tokenTracer/apps/ui && ./scripts/refresh-from-ledger.sh
# or the preview duplicate:
cd /workspace/tokenTracer-ui && ./scripts/refresh-from-ledger.sh
```

What it runs (cwd = ledger root):

```bash
cargo run -p pricing --bin spend -- by-model --currency USD \
  --events fixtures/ac-v1.3/by-model.json
cargo run -p pricing --bin spend -- by-pool --currency USD \
  --events fixtures/ac-v1.3a/cursor-pools.json
# for each kind in all,today,7d,30d,90d:
cargo run -p pricing --bin spend -- by-pool --currency USD \
  --events fixtures/ac-v1.3a/cursor-pools-ranged.json --range KIND
cargo run -p pricing --bin spend -- series --grain day --currency USD \
  --events fixtures/ac-v1.3a/cursor-pools-ranged.json --range KIND
```

Writes pure stdout JSON (human table on stderr is ignored) to:

| File | Source |
|------|--------|
| `src/mock/ledger-by-model.json` | by-model (multi-agent sample) |
| `src/mock/ledger-by-pool.json` | by-pool on `cursor-pools.json` (dual pools) |
| `src/mock/spend-total-{all,today,7d,30d,90d}.json` | real `by-pool --range <kind>` on `cursor-pools-ranged.json` |
| `src/mock/spend-series-{all,7d,30d,90d}.json` | real `series --grain day --range <kind>` (today skipped) |

No invented / scaled prices. Prefer ranged totals for range chips; `ledger-by-pool` is the same-day dual-pool sample.

## Preview / build

```bash
cd /workspace/tokenTracer/apps/ui && npm install && npm run dev
# preview duplicate tree:
cd /workspace/tokenTracer-ui && npm install && npm run dev
```

```bash
cd /workspace/tokenTracer/apps/ui && npm run build
```

Dev server: `http://127.0.0.1:5173/`  
Preview production build: `npm run preview`

## Features

- Collapsed ↔ Expanded toggle (click row / – / Esc)
- Today USD + range total USD (ledger fixtures)
- Range chips: All / 7d / 30d / 90d (re-fetches IPC; no local reprice)
- By-agent bars + `ok` / `partial` status
- By-model + Cursor usage pools (`cursor_models` / `other_models`) — no sentinel “Other model” rows
- Daily trend bars from real CLI series `points`
- `last_imported_at` shown in **Asia/Taipei** (never `computed_at`)
- Cursor **PARTIAL** warning banner
- Footer / disclaimer from ledger `disclaimer` + `pricing_mode`

## Mock IPC (`src/api.ts`)

Same command names as shell-host (plus ledger helpers):

| Command | Params | Returns |
|---------|--------|---------|
| `spend_total` | `range`, `currency` | `SpendSummary` (per-range CLI JSON) |
| `spend_by_model` | `currency` | `SpendSummary` (`ledger-by-model.json`) |
| `spend_by_pool` | `currency` | `SpendSummary` (`ledger-by-pool.json`) |
| `spend_series` | `grain`, `range`, `currency` | `DailySpendSeries` (per-range CLI JSON) |
| `import_status` | — | `ImportMeta` |

Fixtures live under `src/mock/*.json`. Mapping: `src/bind/mapToMiniPanelVM.ts`.

## Tray notes (host; not this webview)

- Win left-click: toggle Collapsed ↔ Expanded  
- Win right-click: refresh / import / quit  
- macOS: menu-bar icon opens dropdown panel  

When Tauri host lands, replace `src/api.ts` mocks with `invoke(...)`.

# tokenTracer mini-panel UI

Vite + TypeScript vanilla shell for the tokenTracer **mini-panel** (儀表).  
Consumes **real ledger CLI** `SpendSummary` JSON fixtures (via `refresh-from-ledger.sh`) — **UI does not price**.

Contracts: UI-IA v0, UI-BIND v0.3, SHELL-HOST v0.

## Refresh fixtures from ledger CLI

Requires the pricing crate at `/home/box/agent-data/projects/token-spend-tracker` (override with `LEDGER_ROOT`).

```bash
cd /workspace/tokenTracer-ui && ./scripts/refresh-from-ledger.sh
```

What it runs (cwd = token-spend-tracker):

```bash
cargo run -p pricing --bin spend -- by-model --currency USD --events fixtures/ac-v1.3/by-model.json
cargo run -p pricing --bin spend -- by-pool --currency USD --events fixtures/ac-v1.3a/cursor-pools.json
```

Writes pure stdout JSON (human table on stderr is ignored) to:

| File | Source |
|------|--------|
| `src/mock/ledger-by-model.json` | by-model (multi-agent sample) |
| `src/mock/ledger-by-pool.json` | by-pool (dual Cursor pools — **primary**) |
| `src/mock/spend-total-{all,today,7d,30d,90d}.json` | same as by-pool; only `range.kind` retagged |

**Note:** Range slicing awaits CLI `--range`. Until then all ranges share the same real totals (no invented scaled amounts). Prefer `ledger-by-pool` / cursor-pools for dual-pool UI; `ledger-by-model` is the multi-agent by-model sample.

## Preview / build

```bash
cd /workspace/tokenTracer-ui && npm install && npm run dev
```

```bash
cd /workspace/tokenTracer-ui && npm install && npm run build
```

Dev server: `http://127.0.0.1:5173/`  
Preview production build: `cd /workspace/tokenTracer-ui && npm run preview`

## Features

- Collapsed ↔ Expanded toggle (click row / – / Esc)
- Today USD + range total USD (ledger fixtures)
- Range chips: All / 7d / 30d / 90d (re-fetches IPC; no local reprice)
- By-agent bars + `ok` / `partial` status
- By-model + Cursor usage pools (`cursor_models` / `other_models`) — no sentinel “Other model” rows
- Daily trend bars from series `points` (series still mock until CLI series lands)
- `last_imported_at` shown in **Asia/Taipei** (never `computed_at`)
- Cursor **PARTIAL** warning banner
- Footer / disclaimer from ledger `disclaimer` + `pricing_mode`

## Mock IPC (`src/api.ts`)

Same command names as shell-host (plus ledger helpers):

| Command | Params | Returns |
|---------|--------|---------|
| `spend_total` | `range`, `currency` | `SpendSummary` (real by-pool retagged) |
| `spend_by_model` | `currency` | `SpendSummary` (`ledger-by-model.json`) |
| `spend_by_pool` | `currency` | `SpendSummary` (`ledger-by-pool.json`) |
| `spend_series` | `grain`, `range`, `currency` | `DailySpendSeries` (mock series) |
| `import_status` | — | `ImportMeta` |

Fixtures live under `src/mock/*.json`. Mapping: `src/bind/mapToMiniPanelVM.ts`.

## Tray notes (host; not this webview)

- Win left-click: toggle Collapsed ↔ Expanded  
- Win right-click: refresh / import / quit  
- macOS: menu-bar icon opens dropdown panel  

When Tauri host lands, replace `src/api.ts` mocks with `invoke(...)`.

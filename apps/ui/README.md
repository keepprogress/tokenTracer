# tokenTracer mini-panel UI

Vite + TypeScript vanilla shell for the tokenTracer **mini-panel** (儀表).  
**UI does not price** — it consumes `SpendSummary` / series / `ImportMeta` JSON from:

1. **Primary (dev / `vite preview`)**: live `spend` CLI via `/api/ipc/*` (Vite middleware in `scripts/spend-dev-bridge.mjs`)
2. **Fallback**: bundled `src/mock/*.json` from `refresh-from-ledger.sh` when the CLI bridge is unavailable

Contracts: UI-IA v0.3, UI-BIND v0.3, SHELL-HOST v0.

## Preview / build (live CLI path)

From the monorepo root, ensure the pricing binary exists (once):

```bash
cd /workspace/tokenTracer-main
cargo build -p pricing --bin spend
```

Then:

```bash
cd /workspace/tokenTracer-main/apps/ui
npm install
npm run dev          # http://127.0.0.1:5173/  — /api/ipc → spend CLI
# or production bundle + preview (bridge still mounted):
npm run build && npm run preview   # http://127.0.0.1:4173/
```

Force fixture-only (skip CLI):

```bash
TOKENTRACER_FORCE_FIXTURES=1 npm run dev
```

Probe the bridge:

```bash
curl -s http://127.0.0.1:5173/api/ipc/_meta | jq .
curl -s 'http://127.0.0.1:5173/api/ipc/spend_total?range=7d&currency=USD' | jq '{total,pricing_mode,range}'
```

## IPC surface (`src/api.ts`)

Same command names as shell-host (plus ledger helpers). Live requests hit `/api/ipc/<command>`:

| Command | Params | CLI |
|---------|--------|-----|
| `spend_total` | `range`, `currency` | `spend by-pool --range … --currency …` on `cursor-pools-ranged.json` |
| `spend_series` | `grain`, `range`, `currency` | `spend series --grain day --range …` |
| `spend_by_model` | `currency`, `range?` | `spend by-model …` on `by-model.json` |
| `spend_by_pool` | `currency`, `range?` | `spend by-pool …` (`cursor-pools.json` when `all`) |
| `import_status` | — | `spend import status --json` |

`dataSource()` returns `"cli"` \| `"fixture"` after the last successful call. Footer copy reflects the active path.

Mapping: `src/bind/mapToMiniPanelVM.ts` (no sentinel `"Other"` model rows — usage_pool groups only).

## Refresh static fixtures from ledger CLI

Still useful for offline / Tauri-embed fallback:

```bash
cd /workspace/tokenTracer-main/apps/ui && ./scripts/refresh-from-ledger.sh
# or: npm run refresh-fixtures
```

Writes:

| File | Source |
|------|--------|
| `src/mock/ledger-by-model.json` | by-model (multi-agent sample) |
| `src/mock/ledger-by-pool.json` | by-pool on `cursor-pools.json` (dual pools) |
| `src/mock/spend-total-{all,today,7d,30d,90d}.json` | `by-pool --range <kind>` on `cursor-pools-ranged.json` |
| `src/mock/spend-series-{all,7d,30d,90d}.json` | `series --grain day --range <kind>` |

## Env (bridge)

| Var | Purpose |
|-----|---------|
| `LEDGER_ROOT` / `TOKENTRACER_LEDGER_ROOT` | Cargo workspace root (default: repo root) |
| `TOKENTRACER_SPEND_BIN` | Explicit path to `spend` binary |
| `TOKENTRACER_EVENTS_RANGED` | Override ranged events fixture |
| `TOKENTRACER_EVENTS_POOL` | Override dual-pool fixture |
| `TOKENTRACER_EVENTS_MODEL` | Override by-model fixture |
| `TOKENTRACER_IMPORT_STATE` | Import-meta state path |
| `TOKENTRACER_FORCE_FIXTURES=1` | Bridge returns 503 → UI fixtures |

## Features

- Collapsed ↔ Expanded toggle (click row / – / Esc)
- Today USD + range total USD from live CLI (or fixtures)
- Range chips: All / 7d / 30d / 90d (re-fetches IPC; no local reprice)
- By-agent bars + `ok` / `partial` status
- By-model + Cursor usage pools (`cursor_models` / `other_models`) — no sentinel “Other model” rows
- Daily trend bars from CLI series `points`
- `last_imported_at` shown in **Asia/Taipei** (never `computed_at`)
- Cursor **PARTIAL** warning banner
- Footer / disclaimer from ledger `disclaimer` + `pricing_mode`

## Tray notes (host; not this webview)

- Win left-click: toggle Collapsed ↔ Expanded  
- Win right-click: refresh / import / quit  
- macOS: menu-bar icon opens dropdown panel  

When Tauri host lands, replace `/api/ipc` fetches in `src/api.ts` with `invoke(...)`.

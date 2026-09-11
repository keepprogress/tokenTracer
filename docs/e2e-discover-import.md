# E2E: discover → import from-discover → spend (POC)

**Status:** local smoke composition — **not a PASS**.  
**Contract:** path-list-v0.2  
**Date:** 2026-09-11 (Asia/Taipei)

Composes **橋樑** `discover` with **帳本** `spend import from-discover` (sole ImportMeta writer), then `total` + `import status`.

## How to run

```bash
# From monorepo
/workspace/tokenTracer/scripts/e2e-discover-import.sh

# Or from bridge checkout
/workspace/tokenTracer-bridge/scripts/e2e-discover-import.sh
```

Env overrides: `BRIDGE_ROOT`, `PRICING_ROOT` (default `/home/box/agent-data/projects/token-spend-tracker`), `FIXTURE_DIR`, `WORKDIR`, `KEEP_WORKDIR=1`, `FORCE_CARGO_RUN=1`.

## Authoritative manual pipeline

```bash
cd /workspace/tokenTracer-bridge
cargo run -q -- discover --json --fixture tests/fixtures > /tmp/discover.json

cd /home/box/agent-data/projects/token-spend-tracker
cargo run -q -p pricing --bin spend -- import from-discover \
  --discover /tmp/discover.json \
  --limit 0 \
  --state /tmp/tt-import-meta.json \
  --events-out /tmp/tt-events.json \
  --report-out /tmp/tt-import-report.json \
  --json

cargo run -q -p pricing --bin spend -- total --currency USD --events /tmp/tt-events.json
cargo run -q -p pricing --bin spend -- import status --state /tmp/tt-import-meta.json --json
```

## path-list-v0.2 / 帳本 alignment

| Rule | Behavior |
|------|----------|
| Discover without `files[]` | `discover --json` (no `--list-files`) |
| Expand / import | `spend import from-discover --limit 0` |
| Source filter | `readable && status ∈ {ok, partial}`; prefer `meta.import_path` |
| Truncated / TT-F2-006 | `from-discover` exits **2** → script fails |
| Cursor local billing | Skipped (`cursor_sources_skipped` in ImportReport) |
| ImportMeta writer | **pricing only** |

## Fixture smoke excerpt (local, 2026-09-11)

```text
events_upserted: 5
SpendSummary.total ≈ 0.000563 USD  (notional_api_estimate, table 2026-09-11.v2)
by_agent: claude_code ≈ 0.000563; codex ≈ 0.0  (tiny stub tokens)
cursor_sources_skipped: 3
ImportMeta.schema_version: import-meta/v0
```

Script exits **0** on successful smoke. That is **not** an AC PASS.

## Open gaps (帳本 / 指揮官)

1. Durable event ledger beyond ImportMeta + events-out file.
2. Live Windows dual-scan (fixtures use `import_path`; real host still needs bridge access_exec/UNC).
3. Cursor vendor-reported / billing path still out of scope for this smoke.
4. Keep `PRICING_ROOT` on agent-data until monorepo crates stay in sync with `from-discover`.
5. Commander: commit/push paths listed in the success report — this POC does not push.

## Files

- `scripts/e2e-discover-import.sh` (also mirrored under `tokenTracer-bridge/scripts/`)
- `docs/e2e-discover-import.md` (also mirrored under `tokenTracer-bridge/docs/`)

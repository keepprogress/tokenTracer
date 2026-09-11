# pricing — tokenTracer spend core

Rust library + `spend` CLI for normalized `UsageEvent` → notional USD spend.

## Build / test

```bash
cd /home/box/agent-data/projects/token-spend-tracker
cargo test -p pricing
```

## CLI examples

```bash
# Historical total (AC-F4 fixture)
cargo run -p pricing --bin spend -- total --currency USD \
  --events ../../fixtures/ac-v1/events.json

# Today (OPEN-UI-1); day boundary default = local Asia/Taipei
cargo run -p pricing --bin spend -- total --currency USD --range today \
  --events ../../fixtures/ac-v1/events.json
# alias:
cargo run -p pricing --bin spend -- today --currency USD \
  --events ../../fixtures/ac-v1/events.json

# Daily series (OPEN-UI-2)
cargo run -p pricing --bin spend -- series --grain day --range 30d --currency USD \
  --events ../../fixtures/ac-v1/events.json
cargo run -p pricing --bin spend -- series --grain day \
  --from 2026-09-09 --to 2026-09-11 --currency USD \
  --events ../../fixtures/ac-v1/events.json

# Import freshness (OPEN-UI-3) — separate from SpendSummary
cargo run -p pricing --bin spend -- import run --events-upserted 3 \
  --source-id EC-claude-code-v1 --source-id EC-codex-v1 --json
cargo run -p pricing --bin spend -- import status --json

# Parse smoke
cargo run -p pricing --bin spend -- parse --agent claude_code \
  --file ../../fixtures/ac-v1/claude-sample.jsonl
cargo run -p pricing --bin spend -- parse --agent codex \
  --file ../../fixtures/ac-v1/codex-sample.jsonl
```

Paths above assume cwd = `crates/pricing`. From workspace root use
`fixtures/ac-v1/events.json` instead.



## Import from bridge discover (path-list-v0.2)

E2E POC with `tokentracer-bridge` fixtures:

```bash
# 1) Emit DiscoverResult
cd /workspace/tokenTracer-bridge
cargo run -q -- discover --json --fixture tests/fixtures > /tmp/discover.json

# 2) Import into ledger events + ImportMeta
cd /home/box/agent-data/projects/token-spend-tracker
cargo run -q -p pricing --bin spend -- import from-discover \
  --discover /tmp/discover.json \
  --limit 0 \
  --state /tmp/tt-import-meta.json \
  --events-out /tmp/tt-events.json \
  --report-out /tmp/tt-import-report.json \
  --json
```

CLI flags:

| Flag | Meaning |
|------|---------|
| `--discover` | Path to DiscoverResult JSON |
| `--limit` | Local expand / files cap; **`0` = unlimited** (use for full import) |
| `--state` | ImportMeta state file (default `.token-tracer/import-meta.json`) |
| `--events-out` | Write UsageEvent JSON array |
| `--report-out` | Write ImportReport JSON |
| `--json` | Print ImportReport to stdout |

Behavior notes:

- Walks `sources` where `readable && status ∈ {ok, partial}`.
- Truncated `files[]` (`truncated: true` / **TT-F2-006**) is **never** treated as complete — raise `--limit` or use `--limit 0`.
- `claude_code` / `codex` → existing JSONL parsers; `UsageEvent.source = source.id`, `meta.host_os = source.host`.
- `cursor` → skipped for local billing (`cursor_skipped_local_billing`); no bubble `tokenCount` events.
- WSL roots: prefer `meta.import_path` / `meta.posix_path`, else strip `wsl:<Distro>:` on Linux hosts.

## Price table sources

Embedded version `2026-09-11.v2` (`data/price_table_2026-09-11.v2.json`):

- Anthropic: https://platform.claude.com/docs/en/about-claude/pricing
- OpenAI: https://openai.com/api/pricing/ (also developers.openai.com)

**Notional vs invoice:** Claude Code / Codex local JSONL has no vendor `$`.
Totals are **notional API estimates** (`cost_nature=notional_api_estimate`),
not subscription invoices. Do not treat as card charges.

## Cursor PARTIAL

Local bubble `tokenCount` is **unreliable** (`EC-cursor-v1`). The
`parse_cursor_local` stub refuses bubble billing and only accepts empty input
or explicit `meta.cost_nature=vendor_reported` aggregates. `by_agent.status`
for cursor is always `partial`.

## Pure API

```rust
price(events, &price_table, fx?) -> SpendSummary
filter_events_by_range(events, &RangeFilterOpts) -> (Vec<UsageEvent>, ResolvedWindow)
daily_spend_series(events, &price_table, fx?, &SeriesOpts) -> DailySpendSeries
```

No hidden globals; pricing / range / series do not touch the filesystem.
`import_meta` is the I/O edge for freshness state.


## AC v1.3 / v1.3a — by_model + usage_pool

```bash
cargo run -p pricing --bin spend -- by-model --currency USD \
  --events ../../fixtures/ac-v1.3/by-model.json
cargo run -p pricing --bin spend -- by-pool --currency USD \
  --events ../../fixtures/ac-v1.3a/cursor-pools.json
```

- `by_model[].model` = concrete ids only (or `cursor:unknown` when missing). **Never** `__other__` / literal `Other`.
- `usage_pool` = `cursor_models` | `other_models` | `unknown` (orthogonal to model id).
- `pricing_mode=notional_api_estimate` + `disclaimer` on every SpendSummary.
- Cursor `tier` map is **PARTIAL** (1→other_models, 2→cursor_models) per EC-cursor-other-models-v1.

## AC v1.4 — SpendingAlign CLI (OPEN-BIND)

Comparison layer only (not UsageEvent / not notional). Default state `.token-tracer/spending-align.json`.

```bash
cargo run -p pricing --bin spend -- spending-align --json
cargo run -p pricing --bin spend -- spending-align set \
  --json ../../fixtures/ac-v1.4/spending-align-manual-p1.json
```

Missing state → `source_mode=none` placeholder. `set` validates `manual_p1` and stamps `computed_at`. Never derive pct from `by_usage_pool.amount` or Admin cents. See `design/ledger/open-bind-spending-align-v0.md`.

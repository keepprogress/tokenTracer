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

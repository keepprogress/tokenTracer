# fixtures/ac-v1 — reproducible spend totals

Hand-crafted fixtures (no real credentials). Price table: `2026-09-11.v2`.

## Rates used (USD per 1M tokens)

| Model | input | output | cache_read | cache_write_5m |
|-------|------:|-------:|-----------:|---------------:|
| `claude-sonnet-4-20250514` | 3.00 | 15.00 | 0.30 | 3.75 |
| `gpt-5-codex` | 1.25 | 10.00 | 0.125 | — |

Formula per event:

```
(input * in_rate + output * out_rate + cache_read * cr_rate + cache_write * cw_rate) / 1_000_000
```

## Claude (`claude-sample.jsonl`)

Two assistant turns after dedup by `message.id` (duplicate streaming line dropped).

### Event A — `msg_dedup_test`

| bucket | tokens | rate | USD |
|--------|-------:|-----:|----:|
| input | 10_000 | 3.00 | 10_000 × 3 / 1e6 = **0.030** |
| output | 2_000 | 15.00 | 2_000 × 15 / 1e6 = **0.030** |
| cache_read | 100_000 | 0.30 | 100_000 × 0.30 / 1e6 = **0.030** |
| cache_write | 8_000 | 3.75 | 8_000 × 3.75 / 1e6 = **0.030** |
| **subtotal A** | | | **0.120** |

### Event B — `msg_second`

| bucket | tokens | rate | USD |
|--------|-------:|-----:|----:|
| input | 5_000 | 3.00 | **0.015** |
| output | 1_000 | 15.00 | **0.015** |
| cache_read / write | 0 | — | **0** |
| **subtotal B** | | | **0.030** |

**Claude agent total = 0.120 + 0.030 = 0.150 USD**

## Codex (`codex-sample.jsonl`)

- One `token_usage_record` counted.
- Companion `event_msg` / `token_count` **not** counted (no double-count).
- Model from preceding `turn_context`: `gpt-5-codex`.
- `input_tokens` stored = `20000 − 8000 = 12000` (CodexScope / non-cached).
- `cache_read_tokens = 8000`, `cache_write_tokens = 0`, `output_tokens = 400`.

| bucket | tokens | rate | USD |
|--------|-------:|-----:|----:|
| input (non-cached) | 12_000 | 1.25 | 12_000 × 1.25 / 1e6 = **0.015** |
| output | 400 | 10.00 | 400 × 10 / 1e6 = **0.004** |
| cache_read | 8_000 | 0.125 | 8_000 × 0.125 / 1e6 = **0.001** |
| cache_write | 0 | — | **0** |
| **Codex agent total** | | | **0.020** |

## Grand total

```
0.150 (claude_code) + 0.020 (codex) = 0.170 USD
```

Expected file: `total-spend.expected.json` (also copied as `total-spend.json` for AC-F4 naming).
Tolerance: absolute ≤ $0.01.

## Cursor

No cursor events in this fixture. Local bubble `tokenCount` is **forbidden** as billed usage (`EC-cursor-v1`). Parser stub returns empty / error; `by_agent.status` for cursor is `partial` when present.

## AC path note

Primary fixtures live under `fixtures/ac-v1/`. Symlink `fixtures/ac-v1.1` → `ac-v1` for AC-F4 path mentions.

## OPEN-UI today / series

Fixture event timestamps are all on **2026-09-11** (UTC mornings → still 2026-09-11 in Asia/Taipei).

With `--day-boundary local --timezone Asia/Taipei` and `now` on 2026-09-11:

- `spend total --range today` → **0.170 USD** (same as all for this fixture)
- `spend series --from 2026-09-09 --to 2026-09-11` → points
  - 2026-09-09: 0
  - 2026-09-10: 0
  - 2026-09-11: 0.170
  - Σ points = spend total for the same window


## by_model arithmetic (AC v1.3 / v1.3a)

Same events → `SpendSummary.by_model` rows keyed by concrete model id (not agent-only):

| model | agent | amount (USD) | priced |
|-------|-------|-------------:|:------:|
| `claude-sonnet-4-20250514` | claude_code | 0.150 | yes |
| `gpt-5-codex` | codex | 0.020 | yes |

Invariant: `Σ by_model.amount where priced ≈ total ≈ Σ by_agent.amount` (≤ $0.01).

Missing model uses `cursor:unknown` in by_model **only as unknown label** — never `__other__`.
Cursor dual pool uses `usage_pool` (`cursor_models` / `other_models` / `unknown`), orthogonal to model id.
Price table version: `2026-09-11.v2`.

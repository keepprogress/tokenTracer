# fixtures/ac-v1.3a — Cursor dual pool (AC-F13′)

Synthetic vendor-reported Cursor aggregates aligned with `EC-cursor-other-models-v1`.

| model | usage_pool | tier (PARTIAL) |
|-------|------------|----------------|
| `composer-1.5` | `cursor_models` | 2 |
| `claude-4.6-opus-high-thinking` | `other_models` | 1 |
| `grok-4.6` | `other_models` | 1 |

**Forbidden:** literal model `"Other"` / `"Other model"` / `__other__` / `cursor:other`.

## Files

| file | purpose |
|------|---------|
| `cursor-pools.json` | Same-day dual-pool sample (by-pool / by-model smoke). |
| `cursor-pools-ranged.json` | Same models / pools / token fields, timestamps spanning ~100d / ~45d / ~10d / today so CLI `--range all\|today\|7d\|30d\|90d` yields different SpendSummary / series. UI mock refresh uses this file. CLI prices events — do not invent unit prices. |

Verify:

```bash
cargo run -p pricing --bin spend -- by-pool --currency USD \
  --events fixtures/ac-v1.3a/cursor-pools.json
cargo run -p pricing --bin spend -- by-model --currency USD \
  --events fixtures/ac-v1.3a/cursor-pools.json
cargo run -p pricing --bin spend -- by-pool --currency USD \
  --events fixtures/ac-v1.3a/cursor-pools-ranged.json --range 90d
cargo run -p pricing --bin spend -- series --grain day --currency USD \
  --events fixtures/ac-v1.3a/cursor-pools-ranged.json --range 7d
```

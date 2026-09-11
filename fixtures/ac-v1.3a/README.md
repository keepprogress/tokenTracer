# fixtures/ac-v1.3a — Cursor dual pool (AC-F13′)

Synthetic vendor-reported Cursor aggregates aligned with `EC-cursor-other-models-v1`.

| model | usage_pool | tier (PARTIAL) |
|-------|------------|----------------|
| `composer-1.5` | `cursor_models` | 2 |
| `claude-4.6-opus-high-thinking` | `other_models` | 1 |
| `grok-4.6` | `other_models` | 1 |

**Forbidden:** literal model `"Other"` / `"Other model"` / `__other__` / `cursor:other`.

Verify:

```bash
cargo run -p pricing --bin spend -- by-pool --currency USD \
  --events fixtures/ac-v1.3a/cursor-pools.json
cargo run -p pricing --bin spend -- by-model --currency USD \
  --events fixtures/ac-v1.3a/cursor-pools.json
```

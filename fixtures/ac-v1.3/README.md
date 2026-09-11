# fixtures/ac-v1.3 — by_model (AC-F11 / v1.3a aligned)

Events for `spend by-model`. Includes:

- ≥2 concrete model ids (`claude-sonnet-4-20250514`, `gpt-5-codex`)
- ≥1 Cursor row with `usage_pool=other_models` and concrete `model=claude-4.6-opus-high-thinking`
- **No** `model=cursor:other`, **no** `__other__`, **no** literal `"Other"` model

## by_model arithmetic

- Claude / Codex rows: notional API list prices (same as `fixtures/ac-v1/`).
- Cursor row: `vendor_reported` `raw_cost_usd` (unpriced in price table → still priced via vendor).
- `Σ by_model.amount where priced ≈ total` (≤ $0.01).
- `usage_pool` is orthogonal to `model` id.

Price table: embedded `2026-09-11.v2`.

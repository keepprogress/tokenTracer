# fixtures/ac-v1.4 — Cursor official Admin reconcile (AC-F14)

Synthetic Team Admin API samples aligned with `EC-cursor-official-api-v1` / `EC-cursor-spending-align-v1` (S1/S2).
**No live network.** No secrets / Admin keys in fixtures.

## Arithmetic (F14)

| Event | model | tier → usage_pool | chargedCents |
|-------|-------|-------------------|--------------|
| 1 | `claude-4.5-sonnet` | 1 → `other_models` | 1234 |
| 2 | `composer-1.5` | 2 → `cursor_models` | 567 |
| 3 | `grok-4.6` | 1 → `other_models` | 199 |
| **Σ** | | | **2000** |

| Spend field | value |
|-------------|-------|
| member A `spendCents` | 1500 |
| member B `spendCents` | 500 |
| **`overallSpendCents`** | **2000** |

```
events_charged_usd  = Σ chargedCents / 100 = 2000 / 100 = $20.00
spend_overall_usd   = overallSpendCents / 100 = 2000 / 100 = $20.00
delta_usd           = |20.00 − 20.00| = $0.00
tolerance_usd       = $0.01
matched             = true
```

Mapping rules (AC-F14 / F19):

- Concrete model ids only — **never** literal `"Other"` / `"Other model"`.
- `raw_cost_usd = chargedCents / 100`
- `meta.cost_nature = vendor_reported`
- `meta.cursor_source_mode = official_admin`
- `usage_pool` from `tier` when present (PARTIAL community map)

## Credential contract (align bridge)

Live Admin fetch (optional; not required for tests) uses env
`TOKENTRACER_CURSOR_ADMIN_API_KEY` (same as bridge `cursor_admin`).
Pricing/bridge do **not** invent secret storage. Missing key → clear error
(`TT-C14-MISSING-KEY` semantics); fixture reconcile needs no key.

## Verify (fixture-only)

```bash
cargo run -p pricing --bin spend -- cursor official-reconcile \
  --events fixtures/ac-v1.4/admin-events.json \
  --spend fixtures/ac-v1.4/admin-spend.json

cargo test -p pricing --test ac_v1_4_cursor_official
```

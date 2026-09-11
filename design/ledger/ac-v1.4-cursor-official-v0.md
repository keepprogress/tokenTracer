# Design: AC v1.4 Cursor official / Spending align (ledger) v0

| Field | Value |
|-------|-------|
| Spec | `specs/AC-v1.4-cursor-official.md` |
| Evidence | `EC-cursor-official-api-v1`, `EC-cursor-spending-align-v1` (S1–G1) |
| Scope | pricing / ledger only — **not UI, not PASS** |
| Date | 2026-09-11 (Asia/Taipei) |

## 1. Source modes

| Mode | Meaning | Default |
|------|---------|---------|
| `official_admin` | Team/Enterprise Admin → S1 `filtered-usage-events` + S2 `/teams/spend` | On when Admin key present at call time |
| `local_enrichment` | Local DB / bubble / context (L1) | Allowed; **must not** be sole Cursor USD / invoice / pool % |
| `undocumented_dashboard` | U1/U2 (`GetCurrentPeriodUsage`, cookie REST, …) | **`false`** |

Enum: `CursorSourceMode` in `pricing::models`.

## 2. Data shapes

### 2.1 Official Admin → `UsageEvent` (F14 / F19)

From S1 event fields:

| Source field | UsageEvent |
|--------------|------------|
| `model` | concrete model id (never literal `"Other"`) |
| `tokenUsage.inputTokens` … | token fields |
| `chargedCents / 100` | `raw_cost_usd` |
| `tier` (if present) | `usage_pool` via PARTIAL map (1→other_models, 2→cursor_models) |
| — | `agent=cursor`, `source` tag `EC-cursor-official-api-v1` |
| — | `meta.cost_nature=vendor_reported` |
| — | `meta.cursor_source_mode=official_admin` |

### 2.2 `SpendSnapshot` (S2)

Normalized from `/teams/spend`: member rows + `overall_spend_cents` + cycle start.

### 2.3 `ReconcileReport` (F14)

`Σ chargedCents` vs spend overall; abs tolerance **$0.01**.

### 2.4 `SpendingAlignSummary` (F17) — **comparison layer, not UsageEvent**

`cursor_models_percent`, `other_models_percent`, `reset_note`, `on_demand`, partial flags.
Personal Ultra align target = Spending UI (P1); no public personal usage API.

### 2.5 Config

```json
{ "undocumented_dashboard": false }
```

Default **false**. Opt-in outputs must say UNSUPPORTED/undocumented; never claim `official_admin`.

## 3. Pure functions (fixture-first)

```text
parse_admin_filtered_usage_events(json) -> Vec<UsageEvent>
parse_teams_spend(json) -> SpendSnapshot
reconcile_charged_cents(events, spend) -> ReconcileReport
```

Optional HTTP client behind trait / feature — **tests use fixtures only**.

## 4. CLI

```bash
spend cursor official-reconcile \
  --events fixtures/ac-v1.4/admin-events.json \
  --spend  fixtures/ac-v1.4/admin-spend.json
```

Live fetch (optional): requires `CURSOR_ADMIN_API_KEY` env (Basic username).
**No secret storage** — same posture as bridge (do not copy credentials into state).
Missing key → **clear error**, no silent L1 fallback.

## 5. Fixtures

See `fixtures/ac-v1.4/README.md` for Σ arithmetic (2000¢ ↔ 2000¢).

## 6. Local enrichment guard (F15)

When Cursor events are **not** `official_admin` / `vendor_reported`:

- `by_agent` Cursor status = `partial`
- Strengthen disclaimer for cursor-only-local (not sole authoritative invoice USD)
- Existing stub keeps refusing bubble `tokenCount` billing

## 7. Non-goals

- Invented API prices for unknown models
- Bubble billing
- Claiming PASS / UI work
- `undocumented_dashboard` default on

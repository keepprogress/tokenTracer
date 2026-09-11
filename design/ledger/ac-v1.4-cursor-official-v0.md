# Design: AC v1.4 Cursor official / Spending align (ledger) v0

| Field | Value |
|-------|-------|
| Spec | `specs/AC-v1.4-cursor-official.md` |
| Evidence | `EC-cursor-official-api-v1`, `EC-cursor-spending-align-v1` (S1–G1) |
| Scope | pricing / ledger only — **not UI, not PASS** |
| Date | 2026-09-11 (Asia/Taipei) |
| Bridge align | `apps/bridge/docs/cursor-official-admin-v0.md` (`TOKENTRACER_CURSOR_ADMIN_API_KEY`) |

## 1. Source modes

| Mode | Meaning | Default |
|------|---------|---------|
| `official_admin` | Team/Enterprise Admin → S1 + S2 | On when Admin key present at call time |
| `local_enrichment` | Local DB / bubble / context (L1) | Allowed; **must not** be sole Cursor USD / invoice / pool % |
| `undocumented_dashboard` | U1/U2 | **`false` / off** |

## 2. Data shapes

### Official Admin → `UsageEvent` (F14 / F19)

`raw_cost_usd = chargedCents/100`; `meta.cost_nature=vendor_reported`;
`meta.cursor_source_mode=official_admin`; concrete model ids; `usage_pool` from tier.

### `SpendSnapshot` / `ReconcileReport` (F14)

Σ chargedCents ↔ spend overall; abs tol **$0.01**.

### `SpendingAlignSummary` (F17) — comparison layer, **not** UsageEvent

`cursor_models_percent`, `other_models_percent`, `reset_note`, `on_demand`, partial flags.

### Config

`undocumented_dashboard: bool` default **false**.

## 3. Pure functions (fixture-first)

```
parse_admin_filtered_usage_events(json) -> Vec<UsageEvent>
parse_teams_spend(json) -> SpendSnapshot
reconcile_charged_cents(events, spend) -> ReconcileReport
```

Optional HTTP behind trait; tests use fixtures only.

## 4. CLI

```bash
spend cursor official-reconcile --events fixtures/ac-v1.4/admin-events.json \
  --spend fixtures/ac-v1.4/admin-spend.json
spend cursor official-fetch-check   # clear error if no Admin key
```

Credential: env `TOKENTRACER_CURSOR_ADMIN_API_KEY` (bridge-aligned). No secret store.

## 5. F15 local enrichment guard

Non-official_admin / non-vendor_reported Cursor → `by_agent` Partial + strengthened disclaimer;
not sole authoritative invoice USD. Bubble tokenCount still refused.

## 6. Non-goals

Invented prices; bubble billing; self-PASS; undocumented default on.

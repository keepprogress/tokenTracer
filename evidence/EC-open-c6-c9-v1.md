# Evidence Card: OPEN-C6…C9（規格派 AC-v1.4 未解項）

| Field | Value |
|-------|-------|
| Evidence ID | `EC-open-c6-c9-v1` |
| Date | 2026-09-11 (Asia/Taipei) |
| Trigger | 規格派 OPEN-C6…C9 |
| Probe | 探針（tokenTracer） |
| Note | **Does NOT amend locked AC v1.4 body** — evidence only for 規格 handoff |
| Live Admin key test | **Not run** this probe (no prior recorded Teams 403 sample invented) |

**Label legend:** CONFIRMED / PARTIAL / UNDOCUMENTED / UNKNOWN / BLOCK

---

## OPEN-C6 — Teams ≠ Enterprise Admin API availability boundary

### Question
Teams≠Enterprise Admin API availability boundary (403 / Enterprise access required) — docs say Enterprise; community sometimes says Team admin.

### Findings

| Claim | Status | Evidence |
|-------|--------|----------|
| API Overview Availability table: Admin API = **Enterprise teams** | **CONFIRMED** | [cursor.com/docs/api](https://cursor.com/docs/api) Availability table (fetched 2026-09-11) |
| Same Overview documents 403 body `{"error":"Forbidden","message":"Enterprise access required"}` for valid key + insufficient plan | **CONFIRMED** | docs/api Common Error Responses 403 + Best Practices snippet treating 403 as Enterprise |
| 403 prose: “Enterprise features on non-Enterprise plan” | **CONFIRMED** | docs/api |
| Several Admin write endpoints marked **Availability: Enterprise only** (e.g. user-spend-limit, remove-member; bulk spend-limits notes 403 if not enabled) | **CONFIRMED** | [Admin API](https://cursor.com/docs/account/teams/admin-api) |
| **Teams pricing** page lists feature: usage stats “also available via the **Admin API**” under Team plans (Teams vs Enterprise section) | **CONFIRMED** (docs contradiction vs Overview) | [Team Pricing](https://cursor.com/docs/account/teams/pricing) — “Centralized team billing and administration, with usage stats also available via the Admin API” |
| DigiUsher (third-party) onboarding: requires Cursor **Team plan** + Team Admin role; “Individual plans… give no access”; does **not** say Enterprise-only | **PARTIAL** (community / vendor docs; not Cursor contract) | [docs.digiusher.com/connecting-cursor](https://docs.digiusher.com/connecting-cursor) |
| Forum “Admin API key is not working” (2025-06): reporter had team admin; got **401 Invalid API key** (auth wiring), staff fixed Basic auth — **not** a recorded Teams→403 Enterprise sample | **PARTIAL** (wrong failure mode for C6) | [forum #109221](https://forum.cursor.com/t/admin-api-key-is-not-working/109221) |
| Live Teams (non-Enterprise) Admin key → 403 Enterprise access required | **UNKNOWN** this run | No live key test; no prior recorded 403 transcript in evidence corpus |

### Confidence
**PARTIAL** overall: official **Overview** strongly Enterprise; official **Teams pricing** text conflicts; vendor DigiUsher implies Team admin suffices; **no** first-party live 403 sample for Teams-only.

### Recommendation to 規格
**HOLD / still OPEN** until a Teams Standard/Premium Admin key is exercised and response body recorded. Spec may cite Overview Availability + 403 message as expected Enterprise gate, and footnote Teams-pricing contradiction + DigiUsher as unresolved.

---

## OPEN-C7 — Do Admin `filtered-usage-events` include Other-Models `tier`?

### Question
Do Admin `filtered-usage-events` include Other-Models `tier` (or equivalent)? Official docs sample has model/kind/tokenUsage/chargedCents — check docs + community for `tier`.

### Findings

| Claim | Status | Evidence |
|-------|--------|----------|
| Admin docs **Response Fields** for `usageEvents[]` list: `timestamp`, `userEmail`, optional service/cloud/automation/conversation ids, `model`, `kind`, `maxMode`, `requestsCosts`, `isTokenBasedCall`, `isChargeable`, `isHeadless`, `tokenUsage{…}`, `chargedCents`, `cursorTokenFee` — **no `tier`** | **CONFIRMED** | [Admin API — filtered-usage-events](https://cursor.com/docs/account/teams/admin-api) (fetched 2026-09-11) |
| Official JSON samples likewise omit `tier` | **CONFIRMED** | same page samples |
| Staff forum reply listing event fields for Included Requests: userEmail, model, kind, requestsCosts, tokenUsage — **no tier** | **PARTIAL** (staff; aligns with docs) | [forum #128253/6](https://forum.cursor.com/t/details-on-ai-code-tracking-api/128253/6) |
| Community `tier` (1=Other / 2=Cursor Models) appears on undocumented **`GetAggregatedUsageEvents`** dashboard RPC, **not** documented on Admin filtered-usage-events | **PARTIAL** | `EC-cursor-other-models-v1`; OpenUsage provider docs |
| Live Admin response actually includes undocumented `tier` | **UNKNOWN** | No Admin key sample this run; do not invent |

### Confidence
**CONFIRMED** that **official Admin contract does not document `tier`**. Whether wire responses ever add it = **UNKNOWN** without sample. Pool signals on Admin events: `model` + `kind` + presence/absence of `cursorTokenFee` (docs: fee on third-party) — **PARTIAL** inference path, not a `tier` field.

### Recommendation to 規格
**Fill (docs):** Admin S1 events have **no documented `tier`**; do not require `tier` for Admin ingest. **Still OPEN** only if product must know whether live payloads secretly include it — needs Enterprise sample. Prefer model-list / `cursorTokenFee` / `kind` for pool tagging when using Admin.

---

## OPEN-C8 — Spending pool % ↔ Ultra included $ allowance formula

### Question
Exact formula: Spending pool % ↔ Ultra included $ allowance (Ultra Other Models ~$400 API agent usage per usage-limits help; screenshot only shows %).

### Findings

| Claim | Status | Evidence |
|-------|--------|----------|
| Live [usage-limits](https://cursor.com/help/models-and-usage/usage-limits) / [models-and-pricing](https://cursor.com/docs/models-and-pricing) / [pricing help](https://cursor.com/help/account-and-billing/pricing) (fetched 2026-09-11) show two pools + Ultra **Included**, **without** the $20/$70/$400 dollar table | **CONFIRMED** (current live pages) | WebFetch this run |
| Search-index / older `.md` snapshots still show Ultra **$400** Other Models / API agent usage table | **PARTIAL** (stale index vs live; treat as historical docs, not current live CONFIRMED) | WebSearch snippets for usage-limits.md / models-and-pricing.md / pricing.md |
| Cursor staff (Colin) forum: guaranteed minimum advertised API usage **$20 / $70 / $400** for Pro / Pro+ / Ultra | **PARTIAL** (staff statement 2026-03-01; not in live help table this fetch) | [forum percentage-based-usage #149736](https://forum.cursor.com/t/percentage-based-usage/149736) post 15 |
| Ultra user on same thread: “exhaust my $400 allowance every month” | **PARTIAL** (user report) | same thread post 7 |
| Human Ultra Spending screenshot: Cursor Models **65%**, Other Models **100%**, **no** dollar remaining on face UI | **CONFIRMED** UI | `screenshots/cursor-spending-ultra-2026-09-11.png` + `EC-cursor-official-api-v1` §7 |
| Exact official formula `apiPercentUsed = f(spend, $400)` for Ultra | **UNKNOWN** | No Cursor formula docs |
| Community Pro empirical fit (`apiPercentUsed * $45 ≈ API spend`) **≠** advertised Pro $20; author notes gap | **PARTIAL** (Pro-only samples; **not** Ultra; contradicts published $) | [cursor-sdk2api issues #18/#19](https://github.com/Sunnyender-org/cursor-sdk2api/issues/18) |
| Undocumented `planUsage.{auto,api}PercentUsed` + cents fields exist in dashboard RPC | **UNDOCUMENTED** + **PARTIAL** | OpenUsage; `EC-cursor-personal-ultra-v1` |

### Confidence
**PARTIAL** for “Ultra Other Models advertised ≈ $400 API usage” (staff + historical docs + user). **UNKNOWN** for exact %↔$ formula and for live Cursor Models pool $ denominator. Screenshot proves UI is %-first.

### Recommendation to 規格
**HOLD** exact conversion formula. May note advertised Other Models minimums as **PARTIAL** (staff forum + prior docs; live help currently omits $ table). Align product to **pool %** (P1/U1), not reconstructed $, until official formula or live Ultra RPC cents÷% audited.

---

## OPEN-C9 — Stable machine source for Grok Bot weekly % (upgrade G1)

### Question
Stable machine source for Grok Bot weekly % — upgrade G1 from UNKNOWN if possible (`GetSandUsageStatus` / `get-sand-usage-status`).

### Findings

| Claim | Status | Evidence |
|-------|--------|----------|
| Product UI: Grok Bot weekly % on Spending (Ultra screenshot 29%, reset ~9/18) | **CONFIRMED** UI | screenshot + prior ECs |
| Official help: Grok Bot included usage is **weekly**; Ultra = highest weekly; on-demand after | **CONFIRMED** | [help/grok-bot/plans](https://cursor.com/help/grok-bot/plans) |
| Public Admin / Analytics API documents Grok Bot **usage %** endpoint | **UNDOCUMENTED** (absent) | Admin API + API Overview — audit has `application_type: grok_bot` / event types, **not** weekly % |
| Community machine path A: `POST https://api2.cursor.sh/aiserver.v1.DashboardService/GetSandUsageStatus` (Connect RPC, Bearer accessToken) | **PARTIAL** (merged OpenUsage PR #1134 + docs; **UNDOCUMENTED**) | [openusage#1134](https://github.com/robinebers/openusage/pull/1134); commit `65324c6`; provider docs |
| Community machine path B: `POST https://cursor.com/api/dashboard/get-sand-usage-status` (session cookie) | **PARTIAL** (CodexBar PR #3127; **UNDOCUMENTED**) | [CodexBar#3127](https://github.com/steipete/CodexBar/pull/3127) |
| Response fields used by OpenUsage mapper | **PARTIAL** (community code + tests, not official schema) | `usagePercent`, `nextResetTimestampUtc`, `hasNonZeroIncludedLimit`, `usesPooledEnterpriseAllowance`, `includedLimitZero` |
| CodexBar decoded struct fields | **PARTIAL** | `usagePercent`, `nextResetTimestampUtc`, `hasNonZeroIncludedLimit`; path `/api/dashboard/get-sand-usage-status` |
| OpenUsage live Ultra verify (PR test plan): exported `grokBot` weekly window | **PARTIAL** (maintainer claim in PR; not NB-T3261) | PR #1134 Tests section |
| This probe live call against NB-T3261 JWT | **UNKNOWN** / not run | Auth material not used (anti-leak) |

### Confidence
G1 machine source upgraded **UNKNOWN → PARTIAL**: endpoints + field names are **community-stable / multi-client**, still **UNDOCUMENTED** and fail-soft. Official weekly semantics **CONFIRMED** in help; official machine API still absent.

### Recommendation to 規格
**Fill G1** as: UI CONFIRMED + help weekly CONFIRMED; machine = **UNDOCUMENTED** `GetSandUsageStatus` / `get-sand-usage-status` (opt-in, fail-soft). **Still OPEN** for official docs or first-party live capture. Do **not** treat Admin audit `grok_bot` events as weekly %.

---

## Explicit: no live Admin key / 403 invention

- No Admin / Teams API call executed this run.
- No fabricated 403 JSON beyond what **official docs** already publish.
- Prior corpus (`EC-cursor-personal-ultra-v1`) records personal `crsr_` → **401 Invalid Team API Key** on `/teams/*` (different boundary: individual vs team key), not Teams-vs-Enterprise 403.

---

## Sources

### Official (fetched 2026-09-11)
- https://cursor.com/docs/api
- https://cursor.com/docs/account/teams/admin-api
- https://cursor.com/docs/account/teams/pricing
- https://cursor.com/docs/models-and-pricing
- https://cursor.com/help/models-and-usage/usage-limits
- https://cursor.com/help/account-and-billing/pricing
- https://cursor.com/help/grok-bot/plans
- https://cursor.com/pricing

### Community / staff / vendor
- https://forum.cursor.com/t/percentage-based-usage/149736 (Colin staff $20/$70/$400)
- https://forum.cursor.com/t/details-on-ai-code-tracking-api/128253/6
- https://forum.cursor.com/t/admin-api-key-is-not-working/109221
- https://docs.digiusher.com/connecting-cursor
- https://github.com/robinebers/openusage/pull/1134 (+ commit `65324c6` patch)
- https://openusage.sh/docs/providers/cursor/ + repo `docs/providers/cursor.md`
- https://github.com/steipete/CodexBar/pull/3127
- https://github.com/Sunnyender-org/cursor-sdk2api/issues/18 (and #19)

### Related parent ECs
- [`EC-cursor-official-api-v1.md`](./EC-cursor-official-api-v1.md) — Admin vs personal; OPEN Teams boundary / tier / % formula / Grok Bot UNKNOWN list
- [`EC-cursor-personal-ultra-v1.md`](./EC-cursor-personal-ultra-v1.md) — undocumented dashboard paths; GetSandUsageStatus row C
- [`EC-cursor-other-models-v1.md`](./EC-cursor-other-models-v1.md) — `tier` on AggregatedUsageEvents; Admin tier UNKNOWN
- [`EC-cursor-spending-align-v1.md`](./EC-cursor-spending-align-v1.md) — S1–G1 handoff table

### UI fixture
- `screenshots/cursor-spending-ultra-2026-09-11.png`

---

## Status rollup (for INDEX / 規格)

| ID | Status after this card | Spec action |
|----|------------------------|-------------|
| OPEN-C6 | **PARTIAL** (docs conflict; no live Teams 403) | **HOLD / still OPEN** |
| OPEN-C7 | **CONFIRMED** docs omit `tier`; live presence **UNKNOWN** | **Fill** “no documented tier”; optional live sample still OPEN |
| OPEN-C8 | **PARTIAL** $400 advertised; formula **UNKNOWN**; live docs omit $ table | **HOLD** formula; % authoritative |
| OPEN-C9 / G1 | **PARTIAL** (community Sand endpoints); official API absent | **Fill** undocumented opt-in; still OPEN for official |

# Cursor official Admin API — credential / source-mode contract v0

| Field | Value |
|-------|-------|
| Version | **cursor-official-admin-v0** |
| Date | 2026-09-11 (Asia/Taipei) |
| Spec | AC v1.4-cursor-official (AC-F14 / F15 / F16 / F17) |
| Producer | 橋樑 (`tokentracer-bridge` → `cursor_admin`) |
| Consumer | 帳本 / 儀表 |
| Status | **Design + local stub** (human-approved AC; no self-PASS) |

---

## 1. Credential resolution order

Admin API key is **never hardcoded**. Resolution (first non-empty wins):

| Order | Source | Notes |
|-------|--------|-------|
| 1 | Env `TOKENTRACER_CURSOR_ADMIN_API_KEY` | Preferred for CI / headless |
| 2 | Optional config file key `cursor.admin_api_key` | Path via `TOKENTRACER_CURSOR_ADMIN_CONFIG` or explicit `AdminCredentialConfig.config_path` |
| 3 | **None** | → personal / no-key path (PARTIAL; see §3) |

Config file (when used) is JSON:

```json
{ "cursor": { "admin_api_key": "<secret>" } }
```

**Redaction:** diagnostics MUST never log the full key. Use `redact_secret` (prefix ≤4 chars + `…` + length).

HTTP auth for official Team Admin (future network layer; **out of stub scope**): Basic `-u KEY:` per Cursor docs (AC-F14).

---

## 2. Source modes

| Mode (wire / serde) | Rust | Default | Meaning |
|---------------------|------|---------|---------|
| `official_admin` | `SourceMode::OfficialAdmin` | Enabled **only** when Admin key resolves | S1 `POST /teams/filtered-usage-events` + S2 `POST /teams/spend` |
| `local_enrichment` | `SourceMode::LocalEnrichment` | Optional | L1 local DB / bubble / context — **FORBIDDEN** as sole Cursor USD / bill / pool % |
| `undocumented_dashboard` | `SourceMode::UndocumentedDashboard` | **`off`** | U1/U2; opt-in only; every emit labels **UNSUPPORTED** |

Flag: `undocumented_dashboard` default **`off`** (AC-F16). Opt-in MUST set `unsupported: true` and MUST NOT claim `official_admin`.

---

## 3. Personal / no-key → PARTIAL contract (AC-F15 / F17)

When no Admin key resolves and undocumented is off, bridge returns a structured **PARTIAL** payload for 儀表／帳本:

| Field | Value |
|-------|-------|
| `status` | `"partial"` |
| `source_mode` | `"local_enrichment"` (enrichment only; not billing) |
| `message` | **`無公開個人 usage API`** (exact contract string) |
| `message_code` | `TT-C14-NO-PERSONAL-USAGE-API` |
| `align_targets` | `["cursor_models_percent","other_models_percent","reset","on_demand"]` |
| `billing_authoritative` | `false` |
| `undocumented_dashboard` | `false` |

Consumers MUST NOT treat local token totals as Spending pool % or subscription USD.

---

## 4. Structured errors (AC-F14)

Shape (aligned with path-list `DiscoverError`):

```text
AdminDiagnostic {
  code: string
  message: string
  next_step: string
  http_status?: number
  redacted_hint?: string   // never full key
}
```

| Code | When | Notes |
|------|------|-------|
| `TT-C14-MISSING-KEY` | No env / config key | Prefer PARTIAL personal path over hard fail for UI; CLI may surface as diagnostic |
| `TT-C14-401` | HTTP 401 Unauthorized | Invalid / revoked Admin key |
| `TT-C14-403-ENTERPRISE` | HTTP 403 Enterprise required | Team/Enterprise Admin feature not available on account |
| `TT-C14-CONFIG-UNREADABLE` | Config path set but unreadable / invalid JSON | Clear next_step; do not invent key |
| `TT-C14-UNSUPPORTED-UNDOCUMENTED` | Undocumented path used while opt-in | Label UNSUPPORTED; still not `official_admin` |

401 / 403 MUST NOT be silent skips.

---

## 5. Non-goals (explicit)

- **No L1-as-billing:** never map bubble / `state.vscdb` → USD / Spending / pool % (BLOCK per OPEN-C2 / AC-F15).
- **No undocumented default on.**
- Stub does **not** perform live Admin HTTP; network client is a later slice.
- F7′ tray smoke is **not** this deliverable (驗收官 / NB-T3261).

---

## 6. Stub API surface

Module: `tokentracer_bridge::cursor_admin`

- `resolve_admin_credential(cfg) -> Result<ResolvedAdminKey, AdminDiagnostic>`
- `select_source_mode(cfg, key_present) -> SourceModeSelection`
- `personal_partial_payload() -> PersonalPartialPayload`
- `UndocumentedDashboardFlag` default `Off`
- Unit tests: env key present; missing → PARTIAL; undocumented default off; L1 never yields `OfficialAdmin`

---

## 7. Evidence / AC map

| AC | Contract coverage |
|----|-------------------|
| F14 | Credential order + 401 / 403 Enterprise diagnostics |
| F15 | local_enrichment not billing; PARTIAL |
| F16 | `undocumented_dashboard` default off + UNSUPPORTED on opt-in |
| F17 | Message `無公開個人 usage API` + align targets |

**Final PASS = 驗收官 only; 橋樑 does not self-PASS.**

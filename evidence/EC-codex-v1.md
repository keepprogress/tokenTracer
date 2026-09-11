# Evidence Card: Codex (OpenAI Codex CLI / coding agent)

- **Agent ID for UsageEvent:** `codex`
- **Card version:** v1
- **Research date:** 2026-09-11 (Asia/Taipei)
- **Scope:** Windows native + WSL2
- **Overall confidence:** **CONFIRMED** for WSL2 local rollout JSONL (`token_usage_record` + `event_msg/token_count`); **PARTIAL** for Windows-native populated `sessions/` tree and subscription `$` billing

---

## 1. Local paths (usage / session / history)

### Primary usage source

| Environment | Path | Format | Confidence | Evidence |
|---|---|---|---|---|
| WSL2 / Linux | `${CODEX_HOME:-~/.codex}/sessions/YYYY/MM/DD/rollout-<ts>-<uuid>.jsonl` | JSONL | **CONFIRMED** | Live probe NB-T3261 WSL `/home/t3261/.codex/sessions/…` — **311 rollouts** observed 2026-09-11; also [tokenuse Codex](https://tokenuse.app/docs/development/tools/codex/), [ccusage Codex](https://ccusage.com/guide/codex/), [developers.openai.com CODEX_HOME](https://developers.openai.com/codex/environment-variables) |
| WSL2 / Linux | `~/.codex/archived_sessions/rollout-*.jsonl` | JSONL (flat) | **CONFIRMED** (docs) / **PARTIAL** (this host) | Documented by [tokenuse](https://tokenuse.app/docs/development/tools/codex/), [ccusage](https://ccusage.com/guide/codex/); not re-listed in same NB-T3261 probe pass |
| Windows native | `%USERPROFILE%\.codex\sessions\YYYY\MM\DD\rollout-*.jsonl` | JSONL | **PARTIAL** | Official: Windows app + native Codex share `%USERPROFILE%\.codex` ([learn.chatgpt.com Windows app](https://learn.chatgpt.com/docs/windows/windows-app)); on NB-T3261 `%USERPROFILE%\.codex` exists with `auth.json`, `config.toml` but full `sessions/` tree **not re-confirmed** in interrupted Win probe |
| WSL → Windows home | `export CODEX_HOME=/mnt/c/Users/<windows-user>/.codex` | — | **CONFIRMED** (official recipe) | [learn.chatgpt.com Windows app — Share config with WSL](https://learn.chatgpt.com/docs/windows/windows-app) |

### Env overrides (CONFIRMED)

| Variable | Default | Role | Source |
|---|---|---|---|
| `CODEX_HOME` | `~/.codex` | Root for config, auth, logs, **sessions**, skills | [developers.openai.com/codex/environment-variables](https://developers.openai.com/codex/environment-variables) |
| `CODEX_SQLITE_HOME` | `CODEX_HOME` | SQLite-backed state home (`state_5.sqlite`, etc.) | Same + [openai/codex state lib](https://github.com/openai/codex/blob/914c8eeb/codex-rs/state/src/lib.rs) (`STATE_DB_FILENAME = "state_5.sqlite"`) |

### Related files (secondary / not primary per-call ledger)

| Path | Role | Confidence | Sources |
|---|---|---|---|
| `~/.codex/auth.json` | Access token / account_id (secrets — do not ship) | CONFIRMED | [openusage Codex](https://openusage.sh/docs/providers/codex/), NB-T3261 probe |
| `~/.codex/config.toml` | Model / provider / `chatgpt_base_url` | CONFIRMED | [developers.openai.com/codex/config-advanced](https://developers.openai.com/codex/config-advanced), probe |
| `~/.codex/history.jsonl` | Prompt history only (`[history]` TOML); **does not** disable rollouts | CONFIRMED | [config-advanced](https://developers.openai.com/codex/config-advanced), [allaboutcoding](https://allaboutcoding.ghinda.com/where-ai-coding-clis-store-session-logs/) |
| `~/.codex/state_5.sqlite` | `threads` metadata incl. `tokens_used` (cumulative total only), `rollout_path` | CONFIRMED (source + community) | [codex-rs/state](https://github.com/openai/codex/blob/914c8eeb/codex-rs/state/src/lib.rs), [AlmanacCode guide](https://raw.githubusercontent.com/AlmanacCode/codealmanac/main/archive/code/guides/processing/codex.md) |
| `~/.codex/logs_2.sqlite` | Large local log DB (~148 MiB on NB-T3261) | CONFIRMED present; **UNKNOWN** as token ledger | Live probe; filename in [codex-rs/state](https://github.com/openai/codex/blob/914c8eeb/codex-rs/state/src/lib.rs) (`LOGS_DB_FILENAME`) |
| `~/.codex/version.json` | Installed version | CONFIRMED | [openusage Codex](https://openusage.sh/docs/providers/codex/) |

**Note:** Official advanced config lists `history.jsonl` under `CODEX_HOME` but does **not** document the `sessions/YYYY/MM/DD/rollout-*.jsonl` tree as a public contract — community + live probes are the practical sources ([allaboutcoding](https://allaboutcoding.ghinda.com/where-ai-coding-clis-store-session-logs/): “config keys documented, transcripts undocumented”).

---

## 2. File formats

| Artifact | Format | Confidence | Sources |
|---|---|---|---|
| Session rollouts | **JSONL** — one object per line: `{ timestamp, ordinal?, type, payload }` | CONFIRMED | Live probe; [tokenuse](https://tokenuse.app/docs/development/tools/codex/); [agsess adapter](https://docs.rs/nativelite-agsess/latest/src/agsess/codex.rs.html); [openai/codex#43320](https://github.com/openai/codex/issues/43320) |
| First line | Usually `type: "session_meta"` | CONFIRMED | [tokenuse](https://tokenuse.app/docs/development/tools/codex/) (validates `originator` contains `"codex"`) |
| SQLite index | `state_5.sqlite` / optional `CODEX_SQLITE_HOME` | CONFIRMED | Official env docs + source |
| Cost in files | **No vendor invoice $** | CONFIRMED | Local = tokens; $ via pricing tables or live ChatGPT usage API |

**Retention:** No documented age-based auto-delete for rollouts; `archive` / `unarchive` / `delete` subcommands since ~v0.136. `[history] persistence = "none"` only affects `history.jsonl`, **not** rollouts ([allaboutcoding](https://allaboutcoding.ghinda.com/where-ai-coding-clis-store-session-logs/)).

**Historical gap:** `token_count` events started ~2025-09-06 (commit 0269096); earlier rollouts lack token metrics ([ccusage Codex](https://ccusage.com/guide/codex/)).

---

## 3. APIs (optional / remote)

Local JSONL is sufficient for per-turn tokens. Optional remotes used by trackers for **plan / credits / rate limits**:

| Endpoint / RPC | Auth | Purpose | Confidence | Sources |
|---|---|---|---|---|
| `GET https://chatgpt.com/backend-api/wham/usage` (or `/api/codex/usage`) | Bearer from `auth.json` + optional `ChatGPT-Account-Id` | Plan, credits, rate-limit windows | PARTIAL (community-documented internal) | [openusage Codex](https://openusage.sh/docs/providers/codex/) |
| `codex app-server` → `account/rateLimits/read` | Local CLI | Individual monthly credit limit / reset | PARTIAL | [openusage Codex](https://openusage.sh/docs/providers/codex/) |
| OTel metrics `codex.turn.token_usage` | If user enables `[otel]` | Observability, not historical ledger | PARTIAL | [config-advanced Observability](https://developers.openai.com/codex/config-advanced) |

**BLOCK:** Do not rely on remote APIs alone for historical UsageEvent import; they can change and need live auth.

---

## 4. Sample fields

### A. `token_usage_record` — preferred per-response counters (CONFIRMED live + public)

**Live NB-T3261 WSL** (`~/.codex/sessions/2026/09/11/rollout-*.jsonl`):

```json
{
  "timestamp": "2026-09-11T01:56:22.654Z",
  "ordinal": 15,
  "type": "token_usage_record",
  "payload": {
    "thread_id": "01a08e2d-61ad-7d72-9972-2cca103e96b6",
    "turn_id": "01a08e2d-ec6a-7940-8fb6-c100586a79b2",
    "session_id": "01a08e2d-61ad-7d72-9972-2cca103e96b6",
    "root_turn_id": "01a08e2d-ec6a-7940-8fb6-c100586a79b2",
    "response_id": "resp_0475cde564512e09016aa35fbd036487d0ad4cd2062e159457",
    "usage": {
      "input_tokens": 25205,
      "cached_input_tokens": 11904,
      "cache_write_input_tokens": 0,
      "output_tokens": 256,
      "reasoning_output_tokens": 0,
      "total_tokens": 25461
    },
    "turn_token_usage": { "...": "may be present — prefer for per-turn when set" },
    "thread_token_usage": { "...": "cumulative within thread — do not double-count" }
  }
}
```

**Public corroboration** (same shape) in [openai/codex#43320](https://github.com/openai/codex/issues/43320) and parser tests in [nativelite-agsess](https://docs.rs/nativelite-agsess/latest/src/agsess/codex.rs.html) (`prefer turn_token_usage over usage`).

| Field | Notes | Confidence |
|---|---|---|
| `payload.usage.input_tokens` | Includes cached input (OpenAI quirk) | CONFIRMED |
| `payload.usage.cached_input_tokens` | Cache read; also aliased as `cache_read_input_tokens` on some builds | CONFIRMED ([tokenuse](https://tokenuse.app/docs/development/tools/codex/)) |
| `payload.usage.cache_write_input_tokens` | Often `0`; present on newer builds | CONFIRMED (probe + #43320 + fixture in [codex rollout test](https://github.com/openai/codex/blob/main/codex-rs/app-server/tests/common/rollout.rs)) |
| `payload.usage.output_tokens` | | CONFIRMED |
| `payload.usage.reasoning_output_tokens` | Fold into output for pricing in most parsers | CONFIRMED |
| `payload.response_id` | Stable dedup key | CONFIRMED |
| Cost USD | **Absent** | CONFIRMED |

### B. `event_msg` / `token_count` — cumulative snapshots + rate limits (CONFIRMED)

Documented heavily by [tokenuse](https://tokenuse.app/docs/development/tools/codex/), [ccusage](https://ccusage.com/guide/codex/), live probe:

```json
{
  "type": "event_msg",
  "payload": {
    "type": "token_count",
    "info": {
      "total_token_usage": {
        "input_tokens": 25205,
        "cached_input_tokens": 11904,
        "cache_write_input_tokens": 0,
        "output_tokens": 256,
        "reasoning_output_tokens": 0,
        "total_tokens": 25461
      },
      "last_token_usage": { "...": "same shape — last request" },
      "model_context_window": 258400
    },
    "rate_limits": {
      "limit_id": "codex",
      "primary": { "used_percent": 7.0, "window_minutes": 300 },
      "secondary": { "used_percent": 30.0, "window_minutes": 10080 },
      "plan_type": "plus"
    }
  }
}
```

**Parser rule (CONFIRMED):** if using `token_count`, prefer **deltas of `total_token_usage`** (or use `last_token_usage`); never sum cumulative snapshots. Prefer **`token_usage_record` when both exist** to avoid double-count ([agsess](https://docs.rs/nativelite-agsess/latest/src/agsess/codex.rs.html) notes both channels exist).

### C. Model (CONFIRMED)

- From `turn_context.payload.model` (latest preceding). Live NB-T3261 observed: `gpt-6-astra`.
- `session_meta` has `model_provider` (e.g. `"openai"`), not always the concrete model ([AlmanacCode](https://raw.githubusercontent.com/AlmanacCode/codealmanac/main/archive/code/guides/processing/codex.md)).

---

## 5. Mapping → UsageEvent

Target:

```ts
UsageEvent {
  id, agent: "codex", source, ts, model,
  input_tokens, output_tokens,
  cache_read_tokens?, cache_write_tokens?,
  raw_cost_usd?, meta?
}
```

| UsageEvent | Mapping | Confidence |
|---|---|---|
| `id` | Prefer `payload.response_id`; else `${session_id}:${turn_id}:${ordinal}` | CONFIRMED |
| `agent` | `"codex"` | — |
| `source` | Absolute rollout path, or `codex:wsl-sessions` / `codex:win-sessions` | CONFIRMED practice |
| `ts` | Top-level `timestamp` (ISO-8601 UTC) | CONFIRMED |
| `model` | Latest preceding `turn_context.payload.model` | CONFIRMED |
| `input_tokens` | `usage.input_tokens - usage.cached_input_tokens` (non-cached) **or** store raw + price with cache bucket | CONFIRMED convention ([tokenuse](https://tokenuse.app/docs/development/tools/codex/), CodexScope) |
| `output_tokens` | `usage.output_tokens` (+ optionally `reasoning_output_tokens`) | CONFIRMED |
| `cache_read_tokens` | `usage.cached_input_tokens` | CONFIRMED |
| `cache_write_tokens` | `usage.cache_write_input_tokens` | CONFIRMED (often 0) |
| `raw_cost_usd` | **Compute** from price table; null from file. Subscription plans ≠ API invoice | CONFIRMED ([openusage](https://openusage.sh/docs/providers/codex/), [ccusage](https://ccusage.com/guide/codex/)) |
| `meta` | `{ thread_id, turn_id, session_id, reasoning_output_tokens, plan_type, cwd, cli_version }` | PARTIAL shape |

**Dedup (CONFIRMED recommendation):** emit **one** UsageEvent per `token_usage_record` (prefer `turn_token_usage` if present, else `usage`). Do **not** also emit from the matching `event_msg/token_count` unless older rollouts lack `token_usage_record`. Skip zero-delta cumulative `token_count` snapshots.

---

## 6. Permissions

| Need | Detail | Confidence |
|---|---|---|
| Read | User-owned `${CODEX_HOME}/sessions/**` (+ `archived_sessions/**`) | CONFIRMED |
| Network | **Not required** for local token ledger | CONFIRMED |
| Secrets | Never copy `auth.json` into fixtures / logs | CONFIRMED |
| Plaintext | Rollouts hold prompts, tool output, paths — treat as sensitive | CONFIRMED ([openai/codex#27131](https://github.com/openai/codex/issues/27131) self-ingest risk) |
| WSL from Windows | `\\wsl$\<distro>\home\<user>\.codex\…` or `wsl.exe -e`; needs ACL to Linux home | CONFIRMED pattern |

---

## 7. Windows vs WSL2 differences

| Topic | Finding | Confidence | Source |
|---|---|---|---|
| Default homes | Win: `%USERPROFILE%\.codex`; WSL: Linux `~/.codex` | CONFIRMED | [learn.chatgpt.com Windows app](https://learn.chatgpt.com/docs/windows/windows-app) |
| Not shared by default | Config, auth, **session history** separate unless linked | CONFIRMED | Same official page |
| Official share recipe | `export CODEX_HOME=/mnt/c/Users/<win>/.codex` inside WSL | CONFIRMED | Same |
| NB-T3261 | Both trees exist; WSL has rich `sessions/2026/…`; Win `sessions/` **PARTIAL** | CONFIRMED / PARTIAL | Live probe |
| Desktop app vs CLI | Win Desktop may hardcode `%USERPROFILE%\.codex` and ignore `CODEX_HOME` | PARTIAL | [openai/codex#34070](https://github.com/openai/codex/issues/34070) |
| tokenTracer implication | Scan **both** Win and WSL homes unless user sets shared `CODEX_HOME` | CONFIRMED requirement |

---

## 8. Known parsers

| Tool | What it reads | Sources |
|---|---|---|
| **ccusage** (`ccusage codex`) | `CODEX_HOME` → `sessions/` + `archived_sessions/`; deltas of `event_msg/token_count`; LiteLLM pricing; `--speed` | [ccusage.com/guide/codex](https://ccusage.com/guide/codex/) |
| **tokenuse** | `rollout-*.jsonl`; `token_count` deltas; rate_limits; fork dedup | [tokenuse.app/docs/development/tools/codex](https://tokenuse.app/docs/development/tools/codex/) |
| **openusage** | Local JSONL + optional ChatGPT usage / app-server quotas; provider id `codex` | [openusage.sh/docs/providers/codex](https://openusage.sh/docs/providers/codex/) |
| **nativelite-agsess** | Explicitly handles top-level `token_usage_record` **and** `event_msg/token_count` | [docs.rs agsess/codex.rs](https://docs.rs/nativelite-agsess/latest/src/agsess/codex.rs.html) |
| **CodexScope** (community) | Rollouts; `input = input − cached` | Cited in prior probe notes / continuum guides |
| **AlmanacCode guide** | JSONL + `state_5.sqlite` enrichment | [codex.md](https://raw.githubusercontent.com/AlmanacCode/codealmanac/main/archive/code/guides/processing/codex.md) |

---

## Confidence summary

| Claim | Level |
|---|---|
| WSL path `~/.codex/sessions/**/rollout-*.jsonl` | **CONFIRMED** (live NB-T3261 + docs) |
| Top-level `type: "token_usage_record"` with `usage` buckets | **CONFIRMED** (live + #43320 + agsess) |
| Companion `event_msg` / `token_count` cumulative + rate_limits | **CONFIRMED** |
| Model via `turn_context.payload.model` | **CONFIRMED** |
| `cached_input_tokens` / `cache_write_input_tokens` field names | **CONFIRMED** |
| `CODEX_HOME` / official Win↔WSL share | **CONFIRMED** |
| Win-native `sessions/` populated on NB-T3261 | **PARTIAL / UNKNOWN** until re-probe |
| `logs_2.sqlite` as alternate token ledger | **UNKNOWN** |
| Exact subscription billed USD from local files | **UNKNOWN** (notional API pricing only) |

---

## UNKNOWN / next investigation steps

| Gap | BLOCK reason | Next steps |
|---|---|---|
| Win-native rollout count on NB-T3261 | Interrupted Win probe | `Get-ChildItem $env:USERPROFILE\.codex\sessions -Recurse -Filter rollout-*.jsonl \| Measure` |
| Prefer `usage` vs `turn_token_usage` when both differ | agsess prefers `turn_token_usage`; live sample often shows per-response in `usage` | Diff both fields across 20 recent records; document rule |
| `logs_2.sqlite` schema | Large file; unknown overlap | Read-only `.schema` / table list |
| Whether older ccusage/tokenuse ignore `token_usage_record` | They document `token_count` primarily | Check current parser source; implement dual-path with dedup |
| Desktop-only vs CLI originator strings | tokenuse checks `originator` contains `"codex"` | Sample `session_meta` from Win Desktop vs WSL CLI |

## BLOCK reasons

- **None** for **WSL2 local token import** of Codex via `token_usage_record` / `token_count`.
- **BLOCK** claiming Win-native historical import until `sessions/**/rollout-*.jsonl` confirmed on that profile.
- **BLOCK** claiming exact billed USD from local files alone (ChatGPT subscription).

---

## Evidence sources

### Live probe
- Host **NB-T3261**, WSL user `t3261`, 2026-09-11 — 311 rollouts under `/home/t3261/.codex/sessions/`; sample `token_usage_record` + `token_count` + `turn_context.model=gpt-6-astra`; Win `%USERPROFILE%\.codex` present (auth/config) with sessions tree unconfirmed

### Official
1. https://developers.openai.com/codex/environment-variables  
2. https://developers.openai.com/codex/config-advanced  
3. https://learn.chatgpt.com/docs/windows/windows-app  

### Open-source / parsers / issues
4. https://tokenuse.app/docs/development/tools/codex/  
5. https://ccusage.com/guide/codex/  
6. https://openusage.sh/docs/providers/codex/  
7. https://docs.rs/nativelite-agsess/latest/src/agsess/codex.rs.html  
8. https://raw.githubusercontent.com/AlmanacCode/codealmanac/main/archive/code/guides/processing/codex.md  
9. https://allaboutcoding.ghinda.com/where-ai-coding-clis-store-session-logs/  
10. https://github.com/openai/codex/issues/43320 (`token_usage_record` samples)  
11. https://github.com/openai/codex/issues/27131 (sessions path + self-ingest)  
12. https://github.com/openai/codex/issues/34070 (Win Desktop vs `CODEX_HOME`)  
13. https://github.com/openai/codex/blob/914c8eeb/codex-rs/state/src/lib.rs (`state_5.sqlite`, `logs_2.sqlite`)  
14. https://github.com/openai/codex/blob/main/codex-rs/app-server/tests/common/rollout.rs (`TokenUsage` fixture incl. `cache_write_input_tokens`)

---

## macOS appendix (AC v1.2) — 2026-09-11

| Field | Value |
|-------|-------|
| Target | macOS 13+; Apple Silicon path; Intel best-effort |
| Live Mac probe | **UNKNOWN** (no Mac host this pass) |
| Rollout path/format | **CONFIRMED** (same as Linux/WSL; `$CODEX_HOME` or `~/.codex`) |

### Paths (vs Win / WSL)

| Artifact | macOS | Notes |
|----------|-------|-------|
| Active sessions | `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl` | Identical sharding; `CODEX_HOME` overrides root |
| Archived | `~/.codex/archived_sessions/rollout-*.jsonl` | Include in historical import |
| Auth / config | `~/.codex/auth.json`, `config.toml` | Do not ship secrets in fixtures |
| Indexes (sidebar) | `state_*.sqlite`, `session_index.jsonl` cited on Mac Desktop issues | **PARTIAL** — prefer rollout JSONL for tokens (`token_usage_record`), not sidebar indexes |

Sources: codeburn providers/codex.md; agent-sessions session-storage-format; openai/codex#20419 (Mac Desktop + `~/.codex`); CodexScope; continuum guide.

Event shapes (`token_usage_record`, `turn_context.model`, `event_msg`/`token_count`) assumed **same** as WSL live probe — format is CLI/app-server shared; **PARTIAL** until Mac sample captured.

### Permissions

| Need | Detail | Confidence |
|------|--------|------------|
| Read `~/.codex/sessions/**` | User home; FDA not required for standard home | **CONFIRMED** pattern |
| No Win/WSL dual scan | Single home tree on Mac | CONFIRMED |
| Optional account API | `codex app-server` usage methods / ChatGPT OAuth | PARTIAL — not required for local token ledger |

### Apple Silicon vs Intel

Paths identical. Intel binary best-effort: **UNKNOWN** this pass.

### Mapping

Reuse `EC-codex-v1` WSL UsageEvent mapping. macOS local import: **CONFIRMED** path/format; field-level live dump: **PARTIAL/UNKNOWN**.

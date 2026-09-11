# Evidence Card: Cursor IDE usage / token / cost (Windows + WSL2)

## Evidence Card ID

`EC-cursor-v1`

## Agent

`cursor` (Cursor IDE desktop / Electron UI; VS Code fork)

Also related but distinct surfaces (do not conflate without evidence):

- Cursor **Admin API** (`api.cursor.com`, team admin key) — official per-event usage
- Cursor **dashboard CSV export** (`cursor.com/api/dashboard/export-usage-events-csv`) — undocumented individual export
- Cursor **Connect RPC** (`api2.cursor.sh` `aiserver.v1.DashboardService`) — undocumented; used by community meters
- Local SQLite / `~/.cursor` stores — conversation + attribution; **not** authoritative billed token totals on current builds

## Confidence

**PARTIAL** overall.

| Area | Confidence | Notes |
|------|------------|-------|
| Platform paths for `state.vscdb` (Win / macOS / Linux) | **CONFIRMED** | Multiple independent docs agree |
| WSL2: UI-on-Windows writes chat/auth to Windows `%APPDATA%` | **CONFIRMED** | agentgrep ADR + backends doc; Traces troubleshooting |
| Auth token key `ItemTable.cursorAuth/accessToken` | **CONFIRMED** | openusage, cursor-usage-meter, AI-Usage-for-Windows |
| DashboardService endpoints + aggregated token fields | **CONFIRMED** (undocumented) | openusage provider doc; AI-Usage-for-Windows samples |
| Official Admin API per-event token/cost shape | **CONFIRMED** | cursor.com Admin API docs |
| Individual CSV export columns (tokens/cost) | **CONFIRMED** (undocumented) | agent-walker; @tokscale/cli types |
| Local bubble `tokenCount` as billed usage | **CONFIRMED unreliable** | Cursor forum staff; tokenuse; codeburn |
| Local cache_read / cache_write | **CONFIRMED absent / zero locally** | tokenuse; codeburn |
| Exact Win vs WSL layout of `~/.cursor/chats` and `ai-tracking` under Remote WSL | **PARTIAL** | Paths documented for `~/.cursor`; which host owns them when UI is Windows is not fully cross-verified in one primary source for every subpath |
| Live field dumps from a Windows/WSL machine in this research pass | **UNKNOWN** | Research-only; no local Cursor install inspected here |

## Paths / API / Format

### 1) Local paths (application support + agent home)

#### Global state DB (`state.vscdb`) — primary IDE store

| OS | Path | Sources |
|----|------|---------|
| Windows | `%APPDATA%\Cursor\User\globalStorage\state.vscdb` (expands to `C:\Users\<user>\AppData\Roaming\Cursor\User\globalStorage\state.vscdb`) | tokenuse, openusage, agentgrep, cursor-usage-meter |
| Linux (native Cursor) | `~/.config/Cursor/User/globalStorage/state.vscdb` | tokenuse, openusage, agentgrep |
| macOS | `~/Library/Application Support/Cursor/User/globalStorage/state.vscdb` | tokenuse, openusage, agentgrep |
| WSL2 reading Windows-hosted Cursor UI | `/mnt/c/Users/<WindowsUsername>/AppData/Roaming/Cursor/User/globalStorage/state.vscdb` | agentgrep cursor-ide.md; agentgrep ADR 0009; Traces WSL note |

Workspace-scoped DBs:

- `%APPDATA%\Cursor\User\workspaceStorage\<hash>\state.vscdb` (+ sibling `workspace.json` with `folder` URI)
- Same under `~/.config/Cursor/...` on Linux / under `/mnt/c/Users/.../AppData/Roaming/Cursor/...` from WSL when UI is Windows

`CURSOR_AGENT_HOME` (tokenuse) can replace `~/.cursor` for agent-side trees.

#### Agent / project home under `~/.cursor`

Documented by tokenuse + reverse-engineering writeups:

| Path | Purpose |
|------|---------|
| `~/.cursor/ai-tracking/ai-code-tracking.db` | AI code attribution / suggestion log (`ai_code_hashes`, `scored_commits`, …) |
| `~/.cursor/chats/<...>/store.db` | Current ordered chat stream (protobuf + JSON blobs) |
| `~/.cursor/projects/<workspace>/agent-transcripts/**/*.{jsonl,txt}` | Lossy transcript fallback |
| `~/.cursor/projects/<workspace>/terminals/`, `agent-tools/`, `mcps/` | Terminal state, tool caches, MCP schemas (context, not billing) |

OpenUsage manual override example also names:

- `tracking_db`: `~/.cursor/ai-tracking/ai-code-tracking.db`
- `state_db`: platform `state.vscdb`

### 2) SQLite shapes

#### `state.vscdb`

Tables commonly cited:

- `ItemTable` — key/value (settings, auth, some indexes)
- `cursorDiskKV` — key/value BLOBs (composers, bubbles, agent KV)

Auth keys in `ItemTable` (AI-Usage-for-Windows, cursor-usage-meter):

| Key | Role |
|-----|------|
| `cursorAuth/accessToken` | JWT bearer for `api2.cursor.sh` |
| `cursorAuth/refreshToken` | Refresh credential |
| `cursorAuth/cachedEmail` | Account email |
| `cursorAuth/stripeMembershipType` | Plan tier string |
| `cursorAuth/stripeSubscriptionStatus` | Subscription status |

`cursorDiskKV` key patterns (tokenuse, agentgrep, DEV.to, tracedecay, cursor-chronicle):

| Key pattern | Contents |
|-------------|----------|
| `composerData:<composerId>` | Session envelope: `fullConversationHeadersOnly`, `modelConfig`, `createdAt`/`lastUpdatedAt`, `promptTokenBreakdown` / `contextTokensUsed`, modes, status |
| `bubbleId:<composerId>:<bubbleId>` | Per-message JSON: text, tools, `tokenCount`, `modelInfo`, timestamps / `turnDurationMs` |
| `messageRequestContext:<...>` | Attached/referenced files, diffs |
| `agentKv:blob:<hash>` / AgentKv | Tool args, provider model name, durations |
| `checkpointId:...`, `codeBlockDiff:...` | Restore / accept-reject (not billing) |

Open read pattern (community tools): SQLite **read-only** (`mode=ro`; fallback `immutable=1`) so live WAL is visible without checkpointing Cursor (tokenuse).

#### `ai-code-tracking.db`

OpenUsage + DEV.to document tables including:

- `ai_code_hashes` — per-suggestion: `source`, `model`, `createdAt`, `conversationId`, …
- `scored_commits` — per-commit AI % / line counts (`aiPercentage` / `v1AiPercentage` / `v2AiPercentage` naming varies by writeup)
- Also cited: `tracked_file_content`, `ai_deleted_files`, `tracking_state`, `conversation_summaries`

**Not a billing token ledger** — attribution / AI-code score.

#### `store.db` (chats)

tokenuse documents:

```sql
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
CREATE TABLE blobs (id TEXT PRIMARY KEY, data BLOB);
```

Decoder: hex-decode `meta['0']` → `latestRootBlobId` → protobuf field `1` = ordered 32-byte blob ids → JSON user/assistant/tool messages. **No per-message clock** in current stores (tokenuse known gap).

### 3) Network APIs

#### A. Undocumented DashboardService (individual / signed-in IDE session)

Base: `https://api2.cursor.sh`  
Auth: `Authorization: Bearer <cursorAuth/accessToken>`  
Headers typically include `Content-Type: application/json`, `Connect-Protocol-Version: 1`  
Source: openusage, AI-Usage-for-Windows, Cursor Usage Status marketplace extension

| Method | Path | Use |
|--------|------|-----|
| POST | `/aiserver.v1.DashboardService/GetCurrentPeriodUsage` | Billing cycle + plan spend (cents) |
| POST | `/aiserver.v1.DashboardService/GetPlanInfo` | Plan name / included amount |
| POST | `/aiserver.v1.DashboardService/GetHardLimit` | Usage-based allowed flag |
| POST | `/aiserver.v1.DashboardService/GetAggregatedUsageEvents` | **Per-model** input/output/cache tokens + `totalCents` |
| POST | `/aiserver.v1.DashboardService/GetUsageLimitPolicyStatus` | Spend-limit policy |
| POST | `/aiserver.v1.DashboardService/GetTeamMembers` | Team members (team plans) |
| GET | `/auth/full_stripe_profile` | Membership / team flags |
| GET | `/auth/usage` | Enterprise-style request buckets (legacy) |
| GET | `/api/usage/summary` | Optional summary / on-demand hints |

Token refresh (AI-Usage-for-Windows):

- `POST https://api2.cursor.sh/oauth/token` with `grant_type=refresh_token`, `client_id=KbZUR41cY7W6zRSdpSUJ7I7mLYBKOCmB`, `refresh_token`

#### B. Undocumented individual CSV export (session cookie)

- `GET https://cursor.com/api/dashboard/export-usage-events-csv?strategy=tokens`
- Cookie: `WorkosCursorSessionToken=<userId>%3A%3A<access_token>` (URL-encoded `::`)
- Sources: agent-walker, robinebers/openusage, @tokscale/cli

#### C. Official Team Admin API (requires admin API key)

- Base: `https://api.cursor.com`
- Auth: Basic (API key as username)
- Per-event: `POST /teams/filtered-usage-events`
- Also: `/teams/spend`, `/teams/daily-usage-data`, …
- Source: https://cursor.com/docs/account/teams/admin-api

**Important:** Solo/Pro users typically **cannot** mint Admin API keys; community tools use (A) or (B) instead (codeburn / openusage notes).

## Sample fields (with source)

### Local — bubble / composer (state.vscdb)

Bubble `tokenCount` shape (tracedecay / Jacques Verré / cursor-chronicle):

```json
{
  "tokenCount": {
    "inputTokens": 0,
    "outputTokens": 0
  }
}
```

Cursor staff (forum): `tokenCount` is **best-effort** and often zero; dashboard is the accurate source  
→ https://forum.cursor.com/t/cursordiskkv-table-records-always-show-0-for-tokencount/155984

Composer context meter (tokenuse / codeburn / better-harness):

- `composerData.promptTokenBreakdown.totalUsedTokens` (fallback `contextTokensUsed`)
- This is a **context-window snapshot**, not cumulative billed output/cache

Model fields (tokenuse precedence):

1. AgentKv `providerOptions.cursor.modelName`
2. Bubble `modelInfo.modelName`
3. Composer `modelConfig.modelName`
4. Tracking `conversation_summaries.model` / `ai_code_hashes.model`
5. Store `lastUsedModel`
6. fallback `cursor-auto`

Timestamps: bubble message times / `turnDurationMs`; composer `createdAt` / `lastUpdatedAt` (ms). Store-only turns use session-level times (`timestamp_quality = session` in tokenuse).

### Aggregated API — `GetAggregatedUsageEvents` (openusage)

Each `aggregations[]` row:

| Field | Notes |
|-------|--------|
| `modelIntent` | Model id string |
| `inputTokens` | Often string → parseInt |
| `outputTokens` | string |
| `cacheWriteTokens` | string |
| `cacheReadTokens` | string |
| `totalCents` | number; divide by 100 → USD |
| `tier` | present in openusage |

### Period usage — `GetCurrentPeriodUsage` (AI-Usage-for-Windows sample)

```jsonc
{
  "billingCycleStart": "1768399334000",  // unix ms string
  "billingCycleEnd": "1771077734000",
  "planUsage": {
    "totalSpend": 23222,       // cents
    "includedSpend": 23222,
    "bonusSpend": 0,
    "remaining": 16778,
    "limit": 40000,
    "autoPercentUsed": 0,
    "apiPercentUsed": 46.444,
    "totalPercentUsed": 15.48
  },
  "spendLimitUsage": {
    "pooledLimit": 50000,
    "pooledUsed": 0,
    "individualLimit": 10000,
    "individualUsed": 0,
    "limitType": "user"
  }
}
```

### CSV export columns (@tokscale/cli + agent-walker)

Headers:

`Date, Model, Input (w/ Cache Write), Input (w/o Cache Write), Cache Read, Output Tokens, Total Tokens, Cost, Cost to you`

Mapping (agent-walker):

- `Input (w/o Cache Write)` → input
- `Input (w/ Cache Write)` → cache_write (disjoint column, not a difference)
- `Cache Read` → cache_read
- `Output Tokens` → output
- `Cost` → reported cost

### Official Admin API event (cursor.com docs)

```json
{
  "timestamp": "1750979225854",
  "userEmail": "developer@company.com",
  "conversationId": "8f2e4a1b-6c3d-4e5f-9a7b-2d1c8e6f4a3b",
  "model": "claude-4.5-sonnet",
  "kind": "Usage-based",
  "isTokenBasedCall": true,
  "tokenUsage": {
    "inputTokens": 126,
    "outputTokens": 450,
    "cacheWriteTokens": 6112,
    "cacheReadTokens": 11964,
    "totalCents": 20.18232
  },
  "chargedCents": 21.36232,
  "cursorTokenFee": 1.18
}
```

## Mapping to UsageEvent

Target shape:  
`UsageEvent { id, agent, source, ts, model, input_tokens, output_tokens, cache_read_tokens, cache_write_tokens, raw_cost_usd, meta }`

### Recommended source priority for **billable** tokens/cost

1. **Admin API** `filtered-usage-events` (team) — CONFIRMED complete token+cost per event  
2. **CSV export** `export-usage-events-csv?strategy=tokens` (individual) — CONFIRMED columns for tokens+cost  
3. **GetAggregatedUsageEvents** — CONFIRMED model-level aggregates only (not per-turn events)  
4. **Local reconstruction** (bubbles / context meter / store.db) — PARTIAL; good for sessions/text/model; **weak/wrong for billed tokens**

### Field mapping table

| UsageEvent field | Admin API | CSV export | GetAggregatedUsageEvents | Local state.vscdb / store |
|------------------|-----------|------------|--------------------------|---------------------------|
| `id` | synthesize e.g. `cursor:admin:{timestamp}:{conversationId}:{model}` or hash of event | synthesize from Date+Model+row hash | `cursor:agg:{billingCycleStart}:{modelIntent}` | `cursor:composer:{composerId}:{bubbleId}` (tokenuse-style) |
| `agent` | `"cursor"` | `"cursor"` | `"cursor"` | `"cursor"` |
| `source` | `"cursor.admin.filtered-usage-events"` | `"cursor.csv.export-usage-events"` | `"cursor.api2.GetAggregatedUsageEvents"` | `"cursor.local.state.vscdb"` / `"cursor.local.store.db"` |
| `ts` | `Number(timestamp)` ms → ISO/Date | `Date` column | cycle window only (no per-event ts) | bubble timestamps / composer `createdAt` |
| `model` | `model` | `Model` | `modelIntent` | model precedence list above |
| `input_tokens` | `tokenUsage.inputTokens` | `Input (w/o Cache Write)` | parse `inputTokens` | bubble `tokenCount.inputTokens` **OR** once-per-composer `promptTokenBreakdown.totalUsedTokens` (estimated/session) |
| `output_tokens` | `tokenUsage.outputTokens` | `Output Tokens` | parse `outputTokens` | bubble `tokenCount.outputTokens` (often 0) |
| `cache_read_tokens` | `tokenUsage.cacheReadTokens` | `Cache Read` | parse `cacheReadTokens` | **not locally available** → 0 / omit (tokenuse) |
| `cache_write_tokens` | `tokenUsage.cacheWriteTokens` | `Input (w/ Cache Write)` | parse `cacheWriteTokens` | **not locally available** → 0 / omit |
| `raw_cost_usd` | prefer `chargedCents/100` (reconciles to spend); else `tokenUsage.totalCents/100` | `Cost` (or Cost to you — clarify product choice) | `totalCents/100` | price from local token estimates (tool-aware pricing in tokenuse) — not Cursor invoice |
| `meta` | `kind`, `conversationId`, `maxMode`, `cursorTokenFee`, `userEmail`, `isHeadless`, … | raw CSV row | `tier`, billing cycle bounds | `composerId`, `bubbleId`, `interaction_mode`, `token_quality`, `timestamp_quality`, workspace URI |

### Explicit non-mappings / traps

- Do **not** treat local `{0,0}` bubble tokens as real zeros for billing (forum + tokenuse).
- Do **not** sum context-meter `totalUsedTokens` across reparses without dedup (`cursor:composer-input:<id>` pattern in tokenuse).
- Aggregated API lacks per-request `id`/`ts` — emit cycle aggregates, not fake turn events.
- `ai_code_hashes` / `scored_commits` map to **code-attribution** metrics, not `UsageEvent` token spend.

## Permissions

| Need | Why | Source |
|------|-----|--------|
| Read access to Cursor `state.vscdb` (+ WAL/SHM) | Auth token + local composers | openusage, cursor-usage-meter |
| Read access to `~/.cursor/ai-tracking/*.db`, `chats/**/store.db`, transcripts | Local enrichment | tokenuse, openusage |
| On Windows: read `%APPDATA%\Cursor\...` | Host UI store | all path docs |
| On WSL2 tracking Windows UI: read `/mnt/c/Users/<user>/AppData/Roaming/Cursor/...` | Cross-host discovery | agentgrep ADR 0009 |
| Network egress to `api2.cursor.sh` and/or `cursor.com` | Usage APIs / CSV | openusage, agent-walker, cursor-usage-meter |
| Optional: Team Admin API key (`admin:*`) | Official per-event ledger | cursor.com Admin API |
| Optional: macOS Full Disk Access | Another app’s Application Support | agent-walker caveat |
| SQLite open **read-only** | Avoid locking / corrupting live Cursor DB | tokenuse |
| WSL Remote: run usage extensions as **UI** kind | Extension host on remote cannot see Windows `state.vscdb` | cursor-usage-meter README (`remote.extensionKind`) |

No evidence found of an official public individual “API key for usage” analogous to the team Admin API; session JWT from local DB (or cookie) is the documented community approach.

## Win vs WSL2

| Topic | Evidence | Confidence |
|-------|----------|------------|
| Cursor UI on Windows editing a WSL project writes **IDE chat/auth** under Windows Roaming Cursor, **not** distro `~/.config/Cursor` | agentgrep `cursor-ide.md`; ADR 0009; Traces WSL article | **CONFIRMED** |
| From WSL, probe `/mnt/c/Users/*/AppData/Roaming/Cursor/User/...` (override `AGENTGREP_WSL_USERS_ROOT`, default `/mnt/c/Users`) | agentgrep | **CONFIRMED** |
| Native Linux path `~/.config/Cursor/...` empty when only Windows Cursor UI used | Traces: CLI in WSL looking at Linux path finds nothing | **CONFIRMED** |
| Remote WSL installs server under `~/.cursor-server/`; UI storage remains on Windows | Traces writeup | **PARTIAL** (one secondary source) |
| Extensions that read `state.vscdb` must run on **local UI** host when using WSL/SSH (`remote.extensionKind: ["ui"]`) | cursor-usage-meter README | **CONFIRMED** |
| `workspace.json` may use `vscode-remote://wsl+<distro>/...` URIs; map to Linux cwd for project attribution | agentgrep ADR 0009 | **CONFIRMED** |
| Opening `\\wsl.localhost\...` as a **local** Windows workspace (not Remote WSL) breaks agent tool pathing | Cursor forum bug thread | **CONFIRMED** (ops caveat; not usage-DB layout) |
| Whether `~/.cursor/ai-tracking` and `~/.cursor/chats` live in Windows `%USERPROFILE%\.cursor` vs WSL `$HOME/.cursor` for Remote-WSL sessions | Mixed: openusage uses `~/.cursor` relative to process home; DEV.to uses user home; no single authoritative matrix for Remote WSL | **UNKNOWN / PARTIAL** — next step: inspect both homes on a dual setup |

Practical discovery order for tokenTracer on WSL2:

1. Detect WSL (`/proc/version` contains `microsoft`).
2. Resolve Windows Cursor User dir via `/mnt/c/Users/<winuser>/AppData/Roaming/Cursor/User`.
3. Also check native `~/.config/Cursor` (rare if UI never ran as Linux build).
4. Resolve `~/.cursor` in **both** `$HOME` and `/mnt/c/Users/<winuser>/.cursor` until empirically confirmed which host owns tracking/chats for the user’s install mode.
5. Prefer API/CSV for costs; use local DBs for session/model enrichment and auth.

## Evidence sources (URLs)

### Primary docs fetched for this card

- https://tokenuse.app/docs/development/tools/cursor/
- https://openusage.sh/docs/providers/cursor/
- https://github.com/tony/agentgrep/blob/master/docs/backends/cursor-ide.md  
  (raw: https://raw.githubusercontent.com/tony/agentgrep/master/docs/backends/cursor-ide.md)
- https://raw.githubusercontent.com/tony/agentgrep/master/docs/dev/adr/0009-cross-host-discovery.md
- https://github.com/tansdf/cursor-usage-meter (README raw: https://raw.githubusercontent.com/tansdf/cursor-usage-meter/master/README.md)
- https://cursor.com/docs/account/teams/admin-api
- https://raw.githubusercontent.com/datell1357/AI-Usage-for-Windows/main/docs/providers/cursor.md
- https://raw.githubusercontent.com/shadeov/cursor-costs-raycast/8333fd45/.cursor/rules/cursor-api.mdc
- https://raw.githubusercontent.com/miiiiiiich/agent-walker/master/docs/cursor.md
- https://raw.githubusercontent.com/robinebers/openusage/main/docs/providers/cursor.md
- https://marketplace.visualstudio.com/items?itemName=ClearMeasureLabs.cursor-usage-status
- https://dev.to/vikram_ray/i-reverse-engineered-cursors-ai-agent-heres-everything-it-does-behind-the-scenes-3d0a
- https://forum.cursor.com/t/cursordiskkv-table-records-always-show-0-for-tokencount/155984
- https://forum.cursor.com/t/how-to-obtain-token-usage-per-request/168317
- https://github.com/getagentseal/codeburn/issues/574 / PR discussion on context tokens
- https://cdn.jsdelivr.net/npm/@qoder-ai/better-harness@0.6.6/docs/specs/2026-08-28-cursor-composer-context-usage.md
- https://cdn.jsdelivr.net/npm/@tokscale/cli@1.2.0/dist/cursor.d.ts
- https://docs.rs/tracedecay/latest/src/tracedecay/sessions/cursor_composer.rs.html
- https://jacquesverre.com/blog/cursor-extension
- https://github.com/mikhailsal/cursor-chronicle/blob/main/README.md
- https://traces.com/s/jn7cjgm19f3tyhbc8gxgzxxk7n82bjvw (WSL path note)
- https://forum.cursor.com/t/agent-tools-fail-on-wsl-workspace-opened-via-wsl-localhost-path-resolves-to-c-home-glob-times-out-shell-intermittent-cursor-3-9-16/165471

### Known tools that already parse Cursor usage

| Tool | What it reads | URL |
|------|---------------|-----|
| **tokenuse.app** | Joined local corpus: `state.vscdb`, `store.db`, AgentKv, transcripts, `ai-code-tracking.db`; estimates tokens; no reliable local cache | https://tokenuse.app/docs/development/tools/cursor/ |
| **openusage** | `api2.cursor.sh` DashboardService + local tracking/state DBs; plan spend + per-model aggregates | https://openusage.sh/docs/providers/cursor/ |
| **agentgrep** | Prompt/chat discovery from `state.vscdb` (+ WSL cross-host); not a billing meter | https://github.com/tony/agentgrep/blob/master/docs/backends/cursor-ide.md |
| **cursor-usage-meter** (tansdf) | `cursorAuth/accessToken` from `state.vscdb` → `api2.cursor.sh` usage; WSL UI-kind note | https://github.com/tansdf/cursor-usage-meter |
| **Cursor Usage Status** (ClearMeasureLabs) | Same DashboardService /auth/usage pattern | Marketplace item above |
| **AI-Usage-for-Windows** | Detailed DashboardService samples + oauth refresh | GitHub docs above |
| **agent-walker** | CSV export for per-event tokens/cost | docs/cursor.md above |
| **codeburn / tokscale** | Context meter + CSV/API discussions | issues / d.ts above |

## Gaps / UNKNOWN / next steps

1. **Live Windows + WSL2 inventory** on a real machine: confirm which of `$HOME/.cursor` vs `/mnt/c/Users/<u>/.cursor` holds `ai-tracking` and `chats` when using Remote WSL vs UNC-local open.  
   - Commands: `ls` both trees; `sqlite3` presence checks; compare mtimes after one Agent turn.
2. **Capture one live `GetAggregatedUsageEvents` JSON** and one CSV header row from a Pro individual account (schema drift risk — undocumented).
3. **Bubble schema drift**: document current `_v` and whether any non-zero `tokenCount` still appears on latest Cursor builds.
4. **Hooks / afterAgentResponse payload** (forum thread) as an alternate per-turn local stream — not fully researched in this card; may supply `input_tokens`/`cache_*` when enabled.
5. **Enterprise vs Pro field differences** for `/auth/usage` vs `planUsage` — partially documented by marketplace extension; needs fixture matrix.
6. **Idempotent event IDs** for CSV rows (no stable server event id in CSV docs) — design hash strategy.
7. **Do not invent** additional paths (e.g. LevelDB, logs) as token sources without new evidence; codeburn reports billed totals are **not** on disk.

## BLOCK reasons if any

| Block | Reason | Impact |
|-------|--------|--------|
| No local Cursor install in this research environment | Cannot empirically verify Win/WSL dual paths or live schemas | Paths remain literature-CONFIRMED; Win/WSL agent-home ownership stays PARTIAL |
| Undocumented individual APIs | `api2.cursor.sh` / CSV may change without notice | Production collector needs version probes + graceful degrade |
| Admin API team-only | Individual users lack official per-event API | Must use JWT/CSV path for solo Pro |
| Local billed tokens unavailable | Current builds zero-out bubble tokenCount; cache not stored | Local-only collectors cannot meet accurate `UsageEvent` cost requirements |

**No policy/safety BLOCK** for this research task itself (public docs only; no exploit/auth theft performed beyond documenting publicly described read-only token location).

---

*Card generated for tokenTracer research. Evidence-only; no claimed support beyond cited sources.*

---

## Local host probe (NB-T3261) — 2026-09-11

Upgrades several UNKNOWN/PARTIAL rows above.

| Observation | Result | Confidence |
|-------------|--------|------------|
| Win `state.vscdb` | `/mnt/c/Users/T3261/AppData/Roaming/Cursor/User/globalStorage/state.vscdb` exists, **~735 MiB**, mtime 2026-08-31 | **CONFIRMED** |
| WSL native `~/.config/Cursor/.../state.vscdb` | **absent** | **CONFIRMED** |
| Tables | `ItemTable` (543), `cursorDiskKV` (72553), `composerHeaders` | **CONFIRMED** |
| Auth keys in ItemTable | `cursorAuth/accessToken`, `refreshToken`, `cachedEmail`, stripe membership keys present (values not exported) | **CONFIRMED** |
| cursorDiskKV | `composerData:*` ≈202 keys; `bubbleId:*` ≈21119; `agentKv:blob:*` abundant | **CONFIRMED** |
| Bubble `tokenCount` | Observed `{inputTokens:0, outputTokens:0}` on sample (matches “current builds write zeros”) | **CONFIRMED** |
| Composer meter | `promptTokenBreakdown.totalUsedTokens` + `contextTokensUsed` present (e.g. 85145); `modelConfig.modelName` e.g. `grok-4.6` | **CONFIRMED** |
| `~/.cursor/ai-tracking/ai-code-tracking.db` (Win profile) | Present (~9.1 MiB); tables `ai_code_hashes` (29518), `scored_commits`, `conversation_summaries`, … | **CONFIRMED** |
| `aiCodeTracking.dailyStats.*` in ItemTable | Line-accept stats only (not token ledger) | **CONFIRMED** |
| `~/.cursor/chats` on Win | Directory exists | **CONFIRMED** |

### Probe-adjusted confidence

| Area | Was | Now |
|------|-----|-----|
| Live Win/WSL field dumps | UNKNOWN | **CONFIRMED** (this host) |
| WSL: Cursor UI data on Windows AppData | CONFIRMED (docs) | **CONFIRMED** (live: Win DB present, WSL config Cursor absent) |
| Local bubble tokens as billing | unreliable | **CONFIRMED unreliable** on this host (zeros) |
| Context-meter input | documented | **CONFIRMED** via `promptTokenBreakdown` |

**Importer note:** Prefer Dashboard/Admin API for spend dollars; local DB is CONFIRMED for discovery/auth/model/context-meter, PARTIAL for per-turn billed tokens.

---

## macOS appendix (AC v1.2) — 2026-09-11

| Field | Value |
|-------|-------|
| Target | macOS 13+; Apple Silicon required path; Intel best-effort |
| Live Mac probe this pass | **UNKNOWN** (no registered macOS machine; research = public docs + prior Win/WSL schema) |
| Path / format confidence | **CONFIRMED** (multi-source path convention; same SQLite schema as Win) |
| Billable token confidence | **PARTIAL** (same as Win: local bubble unreliable; prefer API/CSV/Admin) |

### Paths (vs Win / WSL)

| Artifact | macOS | Win | Notes |
|----------|-------|-----|-------|
| Global state DB | `~/Library/Application Support/Cursor/User/globalStorage/state.vscdb` | `%APPDATA%\\Cursor\\User\\globalStorage\\state.vscdb` | Same `ItemTable` / `cursorDiskKV` / `composerHeaders` shape |
| Workspace DBs | `~/Library/Application Support/Cursor/User/workspaceStorage/<hash>/state.vscdb` (+ `workspace.json`) | under `%APPDATA%\\Cursor\\User\\workspaceStorage\\` | Same |
| Agent home | `~/.cursor/` (`ai-tracking/`, `chats/`, `projects/`) | `%USERPROFILE%\\.cursor\\` | Same relative layout; no WSL dual-host split |
| Insiders | `~/Library/Application Support/Cursor - Insiders/...` | Insiders under AppData | Optional |

Sources: tokenuse, openusage, agentgrep, ccs `cursor-auth.ts`, vibe-replay, PokeTokenBar README, convoptics cursor-schema.

### Permissions (macOS)

| Need | Detail | Confidence |
|------|--------|------------|
| Read own `Application Support/Cursor` + `~/.cursor` | User-owned; **FDA not required** for normal home paths | **PARTIAL** (community: FDA mainly for TCC-protected folders / Terminal tooling on locked volumes) |
| TCC Desktop/Documents/Downloads | Irrelevant to reading Cursor stores under Library / `~/.cursor` | CONFIRMED distinction |
| Live SQLite | Open `mode=ro` (+ `immutable=1` fallback) while Cursor runs | CONFIRMED pattern (same as Win) |
| Optional API | HTTPS + Bearer from `ItemTable.cursorAuth/accessToken` | Same as Win |

**BLOCK if:** claiming FDA is mandatory for every Mac install without evidence — leave as optional troubleshooting, not hard install AC.

### Apple Silicon vs Intel

| Topic | Finding | Confidence |
|-------|---------|------------|
| Data paths | Identical (home + Library) | **CONFIRMED** |
| Live tokenTracer binary on Intel | **UNKNOWN** this pass | AC says Intel best-effort |
| Electron/Cursor on both arches | Public tools document both | PARTIAL |

### Mapping

Reuse Win UsageEvent mapping. No WSL `/mnt/c/...` discovery on Mac. Still do **not** treat bubble `{0,0}` as billed usage.

---

## Related appendix

- **Other Models 池區分：** [`EC-cursor-other-models-v1.md`](./EC-cursor-other-models-v1.md)（`tier` / `apiPercentUsed` ≠ model 字串 `"Other"`）

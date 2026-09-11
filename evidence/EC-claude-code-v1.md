# Evidence Card: Claude Code (Anthropic CLI / agent)

- **Agent ID for UsageEvent:** `claude_code`
- **Card version:** v1
- **Research date:** 2026-09-11 (Asia/Taipei)
- **Scope:** Windows native + WSL2
- **Overall confidence:** **CONFIRMED** for local JSONL usage paths/fields; **PARTIAL** for optional remote quota APIs and Desktop agent-mode AppData paths

---

## 1. Local paths (usage / session / history)

### Primary usage source (CONFIRMED)

Session transcripts with per-turn token usage live under the Claude config root’s `projects/` tree:

| Environment | Path | Confidence | Sources |
|---|---|---|---|
| Linux / macOS / WSL2 | `~/.claude/projects/<encoded-cwd>/<session-uuid>.jsonl` | CONFIRMED | [code.claude.com/docs/en/claude-directory](https://code.claude.com/docs/en/claude-directory), [tokenuse Claude Code](https://tokenuse.app/docs/development/tools/claude-code/), [ccusage Claude](https://ccusage.com/guide/claude/), [openusage Claude Code](https://openusage.sh/docs/providers/claude-code/) |
| Linux XDG fallback | `~/.config/claude/projects/**/*.jsonl` | CONFIRMED (parsers search both) | [ccusage Claude](https://ccusage.com/guide/claude/), [openusage Claude Code](https://openusage.sh/docs/providers/claude-code/), [tokenuse Claude Code](https://tokenuse.app/docs/development/tools/claude-code/) |
| Windows native | `%USERPROFILE%\.claude\projects\<encoded-cwd>\<session-uuid>.jsonl` | CONFIRMED | Official: “On Windows, `~/.claude` means `%USERPROFILE%\.claude`” — [code.claude.com/docs/en/configuration](https://code.claude.com/docs/en/configuration); community sample: [gist BoQsc](https://gist.github.com/BoQsc/8b392c3293107edddbd00117ada0fdd2); [claude-dev.tools JSONL format](https://claude-dev.tools/docs/jsonl-format) |

**Layout details (CONFIRMED):**

- Project folder name = working directory with non-alphanumeric chars (incl. `/`, `\`, `_`) replaced by `-` (lossy encoding). Prefer the JSONL top-level `cwd` field over the directory name. Sources: [allaboutcoding session logs](https://allaboutcoding.ghinda.com/where-ai-coding-clis-store-session-logs/), [tokenuse Claude Code](https://tokenuse.app/docs/development/tools/claude-code/), [gist BoQsc](https://gist.github.com/BoQsc/8b392c3293107edddbd00117ada0fdd2).
- One session = one JSONL file named with the session UUID. Source: [code.claude.com/docs/en/claude-directory](https://code.claude.com/docs/en/claude-directory).
- Subagent transcripts: `projects/<encoded>/<sessionId>/subagents/**/agent-*.jsonl` (and nested workflow paths). Source: [tokenuse Claude Code](https://tokenuse.app/docs/development/tools/claude-code/), [code.claude.com/docs/en/claude-directory](https://code.claude.com/docs/en/claude-directory).
- Large tool outputs: `projects/<encoded>/<sessionId>/tool-results/`. Source: [code.claude.com/docs/en/claude-directory](https://code.claude.com/docs/en/claude-directory).

### Env override (CONFIRMED)

- `CLAUDE_CONFIG_DIR` relocates the home config root (settings, session history, plugins). Can be one path or comma-separated list for parsers. Sources: [code.claude.com/docs/en/configuration](https://code.claude.com/docs/en/configuration), [ccusage env vars](https://ccusage.com/guide/environment-variables), [tokenuse Claude Code](https://tokenuse.app/docs/development/tools/claude-code/).

### Related files (not primary per-turn usage, but useful)

| File | Role | Confidence | Sources |
|---|---|---|---|
| `~/.claude/stats-cache.json` | Aggregated token/cost rollups for `/usage` | CONFIRMED | [code.claude.com/docs/en/claude-directory](https://code.claude.com/docs/en/claude-directory), [openusage Claude Code](https://openusage.sh/docs/providers/claude-code/) |
| `~/.claude/history.jsonl` | Prompt recall history (not billed usage) | CONFIRMED | [code.claude.com/docs/en/claude-directory](https://code.claude.com/docs/en/claude-directory) |
| `~/.claude.json` | OAuth / subscription metadata / org UUID | CONFIRMED | [code.claude.com/docs/en/configuration](https://code.claude.com/docs/en/configuration), [openusage Claude Code](https://openusage.sh/docs/providers/claude-code/) |
| `~/.claude/.credentials.json` | OAuth access token (file-based on Linux/WSL) | CONFIRMED | [openusage Claude Code](https://openusage.sh/docs/providers/claude-code/), [GitHub issue #47661](https://github.com/anthropics/claude-code/issues/47661) |
| `~/.claude/settings.json` | Includes `cleanupPeriodDays` (default 30) | CONFIRMED | [code.claude.com/docs/en/settings](https://code.claude.com/docs/en/settings), [code.claude.com/docs/en/claude-directory](https://code.claude.com/docs/en/claude-directory) |

### Desktop agent-mode paths (PARTIAL — separate from CLI)

tokenuse documents Desktop local-agent-mode sessions:

| Platform | Path |
|---|---|
| Windows | `%APPDATA%/Claude/local-agent-mode-sessions/**/projects/**/*.jsonl` |
| Linux | `~/.config/Claude/local-agent-mode-sessions/**/projects/**/*.jsonl` |
| macOS | `~/Library/Application Support/Claude/local-agent-mode-sessions/**/projects/**/*.jsonl` |

Source: [tokenuse Claude Code](https://tokenuse.app/docs/development/tools/claude-code/). **PARTIAL:** not mirrored in Anthropic’s official claude-directory page as a CLI path; treat as Desktop-only.

---

## 2. File formats

| Artifact | Format | Confidence | Sources |
|---|---|---|---|
| Session transcripts | **JSONL** (one JSON object per line, append-only) | CONFIRMED | [code.claude.com/docs/en/claude-directory](https://code.claude.com/docs/en/claude-directory), [tokenuse](https://tokenuse.app/docs/development/tools/claude-code/), [claude-dev.tools](https://claude-dev.tools/docs/jsonl-format) |
| `stats-cache.json` | **JSON** aggregate | CONFIRMED | [code.claude.com/docs/en/claude-directory](https://code.claude.com/docs/en/claude-directory) |
| `history.jsonl` | **JSONL** prompts | CONFIRMED | [code.claude.com/docs/en/claude-directory](https://code.claude.com/docs/en/claude-directory) |
| SQLite | **Not used** for Claude Code CLI transcripts | CONFIRMED (absence across official + parsers) | No SQLite mentioned in official directory docs or ccusage/tokenuse/openusage Claude parsers |

**Retention:** `cleanupPeriodDays` default **30**, minimum **1**; `0` rejected. Disable writing with `CLAUDE_CODE_SKIP_PROMPT_HISTORY=1`. Sources: [code.claude.com/docs/en/claude-directory](https://code.claude.com/docs/en/claude-directory), [code.claude.com/docs/en/settings](https://code.claude.com/docs/en/settings).

---

## 3. APIs (optional / remote only)

Local JSONL is sufficient for per-turn tokens. Optional remote endpoints used by community trackers for **quota gauges** (not required for UsageEvent token fields):

| Endpoint | Auth | Purpose | Confidence | Sources |
|---|---|---|---|---|
| `GET https://api.anthropic.com/api/oauth/usage` | Claude Code OAuth token from `~/.claude/.credentials.json` | 5h / 7d utilization | PARTIAL (community-documented internal surface) | [openusage Claude Code](https://openusage.sh/docs/providers/claude-code/) |
| `GET https://claude.ai/api/organizations/{org_uuid}/usage` | Browser session cookies (macOS Desktop) | Org-level usage | PARTIAL | [openusage Claude Code](https://openusage.sh/docs/providers/claude-code/) |

**BLOCK reason if relying on remote APIs alone:** these are undocumented/internal; may change without notice; not needed for local token spend tracking.

---

## 4. Sample fields (tokens / model / cost / timestamps)

### Assistant turn (authoritative for tokens) — CONFIRMED

From community Windows sample ([gist BoQsc](https://gist.github.com/BoQsc/8b392c3293107edddbd00117ada0fdd2)) and [tokenuse schema](https://tokenuse.app/docs/development/tools/claude-code/):

```json
{
  "type": "assistant",
  "uuid": "f2fb607f-25ab-4aa5-9947-3d8972dd2c50",
  "timestamp": "2025-08-20T19:42:30.851Z",
  "sessionId": "13d805c4-54f1-415f-b72d-bc296e13bed2",
  "cwd": "C:\\Users\\Windows10_new\\Documents\\quickstuff",
  "requestId": "req_011CSKXqP7gqYtUUbfWT1nLS",
  "message": {
    "id": "msg_01KvmXjTgaNTMV7UFw9Ykkof",
    "role": "assistant",
    "model": "claude-sonnet-4-20250514",
    "usage": {
      "input_tokens": 3,
      "output_tokens": 28,
      "cache_creation_input_tokens": 3296,
      "cache_read_input_tokens": 11331,
      "cache_creation": {
        "ephemeral_5m_input_tokens": 3296,
        "ephemeral_1h_input_tokens": 0
      },
      "service_tier": "standard"
    },
    "content": [{ "type": "text", "text": "..." }]
  }
}
```

**Observed / documented usage fields:**

| Field | Present | Notes | Confidence |
|---|---|---|---|
| `message.usage.input_tokens` | yes | Non-cache input | CONFIRMED |
| `message.usage.output_tokens` | yes | | CONFIRMED |
| `message.usage.cache_creation_input_tokens` | yes | Cache write | CONFIRMED |
| `message.usage.cache_read_input_tokens` | yes | Cache read; **not** included in `input_tokens` | CONFIRMED ([tokenuse](https://tokenuse.app/docs/development/tools/claude-code/)) |
| `message.usage.cache_creation.ephemeral_5m/1h_input_tokens` | yes (newer) | TTL split for pricing | CONFIRMED ([tokenuse](https://tokenuse.app/docs/development/tools/claude-code/)) |
| `message.usage.speed` | optional | `"standard"` \| `"fast"` | CONFIRMED ([tokenuse](https://tokenuse.app/docs/development/tools/claude-code/)) |
| `message.model` | yes | | CONFIRMED |
| `timestamp` | yes | ISO-8601 UTC | CONFIRMED |
| Cost in JSONL | **no** | Cost is **computed locally** from public API rates | CONFIRMED ([openusage](https://openusage.sh/docs/providers/claude-code/), [ccusage](https://ccusage.com/guide/claude/), [tokenuse](https://tokenuse.app/docs/development/tools/claude-code/)) |

**Streaming quirk (CONFIRMED):** one API message may appear as multiple `type:"assistant"` JSONL lines sharing the same `message.id` and repeating the same final `usage`; parsers must dedupe by `message.id`. Source: [tokenuse Claude Code](https://tokenuse.app/docs/development/tools/claude-code/).

---

## 5. Mapping → UsageEvent

Target shape:

```ts
UsageEvent {
  id, agent: "claude_code", source, ts, model,
  input_tokens, output_tokens,
  cache_read_tokens?, cache_write_tokens?,
  raw_cost_usd?, meta?
}
```

| UsageEvent field | Source mapping | Confidence |
|---|---|---|
| `id` | Prefer `message.id`; fallback `uuid` or `${sessionId}:${uuid}` | CONFIRMED pattern ([tokenuse](https://tokenuse.app/docs/development/tools/claude-code/) uses `message.id` as dedup key) |
| `agent` | `"claude_code"` (constant) | — |
| `source` | Absolute path of JSONL file, or `claude_code:jsonl` | CONFIRMED practice |
| `ts` | Top-level `timestamp` | CONFIRMED |
| `model` | `message.model` | CONFIRMED |
| `input_tokens` | `message.usage.input_tokens` | CONFIRMED |
| `output_tokens` | `message.usage.output_tokens` | CONFIRMED |
| `cache_read_tokens` | `message.usage.cache_read_input_tokens` | CONFIRMED |
| `cache_write_tokens` | `message.usage.cache_creation_input_tokens` (or max with ephemeral 5m+1h sum) | CONFIRMED ([tokenuse](https://tokenuse.app/docs/development/tools/claude-code/)) |
| `raw_cost_usd` | **Compute** from model + token buckets + speed; do not read from file | CONFIRMED (no cost field in JSONL) |
| `meta` | Suggested: `{ sessionId, requestId, cwd, uuid, service_tier, speed, cache_creation }` | PARTIAL (shape is implementer choice) |

**Filter:** only emit from `type === "assistant"` lines that have `message.usage`. Ignore pure user/system lines for token events.

---

## 6. Permissions

| Claim | Confidence | Source |
|---|---|---|
| Transcripts are **plaintext**, not encrypted at rest | CONFIRMED | [code.claude.com/docs/en/claude-directory](https://code.claude.com/docs/en/claude-directory): “OS file permissions are the only protection.” |
| Readable by the user account that owns the home directory | CONFIRMED | Same official page |
| May contain secrets if tools printed them | CONFIRMED | Same official page |
| Tracker needs **read** access to `projects/**/*.jsonl` (and optionally stats-cache / settings) | CONFIRMED | Implied by all local parsers |

---

## 7. Windows vs WSL2 differences

| Topic | Evidence | Confidence |
|---|---|---|
| Path root on Windows | `%USERPROFILE%\.claude` | CONFIRMED — [code.claude.com/docs/en/configuration](https://code.claude.com/docs/en/configuration) |
| Path root in WSL2 | Linux `~/.claude` inside the distro (e.g. `/home/<user>/.claude`) | CONFIRMED — separate Linux home; see install notes [claw.aguidetocloud.com](https://claw.aguidetocloud.com/anthropic/claude-code/install-windows/) |
| Native Windows vs WSL installs are **separate** | Settings, credentials, sessions do **not** auto-share | CONFIRMED — [claw.aguidetocloud.com](https://claw.aguidetocloud.com/anthropic/claude-code/install-windows/) |
| Encoded project dirs on Windows | Backslashes → hyphens (e.g. `C--Users-...`) | CONFIRMED — [gist BoQsc](https://gist.github.com/BoQsc/8b392c3293107edddbd00117ada0fdd2) |
| `cwd` field uses Windows paths when run natively | e.g. `C:\\Users\\...` | CONFIRMED — same gist |
| Credential isolation bug with `CLAUDE_CONFIG_DIR` on Linux/WSL2 | Credentials may still fall back to `~/.claude/.credentials.json` | PARTIAL — open bug [anthropics/claude-code#47661](https://github.com/anthropics/claude-code/issues/47661) |
| tokenTracer implication | Must scan **both** `%USERPROFILE%\.claude` (Windows) **and** WSL `\\wsl$\Distro\home\<user>\.claude` (or via WSL path) if user runs both | CONFIRMED separation → implementation requirement |

**Sharing (UNKNOWN / not officially documented for Claude):** Unlike Codex, Anthropic docs do not publish a “point WSL at Windows home” recipe. Next step: verify whether setting `CLAUDE_CONFIG_DIR=/mnt/c/Users/<win>/.claude` works end-to-end on WSL2.

---

## 8. Known parsers

| Tool | Role | Sources |
|---|---|---|
| **ccusage** | Reads `~/.config/claude/projects/` + `~/.claude/projects/`; daily/monthly/session/blocks; costs via LiteLLM pricing | [ccusage.com/guide/claude](https://ccusage.com/guide/claude/), [github.com/ryoppippi/ccusage](https://github.com/ryoppippi/ccusage) / [ccusage/ccusage](https://github.com/ccusage/ccusage) |
| **tokenuse** | Full JSONL → ParsedCall mapping; Desktop paths; cache TTL pricing; statusLine sidecar for rate limits | [tokenuse.app/docs/development/tools/claude-code](https://tokenuse.app/docs/development/tools/claude-code/) |
| **openusage** | Local JSONL + stats-cache; optional OAuth usage API; provider id `claude_code` | [openusage.sh/docs/providers/claude-code](https://openusage.sh/docs/providers/claude-code/) |
| **claude-devtools** | JSONL field reference / UI | [claude-dev.tools/docs/jsonl-format](https://claude-dev.tools/docs/jsonl-format) |

---

## UNKNOWN / next investigation steps

| Gap | BLOCK reason | Next steps |
|---|---|---|
| Exact on-disk layout differences between Claude Code versions (flat `*.jsonl` vs nested `sessions/` subdir) | Community blogs disagree slightly with official `projects/<encoded>/<uuid>.jsonl` | Confirm against a live install; prefer official path + `**/*.jsonl` recursive discovery |
| Whether WSL can share Windows `.claude` via `CLAUDE_CONFIG_DIR` | Not in official Win/WSL sharing docs (unlike Codex) | Test on WSL2; watch [issue #47661](https://github.com/anthropics/claude-code/issues/47661) |
| Desktop AppData agent-mode as first-class for tokenTracer | Only in tokenuse, not official CLI docs | Decide product scope: CLI-only vs Desktop too |
| Official Anthropic billing API for Pro/Max dollar spend | Subscription ≠ API cost; local costs are estimates | Do not treat `raw_cost_usd` as card charge |

---

## Sources index

1. https://code.claude.com/docs/en/claude-directory  
2. https://code.claude.com/docs/en/configuration  
3. https://code.claude.com/docs/en/settings  
4. https://tokenuse.app/docs/development/tools/claude-code/  
5. https://ccusage.com/guide/claude/  
6. https://ccusage.com/guide/environment-variables  
7. https://openusage.sh/docs/providers/claude-code/  
8. https://claude-dev.tools/docs/jsonl-format  
9. https://allaboutcoding.ghinda.com/where-ai-coding-clis-store-session-logs/  
10. https://gist.github.com/BoQsc/8b392c3293107edddbd00117ada0fdd2  
11. https://github.com/anthropics/claude-code/issues/47661  
12. https://claw.aguidetocloud.com/anthropic/claude-code/install-windows/

---

## Local host probe (NB-T3261) — 2026-09-11

| Observation | Result | Confidence |
|-------------|--------|------------|
| WSL `~/.claude` | Present; `projects/` with many encoded cwd dirs (e.g. `-home-t3261-projects-omscrm`) | **CONFIRMED** |
| WSL JSONL volume | **1427** `*.jsonl` under `projects/`; **564** under `subagents/` | **CONFIRMED** |
| Sample assistant usage | Live fields: `input_tokens`, `output_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`, `cache_creation.ephemeral_1h_input_tokens` / `ephemeral_5m_input_tokens`, `speed`, `output_tokens_details.thinking_tokens`; top-level `timestamp`, `requestId`, `sessionId`/`session_id`, `cwd`, `message.model`, `message.id` | **CONFIRMED** |
| Dedup risk | Same `message.id` + `requestId` seen on consecutive assistant lines (thinking then tool_use) — must unique_by message+request | **CONFIRMED** |
| Win `%USERPROFILE%\.claude` | Directory exists (native install separate from WSL) | **CONFIRMED** |
| Credentials | WSL `~/.claude/.credentials.json` present — **do not** copy into fixtures | **CONFIRMED** |

### Probe-adjusted confidence

Local JSONL → UsageEvent path for WSL Claude Code: **CONFIRMED**. Win-native `projects/**/*.jsonl` population: still **PARTIAL** (dir exists; session count not finalized in interrupted probe).

---

## macOS appendix (AC v1.2) — 2026-09-11

| Field | Value |
|-------|-------|
| Target | macOS 13+; Apple Silicon path; Intel best-effort |
| Live Mac probe | **UNKNOWN** (no Mac host this pass) |
| Session JSONL path/format | **CONFIRMED** (same Linux layout; official `~/.claude`) |
| OAuth for optional remote | **PARTIAL** (Keychain vs file) |

### Paths (vs Win / WSL)

| Artifact | macOS | Win | WSL |
|----------|-------|-----|-----|
| Sessions | `~/.claude/projects/<encoded-cwd>/<uuid>.jsonl` (+ `.../subagents/`) | `%USERPROFILE%\\.claude\\projects\\...` | `~/.claude/projects/...` |
| Settings / history | `~/.claude/settings.json`, `history.jsonl`, `stats-cache.json` | same under profile | same |
| Config override | `CLAUDE_CONFIG_DIR` | same | same |
| Managed settings | `/Library/Application Support/ClaudeCode/` | `C:\\Program Files\\ClaudeCode\\` | `/etc/claude-code/` |
| Credentials | **Keychain** service `Claude Code-credentials` (primary on recent macOS builds); file `~/.claude/.credentials.json` often absent / not preferred | file `.credentials.json` | file `.credentials.json` |

Sources: code.claude.com settings / claude-directory; ccusage; tokenuse; ai-usagebar keychain.rs; GitHub issues #77697, #89985.

**UsageEvent does not need Keychain** — tokens live in JSONL under `projects/`. Keychain only for optional OAuth quota APIs.

### Permissions

| Need | Detail | Confidence |
|------|--------|------------|
| Read `~/.claude/projects/**` | User home; **FDA not required** for usage import | **CONFIRMED** (home not Desktop/Documents TCC set) |
| TCC Desktop/Documents/Downloads | Affects **Claude Code working on projects there**, not reading `~/.claude` itself | CONFIRMED (issues #90373, #37064, continuum guide) |
| Keychain (optional) | May prompt via `/usr/bin/security`; ACL quirks | PARTIAL — avoid for v1 import path |
| Bundle ID | `com.anthropic.claude-code` for TCC resets | CONFIRMED community |

### Apple Silicon vs Intel

Paths identical. Live Intel runtime for tokenTracer: **UNKNOWN**.

### Mapping

Same as WSL CONFIRMED mapping (`message.usage.*`, dedup `message.id`+`requestId`, include subagents). macOS cell for local JSONL import: **CONFIRMED** (path/format); live host verification: **UNKNOWN**.

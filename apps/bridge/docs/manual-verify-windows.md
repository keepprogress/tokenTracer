# Manual verify — NB-T3261 (Windows + WSL2, read-only)

Date baseline: 2026-09-11 · Host: **NB-T3261** · See also `docs/live-probe-NB-T3261.md`.

**Rules:** read-only; do **not** copy `.credentials.json`, `auth.json`, or Cursor auth tokens; do **not** treat Cursor bubble `tokenCount` as billed.

---

## 0. Distros

```powershell
wsl.exe -l -v
# Expect: Ubuntu-Work Running (default), Ubuntu Stopped — both Version 2
```

---

## 1. Windows native paths

```powershell
$up = $env:USERPROFILE
$ad = $env:APPDATA

# Claude Code
(Get-ChildItem -Path "$up\.claude\projects" -Recurse -Filter *.jsonl -ErrorAction SilentlyContinue |
  Measure-Object).Count
# Live baseline: 36

# Codex sessions
(Get-ChildItem -Path "$up\.codex\sessions" -Recurse -Filter rollout-*.jsonl -ErrorAction SilentlyContinue |
  Measure-Object).Count
# Live baseline: 155

# Codex archived (may be absent)
Test-Path "$up\.codex\archived_sessions"

# Cursor global state (size only — do not parse as billed usage)
Get-Item "$ad\Cursor\User\globalStorage\state.vscdb" |
  Select-Object FullName, Length, LastWriteTime
# Live baseline: ~770 MiB
```

---

## 2. WSL2 Ubuntu-Work (via UNC)

```powershell
$unc = '\\wsl$\Ubuntu-Work\home\t3261'
Test-Path "$unc\.claude\projects"
Test-Path "$unc\.codex\sessions"
# Optional count (may be slow):
(Get-ChildItem -Path "$unc\.claude\projects" -Recurse -Filter *.jsonl -ErrorAction SilentlyContinue |
  Measure-Object).Count
# Live baseline: 1427
(Get-ChildItem -Path "$unc\.codex\sessions" -Recurse -Filter rollout-*.jsonl -ErrorAction SilentlyContinue |
  Measure-Object).Count
# Live baseline: 311
```

---

## 3. WSL2 via `wsl -d` (alternate access)

```powershell
wsl -d Ubuntu-Work -- bash -lc 'echo HOME=$HOME; ls -la ~/.claude/projects 2>/dev/null | head'
wsl -d Ubuntu-Work -- bash -lc 'find ~/.claude/projects -name "*.jsonl" 2>/dev/null | wc -l'
wsl -d Ubuntu-Work -- bash -lc 'find ~/.codex/sessions -name "rollout-*.jsonl" 2>/dev/null | wc -l'
wsl -d Ubuntu-Work -- bash -lc 'ls -la ~/.cursor 2>/dev/null | head'
```

Stopped distro (optional — expect still listed):

```powershell
wsl -d Ubuntu -- bash -lc 'echo ok'   # may start the distro; skip if you must not wake it
```

---

## 4. Bridge CLI (when built on Windows)

```powershell
tokentracer-bridge discover --json
# Confirm sources for claude_code / codex on host=windows AND host=wsl2
# Confirm cursor status=partial
# Confirm no files[] unless --list-files
```

---

## 5. Dual-scan acceptance check

| Check | Pass if |
|-------|---------|
| Win Claude count &gt; 0 | yes (baseline 36) |
| WSL Claude count &gt; 0 | yes (baseline 1427) |
| Win Codex count &gt; 0 | yes (baseline 155) |
| WSL Codex count &gt; 0 | yes (baseline 311) |
| Cursor path listed | yes; **partial**; not billed from tokenCount |
| Missing path | emits TT-F2-003 + next_step (not silent) |

**PASS of this sheet ≠ product PASS** — probe/verification only.

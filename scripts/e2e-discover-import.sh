#!/usr/bin/env bash
# e2e-discover-import.sh — bridge discover → spend import from-discover → total → import status
#
# path-list-v0.2 + 帳本 `spend import from-discover` (authoritative composition).
# ImportMeta sole writer = pricing. NOT a PASS — local POC smoke only.
set -euo pipefail

die() { echo "ERROR: $*" >&2; exit 1; }
log() { echo "$*" >&2; }

# Prefer sibling bridge checkout; fall back to monorepo apps/bridge.
if [[ -z "${BRIDGE_ROOT:-}" ]]; then
  if [[ -f /workspace/tokenTracer-bridge/Cargo.toml ]]; then
    BRIDGE_ROOT=/workspace/tokenTracer-bridge
  elif [[ -f /workspace/tokenTracer/apps/bridge/Cargo.toml ]]; then
    BRIDGE_ROOT=/workspace/tokenTracer/apps/bridge
  else
    HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    if [[ -f "$HERE/src/bin/tokentracer-bridge.rs" ]]; then
      BRIDGE_ROOT="$HERE"
    else
      die "bridge not found; set BRIDGE_ROOT"
    fi
  fi
fi

if [[ -z "${PRICING_ROOT:-}" ]]; then
  if [[ -f /home/box/agent-data/projects/token-spend-tracker/crates/pricing/Cargo.toml ]]; then
    PRICING_ROOT=/home/box/agent-data/projects/token-spend-tracker
  elif [[ -f /workspace/tokenTracer/crates/pricing/Cargo.toml ]]; then
    PRICING_ROOT=/workspace/tokenTracer
  else
    die "pricing not found; set PRICING_ROOT"
  fi
fi

FIXTURE_DIR="${FIXTURE_DIR:-$BRIDGE_ROOT/tests/fixtures}"
[[ -d "$FIXTURE_DIR" ]] || die "fixture dir missing: $FIXTURE_DIR"

WORKDIR="${WORKDIR:-$(mktemp -d -t tt-e2e-XXXXXX)}"
mkdir -p "$WORKDIR"
DISCOVER_JSON="${DISCOVER_JSON:-$WORKDIR/discover.json}"
EVENTS_JSON="${EVENTS_JSON:-$WORKDIR/events.json}"
IMPORT_STATE="${IMPORT_STATE:-$WORKDIR/import-meta.json}"
IMPORT_REPORT="${IMPORT_REPORT:-$WORKDIR/import-report.json}"
TOTAL_JSON="${TOTAL_JSON:-$WORKDIR/total.json}"
STATUS_JSON="${STATUS_JSON:-$WORKDIR/import-status.json}"
KEEP_WORKDIR="${KEEP_WORKDIR:-0}"

bridge() {
  if [[ "$BRIDGE_ROOT" == */apps/bridge ]]; then
    local ws
    ws="$(cd "$BRIDGE_ROOT/../.." && pwd)"
    if [[ -f "$ws/Cargo.toml" ]] && grep -q 'apps/bridge' "$ws/Cargo.toml" 2>/dev/null; then
      (cd "$ws" && cargo run -p tokentracer-bridge --quiet -- "$@")
      return
    fi
  fi
  (cd "$BRIDGE_ROOT" && cargo run --quiet -- "$@")
}

spend() {
  local bin=""
  if [[ "${FORCE_CARGO_RUN:-0}" != "1" ]]; then
    if [[ -x "$PRICING_ROOT/target/release/spend" ]]; then
      bin="$PRICING_ROOT/target/release/spend"
    elif [[ -x "$PRICING_ROOT/target/debug/spend" ]]; then
      bin="$PRICING_ROOT/target/debug/spend"
    fi
  fi
  if [[ -n "$bin" ]]; then
    "$bin" "$@"
  else
    (cd "$PRICING_ROOT" && cargo run -p pricing --bin spend --quiet -- "$@")
  fi
}

log "== tokenTracer e2e discover→from-discover→total (POC) =="
log "BRIDGE_ROOT=$BRIDGE_ROOT"
log "PRICING_ROOT=$PRICING_ROOT"
log "FIXTURE_DIR=$FIXTURE_DIR"
log "WORKDIR=$WORKDIR"

log ""
log "-- 1) bridge discover --json --fixture …"
bridge discover --json --fixture "$FIXTURE_DIR" >"$DISCOVER_JSON"
log "wrote $DISCOVER_JSON"

# Behavior (帳本): readable && status ok|partial; prefer meta.import_path;
# truncated/TT-F2-006 → exit 2; Cursor skipped for local billing.
log ""
log "-- 2) spend import from-discover --limit 0 …"
set +e
spend import from-discover \
  --discover "$DISCOVER_JSON" \
  --limit 0 \
  --state "$IMPORT_STATE" \
  --events-out "$EVENTS_JSON" \
  --report-out "$IMPORT_REPORT" \
  --json | tee "$WORKDIR/from-discover-stdout.json"
fd_rc=${PIPESTATUS[0]}
set -e
if [[ "$fd_rc" -eq 2 ]]; then
  die "from-discover exit 2 (truncated / TT-F2-006) — see $IMPORT_REPORT"
fi
[[ "$fd_rc" -eq 0 ]] || die "from-discover failed with exit $fd_rc"

log ""
log "-- 3) spend total --currency USD --events …"
spend total --currency USD --events "$EVENTS_JSON" | tee "$TOTAL_JSON"

log ""
log "-- 4) spend import status --json --state …"
spend import status --state "$IMPORT_STATE" --json | tee "$STATUS_JSON"

log ""
log "== E2E summary (not a PASS) =="
python3 - "$TOTAL_JSON" "$STATUS_JSON" "$IMPORT_REPORT" <<'PY'
import json, sys
total = json.load(open(sys.argv[1]))
meta = json.load(open(sys.argv[2]))
report = json.load(open(sys.argv[3]))
excerpt = {k: total[k] for k in ("currency", "total", "pricing_mode", "price_table_version") if k in total}
if "by_agent" in total:
    excerpt["by_agent"] = total["by_agent"]
print("SpendSummary excerpt:", json.dumps(excerpt, indent=2), file=sys.stderr)
print("ImportMeta:", json.dumps(meta, indent=2), file=sys.stderr)
print(
    "ImportReport: events_upserted=%s parse_error_count=%s truncated_sources=%s cursor_skipped=%s"
    % (
        report.get("events_upserted"),
        report.get("parse_error_count"),
        report.get("truncated_sources"),
        len(report.get("cursor_sources_skipped") or []),
    ),
    file=sys.stderr,
)
print("---")
print(f"SPEND_TOTAL_USD={total.get('total')}")
print(f"CURRENCY={total.get('currency')}")
print(f"EVENTS_UPSERTED={meta.get('events_upserted')}")
print(f"IMPORT_SOURCE_IDS={meta.get('last_import_source_ids')}")
PY

log "artifacts: DISCOVER=$DISCOVER_JSON EVENTS=$EVENTS_JSON REPORT=$IMPORT_REPORT TOTAL=$TOTAL_JSON STATE=$IMPORT_STATE"
if [[ "$KEEP_WORKDIR" == "1" ]]; then
  log "KEEP_WORKDIR=1 — left $WORKDIR"
fi
log "exit 0 (POC smoke complete; not a PASS)"
exit 0

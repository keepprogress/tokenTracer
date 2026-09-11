#!/usr/bin/env bash
# Refresh UI mock fixtures from real ledger CLI SpendSummary / series JSON (UI-BIND).
# Does not invent prices — parse stdout JSON only from pricing crate.
# Each range.kind is produced by `spend … --range <kind>` on cursor-pools-ranged.json.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"          # apps/ui
MOCK="$ROOT/src/mock"

# Detect ledger repo root (Cargo workspace with pricing crate).
if [[ -n "${LEDGER_ROOT:-}" ]]; then
  LEDGER="$LEDGER_ROOT"
elif [[ -f "$ROOT/../../Cargo.toml" && -d "$ROOT/../../crates/pricing" ]]; then
  LEDGER="$(cd "$ROOT/../.." && pwd)"
elif [[ -f /workspace/tokenTracer/Cargo.toml ]]; then
  LEDGER="/workspace/tokenTracer"
else
  LEDGER="/home/box/agent-data/projects/token-spend-tracker"
fi

if [[ ! -d "$LEDGER/crates/pricing" ]]; then
  echo "error: ledger project not found at $LEDGER (set LEDGER_ROOT)" >&2
  exit 1
fi

EVENTS_RANGED="fixtures/ac-v1.3a/cursor-pools-ranged.json"
EVENTS_POOL="fixtures/ac-v1.3a/cursor-pools.json"
EVENTS_MODEL="fixtures/ac-v1.3/by-model.json"

mkdir -p "$MOCK"
TMPDIR_LOCAL="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_LOCAL"' EXIT

# Extract first top-level JSON object from possibly mixed text → pretty file.
extract_json_file() {
  local in_file="$1"
  local out_path="$2"
  python3 - "$in_file" "$out_path" <<'PY'
import json, sys
raw = open(sys.argv[1], encoding="utf-8").read()
start = raw.find("{")
if start < 0:
    sys.stderr.write("no JSON object found in stdout\n")
    sys.exit(1)
depth = 0
in_str = False
esc = False
end = None
for i, ch in enumerate(raw[start:], start):
    if in_str:
        if esc:
            esc = False
        elif ch == "\\":
            esc = True
        elif ch == '"':
            in_str = False
        continue
    if ch == '"':
        in_str = True
    elif ch == "{":
        depth += 1
    elif ch == "}":
        depth -= 1
        if depth == 0:
            end = i + 1
            break
if end is None:
    sys.stderr.write("unterminated JSON object\n")
    sys.exit(1)
obj = json.loads(raw[start:end])
if not isinstance(obj, dict):
    sys.stderr.write("expected JSON object\n")
    sys.exit(1)
out_path = sys.argv[2]
with open(out_path, "w", encoding="utf-8") as f:
    json.dump(obj, f, indent=2, ensure_ascii=False)
    f.write("\n")
total = obj.get("total")
pts = obj.get("points")
extra = f" points={len(pts)}" if isinstance(pts, list) else f" total={total}"
print(f"wrote {out_path} pricing_mode={obj.get('pricing_mode')}{extra}")
PY
}

run_spend() {
  local subcmd="$1"; shift
  local out_base="$1"; shift
  (
    cd "$LEDGER"
    # Human table may appear on stderr — capture stdout only.
    cargo run -p pricing --bin spend -- "$subcmd" "$@" \
      >"$TMPDIR_LOCAL/${out_base}.out" 2>"$TMPDIR_LOCAL/${out_base}.err"
  )
}

echo "==> by-model ($EVENTS_MODEL) → ledger-by-model.json"
run_spend by-model by-model --currency USD --events "$EVENTS_MODEL"
extract_json_file "$TMPDIR_LOCAL/by-model.out" "$MOCK/ledger-by-model.json"

echo "==> by-pool ($EVENTS_POOL) → ledger-by-pool.json (all, non-ranged fixture)"
run_spend by-pool by-pool --currency USD --events "$EVENTS_POOL"
extract_json_file "$TMPDIR_LOCAL/by-pool.out" "$MOCK/ledger-by-pool.json"

# Per-range SpendSummary + series from ranged events (real CLI --range).
KINDS=(all today 7d 30d 90d)
for KIND in "${KINDS[@]}"; do
  echo "==> by-pool --range $KIND ($EVENTS_RANGED) → spend-total-$KIND.json"
  run_spend by-pool "total-$KIND" --currency USD --events "$EVENTS_RANGED" --range "$KIND"
  extract_json_file "$TMPDIR_LOCAL/total-$KIND.out" "$MOCK/spend-total-$KIND.json"

  if [[ "$KIND" == "today" ]]; then
    # Panel has no today series; skip (UI maps panel ranges only).
    echo "    (skip series for today)"
    continue
  fi

  echo "==> series --grain day --range $KIND → spend-series-$KIND.json"
  run_spend series "series-$KIND" --grain day --currency USD --events "$EVENTS_RANGED" --range "$KIND"
  extract_json_file "$TMPDIR_LOCAL/series-$KIND.out" "$MOCK/spend-series-$KIND.json"
done

echo "==> done."
echo "    spend-total-*.json / spend-series-*.json ← real --range on cursor-pools-ranged.json"
echo "    ledger-by-pool.json ← cursor-pools.json (all)"
echo "    ledger-by-model.json ← by-model.json"

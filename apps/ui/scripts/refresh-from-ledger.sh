#!/usr/bin/env bash
# Refresh UI mock fixtures from real ledger CLI SpendSummary JSON (UI-BIND v0.3).
# Does not invent prices — parse stdout JSON only from pricing crate.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LEDGER="${LEDGER_ROOT:-/home/box/agent-data/projects/token-spend-tracker}"
MOCK="$ROOT/src/mock"

if [[ ! -d "$LEDGER" ]]; then
  echo "error: ledger project not found at $LEDGER (set LEDGER_ROOT)" >&2
  exit 1
fi

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
print(f"wrote {out_path} total={obj.get('total')} pricing_mode={obj.get('pricing_mode')}")
PY
}

echo "==> by-model (fixtures/ac-v1.3/by-model.json)"
(
  cd "$LEDGER"
  # Human table may appear on stderr — capture stdout only.
  cargo run -p pricing --bin spend -- by-model --currency USD --events fixtures/ac-v1.3/by-model.json \
    >"$TMPDIR_LOCAL/by-model.out" 2>"$TMPDIR_LOCAL/by-model.err"
)
# Show non-cargo stderr noise (human table) for visibility
if grep -q 'by_model\|model' "$TMPDIR_LOCAL/by-model.err" 2>/dev/null; then
  echo "(by-model human table on stderr — ignored; using stdout JSON)"
fi
extract_json_file "$TMPDIR_LOCAL/by-model.out" "$MOCK/ledger-by-model.json"

echo "==> by-pool (fixtures/ac-v1.3a/cursor-pools.json) — primary all summary"
(
  cd "$LEDGER"
  cargo run -p pricing --bin spend -- by-pool --currency USD --events fixtures/ac-v1.3a/cursor-pools.json \
    >"$TMPDIR_LOCAL/by-pool.out" 2>"$TMPDIR_LOCAL/by-pool.err"
)
extract_json_file "$TMPDIR_LOCAL/by-pool.out" "$MOCK/ledger-by-pool.json"

# spend-total-*.json: SAME real by-pool totals; only range.kind differs.
# Honest: CLI --range not wired yet; UI does not invent scaled amounts.
python3 - "$MOCK" <<'PY'
import json, sys
from pathlib import Path
mock = Path(sys.argv[1])
pool = json.loads((mock / "ledger-by-pool.json").read_text())
for kind, name in [
    ("all", "spend-total-all.json"),
    ("today", "spend-total-today.json"),
    ("7d", "spend-total-7d.json"),
    ("30d", "spend-total-30d.json"),
    ("90d", "spend-total-90d.json"),
]:
    out = dict(pool)
    out["range"] = {**(pool.get("range") or {}), "kind": kind}
    path = mock / name
    path.write_text(json.dumps(out, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"wrote {path} range.kind={kind} total={out.get('total')}")
PY

echo "==> done."
echo "    spend-total-*.json ← ledger-by-pool (cursor-pools dual pools)."
echo "    ledger-by-model.json ← by-model multi-agent fixture."

#!/usr/bin/env bash
# RGS aci-smoke.sh v0.1 — ULYS-191 §4.3.1 brief v0.1
#
# 4 step verify (per IDE1.0 cli_smoke.sh 1:1 pattern):
#   1. cargo build --release -p rgs-flash-mock  (含 aci-emitter git dep)
#   2. cargo test -p rgs-flash-mock --test aci_integration test_emit_smoke_assertion_v0_1 -- --nocapture
#      (RGS 是 HTTP server, 没有 CLI, smoke 走 cargo test 间接 emit)
#   3. 验 IT-2 输出 JSON 含 10 必填字段
#   4. 验 schema_version == "0.1.0-draft"
#
# 守门:
# - #9 subprocess: 仅调 cargo build/test, 不调任意用户脚本
# - #13 W/T/M: 系统层 smoke (集成层由 tests/aci_integration.rs 覆盖)
set -euo pipefail

WORKDIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$WORKDIR"

echo "=== RGS aci-smoke.sh v0.1 ==="
echo "WORKDIR=$WORKDIR"
echo

# Step 1: 编译 (含 aci-emitter git dep)
echo "[step 1/4] cargo build --release -p rgs-flash-mock"
cargo build --release -p rgs-flash-mock 2>&1 | tail -5
echo

# Step 2: emit (通过 cargo test --nocapture 触发 emit_smoke_assertion_json)
echo "[step 2/4] cargo test -p rgs-flash-mock --test aci_integration test_emit_smoke_assertion_v0_1 -- --nocapture"
# 抓 IT-2 的 stdout 输出 (=== emit_smoke_assertion JSON === ... === END ===)
JSON_OUT="$(cargo test -p rgs-flash-mock --test aci_integration test_emit_smoke_assertion_v0_1 -- --nocapture 2>&1 || true)"
echo "$JSON_OUT" | grep -E "(test result|test_emit_smoke_assertion_v0_1|emit_smoke_assertion JSON)" | head -10
echo

# 抽 eprintln 出来的 pretty JSON (在 === emit_smoke_assertion JSON === 与 === END === 之间)
PRETTY_JSON="$(echo "$JSON_OUT" | sed -n '/=== emit_smoke_assertion JSON ===/,/=== END ===/p' | sed '1d;$d')"
if [ -z "$PRETTY_JSON" ]; then
    echo "FAIL: cannot extract pretty JSON from cargo test output"
    echo "$JSON_OUT" | tail -20
    exit 1
fi

# Step 3: 验 10 必填字段全在
echo "[step 3/4] verify 10 required fields"
MISSING=""
for field in assertion_id aci_version layer scope expect actual status severity reasoning captured_at; do
    if ! echo "$PRETTY_JSON" | grep -q "\"$field\""; then
        MISSING="$MISSING $field"
    fi
done
if [ -n "$MISSING" ]; then
    echo "FAIL: missing fields:$MISSING"
    exit 1
fi
echo "  OK: all 10 required fields present"
echo

# Step 4: 验 aci_version == 0.1.0-draft
echo "[step 4/4] verify aci_version == 0.1.0-draft"
if ! echo "$PRETTY_JSON" | grep -q '"aci_version": "0.1.0-draft"'; then
    echo "FAIL: aci_version != 0.1.0-draft"
    echo "$PRETTY_JSON" | head -5
    exit 1
fi
echo "  OK: aci_version = 0.1.0-draft"
echo

echo "PASS: aci-smoke.sh 4 step verify complete"

#!/usr/bin/env bash
# rgs-flash-mock 新增域 fixture 覆盖自检
# 单一事实来源: 期望清单从 scripts/regression-test-8-domain-extension.sh 解析, 不手写第二份
# 校验项: fixture 文件存在 / JSON 合法 (python json.load) / rpcs.rpc_name 覆盖期望 RPC
# 退出码: 0 = 全部满足; 1 = 有缺失 / JSON 非法 / 未覆盖 (打印具体缺什么)
set -uo pipefail
cd "$(dirname "$0")/.."

files_ok=0; files_total=0; rpcs_ok=0; rpcs_total=0; fail=0

EXPECTED_RPCS=$(mktemp 2>/dev/null) || { echo "❌ mktemp 失败 (无法建临时文件)"; exit 1; }
WANT_RPCS=$(mktemp 2>/dev/null) || { rm -f "$EXPECTED_RPCS"; echo "❌ mktemp 失败 (无法建临时文件)"; exit 1; }
trap 'rm -f "$EXPECTED_RPCS" "$WANT_RPCS"' EXIT INT TERM

# 从 8 域脚本解析 "scene_ext/GetCombatScene" 形式的 RPC 清单
extract_expected() {
  python - "$1" <<'PY'
import re, pathlib, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
src = pathlib.Path("scripts/regression-test-8-domain-extension.sh")
txt = src.read_text(encoding="utf-8", errors="replace")
items = sorted({f"{a}/{b}" for a, b in re.findall(r"([a-z0-9_]+_ext)/([A-Za-z0-9_]+)", txt)})
pathlib.Path(sys.argv[1]).write_text("".join(f"{i}\n" for i in items), encoding="utf-8")
print(f"  期望清单来源: {src.as_posix()} (解析出 {len(items)} RPC)")
PY
}

# 校验单个 fixture: JSON 合法 + rpcs.rpc_name 覆盖 want 文件里的期望 RPC
validate_fixture() {
  python - "$1" "$2" <<'PY'
import json, pathlib, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
fixture = pathlib.Path(sys.argv[1])
want = [l.strip() for l in pathlib.Path(sys.argv[2]).read_text(encoding="utf-8").splitlines() if l.strip()]
try:
    data = json.loads(fixture.read_text(encoding="utf-8"))
except Exception as exc:
    print(f"  ❌ JSON 非法: {fixture.as_posix()}: {exc}")
    sys.exit(1)
rpcs = data.get("rpcs")
if not isinstance(rpcs, dict):
    print(f"  ❌ 缺 rpcs 对象: {fixture.as_posix()}")
    sys.exit(1)
names = {v.get("rpc_name") for v in rpcs.values()
         if isinstance(v, dict) and isinstance(v.get("rpc_name"), str)}
missing = [r for r in want if r not in names]
if missing:
    have = ", ".join(sorted(names)) or "无"
    print(f"  ❌ {fixture.as_posix()} 缺 RPC: {', '.join(missing)} (已有: {have})")
    sys.exit(1)
print(f"  ✅ {fixture.as_posix()}: {len(want)}/{len(want)} RPC 覆盖")
PY
}

echo "=== rgs-flash-mock 新增域 fixture 覆盖自检 ==="

# 1) 从 8 域脚本解析期望 RPC 清单
if ! extract_expected "$EXPECTED_RPCS"; then
  echo "  ❌ 解析 scripts/regression-test-8-domain-extension.sh 失败"
  exit 1
fi

if [ ! -s "$EXPECTED_RPCS" ]; then
  echo "  ❌ 期望 RPC 清单为空 (8 域脚本清单块缺失或格式变更)"
  exit 1
fi

# 2) 逐个 fixture 校验存在 / JSON 合法 / 覆盖对应 RPC
declare -A WANT_BY_FIXTURE=()
MODS=()
while read -r item; do
  [ -n "$item" ] || continue
  mod="${item%%/*}"
  rpc="${item#*/}"
  if [ -z "${WANT_BY_FIXTURE[$mod]:-}" ]; then
    MODS+=("$mod")
  fi
  WANT_BY_FIXTURE[$mod]="${WANT_BY_FIXTURE[$mod]:-}$rpc"$'\n'
done < "$EXPECTED_RPCS"

for mod in "${MODS[@]}"; do
  fixture="mock_data/${mod}.json"
  files_total=$((files_total + 1))
  printf '%s' "${WANT_BY_FIXTURE[$mod]}" > "$WANT_RPCS"
  want_count=$(grep -c . "$WANT_RPCS" || true)
  rpcs_total=$((rpcs_total + want_count))
  if [ ! -f "$fixture" ]; then
    echo "  ❌ fixture 缺失: $fixture (期望 $want_count RPC)"
    fail=1
    continue
  fi
  if validate_fixture "$fixture" "$WANT_RPCS"; then
    files_ok=$((files_ok + 1))
    rpcs_ok=$((rpcs_ok + want_count))
  else
    fail=1
  fi
done

echo "fixture coverage: ${files_ok}/${files_total} 文件, ${rpcs_ok}/${rpcs_total} RPC"
if [ "$fail" -ne 0 ]; then
  exit 1
fi
exit 0

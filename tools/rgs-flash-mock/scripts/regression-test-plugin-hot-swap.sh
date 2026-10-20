#!/usr/bin/env bash
# rgs-flash-mock plugin 集群 hot-swap 回归测试 (v0.1)
#
# 覆盖 RGS-TEST-CASES-2026-09-07_v0.2 §3.2 (L303-315) 的 PLUGIN-001 / 006 / 007 (config-binding 级可观测部分),
# PLUGIN-002 / 003 / 004 / 005 本地不可观测 → 以 known gap 形式打印, 不做假断言 (per 缺标比错标安全).
# 缺口来源: tools/rgs-flash-mock/README.md:107 「plugin 集群 — draw_card_probability PoC 仅 v0.1, 缺 hot-swap E2E 实战验证」
#
# 设计约束:
# - 全程只操作 mktemp 临时目录里的 .aci.json / .mock-cluster.json 副本, tracked 文件零改动 (脚本末尾自校验 sha256)
# - fixture = mock_data/plugin_hot_swap.json, 期望值从 fixture 读取 (fixture 漂移 → 断言失败)
# - 断言对象 = scripts/_lib_mock_switch_rgs.py 的真实输出 (read-plugins / is-enabled / get-mode / trace / validate-compat)
#
# 用法: bash scripts/regression-test-plugin-hot-swap.sh [--aci-config <path>] [--cluster-config <path>]
#   --aci-config      指定被测 .aci.json 源文件 (默认: tracked tools/rgs-flash-mock/.aci.json)
#   --cluster-config  指定被测 .mock-cluster.json 源文件 (默认: tracked tools/rgs-flash-mock/.mock-cluster.json)
#   源文件只读, 脚本始终先复制到 mktemp 临时目录再操作; 退出前自校验源文件 sha256 未变.
# 退出码: 0 = 全部断言通过 / 1 = 存在断言失败 / 2 = 环境或前置缺失 (fixture/python/配置缺失)

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MOCK_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LIB="$MOCK_ROOT/scripts/_lib_mock_switch_rgs.py"
FIXTURE="$MOCK_ROOT/mock_data/plugin_hot_swap.json"
TRACKED_ACI="$MOCK_ROOT/.aci.json"
TRACKED_CLUSTER="$MOCK_ROOT/.mock-cluster.json"

ACI_SRC="$TRACKED_ACI"
CLUSTER_SRC="$TRACKED_CLUSTER"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --aci-config)
      [ "$#" -ge 2 ] || { printf 'ENV-ERROR: --aci-config 需要参数\n' >&2; exit 2; }
      ACI_SRC="$2"; shift 2 ;;
    --cluster-config)
      [ "$#" -ge 2 ] || { printf 'ENV-ERROR: --cluster-config 需要参数\n' >&2; exit 2; }
      CLUSTER_SRC="$2"; shift 2 ;;
    -h|--help)
      printf 'usage: %s [--aci-config <path>] [--cluster-config <path>]\n' "$0"; exit 0 ;;
    *)
      printf 'ENV-ERROR: 未知参数: %s\n' "$1" >&2; exit 2 ;;
  esac
done

PASS_COUNT=0
FAIL_COUNT=0
GAP_COUNT=0

# ---------- 输出helpers ----------
ok()  { PASS_COUNT=$((PASS_COUNT + 1)); printf '  [OK]   %-24s %s\n' "$1" "$2"; }
bad() { FAIL_COUNT=$((FAIL_COUNT + 1)); printf '  [FAIL] %-24s %s\n' "$1" "$2"; }
gap() { GAP_COUNT=$((GAP_COUNT + 1)); printf '  [GAP]  %-24s %s\n' "$1" "$2"; }

assert_eq() { # id desc expected actual
  if [ "$3" = "$4" ]; then ok "$1" "$2 (= $3)"; else bad "$1" "$2 (expected=[$3] actual=[$4])"; fi
}
assert_ne() { # id desc not_expected actual
  if [ "$3" != "$4" ]; then ok "$1" "$2 (!= $3)"; else bad "$1" "$2 (expected!= $3, actual=$4)"; fi
}
assert_contains() { # id desc needle haystack
  case "$4" in
    *"$3"*) ok "$1" "$2 (contains '$3')" ;;
    *)      bad "$1" "$2 (missing '$3' in [$4])" ;;
  esac
}
assert_not_empty() { # id desc value
  if [ -n "$3" ]; then ok "$1" "$2 (len=${#3})"; else bad "$1" "$2 (empty)"; fi
}

die_env() { printf 'ENV-ERROR: %s\n' "$1" >&2; exit 2; }

# ---------- 前置检查 ----------
PY=""
for cand in python python3; do
  if command -v "$cand" >/dev/null 2>&1; then PY="$cand"; break; fi
done
[ -n "$PY" ] || die_env "python 未找到 (需要 python3.13+ / PATH 中 python)"

[ -f "$LIB" ]         || die_env "缺少 helper: $LIB"
[ -f "$FIXTURE" ]     || die_env "缺少 fixture: $FIXTURE"
[ -f "$ACI_SRC" ] || die_env "缺少被测配置: $ACI_SRC"
[ -f "$CLUSTER_SRC" ] || die_env "缺少被测配置: $CLUSTER_SRC"

TMP="$(mktemp -d 2>/dev/null)" || die_env "mktemp -d 失败"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

# ---------- 临时 python 工具 (不入 commit) ----------
MOCKTOOL="$TMP/mocktool.py"
cat >"$MOCKTOOL" <<'PYEOF'
#!/usr/bin/env python3
"""mock 回归测试辅助工具 (运行时生成于临时目录, 不入 commit)."""
import json
import re
import sys
from pathlib import Path


def load(p):
    return json.loads(Path(p).read_text(encoding="utf-8"))


def save(p, obj):
    Path(p).write_text(json.dumps(obj, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def dig(obj, path):
    cur = obj
    if path:
        for part in path.split("."):
            if isinstance(cur, list):
                cur = cur[int(part)]
            else:
                cur = cur[part]
    return cur


def render(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, (int, float)):
        return str(v)
    if isinstance(v, str):
        return v
    return json.dumps(v, ensure_ascii=False, sort_keys=True)


def cmd_get(argv):
    print(render(dig(load(argv[2]), argv[3])))


def cmd_len(argv):
    print(len(dig(load(argv[2]), argv[3])))


def cmd_scenario(argv):
    for s in load(argv[2])["scenarios"]:
        if s["scenario"] == argv[3]:
            print(json.dumps(s, ensure_ascii=False, sort_keys=True))
            return 0
    sys.stderr.write("scenario-not-found: %s\n" % argv[3])
    return 3


def cmd_scenarios(argv):
    for s in load(argv[2])["scenarios"]:
        print("%s\t%s\t%s" % (s["scenario"], str(s.get("executable", False)).lower(), s.get("coverage", "")))
    return 0


def cmd_set_cluster_enabled(argv):
    cfg = load(argv[2])
    cfg["enabled"] = (argv[3] == "true")
    save(argv[2], cfg)


def cmd_set_cluster_compat(argv):
    cfg = load(argv[2])
    cfg["aci_compat_version"] = argv[3]
    save(argv[2], cfg)


def cmd_disable_module(argv):
    aci = load(argv[2])
    aci["plugins"][argv[3]]["modules"][argv[4]]["enabled"] = False
    save(argv[2], aci)


def cmd_add_module(argv):
    aci = load(argv[2])
    aci["plugins"][argv[3]]["modules"][argv[4]] = {
        "module_id": argv[4], "enabled": True, "mode": argv[5]
    }
    save(argv[2], aci)


def cmd_resolve(argv):
    """解析一次调用在给定 (aci, cluster) 快照下的绑定 —— 即 read-plugins 之外的调用绑定视角."""
    aci = load(argv[2])
    cluster = load(argv[3]) if Path(argv[3]).exists() else {}
    p = aci.get("plugins", {}).get(argv[4])
    m = (p or {}).get("modules", {}).get(argv[5])
    binding = {
        "plugin_id": argv[4],
        "module_id": argv[5],
        "plugin_present": p is not None,
        "module_present": m is not None,
        "plugin_enabled": bool((p or {}).get("enabled", False)),
        "module_enabled": bool((m or {}).get("enabled", False)),
        "mode": (m or {}).get("mode", "offline"),
        "cluster_enabled": bool(cluster.get("enabled", False)),
        "cluster_mode": cluster.get("mode", "unknown"),
    }
    print(json.dumps(binding, ensure_ascii=False, sort_keys=True))


def cmd_equal(argv):
    a = load(argv[2])
    b = load(argv[3])
    print("true" if a == b else "false")


def cmd_consistency(argv):
    """.aci.json ↔ .mock-cluster.json ↔ read-plugins 三方一致性报告."""
    aci = load(argv[2])
    cluster = load(argv[3])
    rp = load(argv[4])
    trace = argv[5] if len(argv) > 5 else ""

    aci_plugins = aci.get("plugins", {})
    rp_plugins = rp.get("plugins", {})

    report = {}
    report["plugin_id_sets_equal"] = sorted(aci_plugins.keys()) == sorted(rp_plugins.keys())
    module_ok = True
    per_plugin_ok = True
    for pid, pconf in aci_plugins.items():
        aci_mods = pconf.get("modules", {})
        rp_mods = rp_plugins.get(pid, {}).get("modules", {})
        if sorted(aci_mods.keys()) != sorted(rp_mods.keys()):
            module_ok = False
        rpe = rp_plugins.get(pid, {})
        want_total = len(aci_mods)
        want_enabled = sum(1 for m in aci_mods.values() if m.get("enabled", False))
        if rpe.get("modules_total") != want_total or rpe.get("modules_enabled") != want_enabled:
            per_plugin_ok = False
    report["module_id_sets_equal"] = module_ok
    report["per_plugin_counts_match"] = per_plugin_ok
    report["aci_module_count_total"] = aci.get("module_count_total")
    report["aci_module_count_enabled"] = aci.get("module_count_enabled")
    report["cluster_module_count_total"] = cluster.get("module_count_total")
    report["cluster_module_count_enabled"] = cluster.get("module_count_enabled")
    report["read_plugins_modules_total"] = rp.get("modules_total")
    report["read_plugins_modules_enabled"] = rp.get("modules_enabled")
    report["read_plugins_plugins_total"] = rp.get("plugins_total")

    trace_counts = {}
    for pid, n in re.findall(r"([A-Za-z0-9_]+)\((\d+)m\)", trace):
        trace_counts[pid] = int(n)
    rp_counts = {pid: v.get("modules_total") for pid, v in rp_plugins.items()}
    report["trace_per_plugin_counts_match"] = (trace_counts == rp_counts and len(trace_counts) > 0)
    report["trace_per_plugin_counts"] = trace_counts
    tail = re.search(r"=(\d+)/(\d+) modules", trace)
    if tail:
        report["trace_tail_enabled"] = int(tail.group(1))
        report["trace_tail_total"] = int(tail.group(2))
        report["trace_tail_counts_match"] = (
            int(tail.group(1)) == rp.get("modules_enabled")
            and int(tail.group(2)) == rp.get("modules_total")
        )
    else:
        report["trace_tail_enabled"] = None
        report["trace_tail_total"] = None
        report["trace_tail_counts_match"] = False

    print(json.dumps(report, ensure_ascii=False, sort_keys=True))


def main(argv):
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass
    cmd = argv[1]
    table = {
        "get": cmd_get,
        "len": cmd_len,
        "scenario": cmd_scenario,
        "scenarios": cmd_scenarios,
        "set-cluster-enabled": cmd_set_cluster_enabled,
        "set-cluster-compat": cmd_set_cluster_compat,
        "disable-module": cmd_disable_module,
        "add-module": cmd_add_module,
        "resolve": cmd_resolve,
        "equal": cmd_equal,
        "consistency": cmd_consistency,
    }
    if cmd not in table:
        sys.stderr.write("unknown cmd: %s\n" % cmd)
        return 2
    return table[cmd](argv)


sys.exit(main(sys.argv))
PYEOF

mt() { "$PY" "$MOCKTOOL" "$@"; }

# ---------- mock CLI 包装 ----------
run_mock() { # run_mock <aci> <cluster> <cmd...>
  local aci="$1" cluster="$2"
  shift 2
  "$PY" "$LIB" --aci-config "$aci" --cluster-config "$cluster" "$@"
}

# fixture 取值
fx() { # fx <scenario> <dotted.path>
  local scen="$1" path="$2"
  mt scenario "$FIXTURE" "$scen" | "$PY" -c '
import json,sys
s=json.load(sys.stdin)
cur=s
for part in sys.argv[1].split("."):
    if part=="":
        continue
    cur=cur[part]
if isinstance(cur,bool):
    print("true" if cur else "false")
elif isinstance(cur,(int,float)):
    print(cur)
elif isinstance(cur,str):
    print(cur)
else:
    print(json.dumps(cur,ensure_ascii=False,sort_keys=True))
' "$path"
}

echo "=== rgs-flash-mock plugin hot-swap 回归测试 (per TEST-CASES §3.2 L303-315) ==="
echo "  mock 根: $MOCK_ROOT"
echo "  临时目录: $TMP"
echo "  fixture: mock_data/plugin_hot_swap.json"
echo

# ============================================================================
echo "[1/8] 前置 + 基线读取 (PLUGIN-007 契约快照)"
# ============================================================================
BASE_ACI="$TMP/base.aci.json"
BASE_CLUSTER="$TMP/base.mock-cluster.json"
cp "$ACI_SRC" "$BASE_ACI"
cp "$CLUSTER_SRC" "$BASE_CLUSTER"

# 源文件基线 sha256 (脚本末尾自校验: 源文件全程只读)
sha_before_aci="$(sha256sum "$ACI_SRC" | awk '{print $1}')"
sha_before_cluster="$(sha256sum "$CLUSTER_SRC" | awk '{print $1}')"

BASE_RP="$TMP/base.read-plugins.json"
out="$(run_mock "$BASE_ACI" "$BASE_CLUSTER" read-plugins 2>&1)"; rc=$?
assert_eq "P0-1" "read-plugins exit code" "0" "$rc"
if [ "$rc" -eq 0 ]; then
  printf '%s' "$out" >"$BASE_RP"
  if "$PY" -c 'import json,sys; json.load(open(sys.argv[1],encoding="utf-8"))' "$BASE_RP" 2>/dev/null; then
    ok "P0-2" "read-plugins 输出是合法 JSON"
  else
    bad "P0-2" "read-plugins 输出不是合法 JSON"
  fi
else
  bad "P0-2" "read-plugins 输出不是合法 JSON (exit=$rc)"
fi

base_plugins_total="$(mt get "$BASE_RP" plugins_total)"
base_plugins_enabled="$(mt get "$BASE_RP" plugins_enabled)"
base_modules_total="$(mt get "$BASE_RP" modules_total)"
base_modules_enabled="$(mt get "$BASE_RP" modules_enabled)"

assert_eq "P1-1" "plugins_total == fixture" "$(fx PLUGIN-001 expected.pre_switch.plugins_total)" "$base_plugins_total"
assert_eq "P1-2" "plugins_enabled == fixture" "$(fx PLUGIN-001 expected.pre_switch.plugins_enabled)" "$base_plugins_enabled"
assert_eq "P1-3" "modules_total == fixture" "$(fx PLUGIN-001 expected.pre_switch.modules_total)" "$base_modules_total"
assert_eq "P1-4" "modules_enabled == fixture" "$(fx PLUGIN-001 expected.pre_switch.modules_enabled)" "$base_modules_enabled"

out="$(run_mock "$BASE_ACI" "$BASE_CLUSTER" is-enabled 2>&1)"; rc=$?
assert_eq "P1-5" "is-enabled exit (enabled=true)" "$(fx PLUGIN-001 expected.cluster_enabled_flip.enabled_true_exit)" "$rc"
assert_eq "P1-6" "is-enabled stdout" "CLUSTER_ENABLED=true" "$out"

out="$(run_mock "$BASE_ACI" "$BASE_CLUSTER" get-mode 2>&1)"; rc=$?
assert_eq "P1-7" "get-mode exit code" "0" "$rc"
base_mode="$out"
assert_contains "P1-8" "get-mode stdout" "CLUSTER_MODE=" "$out"

out="$(run_mock "$BASE_ACI" "$BASE_CLUSTER" trace 2>&1)"; rc=$?
assert_eq "P1-9" "trace exit code" "0" "$rc"
assert_not_empty "P1-10" "trace 输出非空" "$out"
base_trace="$out"
trace_idx=0
for pid in $(mt get "$BASE_RP" plugins | "$PY" -c 'import json,sys; print(" ".join(sorted(json.load(sys.stdin).keys())))'); do
  trace_idx=$((trace_idx + 1))
  assert_contains "P1-11-$trace_idx" "trace 含 plugin [$pid]" "$pid" "$out"
done

out="$(run_mock "$BASE_ACI" "$BASE_CLUSTER" validate-compat 2>&1)"; rc=$?
assert_eq "P1-12" "validate-compat exit code" "0" "$rc"
assert_contains "P1-13" "validate-compat stdout" "ACI_COMPAT=OK" "$out"

# ============================================================================
echo
echo "[2/8] cluster.enabled 严格翻转 (PLUGIN-001 cluster_enabled_flip)"
# ============================================================================
FLIP_CLUSTER="$TMP/flip.mock-cluster.json"
cp "$BASE_CLUSTER" "$FLIP_CLUSTER"
mt set-cluster-enabled "$FLIP_CLUSTER" false

out="$(run_mock "$BASE_ACI" "$FLIP_CLUSTER" is-enabled 2>&1)"; rc_flip=$?
assert_eq "P2-1" "enabled=false 时 is-enabled exit" "$(fx PLUGIN-001 expected.cluster_enabled_flip.enabled_false_exit)" "$rc_flip"
assert_eq "P2-2" "enabled=false 时 stdout" "CLUSTER_ENABLED=false" "$out"

out="$(run_mock "$BASE_ACI" "$FLIP_CLUSTER" trace 2>&1)"; rc=$?
assert_eq "P2-3" "enabled=false 时 trace exit" "0" "$rc"
assert_contains "P2-4" "enabled=false 时 trace token" "$(fx PLUGIN-001 expected.cluster_enabled_flip.trace_enabled_false_token)" "$out"

# 恢复 → 必须回到 0 (严格翻转)
cp "$BASE_CLUSTER" "$FLIP_CLUSTER"
out="$(run_mock "$BASE_ACI" "$FLIP_CLUSTER" is-enabled 2>&1)"; rc_restore=$?
assert_eq "P2-5" "恢复后 is-enabled exit" "$(fx PLUGIN-001 expected.cluster_enabled_flip.restored_exit)" "$rc_restore"
assert_ne "P2-6" "exit code 严格翻转 0→1" "0" "$rc_flip"

# ============================================================================
echo
echo "[3/8] validate-compat fail-closed (副本改坏 compat 版本)"
# ============================================================================
BAD_CLUSTER="$TMP/bad.mock-cluster.json"
cp "$BASE_CLUSTER" "$BAD_CLUSTER"
mt set-cluster-compat "$BAD_CLUSTER" "0.0.0-mismatch"
out="$(run_mock "$BASE_ACI" "$BAD_CLUSTER" validate-compat 2>&1)"; rc=$?
assert_eq "P3-1" "compat 不一致时 exit 非 0" "1" "$rc"
assert_contains "P3-2" "compat 不一致时 stdout" "ACI_COMPAT=FAIL" "$out"

# ============================================================================
echo
echo "[4/8] hot-swap 三段: 切走 / 切回 / 回滚 (PLUGIN-001)"
# ============================================================================
TGT_PLUGIN="$(fx PLUGIN-001 expected.switch_off.target_plugin)"
TGT_MODULE="$(fx PLUGIN-001 expected.switch_off.target_module)"

# --- 段 1: 切走 (关闭目标 module) ---
W1_ACI="$TMP/w1.aci.json"
W1_RP="$TMP/w1.read-plugins.json"
cp "$BASE_ACI" "$W1_ACI"
mt disable-module "$W1_ACI" "$TGT_PLUGIN" "$TGT_MODULE"
out="$(run_mock "$W1_ACI" "$BASE_CLUSTER" read-plugins 2>&1)"; rc=$?
assert_eq "P4-1" "切走后 read-plugins exit" "0" "$rc"
printf '%s' "$out" >"$W1_RP"

w1_modules_enabled="$(mt get "$W1_RP" modules_enabled)"
delta="$((w1_modules_enabled - base_modules_enabled))"
assert_eq "P4-2" "切走后 modules_enabled delta" "$(fx PLUGIN-001 expected.switch_off.modules_enabled_delta)" "$delta"
assert_eq "P4-3" "切走后目标 module enabled" "$(fx PLUGIN-001 expected.switch_off.target_module_enabled)" "$(mt get "$W1_RP" "plugins.$TGT_PLUGIN.modules.$TGT_MODULE.enabled")"
assert_eq "P4-4" "切走后目标 plugin 仍 enabled" "$(fx PLUGIN-001 expected.switch_off.target_plugin_enabled)" "$(mt get "$W1_RP" "plugins.$TGT_PLUGIN.enabled")"

out="$(run_mock "$W1_ACI" "$BASE_CLUSTER" is-enabled 2>&1)"; rc=$?
assert_eq "P4-5" "切走后 cluster 仍 enabled (app 不重启)" "$(fx PLUGIN-001 expected.switch_off.cluster_enabled)" "$([ "$rc" -eq 0 ] && echo true || echo false)"
out="$(run_mock "$W1_ACI" "$BASE_CLUSTER" trace 2>&1)"
assert_contains "P4-6" "切走后 trace cluster token 不变" "$(fx PLUGIN-001 expected.cluster_enabled_flip.trace_enabled_true_token)" "$out"

# --- 段 2: 切回 ---
W2_ACI="$TMP/w2.aci.json"
W2_RP="$TMP/w2.read-plugins.json"
cp "$BASE_ACI" "$W2_ACI"
out="$(run_mock "$W2_ACI" "$BASE_CLUSTER" read-plugins 2>&1)"; rc=$?
assert_eq "P4-7" "切回后 read-plugins exit" "0" "$rc"
printf '%s' "$out" >"$W2_RP"
assert_eq "P4-8" "切回后与基线深度相等" "true" "$(mt equal "$W2_RP" "$BASE_RP")"
w2_modules_enabled="$(mt get "$W2_RP" modules_enabled)"
assert_eq "P4-9" "切回后 modules_enabled delta" "$(fx PLUGIN-001 expected.switch_back.modules_enabled_delta)" "$((w2_modules_enabled - base_modules_enabled))"
assert_eq "P4-10" "切回后目标 module 恢复 enabled" "$(fx PLUGIN-001 expected.switch_back.target_module_enabled)" "$(mt get "$W2_RP" "plugins.$TGT_PLUGIN.modules.$TGT_MODULE.enabled")"

# --- 段 3: 回滚 (先叠加 v2 变更, 再回滚到基线) ---
W3_ACI="$TMP/w3.aci.json"
W3_RP="$TMP/w3.read-plugins.json"
cp "$BASE_ACI" "$W3_ACI"
mt add-module "$W3_ACI" "$TGT_PLUGIN" "${TGT_MODULE}_v2" "offline"
out="$(run_mock "$W3_ACI" "$BASE_CLUSTER" read-plugins 2>&1)"; rc=$?
assert_eq "P4-11" "v2 态 read-plugins exit" "0" "$rc"
printf '%s' "$out" >"$W3_RP"
assert_eq "P4-12" "v2 态 modules_total = 基线+1" "$((base_modules_total + 1))" "$(mt get "$W3_RP" modules_total)"

ROLLBACK_ACI="$TMP/rollback.aci.json"
ROLLBACK_RP="$TMP/rollback.read-plugins.json"
cp "$BASE_ACI" "$ROLLBACK_ACI"
out="$(run_mock "$ROLLBACK_ACI" "$BASE_CLUSTER" read-plugins 2>&1)"; rc=$?
assert_eq "P4-13" "回滚后 read-plugins exit" "0" "$rc"
printf '%s' "$out" >"$ROLLBACK_RP"
assert_eq "P4-14" "回滚后与基线深度相等" "$(fx PLUGIN-001 expected.rollback.deep_equal_baseline)" "$(mt equal "$ROLLBACK_RP" "$BASE_RP")"
assert_eq "P4-15" "回滚后 modules_total delta" "$(fx PLUGIN-001 expected.rollback.modules_total_delta)" "$(( $(mt get "$ROLLBACK_RP" modules_total) - base_modules_total ))"
assert_eq "P4-16" "回滚后 modules_enabled delta" "$(fx PLUGIN-001 expected.rollback.modules_enabled_delta)" "$(( $(mt get "$ROLLBACK_RP" modules_enabled) - base_modules_enabled ))"
assert_eq "P4-17" "回滚后 v2 module 消失" "$(fx PLUGIN-001 expected.rollback.extra_module_removed)" "$([ -z "$(mt get "$ROLLBACK_RP" "plugins.$TGT_PLUGIN.modules.${TGT_MODULE}_v2" 2>/dev/null)" ] && echo true || echo false)"

# ============================================================================
echo
echo "[5/8] in-flight 语义: 在途调用走旧版本 / 新调用走新版本 (PLUGIN-001)"
# ============================================================================
# 在途调用: 切换前捕获绑定 (capture-time binding), 切换后不再重新解析
INFLIGHT_BEFORE="$(mt resolve "$BASE_ACI" "$BASE_CLUSTER" "$TGT_PLUGIN" "$TGT_MODULE")"
printf '%s' "$INFLIGHT_BEFORE" >"$TMP/inflight_before.json"
assert_eq "P5-1" "切换前 module_enabled" "$(fx PLUGIN-001 expected.in_flight.module_enabled_before)" "$(mt get "$TMP/inflight_before.json" module_enabled)"
assert_eq "P5-2" "切换前 mode" "$(fx PLUGIN-001 expected.in_flight.mode_before)" "$(mt get "$TMP/inflight_before.json" mode)"

# 切换发生 (W1 = card module 已关闭)
# 在途调用仍按切换前捕获的绑定完成 (旧版本) —— ledger 不被重新解析
assert_eq "P5-3" "in-flight 绑定保持旧版本" "$(fx PLUGIN-001 expected.in_flight.in_flight_binding_kept)" "$(mt get "$TMP/inflight_before.json" module_enabled)"

# 负控: 新调用按新配置重新解析 → 必须与 in-flight 绑定不同 (证明配置真的切了)
NEW_CALL="$(mt resolve "$W1_ACI" "$BASE_CLUSTER" "$TGT_PLUGIN" "$TGT_MODULE")"
printf '%s' "$NEW_CALL" >"$TMP/new_call.json"
assert_eq "P5-4" "in-flight 绑定 vs 新解析 不同" "true" "$([ "$(mt get "$TMP/inflight_before.json" module_enabled)" != "$(mt get "$TMP/new_call.json" module_enabled)" ] && echo true || echo false)"
assert_eq "P5-5" "新调用重新绑定 (负控)" "$(fx PLUGIN-001 expected.in_flight.new_call_rebinds)" "$([ "$NEW_CALL" != "$INFLIGHT_BEFORE" ] && echo true || echo false)"
assert_eq "P5-6" "新调用 module_enabled" "$(fx PLUGIN-001 expected.in_flight.module_enabled_after)" "$(mt get "$TMP/new_call.json" module_enabled)"

# ============================================================================
echo
echo "[6/8] native fallback: plugin Paused 时 app 仍可路由 (PLUGIN-006)"
# ============================================================================
assert_eq "P6-1" "Paused 后 module 仍在 registry" "$(fx PLUGIN-006 expected.paused.target_module_present_in_registry)" "$([ -n "$(mt get "$W1_RP" "plugins.$TGT_PLUGIN.modules.$TGT_MODULE.module_id" 2>/dev/null)" ] && echo true || echo false)"
assert_eq "P6-2" "Paused 后 module enabled=false" "$(fx PLUGIN-006 expected.paused.target_module_enabled)" "$(mt get "$W1_RP" "plugins.$TGT_PLUGIN.modules.$TGT_MODULE.enabled")"
assert_eq "P6-3" "Paused 后 plugin 仍 enabled" "$(fx PLUGIN-006 expected.paused.target_plugin_enabled)" "$(mt get "$W1_RP" "plugins.$TGT_PLUGIN.enabled")"
assert_eq "P6-4" "Paused 后其余 plugin 不受影响" "$(fx PLUGIN-006 expected.paused.plugins_enabled)" "$(mt get "$W1_RP" plugins_enabled)"
assert_eq "P6-5" "Paused 后 modules_enabled delta" "$(fx PLUGIN-006 expected.paused.modules_enabled_delta)" "$(( $(mt get "$W1_RP" modules_enabled) - base_modules_enabled ))"
out="$(run_mock "$W1_ACI" "$BASE_CLUSTER" is-enabled 2>&1)"; rc=$?
assert_eq "P6-6" "Paused 后 cluster 仍 enabled" "$(fx PLUGIN-006 expected.paused.cluster_enabled)" "$([ "$rc" -eq 0 ] && echo true || echo false)"
out="$(run_mock "$W1_ACI" "$BASE_CLUSTER" get-mode 2>&1)"
assert_eq "P6-7" "Paused 后 cluster mode 不变" "$(fx PLUGIN-006 expected.paused.cluster_mode_unchanged)" "$([ "$out" = "$base_mode" ] && echo true || echo false)"

# ============================================================================
echo
echo "[7/8] 三方一致性 (.aci.json ↔ .mock-cluster.json ↔ read-plugins) (PLUGIN-007)"
# ============================================================================
CONS="$TMP/consistency.json"
mt consistency "$BASE_ACI" "$BASE_CLUSTER" "$BASE_RP" "$base_trace" >"$CONS"
assert_eq "P7-1" "plugin id 集合一致" "$(fx PLUGIN-007 expected.plugin_id_sets_equal)" "$(mt get "$CONS" plugin_id_sets_equal)"
assert_eq "P7-2" "module id 集合一致" "$(fx PLUGIN-007 expected.module_id_sets_equal)" "$(mt get "$CONS" module_id_sets_equal)"
assert_eq "P7-3" "per-plugin 计数一致" "$(fx PLUGIN-007 expected.per_plugin_counts_match)" "$(mt get "$CONS" per_plugin_counts_match)"
assert_eq "P7-4" "aci module_count_total" "$(fx PLUGIN-007 expected.aci_module_count_total)" "$(mt get "$CONS" aci_module_count_total)"
assert_eq "P7-5" "aci module_count_enabled" "$(fx PLUGIN-007 expected.aci_module_count_enabled)" "$(mt get "$CONS" aci_module_count_enabled)"
assert_eq "P7-6" "cluster module_count_total" "$(fx PLUGIN-007 expected.cluster_module_count_total)" "$(mt get "$CONS" cluster_module_count_total)"
assert_eq "P7-7" "cluster module_count_enabled" "$(fx PLUGIN-007 expected.cluster_module_count_enabled)" "$(mt get "$CONS" cluster_module_count_enabled)"
assert_eq "P7-8" "read-plugins modules_total" "$(fx PLUGIN-007 expected.modules_total)" "$(mt get "$CONS" read_plugins_modules_total)"
assert_eq "P7-9" "read-plugins modules_enabled" "$(fx PLUGIN-007 expected.modules_enabled)" "$(mt get "$CONS" read_plugins_modules_enabled)"
assert_eq "P7-10" "read-plugins plugins_total" "$(fx PLUGIN-007 expected.plugins_total)" "$(mt get "$CONS" read_plugins_plugins_total)"
assert_eq "P7-11" "trace per-plugin 计数一致" "$(fx PLUGIN-007 expected.trace_per_plugin_counts_match)" "$(mt get "$CONS" trace_per_plugin_counts_match)"
assert_eq "P7-12" "trace 尾部计数一致" "$(fx PLUGIN-007 expected.trace_tail_counts_match)" "$(mt get "$CONS" trace_tail_counts_match)"

# ============================================================================
echo
echo "[8/8] tracked 文件零改动自校验 + known gap 登记"
# ============================================================================
sha_after_aci="$(sha256sum "$ACI_SRC" | awk '{print $1}')"
sha_after_cluster="$(sha256sum "$CLUSTER_SRC" | awk '{print $1}')"
assert_eq "P8-1" "源 .aci.json sha256 未变" "$sha_before_aci" "$sha_after_aci"
assert_eq "P8-2" "源 .mock-cluster.json sha256 未变" "$sha_before_cluster" "$sha_after_cluster"

# known gap: fixture 中 executable=false 的用例
while IFS=$'\t' read -r sid exec_flag cov; do
  [ -n "$sid" ] || continue
  cov="${cov%$'\n'}"
  [ "$exec_flag" = "false" ] || continue
  reason="$(fx "$sid" known_gap.reason)"
  gap "$sid" "known gap ($cov): $reason"
done < <(mt scenarios "$FIXTURE")

# ---------------------------------------------------------------------------
echo
echo "=== 汇总 ==="
echo "  PASS: $PASS_COUNT  FAIL: $FAIL_COUNT  KNOWN-GAP: $GAP_COUNT"
echo "  临时目录已清理: $TMP"
if [ "$FAIL_COUNT" -gt 0 ]; then
  echo "  结果: ❌ 有断言失败"
  exit 1
fi
echo "  结果: ✅ 全部断言通过 (known gap 未计入失败)"
exit 0

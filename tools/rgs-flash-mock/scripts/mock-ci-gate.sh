#!/usr/bin/env bash
# mock-ci-gate.sh — rgs-flash-mock 本地 CI 口径 gate (per flash-mock-ci.yml)
#
# 用途:
#   仓库里只有 .github/workflows/flash-mock-ci.yml（CI 侧），本地没有可复跑的等价入口。
#   本脚本把该 workflow 的 7 个 step 原样搬下来，让本地一次命令跑完 mock 项目的
#   全套 mock 口径回归，产出可直接进 regression 报告的 PASS/FAIL 明细。
#
# 与 flash-mock-ci.yml 的 step 对照 (1:1，不多不少):
#   1. cargo check --tests --locked          (编译门禁)
#   2. cargo test --locked                   (38 lib + 3 aci IT + 7 saga IT + 1 doc-test = 49)
#   3. python unittest discover              (mock_switch 18 tests)
#   4. mock_switch CLI exit-code 规范校验     (5 子命令)
#   5. bash -n 语法检查                       (scripts/*.sh)
#   6. mock 缺口闭环回归                      (3 脚本 + 8 域扩展)
#   7. 跨域故事链 + 跨域事件链回归            (RGS_MOCK_SKIP_CARGO_CHECK=1)
#
# 派生约束:
#   - 凭据永不打印 (8/27 11:06 JST hard ban): 本脚本不 echo 任何 env 值
#   - fail-closed: 任一 step 失败 -> 整体退出码非 0，不吞错
#   - 测试脚本归入 mock 项目 (9/4 17:47 JST)
#   - CARGO_TARGET_DIR 默认走 E 盘 devcache (per AGENTS.md L11/L30)，可用环境变量覆盖
#
# 用法:
#   bash scripts/mock-ci-gate.sh
#   MOCK_GATE_SKIP_BUILD=1 bash scripts/mock-ci-gate.sh   # 跳过两个 cargo step（暖缓存复跑用）
#
# 输出:
#   - 逐 step 明细 + 失败证据尾部
#   - 末尾汇总表 + 退出码 (0 = 全过)

set -uo pipefail

# 切到 mock 项目根
cd "$(dirname "$0")/.." || exit 2

# Python UTF-8 模式 (per 2026-10-21 实测订正): 本机 locale=cp936(GBK),
# mock_data/*.json 是 UTF-8 中文 -> 不开 UTF-8 模式会被判成 "JSON invalid"
export PYTHONUTF8=1

# cargo 不一定在 Git Bash 的 PATH 里，显式挂上 (Windows 常见布局)
if ! command -v cargo >/dev/null 2>&1; then
  for cand in "/e/DevCache/cargo/bin" "$HOME/.cargo/bin"; do
    if [ -d "$cand" ]; then
      export PATH="$cand:$PATH"
      break
    fi
  done
fi

# CARGO_TARGET_DIR 默认 E 盘 devcache (D 盘不写, per L11/L30)
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-E:/DevCache/cargo/rgs-flash-mock}"

# 并发限流 (per 2026-10-21 现场实测):
#   整机 RAM 31.8 GB, 但常态 free 仅 0.3~1.8 GB (commit 80.5/85.5 GB)。
#   cargo 默认 -j = CPU 核数 (本机 24+ 路) 会把 rustc 挤爆,
#   表现为 "can't find crate for std" 这类**假失败**——不是代码问题。
#   限流到 4 路, 可用 MOCK_GATE_JOBS 覆盖。
CARGO_JOBS="${MOCK_GATE_JOBS:-4}"

LOG_DIR="logs/mock-ci-gate"
mkdir -p "$LOG_DIR"
TS=$(date +%Y%m%d-%H%M%S)
GATE_LOG="$LOG_DIR/gate-$TS.log"

# 机读结果归档到 mock_data/<batch>/ (per 9/4 17:47 JST 测试数据归入 mock 项目;
# 与 aggregate-regression.ps1 的 mock_data/<batch>/<lane>.json 约定一致)
BATCH="${MOCK_GATE_BATCH:-mock-ci-gate-$(date +%Y-%m-%d)}"
MOCK_DATA_DIR="mock_data/$BATCH"
mkdir -p "$MOCK_DATA_DIR"

STEP_NAMES=()
STEP_VERDICTS=()
STEP_EVIDENCE=()
FAILED=0

record() {
  # record <name> <verdict> <evidence>
  STEP_NAMES+=("$1")
  STEP_VERDICTS+=("$2")
  STEP_EVIDENCE+=("$3")
  if [ "$2" != "PASS" ]; then
    FAILED=$((FAILED + 1))
  fi
}

log() {
  echo "$@" | tee -a "$GATE_LOG"
}

log "=== rgs-flash-mock 本地 CI 口径 gate ==="
log "  项目根    : $(pwd)"
log "  target dir: $CARGO_TARGET_DIR"
log "  jobs      : $CARGO_JOBS (限流, 防 RAM 挤爆 rustc 假失败)"
log "  日志      : $GATE_LOG"
log "  cargo     : $(command -v cargo || echo '(未找到)')"
log ""

# ── 1. cargo check --tests --locked (编译门禁) ──
if [ "${MOCK_GATE_SKIP_BUILD:-0}" = "1" ]; then
  log "[1/7] cargo check --tests  -> SKIP (MOCK_GATE_SKIP_BUILD=1)"
  record "cargo check --tests" "SKIP" "MOCK_GATE_SKIP_BUILD=1"
else
  log "[1/7] cargo check --tests --locked ..."
  L1_LOG="$LOG_DIR/step1-cargo-check-$TS.log"
  if timeout 1800 cargo check --tests --locked -j "$CARGO_JOBS" >"$L1_LOG" 2>&1; then
    log "  ✅ cargo check 0 error"
    record "cargo check --tests" "PASS" "0 error"
  else
    L1_RC=$?
    log "  ❌ cargo check 失败 (rc=$L1_RC)"
    tail -20 "$L1_LOG" | sed 's/^/    /' | tee -a "$GATE_LOG"
    record "cargo check --tests" "FAIL" "rc=$L1_RC, 见 $L1_LOG"
  fi
  log ""
fi

# ── 2. cargo test --locked (38 lib + 3 aci IT + 7 saga IT + 1 doc-test = 49) ──
if [ "${MOCK_GATE_SKIP_BUILD:-0}" = "1" ]; then
  log "[2/7] cargo test --locked -> SKIP (MOCK_GATE_SKIP_BUILD=1)"
  record "cargo test" "SKIP" "MOCK_GATE_SKIP_BUILD=1"
else
  log "[2/7] cargo test --locked ..."
  L11_LOG="$LOG_DIR/step2-cargo-test-$TS.log"
  if timeout 1800 cargo test --locked --no-fail-fast -j "$CARGO_JOBS" >"$L11_LOG" 2>&1; then
    SUMMARY=$(grep -E "^test result:" "$L11_LOG" | tail -1)
    log "  ✅ cargo test 全过"
    [ -n "$SUMMARY" ] && log "     $SUMMARY"
    record "cargo test" "PASS" "${SUMMARY:-all passed}"
  else
    L11_RC=$?
    FAILED_LIST=$(grep -E "^test .* FAILED" "$L11_LOG" | head -20)
    log "  ❌ cargo test 失败 (rc=$L11_RC)"
    grep -E "^test result:" "$L11_LOG" | tail -3 | sed 's/^/    /' | tee -a "$GATE_LOG"
    [ -n "$FAILED_LIST" ] && echo "$FAILED_LIST" | sed 's/^/    /' | tee -a "$GATE_LOG"
    record "cargo test" "FAIL" "rc=$L11_RC, 见 $L11_LOG"
  fi
  log ""
fi

# ── 3. Python unittest (mock_switch 18 tests) ──
log "[3/7] python -m unittest discover -s tests -p 'test_*.py' ..."
PY_LOG="$LOG_DIR/step3-python-unittest-$TS.log"
if timeout 600 python -m unittest discover -s tests -p "test_*.py" -v >"$PY_LOG" 2>&1; then
  PY_SUMMARY=$(grep -E "^Ran [0-9]+ test" "$PY_LOG" | tail -1)
  log "  ✅ python unittest 通过"
  [ -n "$PY_SUMMARY" ] && log "     $PY_SUMMARY"
  record "python unittest" "PASS" "${PY_SUMMARY:-ok}"
else
  PY_RC=$?
  log "  ❌ python unittest 失败 (rc=$PY_RC)"
  tail -25 "$PY_LOG" | sed 's/^/    /' | tee -a "$GATE_LOG"
  record "python unittest" "FAIL" "rc=$PY_RC, 见 $PY_LOG"
fi
log ""

# ── 4. mock_switch CLI exit-code 规范校验 ──
log "[4/7] mock_switch CLI exit-code 规范校验 (5 子命令) ..."
CLI_FAIL=0
CLI_MSG=""
for cmd in get-mode trace validate-compat read-plugins; do
  if ! python scripts/_lib_mock_switch_rgs.py --aci-config .aci.json "$cmd" >/dev/null 2>&1; then
    CLI_MSG="$CLI_MSG $cmd: 应当 exit 0，实际非 0;"
    CLI_FAIL=1
  fi
done
# is-enabled: exit 0 = enabled=true / exit 1 = enabled=false，两者都合法
python scripts/_lib_mock_switch_rgs.py --aci-config .aci.json is-enabled >/dev/null 2>&1
CLI_RC=$?
if [ "$CLI_RC" -ne 0 ] && [ "$CLI_RC" -ne 1 ]; then
  CLI_MSG="$CLI_MSG is-enabled: 应当 exit 0 或 1，实际 $CLI_RC;"
  CLI_FAIL=1
fi
if [ "$CLI_FAIL" -eq 0 ]; then
  log "  ✅ 5 子命令 exit-code 全部合规"
  record "mock_switch CLI exit-code" "PASS" "5/5 合规"
else
  log "  ❌ CLI exit-code 违规:$CLI_MSG"
  record "mock_switch CLI exit-code" "FAIL" "$CLI_MSG"
fi
log ""

# ── 5. bash -n 语法检查 (全部 mock 脚本) ──
log "[5/7] bash -n 语法检查 (scripts/*.sh) ..."
SYN_FAIL=0
SYN_MSG=""
for f in scripts/*.sh; do
  if ! bash -n "$f" 2>/dev/null; then
    SYN_MSG="$SYN_MSG $f;"
    SYN_FAIL=1
  fi
done
if [ "$SYN_FAIL" -eq 0 ]; then
  log "  ✅ 全部脚本语法通过"
  record "bash -n 语法检查" "PASS" "$(ls scripts/*.sh | wc -l) 个脚本"
else
  log "  ❌ 语法检查失败:$SYN_MSG"
  record "bash -n 语法检查" "FAIL" "$SYN_MSG"
fi
log ""

# ── 6. mock 缺口闭环回归 (3 脚本 + 8 域扩展) ──
log "[6/7] mock 缺口闭环回归 (fixture 覆盖 / plugin hot-swap / app+ops 用例 / 8 域扩展) ..."
GAP_FAIL=0
GAP_MSG=""
for s in scripts/regression-test-new-domain-fixture-coverage.sh \
         scripts/regression-test-plugin-hot-swap.sh \
         scripts/regression-test-app-deploy-ops-ui.sh; do
  GAP_LOG="$LOG_DIR/step6-$(basename "$s" .sh)-$TS.log"
  if timeout 900 bash "$s" >"$GAP_LOG" 2>&1; then
    log "  ✅ $(basename "$s") PASS"
  else
    GAP_RC=$?
    log "  ❌ $(basename "$s") FAIL (rc=$GAP_RC)"
    tail -12 "$GAP_LOG" | sed 's/^/    /' | tee -a "$GATE_LOG"
    GAP_MSG="$GAP_MSG $(basename "$s")(rc=$GAP_RC);"
    GAP_FAIL=1
  fi
done
# 8 域扩展: cargo check 已由 step 1 跑过, 显式跳过避免重复编译
EXT_LOG="$LOG_DIR/step6-regression-test-8-domain-extension-$TS.log"
if RGS_MOCK_SKIP_CARGO_CHECK=1 timeout 900 bash scripts/regression-test-8-domain-extension.sh >"$EXT_LOG" 2>&1; then
  log "  ✅ regression-test-8-domain-extension.sh PASS"
else
  EXT_RC=$?
  log "  ❌ regression-test-8-domain-extension.sh FAIL (rc=$EXT_RC)"
  tail -12 "$EXT_LOG" | sed 's/^/    /' | tee -a "$GATE_LOG"
  GAP_MSG="$GAP_MSG regression-test-8-domain-extension.sh(rc=$EXT_RC);"
  GAP_FAIL=1
fi
if [ "$GAP_FAIL" -eq 0 ]; then
  record "mock 缺口闭环回归" "PASS" "4/4 脚本通过"
else
  record "mock 缺口闭环回归" "FAIL" "$GAP_MSG"
fi
log ""

# ── 7. 跨域故事链 + 跨域事件链回归 ──
log "[7/7] 跨域故事链 + 跨域事件链回归 (5 域链 / 悬空事件 / 顺序错乱 / 域归属) ..."
SAGA_LOG="$LOG_DIR/step7-cross-domain-event-$TS.log"
if RGS_MOCK_SKIP_CARGO_CHECK=1 timeout 900 bash scripts/regression-test-cross-domain-event.sh >"$SAGA_LOG" 2>&1; then
  log "  ✅ regression-test-cross-domain-event.sh PASS"
  record "跨域故事链 + 事件链" "PASS" "通过"
else
  SAGA_RC=$?
  log "  ❌ regression-test-cross-domain-event.sh FAIL (rc=$SAGA_RC)"
  tail -15 "$SAGA_LOG" | sed 's/^/    /' | tee -a "$GATE_LOG"
  record "跨域故事链 + 事件链" "FAIL" "rc=$SAGA_RC, 见 $SAGA_LOG"
fi
log ""

# ── 汇总 ──
log "=== gate 汇总 ==="
log ""
log "| step | 判定 | 证据 |"
log "|---|---|---|"
for i in "${!STEP_NAMES[@]}"; do
  log "| ${STEP_NAMES[$i]} | ${STEP_VERDICTS[$i]} | ${STEP_EVIDENCE[$i]} |"
done
log ""
if [ "$FAILED" -gt 0 ]; then
  log "  ❌ gate 失败 ($FAILED 个 step 未通过)"
else
  log "  ✅ gate 全过 (${#STEP_NAMES[@]} 个 step)"
fi
log "  日志: $GATE_LOG"

# ---- 机读结果 (mock_data/<batch>/gate.json) ----
# 与 aggregate-regression.ps1 的 mock_data/<batch>/<lane>.json 约定一致:
# 归因后的结果入 mock_data/, 原始 log 留在 logs/ (gitignore, 不入库)。
STEPS_TSV="$LOG_DIR/steps-$TS.tsv"
: > "$STEPS_TSV"
for i in "${!STEP_NAMES[@]}"; do
  printf '%s\t%s\t%s\n' "${STEP_NAMES[$i]}" "${STEP_VERDICTS[$i]}" "${STEP_EVIDENCE[$i]}" >> "$STEPS_TSV"
done
JSON_OUT="$MOCK_DATA_DIR/gate.json"
python - "$STEPS_TSV" "$JSON_OUT" "$BATCH" "$GATE_LOG" <<'PY'
import datetime
import json
import sys

tsv, out, batch, gate_log = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]

steps = []
with open(tsv, encoding='utf-8') as f:
    for line in f:
        line = line.rstrip('\n')
        if not line:
            continue
        parts = line.split('\t')
        if len(parts) != 3:
            continue
        steps.append({'step': parts[0], 'verdict': parts[1], 'evidence': parts[2]})

passed = sum(1 for s in steps if s['verdict'] == 'PASS')
skipped = sum(1 for s in steps if s['verdict'] == 'SKIP')
failed = sum(1 for s in steps if s['verdict'] == 'FAIL')

obj = {
    'batch': batch,
    'generated_at': datetime.datetime.now().astimezone().isoformat(),
    'generator': 'tools/rgs-flash-mock/scripts/mock-ci-gate.sh',
    'ci_equivalent': '.github/workflows/flash-mock-ci.yml',
    'total_steps': len(steps),
    'passed': passed,
    'skipped': skipped,
    'failed': failed,
    'verdict': 'PASS' if failed == 0 else 'FAIL',
    'steps': steps,
    'raw_log': 'tools/rgs-flash-mock/' + gate_log,
}

with open(out, 'w', encoding='utf-8') as f:
    json.dump(obj, f, ensure_ascii=False, indent=2)
    f.write('\n')

print('  机读结果: %s' % out)
PY
log ""

if [ "$FAILED" -gt 0 ]; then
  exit 1
fi
exit 0

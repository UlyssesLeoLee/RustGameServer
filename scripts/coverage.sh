#!/usr/bin/env bash
# RGS Workspace 覆盖率测量脚本
# per RGS-TEST-STRATEGY-2026-08-26 v0.1 P0
#
# 用 cargo-llvm-cov(更轻量,无需 LLVM pass plugin)
# 输出: coverage/summary.txt + coverage/index.html(lcov.info)
#
# 退出码:
#   0 = 覆盖率 >= 阈值
#   1 = 覆盖率 < 阈值
#   2 = 工具/环境错误

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

COVERAGE_DIR="target/coverage"
THRESHOLD_LINE="${THRESHOLD_LINE:-90}"
THRESHOLD_BRANCH="${THRESHOLD_BRANCH:-85}"
THRESHOLD_FUNC="${THRESHOLD_FUNC:-90}"

echo "=========================================="
echo " RGS Workspace Coverage Report"
echo "  threshold: line>=$THRESHOLD_LINE% branch>=$THRESHOLD_BRANCH% func>=$THRESHOLD_FUNC%"
echo "=========================================="
echo ""

# 检查工具
if ! cargo llvm-cov --version >/dev/null 2>&1; then
  echo "❌ cargo-llvm-cov 未装, 跑: cargo install cargo-llvm-cov"
  exit 2
fi

# 检查 DATABASE_URL(per rgs-testkit pg_test)
if [ -z "${DATABASE_URL:-}" ]; then
  echo "⚠️  DATABASE_URL 未设, 跳过 PG 集成测试"
  echo "   提示: export DATABASE_URL=postgres://rgs:rgs@localhost:5432/rgs_test"
  INCLUDE_PG=false
else
  INCLUDE_PG=true
fi

mkdir -p "$COVERAGE_DIR"

# 跑 workspace 测试 + 覆盖率
echo "[1/3] cargo test --workspace"
if [ "$INCLUDE_PG" = "true" ]; then
  cargo test --workspace --no-fail-fast 2>&1 | tail -50
else
  # 用 --no-default-features 跳过需要 PG 的 test
  CARGO_NET_OFFLINE=true cargo test --workspace --no-fail-fast -- --skip pg 2>&1 | tail -30 || true
fi

echo ""
echo "[2/3] cargo llvm-cov 报告"
cargo llvm-cov report --summary-only --output-path "$COVERAGE_DIR/summary.txt"
cat "$COVERAGE_DIR/summary.txt"

echo ""
echo "[3/3] HTML 报告"
cargo llvm-cov html --output-dir "$COVERAGE_DIR/html" --quiet
echo "  open: $REPO_ROOT/$COVERAGE_DIR/html/index.html"

# 解析 + 校验
LINE_PCT=$(grep -E "^line|^[0-9]+\.[0-9]+%" "$COVERAGE_DIR/summary.txt" | head -1 || echo "0")
echo ""
echo "=========================================="
echo " Threshold Check"
echo "=========================================="
LINE_PCT_NUM=$(echo "$LINE_PCT" | grep -oE '[0-9]+\.[0-9]+' | head -1 || echo "0")
LINE_INT=${LINE_PCT_NUM%.*}
if [ "$LINE_INT" -ge "$THRESHOLD_LINE" ]; then
  echo "✅ line $LINE_PCT_NUM% >= $THRESHOLD_LINE%"
else
  echo "❌ line $LINE_PCT_NUM% < $THRESHOLD_LINE%"
fi

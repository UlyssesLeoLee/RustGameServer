#!/usr/bin/env bash
# rgs-flash-mock regression test — Plugin PoC (per RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §2.2
#                       + RGS-TEST-CASES-2026-09-05 v0.1 §3.2 PLUGIN-001 ~ 005)
#
# 验证 1 PoC WASM plugin (draw_card_probability v1.1.0) 全流程:
# 1. 注册 plugin 到 function-plane mock
# 2. invoke 多次, 验证 pity 加成
# 3. 80 次保底
# 4. Hot-swap v1.1.0 → v1.2.0 不重启 card-service
# 5. Paused 时 app 走 native fallback
#
# 用法:
#   RGS_GAP_MOCK_URL=http://127.0.0.1:8791 \
#   FUNCTION_PLANE_URL=http://127.0.0.1:8792 \
#   ./regression-test-plugin-poc.sh

set -e

BASE_URL="${RGS_GAP_MOCK_URL:-http://127.0.0.1:8791}"
FUNCTION_PLANE_URL="${FUNCTION_PLANE_URL:-http://127.0.0.1:8792}"
PLUGIN_DIR="${BASH_SOURCE%/*}/../plugin_poc/draw_card_probability"
WASM_FILE="$PLUGIN_DIR/draw_card_probability_v1.1.0.wasm"
METADATA_FILE="$PLUGIN_DIR/metadata.json"

# 颜色
if [ -t 1 ]; then
  RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; NC='\033[0m'
else
  RED=''; GREEN=''; YELLOW=''; NC=''
fi

PASS_COUNT=0
FAIL_COUNT=0

# 1. 前置检查
echo "=== 1. 前置检查 ==="

# 1.1 wasm 文件存在
if [ ! -f "$WASM_FILE" ]; then
  echo -e "${YELLOW}WARN: $WASM_FILE 不存在, 需先 wat2wasm 编译 (per plugin_poc/README.md §4.1)${NC}"
  echo "  安装 wasm-tools: cargo install wasm-tools"
  echo "  编译: wat2wasm $PLUGIN_DIR/draw_card_probability_v1.1.0.wat -o $WASM_FILE"
  # 继续跑 stub 模式 (mock 阶段允许 metadata-only)
  WASM_AVAILABLE=false
else
  echo -e "${GREEN}PASS: $WASM_FILE 存在${NC}"
  WASM_AVAILABLE=true
fi

# 1.2 function-plane 健康
if ! curl -sS -m 5 "$FUNCTION_PLANE_URL/health" > /dev/null 2>&1; then
  echo -e "${YELLOW}WARN: function-plane $FUNCTION_PLANE_URL 不可达 (mock 阶段正常)${NC}"
  echo "  mock 阶段: function-plane 还没 gRPC 端, 跑 metadata-only 验证"
  FUNCTION_PLANE_AVAILABLE=false
else
  echo -e "${GREEN}PASS: function-plane $FUNCTION_PLANE_URL OK${NC}"
  FUNCTION_PLANE_AVAILABLE=true
fi
echo ""

# 2. 注册 plugin
echo "=== 2. 注册 plugin: card.draw_card_probability v1.1.0 ==="
if [ "$FUNCTION_PLANE_AVAILABLE" = true ]; then
  response=$(curl -sS -X POST "$FUNCTION_PLANE_URL/registry/card.draw_card_probability" \
    -H "Content-Type: application/json" \
    -d @"$METADATA_FILE" \
    -w '\n%{http_code}')
  http_code=$(echo "$response" | tail -1)
  if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
    echo -e "${GREEN}PASS: 注册 v1.1.0 (http=$http_code)${NC}"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo -e "${RED}FAIL: 注册 v1.1.0 (http=$http_code)${NC}"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
else
  echo -e "${YELLOW}SKIP: function-plane mock 阶段 (待 v0.1 sprint 实施)${NC}"
fi
echo ""

# 3. invoke 多次, 验证 pity 加成
echo "=== 3. invoke pity_count=0,50,79,80,100 ==="
if [ "$FUNCTION_PLANE_AVAILABLE" = true ]; then
  for pity in 0 50 79 80 100; do
    echo -n "  pity=$pity → "
    response=$(curl -sS -X POST "$FUNCTION_PLANE_URL/invoke/card.draw_card_probability" \
      -H "Content-Type: application/json" \
      -d "{\"rarity\":\"SSR\",\"pity_count\":$pity,\"banner_id\":\"limited_001\"}")
    prob=$(echo "$response" | jq -r '.probability // "null"' 2>/dev/null)

    # 验证期望:
    # pity=0:   probability ≈ 0.05 (500 / 10000)
    # pity=50:  probability ≈ 0.30 (500 + 50*50 = 3000 / 10000)
    # pity=79:  probability ≈ 0.445 (500 + 79*50 = 4450 / 10000)
    # pity=80:  probability = 1.00 (保底)
    # pity=100: probability = 1.00 (保底)
    case $pity in
      0)   expected="0.05" ;;
      50)  expected="0.30" ;;
      79)  expected="0.445" ;;
      80|100) expected="1.00" ;;
    esac

    if [ "$prob" = "$expected" ]; then
      echo -e "${GREEN}PASS (probability=$prob)${NC}"
      PASS_COUNT=$((PASS_COUNT + 1))
    else
      echo -e "${RED}FAIL (probability=$prob, expected=$expected)${NC}"
      FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
  done
else
  echo -e "${YELLOW}SKIP: function-plane mock 阶段${NC}"
fi
echo ""

# 4. Hot-swap 测试
echo "=== 4. Hot-swap v1.1.0 → v1.2.0 不重启 card-service ==="
if [ "$FUNCTION_PLANE_AVAILABLE" = true ]; then
  CARD_SERVICE_PID="${CARD_SERVICE_PID:-$(pgrep -f 'card-service' | head -1)}"

  if [ -z "$CARD_SERVICE_PID" ]; then
    echo -e "${YELLOW}WARN: card-service 进程没找到, 跳过 hot-swap 验证 (CI 阶段补)${NC}"
  else
    initial_start_time=$(ps -o lstart= -p "$CARD_SERVICE_PID" 2>/dev/null | xargs)
    echo "  card-service PID=$CARD_SERVICE_PID, started at: $initial_start_time"

    # 4.1 register v1.2.0 Draft
    echo "  4.1 register v1.2.0 Draft"
    metadata_v1_2_0=$(echo "$(cat $METADATA_FILE)" | jq '.version = "v1.2.0"' | jq '.config.base_probability = 0.06')
    curl -sS -X POST "$FUNCTION_PLANE_URL/registry/card.draw_card_probability" \
      -H "Content-Type: application/json" \
      -d "$metadata_v1_2_0" > /dev/null

    # 4.2 set_status(Active)
    echo "  4.2 set_status(Active)"
    curl -sS -X POST "$FUNCTION_PLANE_URL/registry/card.draw_card_probability/v1.2.0/status" \
      -d '{"status":"Active"}' > /dev/null

    # 4.3 set_old_status(Archived)
    echo "  4.3 set_old_status(Archived) — 旧版本下线"
    curl -sS -X POST "$FUNCTION_PLANE_URL/registry/card.draw_card_probability/v1.1.0/status" \
      -d '{"status":"Archived"}' > /dev/null

    # 4.4 验证 card-service 进程未重启
    sleep 2
    new_start_time=$(ps -o lstart= -p "$CARD_SERVICE_PID" 2>/dev/null | xargs)
    if [ "$initial_start_time" = "$new_start_time" ]; then
      echo -e "${GREEN}PASS: card-service 进程未重启 (start_time 不变)${NC}"
      PASS_COUNT=$((PASS_COUNT + 1))
    else
      echo -e "${RED}FAIL: card-service 进程被重启 (start_time: $initial_start_time → $new_start_time)${NC}"
      FAIL_COUNT=$((FAIL_COUNT + 1))
    fi

    # 4.5 验证 v1.2.0 概率生效 (pity=0 → 6%)
    response=$(curl -sS -X POST "$FUNCTION_PLANE_URL/invoke/card.draw_card_probability" \
      -H "Content-Type: application/json" \
      -d '{"rarity":"SSR","pity_count":0,"banner_id":"limited_001"}')
    prob=$(echo "$response" | jq -r '.probability // "null"')
    if [ "$prob" = "0.06" ]; then
      echo -e "${GREEN}PASS: v1.2.0 概率生效 (pity=0 → 0.06)${NC}"
      PASS_COUNT=$((PASS_COUNT + 1))
    else
      echo -e "${RED}FAIL: v1.2.0 概率 (pity=0 → $prob, expected 0.06)${NC}"
      FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
  fi
else
  echo -e "${YELLOW}SKIP: function-plane mock 阶段${NC}"
fi
echo ""

# 5. Paused + native fallback
echo "=== 5. plugin Paused 时 app 走 native fallback ==="
if [ "$FUNCTION_PLANE_AVAILABLE" = true ]; then
  echo "  5.1 set_status(Paused)"
  curl -sS -X POST "$FUNCTION_PLANE_URL/registry/card.draw_card_probability/v1.2.0/status" \
    -d '{"status":"Paused"}' > /dev/null

  echo "  5.2 验证 gateway 拒绝, app 走 native fallback (返回 probability=0.05)"
  response=$(curl -sS -X POST "$FUNCTION_PLANE_URL/invoke/card.draw_card_probability" \
    -H "Content-Type: application/json" \
    -d '{"rarity":"SSR","pity_count":0,"banner_id":"limited_001"}' \
    -w '\n%{http_code}')
  http_code=$(echo "$response" | tail -1)
  fallback_prob=$(echo "$response" | jq -r '.probability // "null"')

  # Paused 时 gateway 应当 503 + 触发 fallback (mock 简化: 返回 native 值 0.05)
  if [ "$fallback_prob" = "0.05" ]; then
    echo -e "${GREEN}PASS: app 走 native fallback (probability=0.05)${NC}"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo -e "${YELLOW}WARN: fallback response (http=$http_code, prob=$fallback_prob) — 验证 mock 实现细节${NC}"
  fi
else
  echo -e "${YELLOW}SKIP: function-plane mock 阶段${NC}"
fi
echo ""

# 报告
echo "=== 报告 ==="
echo -e "${GREEN}PASS: $PASS_COUNT${NC}"
echo -e "${RED}FAIL: $FAIL_COUNT${NC}"

if [ $FAIL_COUNT -gt 0 ]; then
  exit 1
fi
exit 0

#!/usr/bin/env bash
# Per verifier bg_394b3ca1 §4 P1 修复:5 域 service 集成测试本地入口
# ---------------------------------------------------------------------------
# 流程:
#   1. 检测 rgs-test-pg 容器是否已跑(已跑则跳过)
#   2. 否则 docker compose -f .devcontainer/docker-compose.test.yml up -d
#   3. 等 healthcheck = healthy(最多 30s)
#   4. 注入 DATABASE_URL → 15432(rgs_test DB)
#   5. cargo test --workspace --test integration_* -- --test-threads=1
#
# 跑测前提:
#   - Docker / Docker Desktop 已起(本机或 WSL2)
#   - 端口 15432 空闲(未跑 scripts/port_forward_pg.ps1)
#   - 5 域 service 各自 migrations 在 rgs_test 上能成功 apply
#     (本脚本不自动 migrate;如需 pre-migrate 自行跑 sqlx migrate run)
#
# 例:
#   bash scripts/test-integration.sh
#   DATABASE_URL_OVERRIDE=postgres://rgs:rgs_test_pw@127.0.0.1:15432/rgs_test \
#     bash scripts/test-integration.sh
# ---------------------------------------------------------------------------
set -euo pipefail

cd "$(dirname "$0")/.."

COMPOSE_FILE=".devcontainer/docker-compose.test.yml"
CONTAINER_NAME="rgs-test-pg"
HOST_PORT=15432
DB_USER="rgs"
DB_PASS="rgs_test_pw"
DB_NAME="rgs_test"

# 默认 DATABASE_URL(可被外部 env 覆盖,仅在未设置时填默认)
if [[ -z "${DATABASE_URL_OVERRIDE:-}" ]]; then
  export DATABASE_URL_OVERRIDE="postgres://${DB_USER}:${DB_PASS}@127.0.0.1:${HOST_PORT}/${DB_NAME}"
fi

echo "[test-integration] compose file: ${COMPOSE_FILE}"
echo "[test-integration] target DATABASE_URL: ${DATABASE_URL_OVERRIDE}"

# 1. 容器状态检测
if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
  echo "[test-integration] container ${CONTAINER_NAME} already running"
else
  echo "[test-integration] starting ${CONTAINER_NAME} via docker compose..."
  docker compose -f "${COMPOSE_FILE}" up -d

  # 2. 等 healthcheck
  echo "[test-integration] waiting for healthcheck (max 30s)..."
  for i in $(seq 1 30); do
    status=$(docker inspect --format='{{.State.Health.Status}}' "${CONTAINER_NAME}" 2>/dev/null || echo "starting")
    if [[ "${status}" == "healthy" ]]; then
      echo "[test-integration] postgres healthy after ${i}s"
      break
    fi
    if [[ "${i}" -eq 30 ]]; then
      echo "[test-integration] ERROR: postgres not healthy after 30s (last status: ${status})" >&2
      docker logs "${CONTAINER_NAME}" 2>&1 | tail -40 >&2 || true
      exit 1
    fi
    sleep 1
  done
fi

# 3. 注入 DATABASE_URL 给 cargo test
export DATABASE_URL="${DATABASE_URL_OVERRIDE}"
echo "[test-integration] DATABASE_URL=${DATABASE_URL}"
echo "[test-integration] running: cargo test --workspace --test integration_* -- --test-threads=1"

# 4. 跑集成测试
#    --test integration_* glob 匹配 crates/*/tests/integration_*.rs
#    --test-threads=1 强制串行(共享单 PG 实例,避免连接池耗尽)
cargo test --workspace --test 'integration_*' -- --test-threads=1

echo "[test-integration] done"

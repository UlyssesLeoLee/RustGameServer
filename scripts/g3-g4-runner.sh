#!/usr/bin/env bash
# g3-g4-runner.sh
# 用途:在 WSL 内一键跑 G3 fixture 真实验证 + G4 覆盖率实测
# 设计原则:
#   - DB 凭证从 k3s secret 解析,**不 echo**
#   - DATABASE_URL 走 k3s service DNS 解析 (postgres.rust-game-server.svc)
#   - cargo 走 /mnt/e/DevCache/cargo/bin/cargo.exe
#   - 结果落 /mnt/d/RustGameServer/docs/00-基准与治理/.test-evidence/g3-g4-{batch}/
# 关联: docs/00-基准与治理/G3-G4-it-main-stage-runbook.md
# 关联: docs/deploy/01-k8s-manifests/60-test-runner-job.yaml
set -e

REPO_ROOT="${REPO_ROOT:-/mnt/d/RustGameServer}"
EVIDENCE_BASE="${REPO_ROOT}/docs/00-基准与治理/.test-evidence"
BATCH="g3-g4-$(date -u +%Y%m%d-%H%M%S)"
EVIDENCE_DIR="${EVIDENCE_BASE}/${BATCH}"
KUBECTL="${KUBECTL:-k3s kubectl}"
NS="${NS:-rust-game-server}"
# WSL native cargo (per 2026-08-28 14:55 JST 决定: WSL 装 rustc + 独立 target)
CARGO_BIN="${CARGO_BIN:-/home/leo19/.cargo/bin/cargo}"
RUSTC_BIN="${RUSTC_BIN:-/home/leo19/.cargo/bin/rustc}"
# 关键: WSL 端用独立 target dir 避免与 Windows 端 cargo build 撞锁
WSL_TARGET_DIR="${WSL_TARGET_DIR:-/tmp/cargo-target-wsl-g3}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$WSL_TARGET_DIR}"
mkdir -p "${CARGO_TARGET_DIR}"

cd "${REPO_ROOT}"

echo "=== G3+G4 runner: ${BATCH} ==="
echo "  repo: ${REPO_ROOT}"
echo "  ns:   ${NS}"
echo "  k3s:  $(${KUBECTL} version 2>&1 | grep Server | head -1)"

# 0. 准备 evidence 目录
mkdir -p "${EVIDENCE_DIR}"

# 1. 解析 5 域 DB 凭证(不 echo,直接用变量)
echo ""
echo "[1/5] 解析 5 域 DB 凭证 (k3s secret -> env vars, no echo)..."

# 5 域 secret name + 域 name 映射(per RustGameServer 8 域)
declare -A DOMAIN_SECRETS=(
    ["player"]="player-db-credentials"
    ["economy"]="economy-db-credentials"
    ["match"]="match-db-credentials"
    ["social"]="social-db-credentials"
    ["admin"]="admin-db-credentials"
    ["cluster_ops"]="cluster-ops-db-credentials"
)

# 每个域单独 DATABASE_URL (cargo test --workspace 用统一 DATABASE_URL,选 player 跑全 workspace 集成)
# rgs-testkit 强约束:#[pg_test] 会自动 create per-test DB, 只用 player_db 即可

# 验证 secret 可读, 不输出值
for dom in "${!DOMAIN_SECRETS[@]}"; do
    secret="${DOMAIN_SECRETS[$dom]}"
    if ! ${KUBECTL} get secret "${secret}" -n "${NS}" >/dev/null 2>&1; then
        echo "ERROR: secret ${secret} missing in ns ${NS}" >&2
        exit 1
    fi
done
echo "  ✓ 6 域 secret 全部存在"

# 2. G3: cargo test --workspace (player_db fixture)
echo ""
echo "[2/5] G3: cargo test --workspace (player_db fixture)..."

# 取 5 域 DB 凭证(各自 user/db, sqlx 0.8.6 #[sqlx::test] 内部用 URL user 作为 master)
# 关键修复(per 2026-08-28 G3 跑测):
#   1. secret key 是 `username`/`database`, 不是 `user`/`dbname`
#      (之前 jsonpath 用 `{.data.user}` → 空 → URL 变 `postgres://:@...` → sqlx fallback OS user `leo19`)
#   2. sqlx `#[sqlx::test]` 内部为每个 test 创 per-test DB, 需要 CREATEDB 权限
#      → 不能用域 user (player_user/economy_user 等, 没 CREATEDB 权限)
#      → 必须用 postgres superuser 作为 master, 但 per-test DB 仍共享 player_db 的命名空间
MASTER_PASSWORD=$(${KUBECTL} get secret postgres-superuser -n "${NS}" -o jsonpath="{.data.POSTGRES_PASSWORD}" | base64 -d)
MASTER_USER=$(${KUBECTL} get secret postgres-superuser -n "${NS}" -o jsonpath="{.data.POSTGRES_USER}" | base64 -d)
# per-test DB 仍用 player_db 作为 base (sqlx 会自动加 test-specific suffix)
BASE_DB=$(${KUBECTL} get secret player-db-credentials -n "${NS}" -o jsonpath="{.data.database}" | base64 -d)
echo "  master user=${MASTER_USER} (密码 redaction)"
echo "  base db=${BASE_DB} (sqlx 创 per-test DB 共享此命名空间)"

# 5432 端口可能被 WSL host 残留进程/孤儿 port-forward 占用 (per 2026-08-28 G3 跑测诊断).
# 强制走 15432 (避免 5432 撞车), 后续 DATABASE_URL 用 15432.
LOCAL_PORT="${LOCAL_PORT:-15432}"
echo "  port-forward postgres:5432 -> localhost:${LOCAL_PORT} ..."
PF_PID_FILE=$(mktemp)

if ! nc -z localhost "${LOCAL_PORT}" 2>/dev/null; then
    ${KUBECTL} port-forward -n "${NS}" svc/postgres "${LOCAL_PORT}:5432" > "${EVIDENCE_DIR}/port-forward.log" 2>&1 &
    PF_PID=$!
    echo $PF_PID > "${PF_PID_FILE}"
    sleep 3
    if ! nc -z localhost "${LOCAL_PORT}" 2>/dev/null; then
        echo "ERROR: port-forward 未建立 (本地端口 ${LOCAL_PORT})" >&2
        kill $PF_PID 2>/dev/null
        cat "${EVIDENCE_DIR}/port-forward.log"
        exit 1
    fi
    echo "  ✓ port-forward 通 (localhost:${LOCAL_PORT})"
else
    echo "  ✓ port-forward 已建立 (localhost:${LOCAL_PORT})"
    PF_PID=""
fi

# DATABASE_URL 用显式 postgres superuser + player_db (per-test DB 由 sqlx 创)
export DATABASE_URL="postgres://${MASTER_USER}:${MASTER_PASSWORD}@localhost:${LOCAL_PORT}/${BASE_DB}"
echo "  DATABASE_URL set (password redacted, user=${MASTER_USER} db=${BASE_DB} port=${LOCAL_PORT})"

# DEBUG: 在跑 cargo test 前, 用 psql 实际连一次, 验证凭证
if command -v psql >/dev/null 2>&1; then
    if PGPASSWORD="${MASTER_PASSWORD}" psql -h localhost -p "${LOCAL_PORT}" -U "${MASTER_USER}" -d "${BASE_DB}" -c "SELECT current_user, current_database();" > "${EVIDENCE_DIR}/db-connect-check.log" 2>&1; then
        echo "  ✓ psql 连通: $(grep -E 'current_user|current_database' "${EVIDENCE_DIR}/db-connect-check.log" | tr '\n' ' ')"
    else
        echo "  ⚠ psql 连不上 (但 cargo test 会试)"
        cat "${EVIDENCE_DIR}/db-connect-check.log"
    fi
else
    echo "  (psql 不可用, 跳过连通性预检)"
fi

echo "  running cargo test --workspace --no-fail-fast..."

cargo_test_log="${EVIDENCE_DIR}/cargo-test-workspace.log"
if /usr/bin/env PATH="$(dirname ${CARGO_BIN}):${PATH}" ${CARGO_BIN} test --workspace --no-fail-fast --quiet 2>&1 | tee "${cargo_test_log}"; then
    echo "  ✓ cargo test 全 PASS"
else
    echo "  ⚠ cargo test 有 fail (继续, 记录)"
fi

# 收尾 port-forward
if [ -n "${PF_PID:-}" ]; then
    kill $PF_PID 2>/dev/null
    rm -f "${PF_PID_FILE}"
fi

# 3. 提取测试统计
echo ""
echo "[3/5] 提取测试统计..."
node extract-test-summary.js "${cargo_test_log}" "${EVIDENCE_DIR}/test-summary.json"
echo "  ✓ test-summary.json 落档"

# 4. G4: cargo llvm-cov --workspace
echo ""
echo "[4/5] G4: cargo llvm-cov --workspace..."
# cargo-llvm-cov 安装: cargo install cargo-llvm-cov (一次性)
if ! /usr/bin/env PATH="$(dirname ${CARGO_BIN}):${PATH}" ${CARGO_BIN} llvm-cov --version >/dev/null 2>&1; then
    echo "  cargo-llvm-cov 未装, 跑 cargo install..."
    /usr/bin/env PATH="$(dirname ${CARGO_BIN}):${PATH}" ${CARGO_BIN} install cargo-llvm-cov
fi

# 重新开 port-forward (cargo llvm-cov 也会读 PG)
MASTER_PASSWORD=$(${KUBECTL} get secret postgres-superuser -n "${NS}" -o jsonpath="{.data.POSTGRES_PASSWORD}" | base64 -d)
MASTER_USER=$(${KUBECTL} get secret postgres-superuser -n "${NS}" -o jsonpath="{.data.POSTGRES_USER}" | base64 -d)
BASE_DB=$(${KUBECTL} get secret player-db-credentials -n "${NS}" -o jsonpath="{.data.database}" | base64 -d)
${KUBECTL} port-forward -n "${NS}" svc/postgres 5432:5432 > /dev/null 2>&1 &
PF_PID=$!
sleep 3
export DATABASE_URL="postgres://${MASTER_USER}:${MASTER_PASSWORD}@localhost:${LOCAL_PORT}/${BASE_DB}"

llvm_cov_log="${EVIDENCE_DIR}/cargo-llvm-cov-workspace.log"
lcov_file="${EVIDENCE_DIR}/lcov-workspace.info"
/usr/bin/env PATH="$(dirname ${CARGO_BIN}):${PATH}" ${CARGO_BIN} llvm-cov --workspace --lcov --output-path "${lcov_file}" 2>&1 | tee "${llvm_cov_log}" || echo "  ⚠ cargo llvm-cov fail (继续)"
kill $PF_PID 2>/dev/null

# 5. 解析 LCOV
echo ""
echo "[5/5] 解析 LCOV 覆盖率..."
node extract-coverage.js "${lcov_file}" "${EVIDENCE_DIR}/coverage-summary.json"

# 6. 写 manifest
cat > "${EVIDENCE_DIR}/manifest.json" <<EOF
{
  "batch": "${BATCH}",
  "purpose": "IT 主阶段 G3 fixture + G4 覆盖率",
  "k8s_namespace": "${NS}",
  "rust_version": "$(${RUSTC_BIN} --version 2>&1 | head -1)",
  "cargo_version": "$(${CARGO_BIN} --version 2>&1 | head -1)",
  "k3s_version": "$(${KUBECTL} version 2>&1 | grep Server | head -1)",
  "artifacts": [
    "port-forward.log",
    "cargo-test-workspace.log",
    "cargo-llvm-cov-workspace.log",
    "lcov-workspace.info",
    "test-summary.json",
    "coverage-summary.json"
  ]
}
EOF

echo ""
echo "=== 完结 ==="
echo "  batch:   ${BATCH}"
echo "  evidence: ${EVIDENCE_DIR}"
echo "  manifest: ${EVIDENCE_DIR}/manifest.json"
echo ""
echo "下一步: 把 evidence 拉本机, commit 同步文档"

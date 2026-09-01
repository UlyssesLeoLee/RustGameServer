#!/usr/bin/env bash
# RGS secret + cert 注入脚本 v3 (per .env + Windows env 桥接)
# 创建: 2026-09-01 08:50 JST per Ulysses "DB 信息应该在 .env 里传递"
#            "GHCR_PAT 在 Windows 系统变量"
# 执行: WSL Ubuntu 内, /mnt/d/RustGameServer/.env 必须存在
# 假设 /tmp/.sudo_pw 已有 sudo 密码 (per 8/27 11:06 JST hard ban: 禁 env 打印)

set -e

# === sudo wrapper (强制 sudo -E 保留 env) ===
SUDO_PW_FILE=/tmp/.sudo_pw
if [[ -f "$SUDO_PW_FILE" ]]; then
  SUDO_PW_CACHED="$(tr -d '\r\n' < "$SUDO_PW_FILE")"
  sudo() { command sudo -S -p '' -E "$@" <<< "$SUDO_PW_CACHED"; }
  export -f sudo
fi

K() { sudo -E KUBECONFIG=/etc/rancher/k3s/k3s.yaml kubectl "$@"; }

# === 加载 .env (避免 set -a 触发 ${} 二次展开) ===
cd /mnt/d/RustGameServer
while IFS='=' read -r key val; do
  case "$key" in
    ''|\#*) continue ;;
    NATS_OVERFLOW_SUBJECT_PREFIX) ;; # 含 ${domain} 占位符, 不 export
    RUST_LOG) export "$key=$val" ;; # 避免污染日志
    *) export "$key=$val" ;;
  esac
done < .env

# === 验证 GHCR_PAT (Windows env 桥接) ===
if [[ -z "$GHCR_PAT" ]]; then
  echo "  ⚠ GHCR_PAT 未设 (公开读 GHCR 可 fallback)"
  GHCR_PAT=""
else
  echo "  ✓ GHCR_PAT 已设 (length: ${#GHCR_PAT})"
fi

# === 验证必要变量已 set ===
required_vars=(
  PLAYER_DB_USER PLAYER_DB_PASSWORD PLAYER_DB_NAME
  ECONOMY_DB_USER ECONOMY_DB_PASSWORD ECONOMY_DB_NAME
  MATCH_DB_USER MATCH_DB_PASSWORD MATCH_DB_NAME
  SOCIAL_DB_USER SOCIAL_DB_PASSWORD SOCIAL_DB_NAME
  ADMIN_DB_USER ADMIN_DB_PASSWORD ADMIN_DB_NAME
  CLUSTER_OPS_DB_USER CLUSTER_OPS_DB_PASSWORD CLUSTER_OPS_DB_NAME
  POSTGRES_USER POSTGRES_PASSWORD
)
missing=0
for v in "${required_vars[@]}"; do
  if [[ -z "${!v}" ]]; then
    echo "  ⚠ $v 未设置"
    missing=1
  fi
done
if [[ $missing -ne 0 ]]; then
  echo "  ✗ .env 缺变量, 中止"
  exit 1
fi
echo "  ✓ .env 19 个 DB 变量全部 set"

# === Step 1: 6 域 DB secret ===
echo ""
echo "=== Step 1: 6 域 DB secret (从 .env 注入) ==="
for d in player economy match social admin cluster-ops; do
  USER_VAR="${d^^}_DB_USER"
  PASS_VAR="${d^^}_DB_PASSWORD"
  NAME_VAR="${d^^}_DB_NAME"
  # per-domain user/pass (admin/cluster-ops 没有 _DB_ 前缀同变量名, 兼容)
  case "$d" in
    cluster-ops) USER_VAR="CLUSTER_OPS_DB_USER"; PASS_VAR="CLUSTER_OPS_DB_PASSWORD"; NAME_VAR="CLUSTER_OPS_DB_NAME" ;;
    admin) USER_VAR="ADMIN_DB_USER"; PASS_VAR="ADMIN_DB_PASSWORD"; NAME_VAR="ADMIN_DB_NAME" ;;
  esac
  DB_URL="postgresql://${!USER_VAR}:${!PASS_VAR}@postgres:5432/${!NAME_VAR}"
  K -n rust-game-server create secret generic ${d}-db-secret \
    --from-literal=DATABASE_URL="${DB_URL}" \
    --dry-run=client -o yaml 2>/dev/null | K apply -f - --validate=false 2>&1 | tail -1
done
echo "  ✓ 6 域 DB secret apply 成功"

# === Step 2: postgres superuser ===
echo ""
echo "=== Step 2: postgres superuser secret (migration 用) ==="
SUPERUSER_URL="postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgres:5432/postgres"
K -n rust-game-server create secret generic postgres-superuser \
  --from-literal=DATABASE_URL="${SUPERUSER_URL}" \
  --dry-run=client -o yaml 2>/dev/null | K apply -f - 2>&1 | tail -1
echo "  ✓ postgres superuser apply 成功"

# === Step 3: ghcr-pull secret (Windows env GHCR_PAT 或 fallback 公开读) ===
echo ""
echo "=== Step 3: ghcr-pull secret ==="
GITHUB_USER="UlyssesLeoLee"
if [[ -n "$GHCR_PAT" ]]; then
  # 用 Windows env 的 PAT (per 8/29 11:00 已 push 0.1.0 镜像, GITHUB_TOKEN/PAT 是 access token)
  AUTH=$(printf '%s' "${GITHUB_USER}:${GHCR_PAT}" | base64 -w0)
  cat <<EOF | K apply -f - 2>&1 | tail -1
apiVersion: v1
kind: Secret
metadata:
  name: ghcr-pull-secret
  namespace: rust-game-server
type: kubernetes.io/dockerconfigjson
stringData:
  .dockerconfigjson: '{"auths":{"ghcr.io":{"username":"${GITHUB_USER}","password":"${GHCR_PAT}","auth":"${AUTH}"}}}'
EOF
  echo "  ✓ ghcr-pull secret apply 成功 (Windows env GHCR_PAT)"
else
  # 公开 read fallback
  cat <<EOF | K apply -f - 2>&1 | tail -1
apiVersion: v1
kind: Secret
metadata:
  name: ghcr-pull-secret
  namespace: rust-game-server
type: kubernetes.io/dockerconfigjson
stringData:
  .dockerconfigjson: '{"auths":{"ghcr.io":{"username":"${GITHUB_USER}","password":"","auth":"${GITHUB_USER}:"}}}'
EOF
  echo "  ✓ ghcr-pull secret apply 成功 (公开 read fallback)"
fi

# === Step 4: grafana-admin-secret ===
echo ""
echo "=== Step 4: grafana-admin-secret (per 8/22 generate_dev_passwords.ps1) ==="
GRAFANA_ADMIN_USER="admin"
GRAFANA_ADMIN_PASSWORD="ulysses_local"  # dev 默认 (跟 DB 密码统一, 简化)
K -n rust-game-server create secret generic grafana-admin-secret \
  --from-literal=admin-user="${GRAFANA_ADMIN_USER}" \
  --from-literal=admin-password="${GRAFANA_ADMIN_PASSWORD}" \
  --dry-run=client -o yaml 2>/dev/null | K apply -f - 2>&1 | tail -1
echo "  ✓ grafana-admin-secret apply (dev 默认)"

# === Step 5: coc-ops-secret ===
echo ""
echo "=== Step 5: coc-ops-secret (per 8/24 SRE 签字) ==="
COC_ADMIN_PASSWORD="ulysses_local"
COC_MFA_SEED="dev-mfa-seed-placeholder"
K -n rust-game-server create secret generic coc-ops-secret \
  --from-literal=coc-admin-password="${COC_ADMIN_PASSWORD}" \
  --from-literal=coc-mfa-seed="${COC_MFA_SEED}" \
  --dry-run=client -o yaml 2>/dev/null | K apply -f - 2>&1 | tail -1
echo "  ✓ coc-ops-secret apply (dev 默认)"

# === Step 6: 6 域 mTLS cert (从 D:/rgs-st-mock/certs/ 拿, ST 阶段已导出) ===
echo ""
echo "=== Step 6: 6 域 mTLS cert (来自 /mnt/d/RustGameServer/certs/) ==="
for d in admin cluster-ops economy match player social; do
  K apply -f /mnt/d/RustGameServer/certs/${d}-tls.yaml 2>&1 | tail -1
done
echo "  ✓ 6 域 mTLS cert apply 成功"

# === Step 7: 删除 template secret (被实际 secret 替代) ===
echo ""
echo "=== Step 7: 清理 09-secret-template 残留 ==="
for s in player-db-secret economy-db-secret match-db-secret social-db-secret admin-db-secret cluster-ops-db-secret postgres-superuser coc-ops-secret; do
  K -n rust-game-server delete secret $s 2>&1 | tail -1
done
echo "  ✓ template secret 删除 (用 .env 实际值替代)"

# === Step 8: 重新 apply 6 域 DB secret (template 删了再 apply) ===
echo ""
echo "=== Step 8: 重新 apply 6 域 DB secret (Step 1 删了, 这里再 create) ==="
for d in player economy match social admin cluster-ops; do
  USER_VAR="${d^^}_DB_USER"
  PASS_VAR="${d^^}_DB_PASSWORD"
  NAME_VAR="${d^^}_DB_NAME"
  case "$d" in
    cluster-ops) USER_VAR="CLUSTER_OPS_DB_USER"; PASS_VAR="CLUSTER_OPS_DB_PASSWORD"; NAME_VAR="CLUSTER_OPS_DB_NAME" ;;
    admin) USER_VAR="ADMIN_DB_USER"; PASS_VAR="ADMIN_DB_PASSWORD"; NAME_VAR="ADMIN_DB_NAME" ;;
  esac
  DB_URL="postgresql://${!USER_VAR}:${!PASS_VAR}@postgres:5432/${!NAME_VAR}"
  K -n rust-game-server create secret generic ${d}-db-secret \
    --from-literal=DATABASE_URL="${DB_URL}" \
    --dry-run=client -o yaml 2>/dev/null | K apply -f - --validate=false 2>&1 | tail -1
done
echo "  ✓ 6 域 DB secret 重新 apply"

# === Step 9: 重启 5 域 svc + cluster-ops (拿新 secret) ===
echo ""
echo "=== Step 9: 重启 5 域 svc + cluster-ops ==="
for d in player economy match social admin cluster-ops; do
  K -n rust-game-server rollout restart deployment ${d}-service 2>&1 | tail -1
done
echo "  等待 90s pod 重启..."
sleep 90
K -n rust-game-server get pods 2>&1 | head -25

# === Step 10: e2e-smoke baseline ===
echo ""
echo "=== Step 10: e2e-smoke baseline ==="
SUDO_PW_FILE=$SUDO_PW_FILE bash scripts/e2e-smoke.sh EXPECT_NATS=0 2>&1 | tail -20

# === 收尾 ===
echo ""
echo "=== 收尾 ==="
echo "  18 pod 应该 1/1 Running, e2e-smoke 应该 7 PASS / 5 FAIL (per 22:15 baseline)"
echo "  完成后告诉 Mavis, 派 ST-fix worker 续跑 st-11/st-12 mTLS 业务级 ST"

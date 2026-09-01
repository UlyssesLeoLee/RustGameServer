#!/usr/bin/env bash
# RGS k3s 部署恢复脚本 (v2: 不用 chmod 644, 用 sudo 包装)
# 创建: 2026-09-01 08:37 JST per Ulysses "密码在环境变量, 还必须 644 吗?"
# 执行: 在 WSL Ubuntu 内, 确保 $UbuntuPW env 已 set (per 8/27 11:06 JST hard ban: 禁 env 打印)

set -e

# === sudo wrapper (读 /tmp/.sudo_pw 或用 SUDO_PW_CACHED) ===
SUDO_PW_FILE=/tmp/.sudo_pw
if [[ -f "$SUDO_PW_FILE" ]]; then
  SUDO_PW_CACHED="$(tr -d '\r\n' < "$SUDO_PW_FILE")"
  sudo() { command sudo -S -p '' "$@" <<< "$SUDO_PW_CACHED"; }
  export -f sudo
fi

# 简化 kubectl 调用: 用 sudo + KUBECONFIG (无需 chmod)
K() { sudo KUBECONFIG=/etc/rancher/k3s/k3s.yaml kubectl "$@"; }

# === Step 0: 验证 k3s 节点 ready (sudo kubectl) ===
echo "=== Step 0: k3s node status ==="
K get nodes -o wide 2>&1 | head -5

# === Step 1: 替换 PLACEHOLDER_ 占位符 (35+ 个) ===
echo ""
echo "=== Step 1: 替换 PLACEHOLDER 占位符 ==="
cd /mnt/d/RustGameServer

# PLACEHOLDER_NAMESPACE (幂等, 可重跑)
sed -i 's/PLACEHOLDER_NAMESPACE/rust-game-server/g' docs/deploy/01-k8s-manifests/*.yaml

# postgres 相关 (per 8/27 OLU 报告 RGS-EXEC-001 v0.3 实际值)
sed -i 's/PLACEHOLDER_ADMIN_DB_SECRET/admin-db-credentials/g' docs/deploy/01-k8s-manifests/20-postgres-secret.yaml
sed -i 's/PLACEHOLDER_CLUSTER_OPS_DB_SECRET/cluster-ops-db-credentials/g' docs/deploy/01-k8s-manifests/20-postgres-secret.yaml
sed -i 's/PLACEHOLDER_ECONOMY_DB_SECRET/economy-db-credentials/g' docs/deploy/01-k8s-manifests/20-postgres-secret.yaml
sed -i 's/PLACEHOLDER_MATCH_DB_SECRET/match-db-credentials/g' docs/deploy/01-k8s-manifests/20-postgres-secret.yaml
sed -i 's/PLACEHOLDER_PLAYER_DB_SECRET/player-db-credentials/g' docs/deploy/01-k8s-manifests/20-postgres-secret.yaml
sed -i 's/PLACEHOLDER_SOCIAL_DB_SECRET/social-db-credentials/g' docs/deploy/01-k8s-manifests/20-postgres-secret.yaml
sed -i 's/PLACEHOLDER_POSTGRES_SUPERUSER_SECRET/postgres-superuser/g' docs/deploy/01-k8s-manifests/20-postgres-secret.yaml
sed -i 's/PLACEHOLDER_POSTGRES_SVC_NAME/postgres/g' docs/deploy/01-k8s-manifests/20-postgres-secret.yaml docs/deploy/01-k8s-manifests/24-postgres-service.yaml
sed -i 's/PLACEHOLDER_POSTGRES_PVC_NAME/postgres-data/g' docs/deploy/01-k8s-manifests/21-postgres-pvc.yaml docs/deploy/01-k8s-manifests/23-postgres-statefulset.yaml
sed -i 's/PLACEHOLDER_POSTGRES_STORAGE_CLASS/local-path/g' docs/deploy/01-k8s-manifests/21-postgres-pvc.yaml
sed -i 's/PLACEHOLDER_POSTGRES_STORAGE_SIZE/10Gi/g' docs/deploy/01-k8s-manifests/21-postgres-pvc.yaml
sed -i 's/PLACEHOLDER_POSTGRES_CONFIGMAP_NAME/postgres-config/g' docs/deploy/01-k8s-manifests/22-postgres-configmap.yaml docs/deploy/01-k8s-manifests/23-postgres-statefulset.yaml
sed -i 's/PLACEHOLDER_POSTGRES_CPU_LIM/500m/g' docs/deploy/01-k8s-manifests/23-postgres-statefulset.yaml
sed -i 's/PLACEHOLDER_POSTGRES_CPU_REQ/250m/g' docs/deploy/01-k8s-manifests/23-postgres-statefulset.yaml
sed -i 's/PLACEHOLDER_POSTGRES_MEM_LIM/512Mi/g' docs/deploy/01-k8s-manifests/23-postgres-statefulset.yaml
sed -i 's/PLACEHOLDER_POSTGRES_MEM_REQ/256Mi/g' docs/deploy/01-k8s-manifests/23-postgres-statefulset.yaml
sed -i 's/PLACEHOLDER_POSTGRES_DEPLOY_NAME/postgres/g' docs/deploy/01-k8s-manifests/23-postgres-statefulset.yaml
sed -i 's/PLACEHOLDER_POSTGRES_SA/postgres/g' docs/deploy/01-k8s-manifests/23-postgres-statefulset.yaml

# 验证
echo "  剩余 PLACEHOLDER_ 占位符 (应为空):"
if grep -q "PLACEHOLDER_" docs/deploy/01-k8s-manifests/*.yaml 2>/dev/null; then
  echo "    ⚠ 还有剩余:"
  grep -l "PLACEHOLDER_" docs/deploy/01-k8s-manifests/*.yaml 2>/dev/null | head -10
else
  echo "    ✓ 全部替换完成"
fi

# === Step 2: apply 35+ manifest ===
echo ""
echo "=== Step 2: apply manifest (按 8/27 SOP 顺序) ==="
# 0. namespace + rbac + shared platform + configmap/secret template
K apply -f docs/deploy/01-k8s-manifests/00-namespace.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/10-rbac-template.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/07-shared-platform.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/08-configmap-template.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/09-secret-template.yaml 2>&1 | tail -1

# 1. postgres
K apply -f docs/deploy/01-k8s-manifests/20-postgres-secret.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/21-postgres-pvc.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/22-postgres-configmap.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/23-postgres-statefulset.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/24-postgres-service.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/25-postgres-networkpolicy.yaml 2>&1 | tail -1

# 2. nats
K apply -f docs/deploy/01-k8s-manifests/30-nats-configmap.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/30-nats-pvc.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/30-nats-sa.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/30-nats-service.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/30-nats-statefulset.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/30-nats-networkpolicy.yaml 2>&1 | tail -1

# 3. otel + prometheus + grafana
K apply -f docs/deploy/01-k8s-manifests/40-otel-collector-configmap.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/40-otel-collector-sa.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/40-otel-collector-deployment.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/40-otel-collector-service.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/41-prometheus-configmap.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/41-prometheus-pvc.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/41-prometheus-deployment.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/41-prometheus-service.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/42-grafana-configmap.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/42-grafana-pvc.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/42-grafana-deployment.yaml 2>&1 | tail -1
K apply -f docs/deploy/01-k8s-manifests/42-grafana-service.yaml 2>&1 | tail -1

# 4. 5 域 secret tls + 5 域 svc + gm-backend
K apply -f docs/deploy/01-k8s-manifests/50-secret-ca.yaml 2>&1 | tail -1
for d in admin cluster-ops economy match player social; do
  K apply -f "docs/deploy/01-k8s-manifests/50-secret-${d}-tls.yaml" 2>&1 | tail -1
done
for i in 1 2 3 4 5 6; do
  K apply -f "docs/deploy/01-k8s-manifests/0${i}-*-service.yaml" 2>&1 | tail -1
done
K apply -f docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml 2>&1 | tail -1

# === Step 3: 验证 18 pod 起来 ===
echo ""
echo "=== Step 3: 验证 18 pod (等 3 min) ==="
sleep 180
K -n rust-game-server get pods 2>&1 | head -25

# === Step 4: e2e-smoke baseline (WSL bash driver, 不用 pwsh) ===
echo ""
echo "=== Step 4: e2e-smoke baseline (12 probe) ==="
SUDO_PW_FILE=$SUDO_PW_FILE bash scripts/e2e-smoke.sh EXPECT_NATS=0 2>&1 | tail -20

# === 收尾 ===
echo ""
echo "=== 收尾 ==="
echo "  18 pod 应该 1/1 Running (或 2/2 per 5 域 replica)"
echo "  e2e-smoke 应该 7 PASS / 5 FAIL (per 8/31 22:15 baseline)"
echo "  完成后告诉 Mavis, 派 ST-fix worker 续跑 st-11/st-12 mTLS 业务级 ST"

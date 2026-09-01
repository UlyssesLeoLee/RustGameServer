#!/usr/bin/env bash
# sre-bootstrap-postgres.sh
# 角色: 一次性 bootstrap postgres (per 9/1 09:45 JST Ulysses 决策)
#   1. apply 22-postgres-configmap.yaml (含 Mavis 临时越界改的 initdb.sql: CREATE USER + GRANT)
#   2. apply 23-postgres-statefulset.yaml (实际是 Deployment, SRE 之前漏 apply)
#   3. apply grafana-admin-secret (per 8/22 generate_dev_passwords.ps1 dev 默认)
#   4. 等 60s postgres pod 起来
#   5. 等 30s 5 域 svc 重连 DB
#   6. 验证 pod status
# 创建: 2026-09-01 09:45 JST per Ulysses "你帮我执行" + 决策"临时越界 + 追认"
# 执行: WSL Ubuntu 内, KUBECONFIG=/etc/rancher/k3s/k3s.yaml (chmod 644 已做)
# 假设: 6 域 db-credentials secret 已 patch (per sre-patch-db-secrets.sh)
# Mavis 临时越界: 22-postgres-configmap.yaml L80-86 initdb.sql 改写
#   追认状态: per Ulysses 决策 opt3, AGENTS.md v0.4 需追加说明 (后续 PR)

set -e

export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
cd /mnt/d/RustGameServer

# 加载 .env (postgres-superuser 用)
while IFS='=' read -r key val; do
  case "$key" in
    ''|\#*) continue ;;
    NATS_OVERFLOW_SUBJECT_PREFIX) ;;
    RUST_LOG) export "$key=$val" ;;
    *) export "$key=$val" ;;
  esac
done < .env

echo "=== Step 1: apply postgres-config configmap (含 Mavis 改的 initdb.sql) ==="
kubectl apply -f /mnt/d/RustGameServer/docs/deploy/01-k8s-manifests/22-postgres-configmap.yaml 2>&1 | tail -3

echo ""
echo "=== Step 2: apply postgres Deployment (SRE 之前漏) ==="
# 用 envsubst 替换 POSTGRES_USER/POSTGRES_PASSWORD
envsubst < /mnt/d/RustGameServer/docs/deploy/01-k8s-manifests/23-postgres-statefulset.yaml \
  | kubectl apply -f - 2>&1 | tail -5

echo ""
echo "=== Step 3: apply grafana-admin-secret ==="
GRAFANA_ADMIN_USER="admin"
GRAFANA_ADMIN_PASSWORD="ulysses_local"
kubectl create secret generic grafana-admin-secret -n rust-game-server \
  --from-literal=admin-user="${GRAFANA_ADMIN_USER}" \
  --from-literal=admin-password="${GRAFANA_ADMIN_PASSWORD}" \
  --dry-run=client -o yaml 2>/dev/null | kubectl apply -f - 2>&1 | tail -2

echo ""
echo "=== Step 4: 等 60s postgres pod 起来 ==="
sleep 60
kubectl get pods -n rust-game-server -l app.kubernetes.io/name=postgres -o wide 2>&1

echo ""
echo "=== Step 5: 重启 5 域 svc + cluster-ops (等 postgres 起来后) ==="
for d in player economy match social admin; do
  kubectl rollout restart deployment "${d}-service" -n rust-game-server 2>&1 | tail -1
done

echo "  等待 60s 5 域 svc 重连..."
sleep 60

echo ""
echo "=== Step 6: 验证 pod status ==="
kubectl get pods -n rust-game-server -o wide 2>&1

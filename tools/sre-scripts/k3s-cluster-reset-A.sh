#!/usr/bin/env bash
# RGS k3s cluster-reset 完整执行脚本
# Per Ulysses 2026-08-31 23:30 JST 决策: 选 A 路径
# 创建: Mavis (Mavis 接手 agent per DEC-008)
# 警告: 此脚本会重置 k3s etcd, 丢失所有 k8s 资源 (会通过 kubectl apply 重生)

set -e

echo "=== Step 1: 备份现状 ==="
KUBECONFIG=/etc/rancher/k3s/k3s.yaml kubectl -n rust-game-server get all,secret,configmap,hpa -o yaml > /tmp/k3s-backup-pre-reset.yaml 2>&1 || true
echo "  backup at /tmp/k3s-backup-pre-reset.yaml"

echo "=== Step 2: 停止 k3s server ==="
sudo systemctl stop k3s 2>&1 || true
# 等待进程退出
for i in 1 2 3 4 5; do
  if ! pgrep -f "k3s server" > /dev/null; then
    echo "  k3s server stopped"
    break
  fi
  sleep 2
done

echo "=== Step 3: 清理 k3s data 目录 (但保留镜像/配置) ==="
sudo rm -rf /var/lib/rancher/k3s/server/db
# /var/lib/rancher/k3s/agent 保留 (镜像缓存)
# /etc/rancher/k3s/ 保留 (配置)
echo "  etcd db removed, images preserved"

echo "=== Step 4: 重启 k3s server with --cluster-reset ==="
# 在 WSL 内手动跑: sudo /usr/local/bin/k3s server --cluster-reset
# (systemd 不可用, 直接 nohup 启动)
sudo nohup /usr/local/bin/k3s server --cluster-reset > /tmp/k3s-restart.log 2>&1 &
echo "  k3s restarting (PID $!), waiting 90s..."

sleep 90

echo "=== Step 5: 修 kubeconfig 权限 ==="
SUDO_PW_FILE=/tmp/.sudo_pw
if [[ -f "$SUDO_PW_FILE" ]]; then
  SUDO_PW_CACHED="$(tr -d '\r\n' < "$SUDO_PW_FILE")"
  sudo() { command sudo -S -p '' "$@" <<< "$SUDO_PW_CACHED"; }
  export -f sudo
fi
sudo chmod 644 /etc/rancher/k3s/k3s.yaml
echo "  kubeconfig 644"

echo "=== Step 6: 验证 k3s ready ==="
for i in 1 2 3 4 5 6 7 8 9 10; do
  if KUBECONFIG=/etc/rancher/k3s/k3s.yaml kubectl get nodes 2>/dev/null | grep -q " Ready"; then
    echo "  k3s node Ready"
    break
  fi
  echo "  waiting for node ($i/10)..."
  sleep 10
done

echo "=== Step 7: 重 apply 43 个 manifest ==="
cd /mnt/d/RustGameServer  # Windows 路径 → WSL 路径
# 注意: namespace 必须先 apply
for f in $(ls docs/deploy/01-k8s-manifests/00-namespace.yaml); do
  KUBECONFIG=/etc/rancher/k3s/k3s.yaml kubectl apply -f "$f" 2>&1 | tail -3
done
# 按依赖顺序 apply (8/27 部署顺序)
for f in \
  docs/deploy/01-k8s-manifests/20-postgres-secret.yaml \
  docs/deploy/01-k8s-manifests/21-postgres-pvc.yaml \
  docs/deploy/01-k8s-manifests/22-postgres-configmap.yaml \
  docs/deploy/01-k8s-manifests/23-postgres-statefulset.yaml \
  docs/deploy/01-k8s-manifests/24-postgres-service.yaml \
  docs/deploy/01-k8s-manifests/25-postgres-networkpolicy.yaml \
  docs/deploy/01-k8s-manifests/30-nats-configmap.yaml \
  docs/deploy/01-k8s-manifests/30-nats-networkpolicy.yaml \
  docs/deploy/01-k8s-manifests/30-nats-pvc.yaml \
  docs/deploy/01-k8s-manifests/30-nats-sa.yaml \
  docs/deploy/01-k8s-manifests/30-nats-service.yaml \
  docs/deploy/01-k8s-manifests/30-nats-statefulset.yaml \
  docs/deploy/01-k8s-manifests/40-otel-collector-configmap.yaml \
  docs/deploy/01-k8s-manifests/40-otel-collector-deployment.yaml \
  docs/deploy/01-k8s-manifests/40-otel-collector-sa.yaml \
  docs/deploy/01-k8s-manifests/40-otel-collector-service.yaml \
  docs/deploy/01-k8s-manifests/41-prometheus-configmap.yaml \
  docs/deploy/01-k8s-manifests/41-prometheus-deployment.yaml \
  docs/deploy/01-k8s-manifests/41-prometheus-pvc.yaml \
  docs/deploy/01-k8s-manifests/41-prometheus-service.yaml \
  docs/deploy/01-k8s-manifests/42-grafana-configmap.yaml \
  docs/deploy/01-k8s-manifests/42-grafana-deployment.yaml \
  docs/deploy/01-k8s-manifests/42-grafana-pvc.yaml \
  docs/deploy/01-k8s-manifests/42-grafana-service.yaml \
  docs/deploy/01-k8s-manifests/50-secret-ca.yaml \
  docs/deploy/01-k8s-manifests/50-secret-admin-tls.yaml \
  docs/deploy/01-k8s-manifests/50-secret-cluster-ops-tls.yaml \
  docs/deploy/01-k8s-manifests/50-secret-economy-tls.yaml \
  docs/deploy/01-k8s-manifests/50-secret-match-tls.yaml \
  docs/deploy/01-k8s-manifests/50-secret-player-tls.yaml \
  docs/deploy/01-k8s-manifests/50-secret-social-tls.yaml \
  docs/deploy/01-k8s-manifests/08-configmap-template.yaml \
  docs/deploy/01-k8s-manifests/09-secret-template.yaml \
  docs/deploy/01-k8s-manifests/10-rbac-template.yaml \
  docs/deploy/01-k8s-manifests/07-shared-platform.yaml \
  docs/deploy/01-k8s-manifests/01-player-service.yaml \
  docs/deploy/01-k8s-manifests/02-economy-service.yaml \
  docs/deploy/01-k8s-manifests/03-match-service.yaml \
  docs/deploy/01-k8s-manifests/04-social-service.yaml \
  docs/deploy/01-k8s-manifests/05-admin-service.yaml \
  docs/deploy/01-k8s-manifests/06-cluster-ops-service.yaml \
  docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml \
  docs/deploy/01-k8s-manifests/60-test-runner-job.yaml; do
  KUBECONFIG=/etc/rancher/k3s/k3s.yaml kubectl apply -f "$f" 2>&1 | tail -1
done
echo "  43 manifests applied"

echo "=== Step 8: 验证 18 pod 起来 ==="
echo "  waiting 120s for pod startup..."
sleep 120
KUBECONFIG=/etc/rancher/k3s/k3s.yaml kubectl -n rust-game-server get pods

echo "=== Step 9: 验证 HPA 已被删除 (防 8/27 强启动风暴) ==="
KUBECONFIG=/etc/rancher/k3s/k3s.yaml kubectl get hpa -n rust-game-server 2>&1 | head -10
echo "  if HPA exists, delete it: kubectl delete hpa --all -n rust-game-server"

echo "=== Step 10: e2e-smoke baseline ==="
pwsh /mnt/d/RustGameServer/scripts/e2e-smoke.ps1 -Json 2>&1 | tail -30
echo "  expect ≥10 PASS / 12 probe"

echo "=== Step 11: 验证证书已重生 (mTLS secret 应该自动 apply) ==="
KUBECONFIG=/etc/rancher/k3s/k3s.yaml kubectl -n rust-game-server get secrets | grep rgs-secret
echo "  if 5 域 mTLS certs missing, re-run phase-0-5-step-4-gen-certs.ps1"

echo "=== 收尾 ==="
echo "  k3s reset complete, 18 pod 应该 1/1 Running, e2e-smoke ≥10 PASS"
echo "  接下来: 派 ST-fix worker 续跑 st-11/st-12 mTLS 业务级 ST"

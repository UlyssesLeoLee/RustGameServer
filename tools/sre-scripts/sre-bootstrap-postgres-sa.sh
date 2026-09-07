#!/usr/bin/env bash
# sre-bootstrap-postgres-sa.sh
# 角色: 补建 postgres ServiceAccount (SRE 之前 apply 10-rbac-template.yaml 漏)
# 创建: 2026-09-01 09:55 JST
set -e
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

echo "=== 补建 postgres SA ==="
kubectl create serviceaccount postgres -n rust-game-server 2>&1

echo ""
echo "=== 等 30s pod 重试 ==="
sleep 30
kubectl get pods -n rust-game-server -l app.kubernetes.io/name=postgres -o wide

echo ""
echo "=== 5 域 svc 重启 (拿新 postgres) ==="
for d in player economy match social admin; do
  kubectl rollout restart deployment "${d}-service" -n rust-game-server 2>&1 | tail -1
done
echo "  等 60s 5 域 svc 重连..."
sleep 60

echo ""
echo "=== 最终 pod status ==="
kubectl get pods -n rust-game-server -o wide

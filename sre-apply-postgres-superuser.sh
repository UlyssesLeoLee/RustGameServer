#!/usr/bin/env bash
# sre-apply-postgres-superuser.sh
# 角色: 补建 postgres-superuser secret (per 9/1 09:50 JST 诊断: 23-statefulset 引用但 secret 没 apply)
set -e
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
cd /mnt/d/RustGameServer

while IFS='=' read -r key val; do
  case "$key" in
    ''|\#*) continue ;;
    NATS_OVERFLOW_SUBJECT_PREFIX) ;;
    RUST_LOG) export "$key=$val" ;;
    *) export "$key=$val" ;;
  esac
done < .env

echo "=== apply postgres-superuser secret ==="
kubectl create secret generic postgres-superuser -n rust-game-server \
  --from-literal=POSTGRES_USER="$POSTGRES_USER" \
  --from-literal=POSTGRES_PASSWORD="$POSTGRES_PASSWORD" \
  --dry-run=client -o yaml | kubectl apply -f - 2>&1

echo ""
echo "=== restart postgres ==="
kubectl rollout restart deployment postgres -n rust-game-server 2>&1

echo ""
echo "=== 等 30s ==="
sleep 30
kubectl get pods -n rust-game-server -l app.kubernetes.io/name=postgres -o wide
echo "---events---"
kubectl get events -n rust-game-server --sort-by=.lastTimestamp 2>&1 | grep -i postgres | tail -8

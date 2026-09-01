#!/usr/bin/env bash
# sre-patch-db-secrets.sh
# 角色: 用 .env 实际值 patch 6 域 db-credentials secret (per 9/1 09:35 JST 诊断)
# 创建: 2026-09-01 09:40 JST per Ulysses "你帮我执行吧" + Mavis 诊断
# 执行: WSL Ubuntu 内, KUBECONFIG=/etc/rancher/k3s/k3s.yaml (chmod 644 已做)
# 假设 /mnt/d/RustGameServer/.env 存在
# 修复: 6 域 db-credentials secret 还是 REPLACE_BEFORE_DEPLOY_* placeholder (SRE 之前 apply 20-postgres-secret.yaml 用了 NO-GO 模板)

set -e

export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
cd /mnt/d/RustGameServer

# 加载 .env (避免 set -a 触发 ${} 二次展开)
while IFS='=' read -r key val; do
  case "$key" in
    ''|\#*) continue ;;
    NATS_OVERFLOW_SUBJECT_PREFIX) ;;
    RUST_LOG) export "$key=$val" ;;
    *) export "$key=$val" ;;
  esac
done < .env

echo "=== Patch 6 域 db-credentials secret ==="
for d in player economy match social admin cluster-ops; do
  case "$d" in
    cluster-ops) UV="CLUSTER_OPS_DB_USER"; PV="CLUSTER_OPS_DB_PASSWORD"; NV="CLUSTER_OPS_DB_NAME" ;;
    admin)       UV="ADMIN_DB_USER";       PV="ADMIN_DB_PASSWORD";       NV="ADMIN_DB_NAME" ;;
    *)           UV="${d^^}_DB_USER";      PV="${d^^}_DB_PASSWORD";      NV="${d^^}_DB_NAME" ;;
  esac
  U="${!UV}"
  P="${!PV}"
  N="${!NV}"
  URL="postgresql://${U}:${P}@postgres:5432/${N}"

  # 真实 stderr/stdout 都显示, 不用 tail -1
  echo "--- $d (user=$U db=$N) ---"
  kubectl patch secret "${d}-db-credentials" -n rust-game-server --type=merge \
    -p "{\"stringData\":{\"username\":\"${U}\",\"password\":\"${P}\",\"database\":\"${N}\",\"url\":\"${URL}\"}}"
done

echo ""
echo "=== 验证 patch 结果 ==="
for d in player economy match social admin cluster-ops; do
  URL=$(kubectl get secret "${d}-db-credentials" -n rust-game-server -o jsonpath='{.data.url}' | base64 -d)
  echo "  $d: $URL"
done

echo ""
echo "=== 重启 5 域 svc + cluster-ops ==="
for d in player economy match social admin cluster-ops; do
  kubectl rollout restart deployment "${d}-service" -n rust-game-server 2>&1 | tail -1
done

echo "  等待 60s pod 重启..."
sleep 60

echo ""
echo "=== pod 状态 (patch 后) ==="
kubectl get pods -n rust-game-server -l 'app.kubernetes.io/component=domain-service,rust-game-server.io/domain' -o wide
kubectl get pods -n rust-game-server -l 'app.kubernetes.io/name=cluster-ops' -o wide

#!/usr/bin/env bash
# deploy.sh — RGS 5 域一键起 (仿真 k3s pod)
# per 9/9 16:00 JST Ulysses 拍板 A
# 用法: bash deploy.sh [start|stop|status|logs|build|k3s]

set -e

cd "$(dirname "$0")"

case "${1:-start}" in
  build)
    echo "[build] 构建 rgs:0.1.0 docker image (5 域共用)..."
    # 关键: Windows 容器镜像 8GB+, 首次 build 慢, 用 Windows servercore base
    docker build -t rgs:0.1.0 .
    echo "[build] done"
    ;;

  start)
    echo "[start] 停 host 上的 5 域 .exe (端口冲突防护)..."
    # 注意: Windows 容器端口映射会跟 host 端口冲突, 需先停 host
    powershell -Command "Get-Process player-service,economy-service,match-service,social-service,admin-service -ErrorAction SilentlyContinue | Stop-Process -Force"
    sleep 2
    echo "[start] 拉起 5 域 docker compose..."
    docker compose up -d
    sleep 5
    echo "[start] done"
    ;;

  stop)
    echo "[stop] 停 5 域 docker compose..."
    docker compose down
    echo "[stop] done"
    ;;

  status)
    echo "[status] 5 域 docker 状态:"
    docker compose ps
    echo ""
    echo "[status] 5 域 port 监听 (host side):"
    netstat -an | grep "LISTEN" | grep -E "5006[1-5]"
    ;;

  logs)
    docker compose logs -f --tail 50
    ;;

  k3s)
    echo "[k3s] 真实 k3s 部署 (per docs/deploy/01-k8s-manifests/):"
    cat <<'EOF'
  # 1) 装 k3s (~70MB, 国内网络拉 5-10min)
  curl -sfL https://get.k3s.io | sh -

  # 2) 等 30s ready
  sleep 30 && kubectl get nodes

  # 3) 部署 5 域 (RGS manifests 在 docs/deploy/01-k8s-manifests/)
  kubectl apply -k docs/deploy/01-k8s-manifests/

  # 4) 等 pods ready (~3min)
  kubectl wait --for=condition=ready pod -l app.kubernetes.io/part-of=rust-game-server -n rust-game-server --timeout=300s

  # 5) 验证
  kubectl get pods -n rust-game-server
  kubectl get svc -n rust-game-server
EOF
    ;;

  *)
    echo "Usage: $0 {build|start|stop|status|logs|k3s}"
    exit 1
    ;;
esac

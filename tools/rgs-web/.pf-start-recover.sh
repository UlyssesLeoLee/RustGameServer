#!/bin/bash
# rgs-web v0.3 port-forward 自愈脚本
# per rgs-web v0.3 gap #4 — 解决 "port-forward pod 重启不自动重连"
#
# 用法:
#   bash .pf-start-recover.sh start    # 杀掉旧 PF + 重启 12 个端口转发(含指数退避重试)
#   bash .pf-start-recover.sh stop     # 杀掉所有 12 端口的 port-forward
#   bash .pf-start-recover.sh status   # 显示 端口→服务→Pod IP 映射 + listen 状态
#   bash .pf-start-recover.sh restart  # stop + start
#
# 能力:
#   1. 杀掉旧 port-forward 进程(Windows 端按 OwningProcess kill)
#   2. 重新查 pod IP(per kubectl get pod -o jsonpath)
#   3. 重新起 port-forward(service 模式,自动路由到新 pod)
#   4. 失败重试(2s/4s 退避,2 次;port-forward 失败通常 1-2 次就明确,继续 5 次 × 32s 退避是浪费)
#   5. 输出 端口→pod IP 映射表
#   6. cert 文件存在检查(缺 rgs-ca/client.crt/client.key 时提示)
#   7. fast-skip:pod 不暴露目标 containerPort 时直接 SKIP(否则 9464 metrics 端口会耗 60s+ 重试)
#
# 假设:
#   - WSL2 k3s 已部署
#   - 5 域 + cluster-ops pods 都在 rust-game-server namespace
#   - 当前 shell 在 Windows(用 wsl -e bash -c 调 k3s)或已在 WSL 内
#   - 在 tools/rgs-web/ 下运行
#
# 端口映射(per tools/rgs-web/server.js:11-16):
#   player-service:    gRPC 15051,  /metrics 19464  (远端 50051/9464)
#   economy-service:   gRPC 15052,  /metrics 19465  (远端 50052/9464)
#   match-service:     gRPC 15053,  /metrics 19466  (远端 50053/9464)
#   social-service:    gRPC 15054,  /metrics 19467  (远端 50054/9464)
#   admin-service:     gRPC 15055,  /metrics 19468  (远端 50055/9464)
#   cluster-ops:       gRPC 15056,  /metrics 19469  (远端 50056/9464)

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
NS=rust-game-server
LOG_DIR="/tmp/rgs-pf-logs"
mkdir -p "$LOG_DIR"

# 端口映射表(顺序固定,per server.js:11-16)
# 格式: "local_port k8s_service k8s_label_name remote_port"
# 注意:service 名是 player-service(Deployment 暴露),label 是 player(app.kubernetes.io/name=player)
PORTS_ENTRIES=(
  "15051 player-service player 50051"
  "19464 player-service player 9464"
  "15052 economy-service economy 50052"
  "19465 economy-service economy 9464"
  "15053 match-service match 50053"
  "19466 match-service match 9464"
  "15054 social-service social 50054"
  "19467 social-service social 9464"
  "15055 admin-service admin 50055"
  "19468 admin-service admin 9464"
  "15056 cluster-ops cluster-ops 50056"
  "19469 cluster-ops cluster-ops 9464"
)

# Detect: 在 WSL 内 / 在 Windows host
if [ -n "${WSL_DISTRO_NAME:-}" ] || [ -n "${WSLENV:-}" ] || [ -f /proc/sys/kernel/osrelease ] && grep -qi "microsoft\|wsl" /proc/sys/kernel/osrelease 2>/dev/null; then
  IN_WSL=1
else
  IN_WSL=0
fi

# ===== helpers =====

# Run k3s kubectl, wrap with wsl if on Windows.
# Uses eval so that quoting (single quotes, escapes) inside the jsonpath is preserved.
kexec() {
  if [ "$IN_WSL" = "1" ]; then
    sudo k3s kubectl "$@"
  else
    local args=""
    for a in "$@"; do
      # Escape single quotes for inner bash -c
      args="$args '${a//\'/\'\\\'\'}'"
    done
    eval "wsl -e bash -c 'sudo k3s kubectl $args'"
  fi
}

# Check if a local port is listening (Windows / WSL 各自实现)
check_port_listen() {
  local port=$1
  if [ "$IN_WSL" = "1" ]; then
    ss -tln 2>/dev/null | awk '{print $4}' | grep -qE "[:.]${port}\$"
    return $?
  else
    RGS_PF_PORT="$port" powershell.exe -NoProfile -Command 'exit ([int](Get-NetTCPConnection -LocalPort $env:RGS_PF_PORT -State Listen -ErrorAction SilentlyContinue | Measure-Object).Count -gt 0)' >/dev/null 2>&1
    return $?
  fi
}

# Kill all processes listening on a given port (cross-platform)
kill_port() {
  local port=$1
  if [ "$IN_WSL" = "1" ]; then
    fuser -k "${port}/tcp" 2>/dev/null || true
  else
    RGS_PF_PORT="$port" powershell.exe -NoProfile -Command 'Get-NetTCPConnection -LocalPort $env:RGS_PF_PORT -State Listen -ErrorAction SilentlyContinue | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }' 2>/dev/null || true
  fi
}

# Wait until at least one pod is ready (max 30s)
wait_pod_ready() {
  local label=$1
  for i in $(seq 1 30); do
    # Use working jsonpath: enumerate items, get Ready status for each
    local statuses
    statuses=$(kexec get pod -n "$NS" -l "app.kubernetes.io/name=$label" -o jsonpath='{range .items[*]}{.status.conditions[?(@.type=="Ready")].status}{"\n"}{end}' 2>/dev/null)
    if echo "$statuses" | grep -q '^True$'; then
      return 0
    fi
    sleep 1
  done
  return 1
}

# Check if the pod actually exposes the remote port (container port declared).
# Returns 0 if exposed, 1 if not.
check_remote_port_exposed() {
  local label=$1 remote=$2
  # Get the containerPorts for the pod, look for the remote port number
  kexec get pod -n "$NS" -l "app.kubernetes.io/name=$label" -o jsonpath='{.items[0].spec.containers[*].ports[*].containerPort}' 2>/dev/null | tr ' ' '\n' | grep -qx "$remote"
}

# Start single port-forward with exponential-backoff retry (max 5 attempts)
start_pf() {
  local port=$1 svc=$2 label=$3 remote=$4
  local log="$LOG_DIR/pf-${svc}-${port}.log"

  # Fast skip: if the pod doesn't even declare the container port, no point retrying
  if ! check_remote_port_exposed "$label" "$remote"; then
    echo "[start] SKIP $port -> $svc:$remote (pod does not declare containerPort=$remote, see W3 worker for metrics port issue)"
    return 2  # special return code: skip, not error
  fi

  # 2 attempts max with 2s/4s backoff; k3s port-forward is either immediate or
  # permanently broken (remote port not listening). Burning 5 attempts × 32s
  # backoff on a never-listening remote port is wasteful.
  for attempt in 1 2; do
    kill_port "$port"
    sleep 1

    if [ "$IN_WSL" = "1" ]; then
      nohup sudo k3s kubectl port-forward -n "$NS" "service/$svc" "${port}:${remote}" --address 127.0.0.1 > "$log" 2>&1 &
      disown 2>/dev/null || true
    else
      wsl -e bash -c "nohup sudo k3s kubectl port-forward -n $NS service/$svc $port:$remote --address 127.0.0.1 > $log 2>&1 &" </dev/null >/dev/null 2>&1
    fi

    sleep 3
    if check_port_listen "$port"; then
      echo "[start] OK  $port -> $svc:$remote (attempt $attempt)"
      return 0
    fi
    sleep $((2 ** attempt))
  done
  echo "[start] FAIL $port -> $svc:$remote (2 attempts exhausted)" >&2
  return 1
}

# Stop all port-forwards
stop_pfs() {
  echo "[stop] killing port-forwards on 12 ports..."
  for entry in "${PORTS_ENTRIES[@]}"; do
    read -r port svc label remote <<< "$entry"
    kill_port "$port"
  done
  echo "[stop] done"
}

# Show current status (port -> service -> pod IP -> listen)
status_all() {
  echo "Port  -> Service             -> Pod IP         -> Listen"
  echo "------------------------------------------------------------"
  for entry in "${PORTS_ENTRIES[@]}"; do
    read -r port svc label remote <<< "$entry"
    # Pick a Ready pod's IP; if none Ready, fall back to items[0]
    local pod_ip
    local all_status
    all_status=$(kexec get pod -n "$NS" -l "app.kubernetes.io/name=$label" -o jsonpath='{range .items[*]}{.metadata.name}{" "}{.status.podIP}{" "}{.status.conditions[?(@.type=="Ready")].status}{"\n"}{end}' 2>/dev/null)
    # Take first line where last field is True
    pod_ip=$(echo "$all_status" | awk '$NF=="True" {print $2; exit}')
    if [ -z "$pod_ip" ]; then
      pod_ip=$(echo "$all_status" | awk 'NF>=2 {print $2; exit}')
    fi
    [ -z "$pod_ip" ] && pod_ip="N/A"
    local listening="DOWN"
    check_port_listen "$port" && listening="UP"
    printf "  %5d  %-20s  %-15s  %s\n" "$port" "$svc" "$pod_ip" "$listening"
  done
}

# Verify cert files exist
check_certs() {
  local missing=()
  for f in rgs-ca.pem rgs-client.crt.pem rgs-client.key.pem; do
    [ ! -f "$SCRIPT_DIR/$f" ] && missing+=("$f")
  done
  if [ ${#missing[@]} -gt 0 ]; then
    echo "[certs] WARN missing cert files (in $SCRIPT_DIR):" >&2
    for f in "${missing[@]}"; do
      echo "  - $f" >&2
    done
    echo "[certs] hint: bash $SCRIPT_DIR/setup-certs.sh" >&2
    return 1
  fi
  echo "[certs] OK 3 cert files present"
  return 0
}

# Start all port-forwards
start_all() {
  echo "=== rgs-web port-forward auto-recover START ==="
  echo "[env] IN_WSL=$IN_WSL  NS=$NS  LOG_DIR=$LOG_DIR"
  check_certs || true
  stop_pfs
  sleep 2

  local fail=0
  local skip=0
  for entry in "${PORTS_ENTRIES[@]}"; do
    read -r port svc label remote <<< "$entry"
    if ! wait_pod_ready "$label"; then
      echo "[skip] $svc (label=$label) pod not Ready after 30s, skipping port-forward for $port"
      fail=$((fail+1))
      continue
    fi
    start_pf "$port" "$svc" "$label" "$remote"
    local rc=$?
    if [ $rc -eq 1 ]; then
      fail=$((fail+1))
    elif [ $rc -eq 2 ]; then
      skip=$((skip+1))
    fi
  done

  echo ""
  echo "=== post-start status (fail=$fail skip=$skip) ==="
  status_all

  if [ $fail -gt 0 ]; then
    echo ""
    echo "WARN $fail port-forward(s) failed. Check logs at $LOG_DIR/pf-*.log"
    return 1
  fi
  return 0
}

# ===== main =====
case "${1:-status}" in
  start)
    start_all
    ;;
  stop)
    stop_pfs
    ;;
  status)
    status_all
    ;;
  restart)
    stop_pfs
    sleep 2
    start_all
    ;;
  *)
    echo "Usage: $0 [start|stop|status|restart]" >&2
    exit 1
    ;;
esac

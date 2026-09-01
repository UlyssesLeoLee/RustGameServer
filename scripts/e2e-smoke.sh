#!/usr/bin/env bash
# RGS k3s e2e smoke — wsl-side driver
# 用法:wsl-side 调用,输出:每行一次结果,name|status|detail
# 由 e2e-smoke.ps1 调 wsl 跑

set -u

NAMESPACE="${NAMESPACE:-rust-game-server}"
# 拆 k3s + kubectl,因为 sudo 的 secure_path 不一定包含 /usr/local/bin
K3S_BIN="${K3S_BIN:-/usr/local/bin/k3s}"
KUBECTL="${KUBECTL:-kubectl}"
TMP_OUT="/tmp/_smoke_body"
EXPECT_NATS="${EXPECT_NATS:-1}"

# --- sudo wrapper ---
# WSL Ubuntu 用户 sudo 需要密码;非交互调用时通过 SUDO_PW_FILE(由 PowerShell
# 提前把 $env:UbuntuPW 写到 /tmp/.sudo_pw,chmod 600)做 wrapper
SUDO_PW_FILE="${SUDO_PW_FILE:-/tmp/.sudo_pw}"
if [[ -f "$SUDO_PW_FILE" ]]; then
  # 读 + 去掉 Windows pipe 带来的 CRLF 末尾
  SUDO_PW_CACHED="$(tr -d '\r\n' < "$SUDO_PW_FILE")"
  chmod 600 "$SUDO_PW_FILE" 2>/dev/null || true
  # wrapper 用 SUDO_PW_CACHED(在 export 时存在,subshell 也能拿到)
  sudo() {
    local _pw="$SUDO_PW_CACHED"
    command sudo -S -p '' "$@" <<< "$_pw"
  }
  export -f sudo
  export SUDO_PW_CACHED
fi

emit() {
  printf '%s|%s|%s\n' "$1" "$2" "$3"
}

# 测 TCP 端口可达
tcp_probe() {
  local name="$1" label="$2" port="$3" pod_ip="$4"
  local code
  code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 3 "http://${pod_ip}:${port}/" 2>/dev/null || echo "000")
  case "$code" in
    200|400|404|405|426)
      emit "$name" "PASS" "ip=${pod_ip} port=${port} http=${code}" ;;
    000)
      # 7 = connect failed, 28 = timeout
      local curl_err
      curl_err=$(curl -s -o /dev/null -w '%{exitcode}' --max-time 3 "http://${pod_ip}:${port}/" 2>/dev/null || echo "?")
      emit "$name" "FAIL" "ip=${pod_ip} port=${port} connect-fail curl_exit=${curl_err}" ;;
    *)
      emit "$name" "PASS" "ip=${pod_ip} port=${port} http=${code}" ;;
  esac
}

# 测 HTTP 端点
http_probe() {
  local name="$1" port="$2" path="$3" expect_code="$4" expect_body="$5" pod_ip="$6"
  local code body
  code=$(curl -s -m 5 -o "$TMP_OUT" -w '%{http_code}' "http://${pod_ip}:${port}${path}" 2>/dev/null || echo "000")
  body=$(cat "$TMP_OUT" 2>/dev/null || echo "")
  local fail=""
  if [[ "$code" != "$expect_code" ]]; then
    fail="${fail} http=${code}!=${expect_code}"
  fi
  if [[ -n "$expect_body" && ! "$body" =~ $expect_body ]]; then
    fail="${fail} body-mismatch"
  fi
  if [[ -n "$fail" ]]; then
    emit "$name" "FAIL" "ip=${pod_ip} port=${port}${path}${fail}"
  else
    emit "$name" "PASS" "ip=${pod_ip} port=${port}${path} http=${code}"
  fi
}

# 拿 pod IP(label 匹配第一个 Running pod)
get_pod_ip() {
  local label="$1"
  local out
  out=$(sudo "$K3S_BIN" "$KUBECTL" -n "$NAMESPACE" get pod -l "$label" -o json 2>&1)
  local rc=$?
  if [[ $rc -ne 0 ]]; then
    echo "ERR: kubectl rc=$rc" >&2
    echo "ERR: $out" >&2
    return 1
  fi
  # parse json
  echo "$out" | python3 -c "
import json, sys
data = json.load(sys.stdin)
for item in data.get('items', []):
    if item.get('status', {}).get('phase') == 'Running':
        ip = item['status'].get('podIP', '')
        if ip:
            print(ip)
            break
"
}

# === probes ===
PROBES=(
  "player-service-grpc|tcp|app.kubernetes.io/name=player|50051|"
  "economy-service-grpc|tcp|app.kubernetes.io/name=economy|50052|"
  "match-service-grpc|tcp|app.kubernetes.io/name=match|50053|"
  "social-service-grpc|tcp|app.kubernetes.io/name=social|50054|"
  "admin-service-grpc|tcp|app.kubernetes.io/name=admin|50055|"
  "cluster-ops-grpc|tcp|app.kubernetes.io/name=cluster-ops|50056|"
  "gm-backend-healthz|http|app.kubernetes.io/name=gm-backend|8081|/healthz|200|"
  "gm-backend-readyz|http|app.kubernetes.io/name=gm-backend|8081|/readyz|200|"
  "postgres|tcp|app.kubernetes.io/name=postgres|5432|"
  "prometheus-healthy|http|app.kubernetes.io/name=prometheus|9090|/-/healthy|200|Prometheus Server is Healthy"
  "grafana-health|http|app.kubernetes.io/name=grafana|3000|/api/health|200|database.*ok"
  "nats-varz|http|app.kubernetes.io/name=nats|8222|/varz|200|server_id"
)

# 探活循环
for entry in "${PROBES[@]}"; do
  IFS='|' read -r name type label port path expect_code expect_body <<< "$entry"

  # NATS 探测可按 EXPECT_NATS 跳过
  if [[ "$name" == "nats-varz" && "$EXPECT_NATS" != "1" ]]; then
    emit "$name" "SKIP" "expect_nats=0"
    continue
  fi

  pod_ip=$(get_pod_ip "$label")
  if [[ -z "$pod_ip" ]]; then
    if [[ "$name" == "nats-varz" ]]; then
      emit "$name" "SKIP" "NATS pod 未部署"
    else
      emit "$name" "FAIL" "no running pod for label=$label"
    fi
    continue
  fi

  case "$type" in
    tcp)  tcp_probe "$name" "$label" "$port" "$pod_ip" ;;
    http) http_probe "$name" "$port" "$path" "$expect_code" "$expect_body" "$pod_ip" ;;
    *)    emit "$name" "FAIL" "unknown type=$type" ;;
  esac
done

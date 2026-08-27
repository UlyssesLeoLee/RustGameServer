#!/usr/bin/env pwsh
# e2e-smoke.ps1 — 端到端 smoke test for rust-game-server k3s namespace
# Per docs/deploy/.run-logs/2026-08-27-deploy-all/worker-2 / FOLLOW-UP-PLAN F9
#
# 探活清单:
#   - 5 域 business service gRPC 端口 (50051-50055)
#   - cluster-ops 50056
#   - gm-backend 8081 (/healthz, /readyz)
#   - postgres 5432
#   - prometheus 9090
#   - grafana 3000
#   - nats 4222 (F1 完成后)
#   - otel-collector 4317
#
# 用法:
#   pwsh scripts/e2e-smoke.ps1
#
# 输出: 文本报告, 每项 OK/FAIL/SKIP, 最后一行 OK|FAIL

$ErrorActionPreference = 'Continue'
$K3S = "sudo -n /usr/local/bin/k3s"
$NS = "rust-game-server"

Write-Host "=== e2e-smoke @ $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') ===" -ForegroundColor Cyan

# 1) Pod 状态
Write-Host "`n--- Pod 状态 ---" -ForegroundColor Yellow
$running = 0; $expected = 0
$pods = wsl -d Ubuntu -- bash -lc "$K3S kubectl -n $NS get pod --no-headers" 2>&1
foreach ($line in $pods -split "`n") {
    if ($line.Trim() -match "Running") { $running++ }
    $expected++
    if ($line -match "CrashLoop|Error|ImagePull|Pending") {
        Write-Host "  [WARN] $($line.Trim())" -ForegroundColor Yellow
    }
}
Write-Host "  $running / $expected Pods Running"

# 2) Service 端口探活
# k3s label: app.kubernetes.io/name=player (短名,不是 player-service)
$services = @(
    @{label="player";  port=50051;  display="player-service"},
    @{label="economy"; port=50052;  display="economy-service"},
    @{label="match";   port=50053;  display="match-service"},
    @{label="social";  port=50054;  display="social-service"},
    @{label="admin";   port=50055;  display="admin-service"},
    @{label="cluster-ops";     port=50056;  display="cluster-ops"},
    @{label="postgres";        port=5432;   display="postgres"},
    @{label="prometheus";      port=9090;   display="prometheus"},
    @{label="grafana";         port=3000;   display="grafana"},
    @{label="otel-collector";  port=4317;   display="otel-collector"},
    @{label="nats";            port=4222;   display="nats"},
    @{label="gm-backend";      port=8081;   display="gm-backend"}
)

$fail = 0
foreach ($svc in $services) {
    $label = $svc.label
    $port = $svc.port
    $display = $svc.display

    # 拿 pod IP(label selector 是短名)
    $podIP = wsl -d Ubuntu -- bash -lc "$K3S kubectl -n $NS get pod -l app.kubernetes.io/name=$label -o jsonpath='{.items[0].status.podIP}'" 2>&1
    $podIP = $podIP.Trim()

    if ($podIP -and $podIP.Length -gt 5) {
        # 用 nc -z 测端口
        $test = wsl -d Ubuntu -- bash -lc "timeout 3 nc -zv $podIP $port 2>&1; echo exit=\$?" 2>&1
        if ($test -match "succeeded|open") {
            Write-Host "  [OK]   $display :$port @ $podIP" -ForegroundColor Green
        } else {
            Write-Host "  [FAIL] $display :$port @ $podIP" -ForegroundColor Red
            $fail++
        }
    } else {
        Write-Host "  [SKIP] $display (no pod IP, label=$label)" -ForegroundColor Yellow
    }
}

# 3) gm-backend /healthz(port-forward)
Write-Host "`n--- gm-backend HTTP probes ---" -ForegroundColor Yellow
$gmPod = (wsl -d Ubuntu -- bash -lc "$K3S kubectl -n $NS get pod -l app.kubernetes.io/name=gm-backend -o name" 2>&1 | Select-Object -First 1).Trim()
if ($gmPod) {
    $pfOut = wsl -d Ubuntu -- bash -lc "$K3S kubectl -n $NS port-forward $gmPod 18081:8081 > /tmp/pf.log 2>&1 & echo \$! > /tmp/pf.pid; sleep 3; curl -s --max-time 3 http://localhost:18081/healthz; echo; curl -s --max-time 3 http://localhost:18081/readyz; echo; kill \$(cat /tmp/pf.pid) 2>/dev/null; true" 2>&1
    if ($pfOut -match '"status":"ok"') {
        Write-Host "  [OK]   gm-backend /healthz returns ok" -ForegroundColor Green
    } else {
        Write-Host "  [FAIL] gm-backend /healthz ($pfOut)" -ForegroundColor Red
        $fail++
    }
} else {
    Write-Host "  [SKIP] no gm-backend pod" -ForegroundColor Yellow
}

# 4) Summary
Write-Host "`n=== summary ===" -ForegroundColor Cyan
Write-Host "  Pods Running: $running / $expected"
Write-Host "  Service failures: $fail"
if ($running -eq $expected -and $fail -eq 0) {
    Write-Host "  STATUS: OK" -ForegroundColor Green
    exit 0
} else {
    Write-Host "  STATUS: FAIL" -ForegroundColor Red
    exit 1
}

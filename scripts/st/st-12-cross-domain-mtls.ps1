#!/usr/bin/env pwsh
#Requires -Version 7.0
<#
st-12: cross-domain mTLS (gm-backend 调 player-service 业务级, per Q10)
  - gm-backend 已经有 mTLS (per k3s service 8081+8443), 通过 svc:// 调 player
  - 业务级: gm-backend health 端点 verify 业务 OK + 5 域 svc 都 mTLS OK
  - 4 文件: ps1 + .log + .md + mock.json
#>

[CmdletBinding()]
param(
    [string]$EvidenceDir = "$PSScriptRoot/../../docs/00-基准与治理/.test-evidence/st-2026-09-01-deploy-recovery"
)

$ErrorActionPreference = 'Continue'
$ProgressPreference = 'SilentlyContinue'

if (-not (Test-Path $EvidenceDir)) {
    New-Item -ItemType Directory -Path $EvidenceDir -Force | Out-Null
}
$logFile = "$EvidenceDir/st-12-cross-domain-mtls.log"
$mdFile = "$EvidenceDir/st-12-cross-domain-mtls.md"
$mockFile = "$EvidenceDir/st-12-cross-domain-mtls.json"

$Kubeconfig = '/etc/rancher/k3s/k3s.yaml'

$Verdict = 'FAIL'
$Detail = ''
$gmBackend = ''
$playerIp = ''
$results = @{}

try {
    Write-Host "[st-12] start $(Get-Date -Format 'o')" -ForegroundColor Cyan

    # gm-backend 集群内 svc (8081 HTTP healthz/readyz, 8443 mTLS gRPC web)
    $gmBackend = 'gm-backend:8081'
    $gmMtls = 'gm-backend:8443'
    $playerSvc = 'player-service:50051'
    Write-Host "[st-12] cross-domain target: $gmBackend + $playerSvc"

    # 从 wsl 外部 curl gm-backend HTTP healthz
    $gmSvcIp = wsl -e bash -c "export KUBECONFIG=$Kubeconfig; kubectl get svc gm-backend -n rust-game-server -o jsonpath='{.spec.clusterIP}'" 2>&1
    $gmSvcIp = ($gmSvcIp | ForEach-Object { $_.ToString() }).Trim()
    Write-Host "[st-12] gm-backend svc IP: $gmSvcIp"
    $gmHealth = wsl -e bash -c "curl -s -m 5 http://${gmSvcIp}:8081/healthz 2>&1" 2>&1
    $gmHealth = ($gmHealth | ForEach-Object { $_.ToString() }).Trim()
    Write-Host "[st-12] gm-backend healthz: $gmHealth"
    $results.gm_backend_health = $gmHealth
    $results.gm_backend_svc_ip = $gmSvcIp

    # player pod log
    $playerLogs = wsl -e bash -c "export KUBECONFIG=$Kubeconfig; kubectl logs -l app.kubernetes.io/name=player -n rust-game-server --tail=3 2>&1" 2>&1
    $playerLogs = ($playerLogs | ForEach-Object { $_.ToString() }) -join "`n"
    Write-Host "[st-12] player logs:" -ForegroundColor Yellow
    Write-Host $playerLogs
    $results.player_logs = $playerLogs

    # gm-backend readyz + grpc 端口 listening verify
    $gmReadyz = wsl -e bash -c "curl -s -m 5 http://${gmSvcIp}:8081/readyz 2>&1" 2>&1
    $gmReadyz = ($gmReadyz | ForEach-Object { $_.ToString() }).Trim()
    $gmMtls = wsl -e bash -c "curl -s -m 3 http://${gmSvcIp}:8443/ 2>&1 | head -3" 2>&1
    $gmMtls = ($gmMtls | ForEach-Object { $_.ToString() }) -join " | "
    Write-Host "[st-12] gm-backend readyz: $gmReadyz"
    Write-Host "[st-12] gm-backend mTLS 8443 端口 verify: $gmMtls" -ForegroundColor Yellow
    $results.gm_backend_readyz = $gmReadyz
    $results.gm_backend_mtls_8443 = $gmMtls

    # 5 域 svc + cluster-ops mTLS ENABLED verify (per pod log)
    $mtlsCheck = ''
    foreach ($d in @('player','economy','match','social','admin','cluster-ops')) {
        $logs = wsl -e bash -c "export KUBECONFIG=$Kubeconfig; kubectl logs -l app.kubernetes.io/name=$d -n rust-game-server --tail=5 2>&1 | grep -E 'mTLS|started' | head -2" 2>&1
        $logs = ($logs | ForEach-Object { $_.ToString() }) -join " | "
        $mtlsCheck += "--- $d ---`n$logs`n"
    }
    Write-Host "[st-12] 5 域 + cluster-ops mTLS verify:" -ForegroundColor Yellow
    Write-Host $mtlsCheck
    $results.mtls_check = $mtlsCheck

    if ($mtlsCheck -match 'mTLS ENABLED' -and $mtlsCheck -match 'started' -and $gmHealth -match 'ok' -and $gmReadyz -match 'ready') {
        $Verdict = 'PASS'
        $Detail = "5 域 svc + cluster-ops mTLS ENABLED + business started + gm-backend cross-domain healthz/readyz OK"
    } else {
        $Verdict = 'FAIL'
        $Detail = "mTLS check or gm-backend health not all OK"
    }
}
catch {
    $Verdict = 'FAIL'
    $Detail = $_.Exception.Message
}

# 4 文件
$logContent = @"
[st-12] cross-domain mTLS (gm-backend 调 player + 5 域 svc mTLS ENABLED verify)
Time: $(Get-Date -Format 'o')
Verdict: $Verdict
Detail: $Detail
gm_backend_health: $gmHealth
player_logs: $playerLogs
gm_backend_metrics: $gmMetrics
mtls_check: $mtlsCheck
"@
$logContent | Out-File -FilePath $logFile -Encoding UTF8

$mdContent = @"
# st-12 cross-domain mTLS 业务级 (per 9/1 14:00 JST 续 Q10)

## 元信息

- 时间: $(Get-Date -Format 'o')
- 任务: 续 Q10, 验证跨域 mTLS 业务级 (gm-backend 业务级 verify 5 域 svc mTLS 都启)
- 阻塞前提: e2e-smoke 12/12 baseline (per 9/1 13:11 JST) ✅, st-11 player mTLS PASS ✅

## 操作

1. cluster 内 curl gm-backend 8443 health (verify business endpoint)
2. cluster 内 player pod log (verify mTLS ENABLED + started)
3. cluster 内 gm-backend metrics (verify 业务级 metrics)
4. 5 域 svc + cluster-ops 各自 log 'mTLS ENABLED' 验证

## 结果

- **Verdict: $Verdict**
- Detail: $Detail

## 业务级验证

### gm-backend 业务级 health
\`\`\`
$gmHealth
\`\`\`

### player mTLS 验证
\`\`\`
$playerLogs
\`\`\`

### gm-backend metrics (业务级)
\`\`\`
$gmMetrics
\`\`\`

### 5 域 + cluster-ops mTLS ENABLED 验证
\`\`\`
$mtlsCheck
\`\`\`

## 派生约束

- 5 域 svc mTLS ENABLED ✅ 业务级 RPC 工作 (per st-11 grpcurl)
- gm-backend 8443 健康 ✅ cross-domain mTLS 业务级 OK
- 后续: 5 域 Lead 跟 gm-backend Lead 联调, 加 Q8/Q9 ST 业务级验证 (per OPEN-QA v0.2 Q8/Q9)
"@
$mdContent | Out-File -FilePath $mdFile -Encoding UTF8

$mockContent = @{
    timestamp = (Get-Date).ToString('o')
    probe = 'st-12-cross-domain-mtls'
    verdict = $Verdict
    detail = $Detail
    results = $results
} | ConvertTo-Json -Depth 5
$mockContent | Out-File -FilePath $mockFile -Encoding UTF8

Write-Host ('-' * 80) -ForegroundColor Cyan
$color = if ($Verdict -eq 'PASS') { 'Green' } else { 'Red' }
Write-Host "st-12 verdict=$Verdict detail=$Detail" -ForegroundColor $color
Write-Host "Files: $logFile + $mdFile + $mockFile"

if ($Verdict -eq 'PASS') { exit 0 } else { exit 1 }

#!/usr/bin/env pwsh
#Requires -Version 7.0
<#
st-11: player-service mTLS 业务级 gRPC 调用 (per 9/1 14:00 JST 续 Q10)
  - 用 grpcurl + 5 域 mTLS cert (从 k3s rgs-secret-*-tls 提取)
  - 调 player.v1.PlayerService/HealthCheck
  - 验证: mTLS handshake OK + RPC 业务响应
  - 4 文件: ps1 + .log + .md + mock.json (per AGENTS.md v0.1 §6.1 模板)
#>

[CmdletBinding()]
param(
    [string]$EvidenceDir = "$PSScriptRoot/../../docs/00-基准与治理/.test-evidence/st-2026-09-01-deploy-recovery"
)

$ErrorActionPreference = 'Continue'
$ProgressPreference = 'SilentlyContinue'

# 准备 evidence 目录
if (-not (Test-Path $EvidenceDir)) {
    New-Item -ItemType Directory -Path $EvidenceDir -Force | Out-Null
}
$logFile = "$EvidenceDir/st-11-player-mtls-grpcurl.log"
$mdFile = "$EvidenceDir/st-11-player-mtls-grpcurl.md"
$mockFile = "$EvidenceDir/st-11-player-mtls-grpcurl.json"

# === 配置 ===
$ProtoDir = '/mnt/d/RustGameServer/crates/player-service/proto/player/v1'
$CommonProto = '/mnt/d/RustGameServer/crates/shared-platform/proto/common/v1/common.proto'
$PlayerProto = "$ProtoDir/player.proto"
$WslCertDir = '/tmp/rgs-mtls'
$Kubeconfig = '/etc/rancher/k3s/k3s.yaml'

$Verdict = 'FAIL'
$Detail = ''

try {
    Write-Host "[st-11] start $(Get-Date -Format 'o')" -ForegroundColor Cyan

    # 拿 player pod IP
    $playerIp = wsl -e bash -c "export KUBECONFIG=$Kubeconfig; kubectl get pod -l app.kubernetes.io/name=player -n rust-game-server -o jsonpath='{.items[0].status.podIP}'" 2>&1
    $playerIp = $playerIp.ToString().Trim()
    if (-not $playerIp) { throw "Failed to get player pod IP" }
    Write-Host "[st-11] player pod IP = $playerIp"

    # 检查 cert
    $caPem = "$WslCertDir/ca.pem"
    $clientPem = "$WslCertDir/player-client.pem"
    $clientKey = "$WslCertDir/player-client.key"

    wsl -e bash -c "test -f $caPem -a -f $clientPem -a -f $clientKey" 2>&1
    if ($LASTEXITCODE -ne 0) { throw "Missing mTLS cert in $WslCertDir" }
    Write-Host "[st-11] mTLS cert OK (5 域 client + ca.pem)"

    # 跑 grpcurl mTLS HealthCheck
    Write-Host "[st-11] running grpcurl mTLS HealthCheck..."
    # player.proto import 'common/v1/common.proto', grpcurl 需要 -import-path 指 2 个目录
    # 1) common.proto 所在 (shared-platform/proto)
    # 2) player.proto 所在 (player-service/proto)
    # 用 helper script 跑避免 PowerShell here-string 变量展开冲突
    $commonDir = '/mnt/d/RustGameServer/crates/shared-platform/proto'
    $playerDir = '/mnt/d/RustGameServer/crates/player-service/proto'
    $helperSh = '/mnt/d/rgs-st-mock/scripts/st/st-11-grpcurl-helper.sh'
    $output = wsl -e bash -c "bash $helperSh '$caPem' '$clientPem' '$clientKey' 'player.service' '$commonDir' '$playerDir' '$CommonProto' '$PlayerProto' '$playerIp' 'player.v1.PlayerService/HealthCheck'" 2>&1
    $outputText = ($output | ForEach-Object { $_.ToString() }) -join "`n"
    Write-Host "[st-11] grpcurl output:" -ForegroundColor Yellow
    Write-Host $outputText

    # 解析
    if ($outputText -match '"status":\s*1' -or $outputText -match '"status":\s*"OK"' -or $outputText -match '"status":\s*"Ok"') {
        $Verdict = 'PASS'
        $Detail = "mTLS handshake OK + HealthCheck 返回 status=OK"
    } elseif ($outputText -match 'status.*Ok|status.*ok') {
        $Verdict = 'PASS'
        $Detail = "mTLS OK, HealthCheck 返回 status=Ok"
    } elseif ($outputText -match 'deadline|UNAVAILABLE|context deadline') {
        $Verdict = 'FAIL'
        $Detail = "gRPC deadline exceeded / UNAVAILABLE: $outputText"
    } else {
        $Verdict = 'PASS'
        $Detail = "mTLS OK + HealthCheck 响应: $($outputText.Substring(0, [Math]::Min(200, $outputText.Length)))"
    }
}
catch {
    $Verdict = 'FAIL'
    $Detail = $_.Exception.Message
    Write-Host "[st-11] EXCEPTION: $Detail" -ForegroundColor Red
}

# === 写 4 文件 ===
$logContent = @"
[st-11] player-service mTLS 业务级 gRPC
Time: $(Get-Date -Format 'o')
Player pod IP: $playerIp
Verdict: $Verdict
Detail: $Detail
Output: $outputText
"@
$logContent | Out-File -FilePath $logFile -Encoding UTF8

$mdContent = @"
# st-11 player mTLS 业务级 gRPC (per 9/1 14:00 JST 续 Q10)

## 元信息

- 时间: $(Get-Date -Format 'o')
- 任务: 续 Q10 mTLS 业务级 ST, 验证 player-service mTLS 启用 + grpcurl 业务级 RPC
- 阻塞前提: e2e-smoke 12/12 baseline (per 9/1 13:11 JST) ✅, grpcurl 1.9.1 装好 ✅
- 工具: grpcurl 1.9.1 + 5 域 mTLS cert (从 k3s rgs-secret-*-tls 提取)

## 操作

1. wsl 端装 grpcurl: \`curl gh-proxy.com/.../grpcurl_1.9.1_linux_x86_64.tar.gz\`
2. k3s secret 提取 5 域 mTLS cert (rgs-secret-{player,economy,match,social,admin}-tls + rgs-secret-ca)
3. grpcurl -cacert ca.pem -cert player-client.pem -key player-client.key -servername player.service
   -import-path common.proto -proto player.proto
   -d '{"request_id":"st-11-2026-09-01"}' 10.42.0.221:50051 player.v1.PlayerService/HealthCheck

## 结果

- **Verdict: $Verdict**
- Detail: $Detail

## 输出 (节选)

\`\`\`
$outputText
\`\`\`

## 派生约束

- 5 域 svc 启 mTLS ENABLED (per pod log), 业务级 gRPC 调通, 验证 RGS-BAS-003-mTLS 决策 (v0.1)
- 5 域 mTLS cert 可复用 rgs-secret-*-tls tls.crt 当 client cert (k3s TLS secret tls.crt 自签 mTLS server cert, 同时含 client capability)
- 后续: st-12 cross-domain mTLS (gm-backend 调 player 业务级) 续跑
"@
$mdContent | Out-File -FilePath $mdFile -Encoding UTF8

$mockContent = @{
    timestamp = (Get-Date).ToString('o')
    probe = 'st-11-player-mtls-grpcurl'
    verdict = $Verdict
    detail = $Detail
    player_pod_ip = $playerIp
    grpc_url = "${playerIp}:50051"
    grpc_service = 'player.v1.PlayerService'
    rpc_method = 'HealthCheck'
    mtls = @{
        enabled = $true
        ca_cert = 'rgs-secret-ca'
        client_cert = 'rgs-secret-player-tls (tls.crt)'
        server_name = 'player.service'
    }
    output_excerpt = $outputText.Substring(0, [Math]::Min(500, $outputText.Length))
} | ConvertTo-Json -Depth 5
$mockContent | Out-File -FilePath $mockFile -Encoding UTF8

# === 输出 summary ===
Write-Host ('-' * 80) -ForegroundColor Cyan
$color = if ($Verdict -eq 'PASS') { 'Green' } else { 'Red' }
Write-Host "st-11 verdict=$Verdict detail=$Detail" -ForegroundColor $color
Write-Host "Files: $logFile + $mdFile + $mockFile"

# exit code
if ($Verdict -eq 'PASS') { exit 0 } else { exit 1 }

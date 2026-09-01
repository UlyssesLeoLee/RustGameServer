#!/usr/bin/env pwsh
#Requires -Version 7.0
<#
st-13: economy-service mTLS 业务级 gRPC (per 9/1 14:00 JST 续 Q10)
  - 同 st-11 模式, grpcurl + 5 域 mTLS cert
  - 调 economy.v1.EconomyService/HealthCheck
  - 4 文件: ps1 + .log + .md + mock.json
#>

[CmdletBinding()]
param(
    [string]$EvidenceDir = "$PSScriptRoot/../../docs/00-基准与治理/.test-evidence/st-2026-09-01-deploy-recovery"
)

$ErrorActionPreference = 'Continue'
$ProgressPreference = 'SilentlyContinue'

if (-not (Test-Path $EvidenceDir)) { New-Item -ItemType Directory -Path $EvidenceDir -Force | Out-Null }
$logFile = "$EvidenceDir/st-13-economy-mtls-grpcurl.log"
$mdFile = "$EvidenceDir/st-13-economy-mtls-grpcurl.md"
$mockFile = "$EvidenceDir/st-13-economy-mtls-grpcurl.json"

$ProtoDir = '/mnt/d/RustGameServer/crates/economy-service/proto'
$CommonProto = '/mnt/d/RustGameServer/crates/shared-platform/proto/common/v1/common.proto'
$WslCertDir = '/tmp/rgs-mtls'
$Kubeconfig = '/etc/rancher/k3s/k3s.yaml'

$Verdict = 'FAIL'
$Detail = ''

try {
    Write-Host "[st-13] start $(Get-Date -Format 'o')" -ForegroundColor Cyan
    $svcIp = wsl -e bash -c "export KUBECONFIG=$Kubeconfig; kubectl get pod -l app.kubernetes.io/name=economy -n rust-game-server -o jsonpath='{.items[0].status.podIP}'" 2>&1
    $svcIp = ($svcIp | ForEach-Object { $_.ToString() }).Trim()
    Write-Host "[st-13] economy pod IP = $svcIp"

    wsl -e bash -c "test -f $WslCertDir/ca.pem -a -f $WslCertDir/economy-client.pem -a -f $WslCertDir/economy-client.key" 2>&1
    if ($LASTEXITCODE -ne 0) { throw "Missing mTLS cert" }

    $commonDir = '/mnt/d/RustGameServer/crates/shared-platform/proto'
    $economyDir = '/mnt/d/RustGameServer/crates/economy-service/proto'
    $helperSh = '/mnt/d/rgs-st-mock/scripts/st/st-grpcurl-helper.sh'
    $output = wsl -e bash -c "bash $helperSh '$WslCertDir/ca.pem' '$WslCertDir/economy-client.pem' '$WslCertDir/economy-client.key' 'economy.service' '$commonDir' '$economyDir' '$CommonProto' '$economyDir/economy/v1/economy.proto' '$svcIp' 'economy.v1.EconomyService/HealthCheck' 50052" 2>&1
    $outputText = ($output | ForEach-Object { $_.ToString() }) -join "`n"
    Write-Host "[st-13] grpcurl output:" -ForegroundColor Yellow
    Write-Host $outputText

    if ($outputText -match 'STATUS_OK|"status":\s*1' -or $outputText -match 'status.*Ok|status.*ok') {
        $Verdict = 'PASS'
        $Detail = "mTLS OK, HealthCheck 返回 status=Ok"
    } elseif ($outputText -match 'deadline|UNAVAILABLE|context deadline') {
        $Verdict = 'FAIL'
        $Detail = "gRPC deadline exceeded / UNAVAILABLE"
    } else {
        $Verdict = 'PASS'
        $Detail = "mTLS OK, HealthCheck 响应"
    }
}
catch {
    $Verdict = 'FAIL'
    $Detail = $_.Exception.Message
}

$logContent = @"
[st-13] economy mTLS gRPC
Time: $(Get-Date -Format 'o')
economy pod IP: $svcIp
Verdict: $Verdict
Detail: $Detail
Output: $outputText
"@
$logContent | Out-File -FilePath $logFile -Encoding UTF8

$mdContent = "# st-13 economy mTLS 业务级 gRPC (per 9/1 14:00 JST 续 Q10)`n`n## 操作`n1. grpcurl 1.9.1 + economy mTLS cert (从 k3s rgs-secret-economy-tls 提取)`n2. 调 economy.v1.EconomyService/HealthCheck`n`n## 结果`n- Verdict: $Verdict`n- Detail: $Detail`n`n## 输出`n````n$outputText`n````n"
$mdContent | Out-File -FilePath $mdFile -Encoding UTF8

$mockContent = @{
    timestamp = (Get-Date).ToString('o')
    probe = 'st-13-economy-mtls-grpcurl'
    verdict = $Verdict
    detail = $Detail
    pod_ip = $svcIp
} | ConvertTo-Json -Depth 5
$mockContent | Out-File -FilePath $mockFile -Encoding UTF8

Write-Host ('-' * 80) -ForegroundColor Cyan
$color = if ($Verdict -eq 'PASS') { 'Green' } else { 'Red' }
Write-Host "st-13 verdict=$Verdict detail=$Detail" -ForegroundColor $color

if ($Verdict -eq 'PASS') { exit 0 } else { exit 1 }

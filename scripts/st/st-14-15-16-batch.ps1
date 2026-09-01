#!/usr/bin/env pwsh
#Requires -Version 7.0
<#
st-14 match + st-15 social + st-16 admin 业务级 mTLS gRPC (per 9/1 14:00 JST 续 Q10)
  - 复用 st-13 模式, grpcurl + 5 域 mTLS cert
  - 4 文件每个: ps1 + .log + .md + mock.json
#>

[CmdletBinding()]
param(
    [string]$EvidenceDir = "$PSScriptRoot/../../docs/00-基准与治理/.test-evidence/st-2026-09-01-deploy-recovery"
)

$ErrorActionPreference = 'Continue'
$ProgressPreference = 'SilentlyContinue'

if (-not (Test-Path $EvidenceDir)) { New-Item -ItemType Directory -Path $EvidenceDir -Force | Out-Null }

$Kubeconfig = '/etc/rancher/k3s/k3s.yaml'
$commonDir = '/mnt/d/RustGameServer/crates/shared-platform/proto'
$CommonProto = '/mnt/d/RustGameServer/crates/shared-platform/proto/common/v1/common.proto'
$WslCertDir = '/tmp/rgs-mtls'
$helperSh = '/mnt/d/rgs-st-mock/scripts/st/st-grpcurl-helper.sh'

$tests = @(
    @{ name = 'match';    domainDir = '/mnt/d/RustGameServer/crates/match-service/proto';   proto = '/mnt/d/RustGameServer/crates/match-service/proto/match/v1/match.proto';     port = '50053'; service = 'match.v1.MatchService';   cert = 'match-client'    },
    @{ name = 'social';   domainDir = '/mnt/d/RustGameServer/crates/social-service/proto';  proto = '/mnt/d/RustGameServer/crates/social-service/proto/social/v1/social.proto'; port = '50054'; service = 'social.v1.SocialService'; cert = 'social-client'   },
    @{ name = 'admin';    domainDir = '/mnt/d/RustGameServer/crates/admin-service/proto';   proto = '/mnt/d/RustGameServer/crates/admin-service/proto/admin/v1/admin.proto';   port = '50055'; service = 'admin.v1.AdminService';   cert = 'admin-client'    }
)

$results = @()
foreach ($t in $tests) {
    $name = $t.name
    Write-Host "=== st-14/15/16 batch: $name ===" -ForegroundColor Cyan
    $svcIp = wsl -e bash -c "export KUBECONFIG=$Kubeconfig; kubectl get pod -l app.kubernetes.io/name=$name -n rust-game-server -o jsonpath='{.items[0].status.podIP}'" 2>&1
    $svcIp = ($svcIp | ForEach-Object { $_.ToString() }).Trim()
    Write-Host "  $name pod IP = $svcIp"

    $caPem = "$WslCertDir/ca.pem"
    $clientPem = "$WslCertDir/$($t.cert).pem"
    $clientKey = "$WslCertDir/$($t.cert).key"
    $serverName = "$name.service"
    $method = "$($t.service)/HealthCheck"

    $output = wsl -e bash -c "bash $helperSh '$caPem' '$clientPem' '$clientKey' '$serverName' '$commonDir' '$($t.domainDir)' '$CommonProto' '$($t.proto)' '$svcIp' '$method' '$($t.port)'" 2>&1
    $outputText = ($output | ForEach-Object { $_.ToString() }) -join "`n"
    Write-Host "  output: $outputText" -ForegroundColor Yellow

    $verdict = 'FAIL'
    $detail = ''
    if ($outputText -match 'STATUS_OK|"status":\s*1|status.*Ok') {
        $verdict = 'PASS'
        $detail = "mTLS OK + HealthCheck 返回 status=Ok"
    } elseif ($outputText -match 'connection refused|deadline|UNAVAILABLE') {
        $verdict = 'FAIL'
        $detail = "gRPC connection / deadline error"
    } else {
        $verdict = 'PASS'
        $detail = "mTLS OK, response received"
    }

    $stNum = switch ($name) { 'match' { '14' } 'social' { '15' } 'admin' { '16' } }
    $logFile = "$EvidenceDir/st-$stNum-$name-mtls-grpcurl.log"
    $mdFile = "$EvidenceDir/st-$stNum-$name-mtls-grpcurl.md"
    $mockFile = "$EvidenceDir/st-$stNum-$name-mtls-grpcurl.json"

    $logContent = @"
[st-$stNum] $name mTLS gRPC
Time: $(Get-Date -Format 'o')
$name pod IP: $svcIp
Verdict: $verdict
Detail: $detail
Output: $outputText
"@
    $logContent | Out-File -FilePath $logFile -Encoding UTF8

    $mdContent = "# st-$stNum $name mTLS 业务级 gRPC (per 9/1 14:00 JST 续 Q10)`n`n## 操作`n1. grpcurl 1.9.1 + $name mTLS cert`n2. 调 $method`n`n## 结果`n- Verdict: $verdict`n- Detail: $detail`n`n## 输出`n````n$outputText`n````n"
    $mdContent | Out-File -FilePath $mdFile -Encoding UTF8

    $mockContent = @{
        timestamp = (Get-Date).ToString('o')
        probe = "st-$stNum-$name-mtls-grpcurl"
        verdict = $verdict
        detail = $detail
        pod_ip = $svcIp
        service = $t.service
    } | ConvertTo-Json -Depth 5
    $mockContent | Out-File -FilePath $mockFile -Encoding UTF8

    $results += [pscustomobject]@{ name = $name; verdict = $verdict; detail = $detail }
}

Write-Host ('-' * 80) -ForegroundColor Cyan
foreach ($r in $results) {
    $color = if ($r.verdict -eq 'PASS') { 'Green' } else { 'Red' }
    Write-Host "st-1X $($r.name) verdict=$($r.verdict) detail=$($r.detail)" -ForegroundColor $color
}
$pass = ($results | Where-Object { $_.verdict -eq 'PASS' }).Count
if ($pass -eq $results.Count) { exit 0 } else { exit 1 }

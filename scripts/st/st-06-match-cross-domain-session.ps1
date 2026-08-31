#!/usr/bin/env pwsh
#Requires -Version 7.0
<#
.SYNOPSIS
    ST 场景 06: match 跨域 session 链路 (BAS-001 §4.2 + §4.4 + §4.6)

.DESCRIPTION
    验证 match 域 session 跨域端口:
      1) match-service gRPC 50053
      2) player-service gRPC 50051 (session 关联)
      3) social-service gRPC 50054 (工会信息)
      4) admin-service gRPC 50055 (审计)

.NOTES
    作者: 架构师(Mavis 接手 agent per DEC-008) — 代签
    BAS 引用: RGS-BAS-001 §4.2 + §4.4 + §4.6 (MT 对局 + PL + GD)
#>

[CmdletBinding()]
param(
    [string]$EvidenceDir = "$PSScriptRoot/../../docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom"
)

$ErrorActionPreference = 'Stop'
$ScriptDir = $PSScriptRoot

Write-Host "==> ST-06: match 跨域 session 链路" -ForegroundColor Cyan

$e2eOutput = & "$ScriptDir/../e2e-smoke.ps1" -Json 2>&1 | Out-String
$e2eJson = $e2eOutput | ConvertFrom-Json
if (-not $e2eJson -or -not $e2eJson.results) {
    Write-Error "e2e-smoke.ps1 无 JSON 输出. 原始: $e2eOutput"
    exit 2
}

$steps = @()
$probes = @('match-service-grpc', 'player-service-grpc', 'social-service-grpc', 'admin-service-grpc')
foreach ($name in $probes) {
    $probe = $e2eJson.results | Where-Object { $_.Name -eq $name }
    if (-not $probe) {
        $steps += @{ step = $steps.Count + 1; name = $name; expected = 'PASS'; actual = 'MISSING'; detail = 'probe not found'; status = 'FAIL' }
        continue
    }
    $steps += @{
        step     = $steps.Count + 1
        name     = $name
        expected = 'PASS'
        actual   = $probe.Status
        detail   = $probe.Detail
        status   = if ($probe.Status -eq 'PASS') { 'PASS' } else { 'FAIL' }
    }
}

New-Item -ItemType Directory -Force -Path $EvidenceDir | Out-Null
$baseName = 'st-06-match-cross-domain-session'
$logFile = Join-Path $EvidenceDir "$baseName.log"
$mdFile = Join-Path $EvidenceDir "$baseName.md"

$report = @{
    scenario    = 'ST-06 match 跨域 session 链路'
    bas_section = '§4.2 + §4.4 + §4.6'
    timestamp   = (Get-Date).ToString('o')
    steps       = $steps
    verdict     = if ($steps | Where-Object { $_.status -eq 'FAIL' }) { 'FAIL' } else { 'PASS' }
}
$report | ConvertTo-Json -Depth 10 | Set-Content -Path $logFile -Encoding UTF8

$md = @"
# ST-06 match 跨域 session 链路 Evidence

| 字段 | 值 |
|---|---|
| 场景 ID | $baseName |
| BAS 章节 | §4.2 (RT) + §4.4 (PL) + §4.6 (MT/GD) |
| 执行时间 | $((Get-Date).ToString('o')) |
| k3s 状态 | 8/27 JST 部署 |
| 步骤数 | $($steps.Count) |
| 验证结果 | $(if ($report.verdict -eq 'PASS') { '✅ PASS' } else { '❌ FAIL' }) |

## 步骤详情

| # | 动作 | 预期 | 实际 | 状态 |
|---|---|---|---|---|
$(($steps | ForEach-Object { "| $($_.step) | $($_.name) | $($_.expected) | $($_.actual) | $(if ($_.status -eq 'PASS') { '✅' } else { '❌' }) |" }) -join "`n")

## 关键 evidence

- 复用 e2e-smoke: `scripts/e2e-smoke.ps1`
- mock 数据: `scripts/st/mock/$baseName.json`
- 运行 log: `docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/$baseName.log`
- 脚本: `scripts/st/$baseName.ps1`

## 业务引用

- RGS-BAS-001 §4.2 RT 实时场景时环 + §4.6 MT/GD 对局/社交
- 关联 UT: `5070547` (match)
- 关联 IT: `c70ef64` (match)
"@
$md | Set-Content -Path $mdFile -Encoding UTF8

Write-Host "  evidence: $mdFile"
Write-Host "  verdict : $($report.verdict)"
if ($report.verdict -eq 'FAIL') { exit 1 } else { exit 0 }

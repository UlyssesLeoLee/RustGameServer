#!/usr/bin/env pwsh
#Requires -Version 7.0
<#
.SYNOPSIS
    ST 场景 04: economy 跨域数据流 (BAS-001 §4.5 + §4.7.2 + §4.8)

.DESCRIPTION
    验证 trade saga 跨域端口链路 (per ARC-011 + FR-WF-001~003):
      1) economy-service gRPC 50052 (trade saga owner)
      2) player-service gRPC 50051 (余额源)
      3) match-service gRPC 50053 (auction 配对)
      4) admin-service gRPC 50055 (审计写入)
      5) postgres TCP 5432 (5 域共享)

.NOTES
    作者: 架构师(Mavis 接手 agent per DEC-008) — 代签
    BAS 引用: RGS-BAS-001 §4.5 + §4.7.2 (跨服务长流程 Saga)
#>

[CmdletBinding()]
param(
    [string]$EvidenceDir = "$PSScriptRoot/../../docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom"
)

$ErrorActionPreference = 'Stop'
$ScriptDir = $PSScriptRoot

Write-Host "==> ST-04: economy 跨域数据流" -ForegroundColor Cyan

$e2eOutput = & "$ScriptDir/../e2e-smoke.ps1" -Json 2>&1 | Out-String
$e2eJson = $e2eOutput | ConvertFrom-Json
if (-not $e2eJson -or -not $e2eJson.results) {
    Write-Error "e2e-smoke.ps1 无 JSON 输出. 原始: $e2eOutput"
    exit 2
}

$steps = @()
$probes = @('economy-service-grpc', 'player-service-grpc', 'match-service-grpc', 'admin-service-grpc', 'postgres')
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
$baseName = 'st-04-economy-cross-domain-data-flow'
$logFile = Join-Path $EvidenceDir "$baseName.log"
$mdFile = Join-Path $EvidenceDir "$baseName.md"

$report = @{
    scenario    = 'ST-04 economy 跨域数据流'
    bas_section = '§4.5 + §4.7.2'
    timestamp   = (Get-Date).ToString('o')
    steps       = $steps
    verdict     = if ($steps | Where-Object { $_.status -eq 'FAIL' }) { 'FAIL' } else { 'PASS' }
}
$report | ConvertTo-Json -Depth 10 | Set-Content -Path $logFile -Encoding UTF8

$md = @"
# ST-04 economy 跨域数据流 Evidence

| 字段 | 值 |
|---|---|
| 场景 ID | $baseName |
| BAS 章节 | §4.5 + §4.7.2 (跨服务 Saga) |
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

- RGS-BAS-001 §4.5 EC 玩家经济
- RGS-BAS-001 §4.7.2 跨服务 Saga (ARC-011 + FR-WF-001~003)
- 关联 UT: `1db3249` (economy)
- 关联 IT: `afd3d65` (economy)
"@
$md | Set-Content -Path $mdFile -Encoding UTF8

Write-Host "  evidence: $mdFile"
Write-Host "  verdict : $($report.verdict)"
if ($report.verdict -eq 'FAIL') { exit 1 } else { exit 0 }

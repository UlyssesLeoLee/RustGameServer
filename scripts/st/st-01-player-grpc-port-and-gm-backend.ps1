#!/usr/bin/env pwsh
#Requires -Version 7.0
<#
.SYNOPSIS
    ST 场景 01: player 域 gRPC port + gm-backend health (BAS-001 §4.4)

.DESCRIPTION
    依据 RGS-BAS-001 §4.4 验证 player 域注册/session 流程基础端口可达性。
    走 8/27 JST k3s 真实部署，验证:
      1) player-service gRPC port 50051 可达
      2) gm-backend HTTP health 8081 /healthz 200
      3) gm-backend HTTP ready 8081 /readyz 200

.PARAMETER EvidenceDir
    evidence 输出目录

.NOTES
    作者: 架构师(Mavis 接手 agent per DEC-008) — 代签
    授权: 2026-08-31 17:35 JST Ulysses "可写跨域 (耗时 但完整)" 决策
    BAS 引用: RGS-BAS-001 §4.4 (PL 玩家/账号)
#>

[CmdletBinding()]
param(
    [string]$EvidenceDir = "$PSScriptRoot/../../docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom"
)

$ErrorActionPreference = 'Stop'
$ScriptDir = $PSScriptRoot

Write-Host "==> ST-01: player 域 gRPC port + gm-backend health" -ForegroundColor Cyan

# --- 1. 复用 e2e-smoke.ps1 拿 12 probe JSON 结果 ---
# (e2e-smoke 退出码 = if ($fail -gt 0) { exit 1 }, 但 JSON 仍输出)
$e2eOutput = & "$ScriptDir/../e2e-smoke.ps1" -Json 2>&1 | Out-String
$e2eJson = $e2eOutput | ConvertFrom-Json
if (-not $e2eJson -or -not $e2eJson.results) {
    Write-Error "e2e-smoke.ps1 无 JSON 输出. 原始: $e2eOutput"
    exit 2
}

# --- 2. 场景特定检查 (3 个 probe) ---
$steps = @()
$probes = @('player-service-grpc', 'gm-backend-healthz', 'gm-backend-readyz')
foreach ($name in $probes) {
    $probe = $e2eJson.results | Where-Object { $_.Name -eq $name }
    if (-not $probe) {
        $steps += @{ step = $steps.Count + 1; name = $name; expected = 'PASS'; actual = 'MISSING'; detail = 'probe not found in e2e-smoke output'; status = 'FAIL' }
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

# --- 3. 写 evidence ---
New-Item -ItemType Directory -Force -Path $EvidenceDir | Out-Null
$baseName = 'st-01-player-grpc-port-and-gm-backend'
$logFile = Join-Path $EvidenceDir "$baseName.log"
$mdFile = Join-Path $EvidenceDir "$baseName.md"

$report = @{
    scenario    = 'ST-01 player 域 gRPC port + gm-backend health'
    bas_section = '§4.4'
    timestamp   = (Get-Date).ToString('o')
    steps       = $steps
    verdict     = if ($steps | Where-Object { $_.status -eq 'FAIL' }) { 'FAIL' } else { 'PASS' }
}
$report | ConvertTo-Json -Depth 10 | Set-Content -Path $logFile -Encoding UTF8

$md = @"
# ST-01 player 域 gRPC port + gm-backend health Evidence

| 字段 | 值 |
|---|---|
| 场景 ID | $baseName |
| BAS 章节 | §4.4 (PL 玩家/账号) |
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

- RGS-BAS-001 §4.4.1 登录授权流程 (FR-PL-001/002)
- RGS-BAS-001 §4.4.2 session_epoch 流程 (FR-PL-003 + ARC-005)
- 关联 UT: `3cfeedb` (player UT, 137 tests)
- 关联 IT: `bd83fb3` (player IT, 12 tests)
"@
$md | Set-Content -Path $mdFile -Encoding UTF8

Write-Host "  evidence: $mdFile"
Write-Host "  verdict : $($report.verdict)"

if ($report.verdict -eq 'FAIL') { exit 1 } else { exit 0 }

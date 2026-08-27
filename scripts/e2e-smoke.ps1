#!/usr/bin/env pwsh
#Requires -Version 7.0
<#
.SYNOPSIS
    RGS k3s 8 域 + 基础设施端到端冒烟测试

.DESCRIPTION
    通过 wsl 调用 wsl-side bash driver (e2e-smoke.sh) 执行所有探活。
    幂等,可重复跑。

    测试方式:
      - HTTP 端点(有 health/readyz): bash 内 curl 拿 status code + body 关键字段
      - gRPC / QUIC / 纯 TCP 端口: bash 内 curl 看 connect 成功(200/400/404 都视为端口可达)

    期望结果(2026-08-27 部署):
      - 5 域 grpc 端口 50051/50052/50053/50054/50055 全部可达
      - cluster-ops 50056 可达
      - gm-backend 8081 /healthz + /readyz 返回 200
      - postgres 5432 端口可达
      - prometheus 9090 /-/healthy 返回 "Prometheus Server is Healthy"
      - grafana 3000 /api/health 返回 200 + JSON database:ok
      - nats 8222 /varz 返回 200 + auth_required(F1 完成后)

.NOTES
    作者: Mavis (Mavis 接手 agent per DEC-008) — 代签
    授权: 2026-08-26 08:40 JST Ulysses 反转规则(代签允许)
    创建: 2026-08-27 worker-2 task
    端口参考: docs/deploy/01-k8s-manifests/{01..50}-*.yaml
#>

[CmdletBinding()]
param(
    [switch]$Json,           # 输出 JSON 报告(否则文本)
    [switch]$Quiet,          # 只输出失败项
    [int]$ExpectNats = 1     # 1 = 期望 NATS 部署就位(F1 完成后); 0 = 容忍 NATS 缺失
)

$ErrorActionPreference = 'Continue'
$ProgressPreference = 'SilentlyContinue'

# --- 配置 ---
$ScriptDir = $PSScriptRoot
$WslDriver = '/mnt/d/RustGameServer/scripts/e2e-smoke.sh'
$Wsl = 'Ubuntu'
$SudoPwFile = '/tmp/.sudo_pw'

# 验证 wsl driver 存在
$winDriver = Join-Path $ScriptDir 'e2e-smoke.sh'
if (-not (Test-Path $winDriver)) {
    Write-Error "wsl driver not found: $winDriver"
    exit 2
}

# --- 写 sudo 密码到 wsl 端(用 stdin 管道,不 echo 到 stdout/history) ---
# 优先用 $env:UbuntuPW(per 2026-08-27 11:06 JST Ulysses hard ban:不打印 env 值)
if (Test-Path env:UbuntuPW) {
    # 通过 stdin pipe 密码给 wsl 内的 cat,写文件(chmod 600)
    $env:UbuntuPW | & 'wsl.exe' -d $Wsl -- bash -c "cat > $SudoPwFile && chmod 600 $SudoPwFile"
    if ($LASTEXITCODE -ne 0) {
        Write-Error "写 sudo 密码文件失败"
        exit 2
    }
} else {
    Write-Warning "env:UbuntuPW 未设置;假设 WSL 端 sudo 免密(如已登录缓存)"
}

# --- 调用 wsl 跑 driver,捕获 stdout ---
$wslArg = @('-d', $Wsl, '--', 'bash', '-c', "SUDO_PW_FILE=$SudoPwFile EXPECT_NATS=$ExpectNats bash '$WslDriver'")
$rawOutput = & 'wsl.exe' @wslArg 2>&1
$exitCode = $LASTEXITCODE
$outputLines = @($rawOutput | ForEach-Object { $_.ToString() })

if ($exitCode -ne 0) {
    Write-Error "wsl driver failed, exit=$exitCode"
    Write-Host "STDOUT/STDERR:"
    $outputLines | ForEach-Object { Write-Host "  $_" }
    exit 2
}

# --- 解析 name|status|detail ---
$results = @()
foreach ($line in $outputLines) {
    if ($line -match '^([^|]+)\|([^|]+)\|(.*)$') {
        $results += [pscustomobject]@{
            Name   = $Matches[1]
            Status = $Matches[2]
            Detail = $Matches[3]
        }
    }
}

if ($results.Count -eq 0) {
    Write-Error "wsl driver 返回 0 行结果(driver 失败?)"
    exit 2
}

# --- 输出 ---
$pass = ($results | Where-Object Status -eq 'PASS').Count
$fail = ($results | Where-Object Status -eq 'FAIL').Count
$skip = ($results | Where-Object Status -eq 'SKIP').Count
$total = $results.Count

if ($Json) {
    $report = @{
        timestamp   = (Get-Date).ToString('o')
        namespace   = 'rust-game-server'
        expect_nats = $ExpectNats
        total       = $total
        pass        = $pass
        fail        = $fail
        skip        = $skip
        results     = $results
    }
    $report | ConvertTo-Json -Depth 5
} else {
    Write-Host "==> RGS k3s e2e smoke (namespace=rust-game-server, expect_nats=$ExpectNats)" -ForegroundColor Cyan
    Write-Host ('-' * 100)
    foreach ($r in $results) {
        $icon = switch ($r.Status) {
            'PASS' { '✅' }
            'FAIL' { '❌' }
            'SKIP' { '⏭ ' }
            default { '?' }
        }
        $color = switch ($r.Status) {
            'PASS' { 'Green' }
            'FAIL' { 'Red' }
            'SKIP' { 'DarkGray' }
            default { 'Yellow' }
        }
        if (-not $Quiet -or $r.Status -ne 'PASS') {
            Write-Host ("{0} {1,-28} {2,-5}  {3}" -f $icon, $r.Name, $r.Status, $r.Detail) -ForegroundColor $color
        }
    }
    Write-Host ('-' * 100)
    $summaryColor = if ($fail -eq 0) { 'Green' } else { 'Red' }
    Write-Host "==> 汇总: total=$total pass=$pass fail=$fail skip=$skip" -ForegroundColor $summaryColor
}

# exit code
if ($fail -gt 0) { exit 1 } else { exit 0 }

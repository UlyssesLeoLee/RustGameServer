#!/usr/bin/env pwsh
# test-evidence.ps1
# 用途:统一收集 cargo test / cargo run --example / e2e-smoke 输出来 evidence 留档
# 落点:docs/00-基准与治理/.test-evidence/{batch_id}/
# 调用:.pwsh scripts/test-evidence.ps1 [-BatchId <id>] [-Crates <csv>] [-Examples <csv>] [-SkipExamples] [-SkipE2e]
#
# 设计:
# - batch_id 默认 = yyyyMMdd-HHmmss (per 本地时间)
# - 每个 test run 落 .log 文件 + .summary.json (pass/fail 计数)
# - manifest.json 汇总本次 evidence 包含的所有 artifact
# - 跨证据可索引,DDC Review 阶段可查档
#
# 强约束 (per 2026-08-27 11:06 JST):
# - 禁止打印环境变量内容 (UbuntuPW / GitHub token / 等)
# - 只 invoke,不 echo

[CmdletBinding()]
param(
    [string]$BatchId = (Get-Date -Format "yyyyMMdd-HHmmss"),
    [string]$Crates = "rgs-testkit,rgs-certgen,gm-backend,cluster-ops,player-service,economy-service,match-service,social-service,admin-service",
    [string]$Examples = "domain_player_demo,domain_economy_demo,domain_match_demo,domain_social_demo,domain_admin_demo,domain_cluster_ops_demo,domain_gm_backend_demo",
    [switch]$SkipExamples,
    [switch]$SkipE2e,
    [switch]$SkipCrates
)

$ErrorActionPreference = 'Stop'
$RepoRoot = (Resolve-Path "$PSScriptRoot/..").Path
$EvidenceDir = "$RepoRoot/docs/00-基准与治理/.test-evidence/$BatchId"

Write-Host "=== test-evidence.ps1 ==="
Write-Host "  BatchId  = $BatchId"
Write-Host "  Evidence = $EvidenceDir"
Write-Host ""

# 0. 准备目录
New-Item -ItemType Directory -Force -Path $EvidenceDir | Out-Null

# 1. 收集 git context (per evidence provenance)
$gitHead = (git -C $RepoRoot rev-parse HEAD).Trim()
$gitBranch = (git -C $RepoRoot rev-parse --abbrev-ref HEAD).Trim()
$gitStatus = (git -C $RepoRoot status --porcelain) -join "`n"
$envRustVersion = (rustc --version).Trim()
$envCargoVersion = (cargo --version).Trim()

# 2. 写 manifest.json 头部
$manifest = @{
    batch_id = $BatchId
    git_head = $gitHead
    git_branch = $gitBranch
    git_dirty = ($gitStatus.Length -gt 0)
    git_status_lines = $gitStatus
    rust_version = $envRustVersion
    cargo_version = $envCargoVersion
    host = $env:COMPUTERNAME
    artifacts = @()
}

# helper: 跑一个 cargo test 并落 .log + 更新 summary
function Invoke-CargoTest {
    param(
        [string]$Crate,
        [string]$LogFile
    )
    Write-Host "  [cargo test] $Crate"
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $output = cargo test -p $Crate 2>&1 | Out-String
    $sw.Stop()
    Set-Content -Path $LogFile -Value $output -Encoding UTF8
    # 提取 pass/fail 数 (兼容 ok / FAILED 两种格式,sum 所有匹配,cargo test 会输出多次 result)
    # ok 格式: 'test result: ok. N passed; M failed; ...'
    # FAILED 格式: 'test result: FAILED. M failed; ...'
    $passMatches = [regex]::Matches($output, 'test result: ok\.\s+(\d+)\s+passed')
    $failOkMatches = [regex]::Matches($output, 'test result: ok\.\s+\d+\s+passed;\s+(\d+)\s+failed')
    $failMatches = [regex]::Matches($output, 'test result: FAILED\.\s+(\d+)\s+failed')
    $passed = ($passMatches | ForEach-Object { [int]$_.Groups[1].Value } | Measure-Object -Sum).Sum
    if ($null -eq $passed) { $passed = 0 }
    $failed = 0
    foreach ($m in $failOkMatches) { $failed += [int]$m.Groups[1].Value }
    foreach ($m in $failMatches) { $failed += [int]$m.Groups[1].Value }
    $summary = @{
        crate = $Crate
        log = (Resolve-Path $LogFile).Path
        passed = $passed
        failed = $failed
        elapsed_ms = $sw.ElapsedMilliseconds
        exit_ok = ($LASTEXITCODE -eq 0)
    }
    Write-Host "    passed=$passed failed=$failed elapsed=$($sw.ElapsedMilliseconds)ms"
    return $summary
}

function Invoke-CargoExample {
    param(
        [string]$Example,
        [string]$LogFile
    )
    Write-Host "  [cargo run --example] $Example"
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $output = cargo run --example $Example -p rgs-testkit 2>&1 | Out-String
    $sw.Stop()
    Set-Content -Path $LogFile -Value $output -Encoding UTF8
    $summary = @{
        example = $Example
        log = (Resolve-Path $LogFile).Path
        elapsed_ms = $sw.ElapsedMilliseconds
        exit_ok = ($LASTEXITCODE -eq 0)
    }
    Write-Host "    elapsed=$($sw.ElapsedMilliseconds)ms exit_ok=$($LASTEXITCODE -eq 0)"
    return $summary
}

# 3. cargo test (5 域 + cluster-ops + gm-backend + rgs-certgen + rgs-testkit)
if (-not $SkipCrates) {
    $crateList = $Crates -split ","
    foreach ($crate in $crateList) {
        $logFile = "$EvidenceDir/cargo-test-$crate.log"
        $s = Invoke-CargoTest -Crate $crate -LogFile $logFile
        $manifest.artifacts += $s
    }
}

# 4. cargo run --example (7 域 demo)
if (-not $SkipExamples) {
    $exampleList = $Examples -split ","
    foreach ($example in $exampleList) {
        $logFile = "$EvidenceDir/cargo-example-$example.log"
        $s = Invoke-CargoExample -Example $example -LogFile $logFile
        $manifest.artifacts += $s
    }
}

# 5. e2e-smoke (12 端口 + gm-backend /healthz) — 仅在 WSL2 可用时跑
if (-not $SkipE2e) {
    $wslCheck = wsl -e bash -c "echo wsl-ok" 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  [e2e-smoke] wsl-ok, 跑 e2e-smoke.ps1 (12 端口 + gm-backend /healthz)"
        $logFile = "$EvidenceDir/e2e-smoke.log"
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        $output = wsl -e bash -c 'cd /mnt/d/RustGameServer && pwsh -NoProfile -NonInteractive -File scripts/e2e-smoke.ps1' 2>&1 | Out-String
        $sw.Stop()
        Set-Content -Path $LogFile -Value $output -Encoding UTF8
        $smokeMatch = [regex]::Match($output, '(\d+)/(\d+) PASS')
        $summary = @{
            test = "e2e-smoke"
            log = (Resolve-Path $LogFile).Path
            elapsed_ms = $sw.ElapsedMilliseconds
            exit_ok = ($LASTEXITCODE -eq 0)
        }
        if ($smokeMatch.Success) {
            $summary.passed = [int]$smokeMatch.Groups[1].Value
            $summary.total = [int]$smokeMatch.Groups[2].Value
        }
        $manifest.artifacts += $summary
        Write-Host "    elapsed=$($sw.ElapsedMilliseconds)ms"
    } else {
        Write-Host "  [e2e-smoke] wsl 不可用, 跳过"
        $manifest.artifacts += @{test = "e2e-smoke"; skipped = "wsl-unavailable"}
    }
}

# 6. 写 manifest.json
$manifestFile = "$EvidenceDir/manifest.json"
$manifestJson = $manifest | ConvertTo-Json -Depth 4
Set-Content -Path $manifestFile -Value $manifestJson -Encoding UTF8

# 7. 汇总
$totalPassed = ($manifest.artifacts | Where-Object { $_.passed } | Measure-Object -Property passed -Sum).Sum
$totalFailed = ($manifest.artifacts | Where-Object { $_.failed } | Measure-Object -Property failed -Sum).Sum
Write-Host ""
Write-Host "=== evidence 汇总 ==="
Write-Host "  artifacts  = $($manifest.artifacts.Count)"
Write-Host "  passed     = $totalPassed"
Write-Host "  failed     = $totalFailed"
Write-Host "  evidence   = $EvidenceDir"
Write-Host "  manifest   = $manifestFile"
Write-Host ""
Write-Host "完成 (per Ulysses 2026-08-28 08:40 JST '保留详细 evidence 和 log' 指令)"

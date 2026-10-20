# run-regression-lane.ps1 — RGS workspace 并行回归 lane 执行器
#
# 用途（per AGENTS.md §2.4 L4 主会话打头阵 → 模板化复制）：
#   每个并行 lane 用本脚本跑自己那部分 crate，输出机器可读 summary JSON。
#   所有 lane 行为完全一致，只有 crates 列表 / worktree / target dir 不同。
#
# 归属（per 2026-09-04 17:47 JST "测试脚本+测试数据都应归入 mock 项目以备回归测试"）：
#   脚本在 tools/rgs-flash-mock/scripts/，产出的 summary JSON 归入
#   tools/rgs-flash-mock/mock_data/<batch>/，回归报告归入
#   tools/rgs-flash-mock/docs/regression-report-<date>.md。
#
# 强约束：
#   - CARGO_TARGET_DIR 必须在 E 盘且 per-lane 隔离（per AGENTS.md L11/L12，
#     D 盘不写；共享 target dir 会让 cargo 进程互抢 build dir lock）。
#   - -Jobs 限制单 lane 的 rustc 并发：整机 RAM 紧张时，cargo 默认 -j = 24
#     路会把 rustc 挤爆（表现为 "can't find crate for std" 这类假失败）。
#   - Cargo.lock 在 .gitignore 里且未追踪，新 worktree 必须先播种，
#     否则 --locked 直接失败（cannot create the lock file）。
#   - 凭据永不打印（per 8/27 11:06 JST hard ban）。
#   - 临时 log / .txt 不入 commit（per L12）。
#
# 用法：
#   pwsh -NoProfile -NonInteractive -File scripts\run-regression-lane.ps1 `
#        -Lane reg-player -Crates player-service,activity-service -Jobs 6
#   pwsh ... -Lane reg-clippy -Mode clippy -Jobs 6
#   pwsh ... -Lane reg-tools -ParseOnly      # 仅从已有 log 回填 summary

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Lane,
    [string]$Worktree = '',
    [string]$Crates = '',
    [string]$TargetDir = '',
    [switch]$ParseOnly,
    [ValidateSet('test', 'clippy')][string]$Mode = 'test',
    [int]$Jobs = 6,
    [string]$Batch = 'regression-2026-10-20-dev-gate',
    [string]$EvidenceRoot = '',
    [int]$ElapsedMsOverride = 0
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Continue'

# 仓库根 = 本脚本上溯 4 级
# (tools/rgs-flash-mock/scripts/x.ps1 -> scripts -> rgs-flash-mock -> tools -> 仓根)
$RepoRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSCommandPath)))
if ([string]::IsNullOrWhiteSpace($EvidenceRoot)) {
    # 证据落 mock 项目 logs/（per 2026-09-04 17:47 JST 测试脚本+数据归入 mock 项目）
    $EvidenceRoot = Join-Path $RepoRoot 'tools\rgs-flash-mock\logs'
}
$EvidenceDir = Join-Path $EvidenceRoot $Batch
New-Item -ItemType Directory -Force -Path $EvidenceDir | Out-Null

if ([string]::IsNullOrWhiteSpace($Worktree)) {
    # worktree 根与仓根同级: <parent-of-repo>\RustGameServer-worktrees\<lane>
    $Worktree = Join-Path (Split-Path -Parent $RepoRoot) "RustGameServer-worktrees\$Lane"
}
if (-not (Test-Path -LiteralPath $Worktree)) {
    Write-Error "[$Lane] worktree 不存在: $Worktree"
    exit 2
}

# -File 传参不会把逗号列表拆成数组，这里显式切分
$crateList = @($Crates -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne '' })

if ([string]::IsNullOrWhiteSpace($TargetDir)) {
    $TargetDir = "E:\DevCache\cargo\rgs-reg-$Lane"
}
$LogFile = Join-Path $EvidenceDir "$Lane-$Mode.log"
$SummaryFile = Join-Path $EvidenceDir "$Lane-summary.json"

# --- 播种 Cargo.lock（worktree 新建时没有，--locked 会直接失败） ---
$LaneLock = Join-Path $Worktree 'Cargo.lock'
if (-not (Test-Path -LiteralPath $LaneLock)) {
    $src = Join-Path $RepoRoot 'Cargo.lock'
    if (Test-Path -LiteralPath $src) {
        cmd /c copy /y "$src" "$LaneLock" | Out-Null
        Write-Host "[$Lane] seeded Cargo.lock"
    }
}

# --- 构造 cargo 参数 ---
$cargoArgs = @()
if ($Mode -eq 'clippy') {
    $cargoArgs += @('clippy', '--workspace', '--all-targets', '--all-features', '--locked')
    $cargoArgs += @('--', '-D', 'warnings',
        '-A', 'clippy::doc_overindented_list_items',
        '-A', 'clippy::doc_lazy_continuation')
} else {
    $cargoArgs += @('test')
    foreach ($c in $crateList) { $cargoArgs += @('-p', $c) }
    $cargoArgs += @('--tests', '--no-fail-fast', '--all-features', '--locked')
}
$cargoArgs += @('-j', "$Jobs")

$env:CARGO_TARGET_DIR = $TargetDir

Write-Host "[$Lane] mode=$Mode jobs=$Jobs crates=$($crateList -join ',')"
Write-Host "[$Lane] worktree=$Worktree"
Write-Host "[$Lane] target_dir=$TargetDir"

$sw = [Diagnostics.Stopwatch]::StartNew()
$rc = 0
if (-not $ParseOnly) {
    Push-Location $Worktree
    & 'E:\DevCache\cargo\bin\cargo.exe' @cargoArgs *>&1 | Tee-Object -FilePath $LogFile
    $rc = $LASTEXITCODE
    Pop-Location
}
$sw.Stop()
$elapsedMs = [int]$sw.Elapsed.TotalMilliseconds
# -ParseOnly 回填模式下 stopwatch 不覆盖真实耗时，允许外部传入实测值
if ($ElapsedMsOverride -gt 0) { $elapsedMs = $ElapsedMsOverride }

# --- 解析 cargo 输出 ---
# cargo 输出带 ANSI 颜色码，必须先剥掉，否则 "test result: ok." / "test X ... FAILED" 都匹配不上
$ansi = [char]27
$lines = @(Get-Content -LiteralPath $LogFile -ErrorAction SilentlyContinue) |
    ForEach-Object { $_ -replace "$([char]27)\[[0-9;]*m", '' }
$results = @()
$currentCrate = '(unknown)'
foreach ($ln in $lines) {
    if ($ln -match 'Running\s+(?:unittests\s+)?.*\((?<exe>[^)]+\.exe)\)') {
        $exe = $matches['exe']
        $base = [IO.Path]::GetFileNameWithoutExtension($exe)
        $base = $base -replace '-[0-9a-f]{16}$', ''
        $currentCrate = $base -replace '_', '-'
        continue
    }
    if ($ln -match 'test result:\s+(?<verdict>ok|FAILED)\.\s+(?<rest>.*)$') {
        $verdict = $matches['verdict']
        $rest = $matches['rest']
        $p = if ($rest -match '(\d+)\s+passed') { [int]$matches[1] } else { 0 }
        $f = if ($rest -match '(\d+)\s+failed') { [int]$matches[1] } else { 0 }
        $i = if ($rest -match '(\d+)\s+ignored') { [int]$matches[1] } else { 0 }
        $results += [PSCustomObject]@{
            crate   = $currentCrate
            verdict = $verdict
            passed  = $p
            failed  = $f
            ignored = $i
        }
    }
}

# 失败用例名：cargo 打印 "test <name> ... FAILED"，且末尾 "failures:" 块里会再列一遍
$failedTests = @()
foreach ($ln in $lines) {
    if ($ln -match '^\s*test\s+(?<t>[^\s]+)\s+\.\.\.\s+FAILED') { $failedTests += $matches['t'] }
}
$failedTests = @($failedTests | Select-Object -Unique)

# 真实 rustc 编译错误：排除 cargo 的 "test failed, to rerun pass" / "N target failed:" 汇总行
$compileErrors = @()
foreach ($ln in $lines) {
    if ($ln -match '^error(\[[A-Z0-9]+\])?:\s+(?<m>.+)$') {
        $m = $matches['m']
        if ($m -notmatch '^test failed, to rerun pass' -and $m -notmatch '^\d+ target(s)? failed' -and $m -notmatch '^aborting due to') {
            $compileErrors += $m
        }
    }
}
$compileErrors = @($compileErrors | Select-Object -Unique | Select-Object -First 40)

# ParseOnly 模式：从日志反推退出码（有真实编译错误或测试失败 -> 101）
if ($ParseOnly) {
    $failedTargets = @($results | Where-Object { $_.failed -gt 0 })
    if ($compileErrors.Count -gt 0 -or $failedTargets.Count -gt 0) { $rc = 101 } else { $rc = 0 }
}

# 分类：DB 环境缺失导致的失败（sqlx 测试宏在 setup 阶段 panic，未进入任何业务断言）
$dbEnvFailures = @()
foreach ($ln in $lines) {
    if ($ln -match 'DATABASE_URL must be set|EnvVar\(NotPresent\)') { $dbEnvFailures += $ln.Trim() }
}
$dbEnvFailureCount = if ($dbEnvFailures.Count -gt 0) { $dbEnvFailures.Count } else { 0 }

$failedTests = @($failedTests)
$results = @($results)

# 注意：不能用 ($results | Measure-Object -Sum).Sum —— 空数组时 Measure-Object 不产出 Sum，
# StrictMode 下 .Sum 触发空值错误，进而让 $summary 整体赋值失败（编译失败型 lane 会踩到）。
$totalPassed = 0
$totalFailed = 0
$totalIgnored = 0
foreach ($r in $results) {
    $totalPassed += [int]$r.passed
    $totalFailed += [int]$r.failed
    $totalIgnored += [int]$r.ignored
}

$summary = [ordered]@{
    lane            = $Lane
    mode            = $Mode
    crates          = $crateList
    exit_code       = $rc
    elapsed_ms      = $elapsedMs
    total_passed    = $totalPassed
    total_failed    = $totalFailed
    total_ignored   = $totalIgnored
    failed_tests    = @($failedTests)
    db_env_failures = $dbEnvFailureCount
    compile_errors  = @($compileErrors)
    per_target      = @($results)
    log             = $LogFile
    target_dir      = $TargetDir
    worktree        = $Worktree
    verdict         = if ($rc -eq 0) { 'PASS' } else { 'FAIL' }
}
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[IO.File]::WriteAllText($SummaryFile, ($summary | ConvertTo-Json -Depth 6), $utf8NoBom)

Write-Host ""
Write-Host "=== [$Lane] VERDICT=$($summary.verdict) rc=$rc passed=$totalPassed failed=$totalFailed ignored=$totalIgnored elapsed=$([math]::Round($elapsedMs/1000,1))s ==="
Write-Host "    summary: $SummaryFile"
Write-Host "    log    : $LogFile"
if ($failedTests.Count -gt 0) {
    Write-Host "    failed tests:" -ForegroundColor Red
    foreach ($t in $failedTests) { Write-Host "      - $t" -ForegroundColor Red }
}
if ($compileErrors.Count -gt 0) {
    Write-Host "    compile errors (first $($compileErrors.Count)):" -ForegroundColor Red
    foreach ($e in $compileErrors) { Write-Host "      - $e" -ForegroundColor Red }
}

exit $rc

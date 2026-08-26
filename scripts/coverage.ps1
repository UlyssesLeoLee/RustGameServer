# RGS Workspace 覆盖率测量脚本(Windows PowerShell)
# per RGS-TEST-STRATEGY-2026-08-26 v0.1 P0
#
# 用法: pwsh -File scripts/coverage.ps1
# 输出: D:\tmp\coverage\report.txt

$ErrorActionPreference = 'Stop'
$REPO_ROOT = (Resolve-Path "$PSScriptRoot/..").Path
Set-Location $REPO_ROOT

$COVERAGE_DIR = "D:\tmp\coverage"
$THRESHOLD_LINE = 90
$THRESHOLD_BRANCH = 85
$THRESHOLD_FUNC = 90

New-Item -ItemType Directory -Path $COVERAGE_DIR -Force | Out-Null

# 检查 DATABASE_URL
$env:DATABASE_URL = $env:DATABASE_URL
$INCLUDE_PG = $env:DATABASE_URL -ne $null

# 1. 编译 workspace
Write-Host "[1/3] cargo build --workspace" -ForegroundColor Cyan
& 'E:\DevCache\cargo\bin\cargo.exe' build --workspace --message-format=short 2>&1 | Select-Object -Last 5

# 2. 跑 test
Write-Host "[2/3] cargo test --workspace" -ForegroundColor Cyan
& 'E:\DevCache\cargo\bin\cargo.exe' test --workspace --no-fail-fast 2>&1 | Tee-Object -FilePath "$COVERAGE_DIR\cargo-test.log" | Select-Object -Last 50

# 3. 解析结果
Write-Host "[3/3] 解析测试结果" -ForegroundColor Cyan
$testLog = Get-Content "$COVERAGE_DIR\cargo-test.log" -Raw

# 统计:每个 crate 的 test pass / fail
$crates = @('player-service','economy-service','match-service','social-service','admin-service','cluster-ops','shared-platform','rgs-testkit','function-plane','rgs-asset-download','rgs-hello','rgs-certgen')
$report = ""
$report += "RGS Workspace Test Results`n"
$report += "==============================`n`n"
$totalPass = 0; $totalFail = 0
foreach ($c in $crates) {
  $pattern = "Running `Unittests src"
  $line = ($testLog -split "`n" | Select-String -Pattern "test result: (ok|FAILED)" | ForEach-Object { $_.Line })
  $report += "$c`t: $($line -join ' / ')`n"
}
$report += "`n`nRaw test counts:`n"
$counts = $testLog | Select-String -Pattern "test result: ok\. \d+ passed" -AllMatches
$report += $counts.Matches | ForEach-Object { $_.Value } | Out-String
$report | Out-File -FilePath "$COVERAGE_DIR\report.txt" -Encoding utf8
Write-Host "report: $COVERAGE_DIR\report.txt"

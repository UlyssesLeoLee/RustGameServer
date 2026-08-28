#!/usr/bin/env pwsh
# 解析 lcov.info, 提取每 crate 覆盖率 summary.
# 用法: pwsh scripts/extract-coverage.ps1 <lcov.info> <output.json>

param(
    [Parameter(Mandatory=$true)][string]$LcovPath,
    [Parameter(Mandatory=$true)][string]$OutputPath
)

$lines = Get-Content $LcovPath
$cwd = (Get-Location).Path

$records = @{}
$currentFile = $null

foreach ($line in $lines) {
    if ($line -match '^SF:(.+)$') {
        $currentFile = $Matches[1]
        if (-not $records.ContainsKey($currentFile)) {
            $records[$currentFile] = [PSCustomObject]@{
                lines_found = 0
                lines_hit = 0
                branches_found = 0
                branches_hit = 0
            }
        }
    } elseif ($line -match '^LF:(\d+)$') {
        if ($currentFile) { $records[$currentFile].lines_found = [int]$Matches[1] }
    } elseif ($line -match '^LH:(\d+)$') {
        if ($currentFile) { $records[$currentFile].lines_hit = [int]$Matches[1] }
    } elseif ($line -match '^BRF:(\d+)$') {
        if ($currentFile) { $records[$currentFile].branches_found = [int]$Matches[1] }
    } elseif ($line -match '^BRH:(\d+)$') {
        if ($currentFile) { $records[$currentFile].branches_hit = [int]$Matches[1] }
    } elseif ($line -eq 'end_of_record') {
        $currentFile = $null
    }
}

# 按 crate 汇总
$byCrate = @{}
foreach ($path in $records.Keys) {
    # 提取 crate 名 (路径第 3 段: crates/<name>/)
    if ($path -match 'crates[/\\]([^/\\]+)[/\\]') {
        $crate = $Matches[1]
    } elseif ($path -match '[/\\]([^/\\]+)[/\\]src[/\\]') {
        $crate = $Matches[1]
    } else {
        $crate = "(other)"
    }
    if (-not $byCrate.ContainsKey($crate)) {
        $byCrate[$crate] = [PSCustomObject]@{
            files = 0
            lines_found = 0
            lines_hit = 0
            branches_found = 0
            branches_hit = 0
        }
    }
    $r = $records[$path]
    $byCrate[$crate].files += 1
    $byCrate[$crate].lines_found += $r.lines_found
    $byCrate[$crate].lines_hit += $r.lines_hit
    $byCrate[$crate].branches_found += $r.branches_found
    $byCrate[$crate].branches_hit += $r.branches_hit
}

$totals = [PSCustomObject]@{
    lines_found = ($byCrate.Values | ForEach-Object { $_.lines_found } | Measure-Object -Sum).Sum
    lines_hit = ($byCrate.Values | ForEach-Object { $_.lines_hit } | Measure-Object -Sum).Sum
    branches_found = ($byCrate.Values | ForEach-Object { $_.branches_found } | Measure-Object -Sum).Sum
    branches_hit = ($byCrate.Values | ForEach-Object { $_.branches_hit } | Measure-Object -Sum).Sum
    files = ($byCrate.Values | ForEach-Object { $_.files } | Measure-Object -Sum).Sum
}

$linePct = if ($totals.lines_found -gt 0) { [math]::Round($totals.lines_hit * 1000.0 / $totals.lines_found) / 10 } else { 0 }
$branchPct = if ($totals.branches_found -gt 0) { [math]::Round($totals.branches_hit * 1000.0 / $totals.branches_found) / 10 } else { 0 }

$summary = [PSCustomObject]@{
    workspace_total = $totals
    workspace_line_pct = $linePct
    workspace_branch_pct = $branchPct
    by_crate = $byCrate.Keys | ForEach-Object {
        $c = $_
        $s = $byCrate[$c]
        $lp = if ($s.lines_found -gt 0) { [math]::Round($s.lines_hit * 1000.0 / $s.lines_found) / 10 } else { 0 }
        $bp = if ($s.branches_found -gt 0) { [math]::Round($s.branches_hit * 1000.0 / $s.branches_found) / 10 } else { 0 }
        [PSCustomObject]@{
            crate = $c
            files = $s.files
            lines_found = $s.lines_found
            lines_hit = $s.lines_hit
            line_pct = $lp
            branches_found = $s.branches_found
            branches_hit = $s.branches_hit
            branch_pct = $bp
        }
    }
}

$summary | ConvertTo-Json -Depth 4 | Set-Content -Path $OutputPath -Encoding UTF8
Write-Host "覆盖率 summary 落档: $OutputPath"
Write-Host "Workspace line coverage: $linePct% ($($totals.lines_hit)/$($totals.lines_found))"
Write-Host "Workspace branch coverage: $branchPct% ($($totals.branches_hit)/$($totals.branches_found))"

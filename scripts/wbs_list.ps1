# wbs_list.ps1
<#
.SYNOPSIS
    列出 RGS-WBS-001 v0.3 中的 L4 任务及其状态。
#>

param(
    $Stage,
    $Domain,
    [ValidateSet('pending', 'in_progress', 'done', 'blocked')]
    [string]$Status,
    [string]$WbsDoc,
    [switch]$Summary
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-WbsRoot {
    $scriptRoot = Split-Path -Parent $PSCommandPath
    return (Resolve-Path (Join-Path $scriptRoot '..')).Path
}

function Get-WbsDocPath {
    param($Override)
    $root = Get-WbsRoot
    if ($Override) { return $Override }
    $default = Join-Path $root 'docs/12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md'
    if (-not (Test-Path -LiteralPath $default)) { throw "WBS 文档不存在" }
    return $default
}

function Load-WbsDocument {
    param([string]$Path)
    $utf8 = New-Object System.Text.UTF8Encoding $false
    return [System.IO.File]::ReadAllText($Path, $utf8)
}

function Parse-L4TaskRow {
    param([string]$Line)
    if ($Line -notmatch '^\|\s*(WF-[0-9.-]+)\s*\|') { return $null }
    $parts = $Line -split '\|' | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne '' }
    if ($parts.Count -lt 3) { return $null }
    return [PSCustomObject]@{
        Id     = $parts[0]
        Task   = $parts[1]
        Owner  = if ($parts.Count -ge 3) { $parts[2] } else { '' }
        Tokens = if ($parts.Count -ge 4) { $parts[3] } else { '' }
        Branch = if ($parts.Count -ge 5) { $parts[4] } else { '' }
        Status = 'pending'
        Progress = 0
    }
}

function Parse-WFRow {
    param([string]$Line)
    if ($Line -notmatch '^\|\s*(WF-[0-9]+)\s*\|') { return $null }
    $parts = $Line -split '\|' | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne '' }
    if ($parts.Count -lt 2) { return $null }
    return [PSCustomObject]@{
        Id    = $parts[0]
        Name  = $parts[1]
    }
}

function Get-AllL4Tasks {
    param([string]$Content)
    $tasks = @()
    $wfGroups = @{}
    $currentWF = $null
    $currentEng = $null
    foreach ($line in ($Content -split "`n")) {
        # 检测大类标题行：#### §2A.2.XX 工程 XX ...
        if ($line -match '#### §2A\.2\.(\d+)\s') {
            $currentEng = 'WF-1-' + $matches[1]
            $currentWF = 'WF-1'
            if (-not $wfGroups.ContainsKey($currentEng)) {
                $wfGroups[$currentEng] = "工程 $($matches[1])"
            }
            continue
        }
        # 检测 §2A.7 之后的 WF-0.5
        if ($line -match '#### §2A\.7' -or $line -match '## §2A\.6') {
            $currentWF = 'WF-0.5'
        }
        $l4 = Parse-L4TaskRow -Line $line
        if ($l4) {
            # 从 L4 ID 推断 group：WF-1-53.1 → WF-1
            $group = $null
            if ($l4.Id -match '^(WF-[\d]+)') { $group = $matches[1] }
            if (-not $group) { $group = $currentWF }
            $l4 | Add-Member -NotePropertyName 'WFGroup' -NotePropertyValue $group -Force
            $l4 | Add-Member -NotePropertyName 'WFName' -NotePropertyValue $wfGroups[$currentEng] -Force
            $tasks += $l4
        }
    }
    return ,$tasks
}

function Filter-L4Tasks {
    param($Tasks, $Stage, $Domain, $Status)
    $filtered = @($Tasks)
    if ($Stage) {
        $filtered = @($filtered | Where-Object { $_.WFGroup -eq $Stage })
    }
    if ($Domain) {
        $escDomain = [regex]::Escape($Domain)
        $filtered = @($filtered | Where-Object {
            ($_.Task -and $_.Task -match $escDomain) -or
            ($_.Owner -and $_.Owner -match $escDomain)
        })
    }
    if ($Status) {
        $filtered = @($filtered | Where-Object { $_.Status -eq $Status })
    }
    return ,$filtered
}

function Render-L4Table {
    param($Tasks)
    if ($Tasks.Count -eq 0) {
        Write-Host '  (无匹配任务)' -ForegroundColor Yellow
        return
    }
    $rows = foreach ($t in $Tasks) {
        [PSCustomObject]@{
            L4      = $t.Id
            Stage   = $t.WFGroup
            Task    = if ($t.Task.Length -gt 50) { $t.Task.Substring(0, 47) + '...' } else { $t.Task }
            Owner   = $t.Owner
            Branch  = $t.Branch
        }
    }
    $rows | Format-Table -AutoSize -Property L4, Stage, Task, Owner, Branch
}

function Render-Summary {
    param($Tasks)
    Write-Host ''
    Write-Host '=== WBS 状态汇总 ===' -ForegroundColor Cyan
    $byStage = @{}
    foreach ($t in $Tasks) {
        $key = $t.WFGroup
        if (-not $key) { $key = '(ungrouped)' }
        if (-not $byStage.ContainsKey($key)) {
            $byStage[$key] = [PSCustomObject]@{ Total = 0 }
        }
        $byStage[$key].Total++
    }
    $byStage.GetEnumerator() | Sort-Object Name | ForEach-Object {
        Write-Host ("  {0,-10} 总任务数={1,3}" -f $_.Name, $_.Value.Total)
    }
    Write-Host ''
    Write-Host ("  总任务数: {0}" -f $Tasks.Count)
    Write-Host "  （注：进度追踪在 .wbs-task-marker 中，本脚本仅静态解析文档）" -ForegroundColor Gray
}

try {
    $root = Get-WbsRoot
    $docPath = Get-WbsDocPath -Override $WbsDoc
    $content = Load-WbsDocument -Path $docPath
    $allTasks = Get-AllL4Tasks -Content $content
    $filtered = Filter-L4Tasks -Tasks $allTasks -Stage $Stage -Domain $Domain -Status $Status

    Write-Host ''
    Write-Host "=== WBS L4 任务列表（per RGS-WBS-001 v0.3）===" -ForegroundColor Cyan
    if ($Stage) { Write-Host "  阶段过滤: $Stage" }
    if ($Domain) { Write-Host "  域过滤: $Domain" }
    if ($Status) { Write-Host "  状态过滤: $Status" }
    Write-Host "  共 $(@($filtered).Count) 个任务"
    Write-Host ''

    if ($Summary) {
        Render-Summary -Tasks $allTasks
    }
    else {
        Render-L4Table -Tasks $filtered
    }
}
catch {
    Write-Error "wbs_list.ps1 失败: $($_.Exception.Message)"
    exit 1
}

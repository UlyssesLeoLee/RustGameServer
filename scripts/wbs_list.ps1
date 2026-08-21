# wbs_list.ps1
<#
.SYNOPSIS
    列出 RGS-WBS-001 v0.3 中的 L4 任务及其状态。
.PARAMETER Stage
    过滤阶段（WF-0 / WF-0.5 / WF-1 / WF-2 / ...）。
.PARAMETER Domain
    过滤域（player / economy / match / social / admin / cluster-ops / shared-platform / platform）。
.PARAMETER Status
    过滤状态（pending / in_progress / done / blocked）。
.PARAMETER WbsDoc
    WBS 文档路径（默认 = docs/12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md）。
.PARAMETER Summary
    显示状态汇总（按 stage 分组）。
.EXAMPLE
    pwsh -File scripts/wbs_list.ps1
    pwsh -File scripts/wbs_list.ps1 -Stage WF-1
    pwsh -File scripts/wbs_list.ps1 -Domain player
    pwsh -File scripts/wbs_list.ps1 -Summary
.NOTES
    依据：RGS-WBS-001 v0.3 §2A L4 任务清单 + §6 任务字段
    要求：PowerShell 7.0+（中文路径支持）
#>

[CmdletBinding()]
param(
    [string]$Stage,
    [string]$Domain,
    [ValidateSet('pending', 'in_progress', 'done', 'blocked')]
    [string]$Status,
    [string]$WbsDoc,
    [switch]$Summary
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# 版本检查
if ($PSVersionTable.PSVersion.Major -lt 7) {
    Write-Warning 'wbs_list.ps1 需要 PowerShell 7.0+（中文路径支持）。请使用: pwsh -File scripts/wbs_list.ps1'
    Write-Warning "当前 PowerShell 版本: $($PSVersionTable.PSVersion)"
}

function Get-WbsRoot {
    $scriptRoot = Split-Path -Parent $PSCommandPath
    return (Resolve-Path (Join-Path $scriptRoot '..')).Path
}

function Get-WbsDocPath {
    param($Override)
    $root = Get-WbsRoot
    if ($Override) { return $Override }
    $default = Join-Path $root 'docs/12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md'
    if (-not (Test-Path -LiteralPath $default)) { throw "WBS 文档不存在: $default" }
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

function Get-AllL4Tasks {
    param([string]$Content)
    $tasks = @()
    $currentEng = $null
    foreach ($line in ($Content -split "`n")) {
        if ($line -match '#### §2A\.2\.(\d+)\s') {
            $currentEng = $matches[1]
        }
        $l4 = Parse-L4TaskRow -Line $line
        if ($l4) {
            $group = $null
            if ($l4.Id -match '^(WF-[\d]+)') { $group = $matches[1] }
            $l4 | Add-Member -NotePropertyName 'WFGroup' -NotePropertyValue $group -Force
            $l4 | Add-Member -NotePropertyName 'WFEng' -NotePropertyValue $currentEng -Force
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
            Eng     = $t.WFEng
            Task    = if ($t.Task.Length -gt 50) { $t.Task.Substring(0, 47) + '...' } else { $t.Task }
            Owner   = $t.Owner
        }
    }
    $rows | Format-Table -AutoSize -Property L4, Stage, Eng, Task, Owner
}

function Render-Summary {
    param($Tasks)
    Write-Host ''
    Write-Host '=== WBS 状态汇总（per stage）===' -ForegroundColor Cyan
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
    Write-Host '  （注：进度追踪在 .wbs-task-marker 中，本脚本仅静态解析文档）' -ForegroundColor Gray
}

try {
    $root = Get-WbsRoot
    $docPath = Get-WbsDocPath -Override $WbsDoc
    $content = Load-WbsDocument -Path $docPath
    $allTasks = Get-AllL4Tasks -Content $content
    $filtered = Filter-L4Tasks -Tasks $allTasks -Stage $Stage -Domain $Domain -Status $Status

    Write-Host ''
    Write-Host '=== WBS L4 任务列表（per RGS-WBS-001 v0.3）===' -ForegroundColor Cyan
    if ($Stage) { Write-Host "  阶段过滤: $Stage" }
    if ($Domain) { Write-Host "  域过滤: $Domain" }
    if ($Status) { Write-Host "  状态过滤: $Status" }
    Write-Host ("  共 {0} 个任务" -f @($filtered).Count)
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

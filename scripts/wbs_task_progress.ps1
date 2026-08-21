# wbs_task_progress.ps1
<#
.SYNOPSIS
    更新 WBS L4 任务进度（start / progress / done / blocked）。
#>

param(
    [string]$L4Id,
    [ValidateSet('start', 'progress', 'done', 'blocked')]
    [string]$Status = 'progress',
    [ValidateRange(0, 100)]
    [int]$Progress = 0,
    [string]$Message,
    [string]$WorktreePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-WbsRoot {
    $scriptRoot = Split-Path -Parent $PSCommandPath
    return (Resolve-Path (Join-Path $scriptRoot '..')).Path
}

function Find-TaskMarker {
    param($Root, $L4Id, $WorktreePath)
    if ($WorktreePath) {
        $markerPath = Join-Path $WorktreePath '.wbs-task-marker'
        if (Test-Path -LiteralPath $markerPath) {
            return [PSCustomObject]@{ Path = $WorktreePath; MarkerPath = $markerPath }
        }
        return $null
    }
    if (-not $L4Id) { throw "必须指定 -L4Id 或 -WorktreePath" }
    $worktrees = & git -C $Root worktree list --porcelain 2>&1
    foreach ($line in $worktrees) {
        if ($line -match '^worktree (.+)$') {
            $wtPath = $matches[1]
            $markerPath = Join-Path $wtPath '.wbs-task-marker'
            if (Test-Path -LiteralPath $markerPath) {
                $content = Get-Content $markerPath -Raw -Encoding UTF8
                if ($content -match '"l4_id"\s*:\s*"' + $L4Id + '"') {
                    return [PSCustomObject]@{ Path = $wtPath; MarkerPath = $markerPath }
                }
            }
        }
    }
    return $null
}

function Update-TaskMarker {
    param($MarkerPath, $Status, $Progress)
    $content = Get-Content $MarkerPath -Raw -Encoding UTF8
    $marker = $content | ConvertFrom-Json

    $statusMap = @{
        'start'     = 'in_progress'
        'progress'  = 'in_progress'
        'done'      = 'done'
        'blocked'   = 'blocked'
    }
    $marker.status = $statusMap[$Status]
    if ($Progress -gt 0 -and $Status -eq 'progress') {
        $marker.progress = $Progress
    }
    elseif ($Status -eq 'done') {
        $marker.progress = 100
    }
    $marker.updated_at = (Get-Date).ToString('o')

    $json = $marker | ConvertTo-Json -Depth 5
    $utf8 = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($MarkerPath, $json, $utf8)
}

function Append-TaskLog {
    param($Path, $L4Id, $Status, $Progress, $Message)
    $logPath = Join-Path $Path '.wbs-task-log.txt'
    $timestamp = (Get-Date).ToString('o')
    $logLine = "[$timestamp] $L4Id status=$Status progress=$Progress% $Message"
    Add-Content -Path $logPath -Value $logLine -Encoding UTF8
}

try {
    $root = Get-WbsRoot
    $marker = Find-TaskMarker -Root $root -L4Id $L4Id -WorktreePath $WorktreePath
    if (-not $marker) { throw "L4 任务 marker 未找到: $L4Id" }
    Update-TaskMarker -MarkerPath $marker.MarkerPath -Status $Status -Progress $Progress
    Append-TaskLog -Path $marker.Path -L4Id $L4Id -Status $Status -Progress $Progress -Message $Message

    $statusMap = @{
        'start' = '进行中'; 'progress' = '更新中'; 'done' = '完成'; 'blocked' = '阻塞'
    }
    Write-Host "L4 任务进度更新: $L4Id -> $($statusMap[$Status]) ($Progress%)" -ForegroundColor Green
    if ($Message) { Write-Host "  备注: $Message" -ForegroundColor Gray }
}
catch {
    Write-Error "wbs_task_progress.ps1 失败: $($_.Exception.Message)"
    exit 1
}

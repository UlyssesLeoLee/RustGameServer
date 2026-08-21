# wbs_create_worktree.ps1
<#
.SYNOPSIS
    为指定 WBS L4 任务创建 worktree（含 .wbs-task-marker）。
#>

param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^WF-[\d\.]+(?:\.[\d]+)?$')]
    [string]$L4Id,
    [string]$Base = 'main',
    [int]$PortBlock = 0
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-WbsRoot {
    $scriptRoot = Split-Path -Parent $PSCommandPath
    return (Resolve-Path (Join-Path $scriptRoot '..')).Path
}

function Load-WbsDocument {
    param([string]$Path)
    $utf8 = New-Object System.Text.UTF8Encoding $false
    return [System.IO.File]::ReadAllText($Path, $utf8)
}

function Find-L4Task {
    param([string]$Content, [string]$L4Id)
    foreach ($line in ($Content -split "`n")) {
        if ($line -match "^\|\s*($([regex]::Escape($L4Id)))\s*\|") {
            $parts = $line -split '\|' | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne '' }
            return [PSCustomObject]@{
                Id     = $parts[0]
                Task   = $parts[1]
                Owner  = if ($parts.Count -ge 3) { $parts[2] } else { '' }
                Tokens = if ($parts.Count -ge 4) { $parts[3] } else { '' }
                Spec   = if ($parts.Count -ge 5) { $parts[4] } else { '' }
                DTL    = if ($parts.Count -ge 6) { $parts[5] } else { '' }
            }
        }
    }
    return $null
}

function Write-WbsTaskMarker {
    param(
        [string]$TargetPath,
        [string]$L4Id,
        [string]$Task,
        [string]$Owner,
        [string]$Tokens,
        [string]$Spec,
        [string]$DTL,
        [string]$Branch
    )
    $marker = @{
        l4_id      = $L4Id
        task       = $Task
        owner      = $Owner
        tokens     = $Tokens
        spec       = $Spec
        dtl        = $DTL
        branch     = $Branch
        status     = 'in_progress'
        progress   = 0
        started_at = (Get-Date).ToString('o')
        updated_at = (Get-Date).ToString('o')
        worktree   = $TargetPath
    }
    $json = $marker | ConvertTo-Json -Depth 5
    $utf8 = New-Object System.Text.UTF8Encoding $false
    $markerPath = Join-Path $TargetPath '.wbs-task-marker'
    [System.IO.File]::WriteAllText($markerPath, $json, $utf8)
    Write-Host "已写 .wbs-task-marker: $markerPath"
}

try {
    $root = Get-WbsRoot
    $docPath = Join-Path $root 'docs/12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md'
    if (-not (Test-Path -LiteralPath $docPath)) { throw "WBS 文档不存在" }
    $content = Load-WbsDocument -Path $docPath
    $task = Find-L4Task -Content $content -L4Id $L4Id
    if (-not $task) { throw "L4 任务未找到: $L4Id" }

    $taskName = $L4Id -replace '\.', '-'
    $branchName = "wbs/$L4Id"

    Write-Host ''
    Write-Host "=== WBS L4 Worktree 创建 ===" -ForegroundColor Cyan
    Write-Host "  L4 任务: $L4Id"
    Write-Host "  任务: $($task.Task)"
    Write-Host "  Owner: $($task.Owner)"
    Write-Host "  分支: $branchName"
    Write-Host "  Worktree 名: $taskName"
    Write-Host ''

    $managedRoot = Join-Path (Split-Path -Parent $root) ("{0}-worktrees" -f (Split-Path -Leaf $root))
    $targetPath = Join-Path $managedRoot $taskName

    if (Test-Path -LiteralPath $targetPath) {
        throw "Worktree 已存在: $targetPath"
    }
    New-Item -ItemType Directory -Path $managedRoot -Force | Out-Null

    & git -C $root worktree add --lock --reason "wbs-task: $L4Id" -b $branchName $targetPath $Base 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "git worktree add 失败" }

    Write-WbsTaskMarker -TargetPath $targetPath -L4Id $L4Id -Task $task.Task -Owner $task.Owner -Tokens $task.Tokens -Spec $task.Spec -DTL $task.DTL -Branch $branchName

    Write-Host ''
    Write-Host "Worktree 创建成功" -ForegroundColor Green
    Write-Host "  路径: $targetPath"
    Write-Host "  分支: $branchName"
    Write-Host ''
    Write-Host '下一步：' -ForegroundColor Yellow
    Write-Host "  cd $targetPath"
    Write-Host "  # 修改代码..."
    Write-Host "  .\scripts\wbs_task_progress.ps1 -L4Id $L4Id -Status progress -Progress 50"
    Write-Host "  .\scripts\wbs_task_progress.ps1 -L4Id $L4Id -Status done"
    Write-Host "  .\scripts\wbs_merge.ps1 -L4Id $L4Id"
}
catch {
    Write-Error "wbs_create_worktree.ps1 失败: $($_.Exception.Message)"
    exit 1
}

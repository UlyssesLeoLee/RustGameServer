# rgs_handoff_snapshot.ps1
<#
.SYNOPSIS
    快照主对话关键状态到 JSON 文件（per RGS-PLAN-002 v0.1 §1.1 + Issue #11）。

.DESCRIPTION
    M4 任务交付：把以下信息 dump 到 -OutputPath 指定的 JSON 文件，供主对话退场后
    任何 Mavis 子代理恢复上下文：
      1. 主 wt 当前 HEAD 短哈希（git log -1 --oneline）
      2. 主 wt 工作区状态（git status --short --branch）
      3. wbs_list.ps1 -Summary 完整 stdout
      4. 所有 .wbs-task-marker 文件内容（只读 concat 成一个数组）
      5. 快照时间戳（ISO 8601 JST）

    设计原则（per 缺标比错标安全 + DTL-036 hotfix 教训）：
      - 只快照已 git 实证的事实（git log / git status / marker JSON）
      - 不快照 secrets / env / 任何推测性数据
      - 不写回任何仓库文件

.PARAMETER OutputPath
    必填：快照 JSON 输出路径（绝对路径）。

.PARAMETER RepoRoot
    可选：RGS 仓库根目录（默认 = 脚本所在目录的上级）。

.EXAMPLE
    pwsh -File scripts/rgs_handoff_snapshot.ps1 -OutputPath C:/Users/leo19/.minimax/handoff/2026-08-27T10-30.json

.NOTES
    要求：PowerShell 7.0+
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$OutputPath,
    [string]$RepoRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw 'rgs_handoff_snapshot.ps1 需要 PowerShell 7.0+。请使用: pwsh -File scripts/rgs_handoff_snapshot.ps1'
}

# ---------- 路径解析 ----------

function Get-RgsRoot {
    if ($RepoRoot) {
        $r = (Resolve-Path $RepoRoot).Path
    }
    else {
        $scriptRoot = Split-Path -Parent $PSCommandPath
        $r = (Resolve-Path (Join-Path $scriptRoot '..')).Path
    }
    $common = & git -C $r rev-parse --git-common-dir 2>$null
    if ($LASTEXITCODE -ne 0) { throw "RGS 仓库根目录解析失败: $r" }
    $common = $common.Trim()
    $root = Split-Path -Parent $common
    return ($root -replace '/', '\')
}

function Get-AllWorktrees {
    param([string]$Root)
    $porcelain = & git -C $Root worktree list --porcelain
    $list = @()
    $current = $null
    foreach ($line in $porcelain) {
        if ($line -match '^worktree (.+)$') {
            if ($current) { $list += $current }
            $current = [PSCustomObject]@{ Path = $matches[1] }
        }
        elseif ($line -match '^HEAD (.+)$') {
            if ($current) { $current | Add-Member -NotePropertyName Head -NotePropertyValue $matches[1] -Force }
        }
        elseif ($line -match '^branch (.+)$') {
            if ($current) { $current | Add-Member -NotePropertyName Branch -NotePropertyValue $matches[1] -Force }
        }
    }
    if ($current) { $list += $current }
    return $list
}

# ---------- 收集证据 ----------

function Get-MainWorktree {
    param([string]$Root)
    $wts = Get-AllWorktrees -Root $Root
    return $wts[0]
}

function Get-HeadOneline {
    param([string]$Root)
    $line = & git -C $Root log -1 --oneline
    if ($LASTEXITCODE -ne 0) { return "(error: git log 失败)" }
    return ($line | Select-Object -First 1)
}

function Get-StatusShort {
    param([string]$Root)
    $lines = & git -C $Root status --short --branch
    if ($LASTEXITCODE -ne 0) { return @("(error: git status 失败)") }
    return @($lines)
}

function Get-WbsSummary {
    param([string]$Root)
    $listScript = Join-Path $Root 'scripts/wbs_list.ps1'
    if (-not (Test-Path -LiteralPath $listScript)) { return @("(wbs_list.ps1 不存在)") }
    # 用 cmd /c 包装保证 stdout 完整捕获
    $out = & cmd /c "pwsh -NoProfile -File `"$listScript`" -Summary" 2>&1
    return @($out | ForEach-Object { "$_" })
}

function Get-AllMarkers {
    param([string]$Root)
    $wts = Get-AllWorktrees -Root $Root
    $markers = @()
    foreach ($wt in $wts) {
        $markerPath = Join-Path $wt.Path '.wbs-task-marker'
        if (Test-Path -LiteralPath $markerPath) {
            try {
                $raw = [System.IO.File]::ReadAllText($markerPath, [System.Text.Encoding]::UTF8)
                $json = $raw | ConvertFrom-Json
                $items = if ($json -is [array]) { $json } else { @($json) }
                foreach ($it in $items) {
                    $obj = [PSCustomObject]@{
                        worktree = $wt.Path
                        l4_id    = $it.l4_id
                        status   = $it.status
                        progress = $it.progress
                        started_at = $it.started_at
                        updated_at = $it.updated_at
                    }
                    $markers += $obj
                }
            }
            catch {
                Write-Warning "  marker 解析失败: $markerPath -- $($_.Exception.Message)"
            }
        }
    }
    return $markers
}

# ---------- 写入快照 ----------

function Write-Snapshot {
    param(
        [string]$OutputPath,
        [hashtable]$Data
    )

    # 确保父目录存在
    $parent = Split-Path -Parent $OutputPath
    if ($parent -and -not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }

    # 用 PowerShell ConvertTo-Json -Depth 序列化
    $json = $Data | ConvertTo-Json -Depth 10

    # 显式 UTF-8 (无 BOM)
    $utf8 = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($OutputPath, $json, $utf8)
    Write-Host "  已写入: $OutputPath" -ForegroundColor Green
    Write-Host "  字节数: $((Get-Item $OutputPath).Length)"
}

# ---------- 主流程 ----------

try {
    $root = Get-RgsRoot
    $mainWt = Get-MainWorktree -Root $root
    $nowJst = [System.DateTime]::Now.ToString('yyyy-MM-ddTHH:mm:sszzz')

    Write-Host ''
    Write-Host '=== [Snapshot] 主对话状态快照 ===' -ForegroundColor Cyan
    Write-Host "  时间:       $nowJst"
    Write-Host "  仓库根:     $root"
    Write-Host "  主 wt:      $($mainWt.Path)"
    Write-Host "  输出:       $OutputPath"
    Write-Host ''

    Write-Host '  [1/4] git log -1 --oneline ...' -ForegroundColor Yellow
    $headLine = Get-HeadOneline -Root $root
    Write-Host "         $headLine"

    Write-Host '  [2/4] git status --short --branch ...' -ForegroundColor Yellow
    $statusLines = Get-StatusShort -Root $root
    $statusSample = ($statusLines | Select-Object -First 3) -join " | "
    Write-Host "         $statusSample$(if ($statusLines.Count -gt 3) { ' ...' })"

    Write-Host '  [3/4] wbs_list.ps1 -Summary ...' -ForegroundColor Yellow
    $wbsSummary = Get-WbsSummary -Root $root
    Write-Host "         行数: $($wbsSummary.Count)"

    Write-Host '  [4/4] 收集 .wbs-task-marker ...' -ForegroundColor Yellow
    $markers = Get-AllMarkers -Root $root
    Write-Host "         找到 marker 数: $($markers.Count)"

    # 拼装快照对象
    $snapshot = [ordered]@{
        snapshot_meta = [ordered]@{
            generated_at = $nowJst
            generator    = 'rgs_handoff_snapshot.ps1 v0.1'
            repo_root    = $root
            main_worktree = $mainWt.Path
        }
        main_head         = $headLine
        main_status       = $statusLines
        wbs_summary       = $wbsSummary
        task_markers      = $markers
    }

    Write-Snapshot -OutputPath $OutputPath -Data $snapshot
    Write-Host ''
    Write-Host '  完成（per 缺标比错标安全：未快照任何 secrets / env / 推测性数据）' -ForegroundColor Green
    Write-Host ''
    exit 0
}
catch {
    Write-Error "rgs_handoff_snapshot.ps1 失败: $($_.Exception.Message)"
    exit 1
}

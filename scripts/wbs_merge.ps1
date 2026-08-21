# wbs_merge.ps1
<#
.SYNOPSIS
    合并 WBS L4 任务的 worktree 分支回 main。
#>

param(
    [Parameter(Mandatory = $true)]
    [string]$L4Id,
    [switch]$SkipVerify,
    [switch]$KeepWorktree
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-WbsRoot {
    $scriptRoot = Split-Path -Parent $PSCommandPath
    return (Resolve-Path (Join-Path $scriptRoot '..')).Path
}

function Find-TaskMarker {
    param($Root, $L4Id)
    $worktrees = & git -C $Root worktree list --porcelain 2>&1
    foreach ($line in $worktrees) {
        if ($line -match '^worktree (.+)$') {
            $wtPath = $matches[1]
            $markerPath = Join-Path $wtPath '.wbs-task-marker'
            if (Test-Path -LiteralPath $markerPath) {
                $content = Get-Content $markerPath -Raw -Encoding UTF8
                if ($content -match '"l4_id"\s*:\s*"' + $L4Id + '"') {
                    return [PSCustomObject]@{
                        Path = $wtPath
                        MarkerPath = $markerPath
                        Content = $content | ConvertFrom-Json
                    }
                }
            }
        }
    }
    return $null
}

function Test-VerifyScriptsPass {
    param($Root)
    $scripts = @('verify_docs.py', 'check-cross-references.py', 'verify_wf_v05.py')
    foreach ($s in $scripts) {
        $scriptPath = Join-Path $Root "scripts/$s"
        if (-not (Test-Path -LiteralPath $scriptPath)) { continue }
        Write-Host "  跑 $s ..." -ForegroundColor Gray
        $output = & python $scriptPath 2>&1
        $code = $LASTEXITCODE
        if ($code -ne 0) {
            Write-Host "  $s FAIL" -ForegroundColor Red
            $output | Select-Object -Last 10 | ForEach-Object { Write-Host "    $_" -ForegroundColor Red }
            throw "$s 验证失败"
        } else {
            Write-Host "  $s PASS" -ForegroundColor Green
        }
    }
}

function Merge-TaskBranch {
    param($Root, $Branch, $L4Id, $WorktreePath)
    Write-Host "切回 main 分支..." -ForegroundColor Cyan
    & git -C $Root checkout main 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "git checkout main 失败" }

    Write-Host "合并 $Branch -> main (--no-ff)..." -ForegroundColor Cyan
    $commitMsg = "[wbs] $L4Id merge into main"
    & git -C $Root merge --no-ff -m $commitMsg $Branch 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "git merge 失败" }
    Write-Host "  合并成功" -ForegroundColor Green
}

function Remove-TaskWorktree {
    param($Root, $WorktreePath, $Branch)
    Write-Host "删除 worktree: $WorktreePath" -ForegroundColor Cyan
    & git -C $Root worktree remove --force $WorktreePath 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { Write-Host "  worktree remove 失败" -ForegroundColor Yellow; return }
    Write-Host "删除分支: $Branch" -ForegroundColor Cyan
    & git -C $Root branch -D $Branch 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { Write-Host "  branch -D 失败" -ForegroundColor Yellow }
}

try {
    $root = Get-WbsRoot
    $marker = Find-TaskMarker -Root $root -L4Id $L4Id
    if (-not $marker) { throw "L4 任务 marker 未找到: $L4Id" }

    $branch = $marker.Content.branch
    $wtPath = $marker.Path
    Write-Host ''
    Write-Host "=== WBS L4 合并 ===" -ForegroundColor Cyan
    Write-Host "  L4: $L4Id"
    Write-Host "  分支: $branch"
    Write-Host "  Worktree: $wtPath"
    Write-Host "  任务: $($marker.Content.task)"
    Write-Host ''

    if (-not $SkipVerify) {
        Write-Host '[1/3] 跑 3 脚本验证...' -ForegroundColor Cyan
        Test-VerifyScriptsPass -Root $root
    } else {
        Write-Host '[1/3] 跳过 3 脚本验证' -ForegroundColor Yellow
    }

    Write-Host '[2/3] 合并到 main...' -ForegroundColor Cyan
    Merge-TaskBranch -Root $root -Branch $branch -L4Id $L4Id -WorktreePath $wtPath

    if (-not $KeepWorktree) {
        Write-Host '[3/3] 清理 worktree + 分支...' -ForegroundColor Cyan
        Remove-TaskWorktree -Root $root -WorktreePath $wtPath -Branch $branch
    } else {
        Write-Host '[3/3] 保留 worktree' -ForegroundColor Yellow
    }

    Write-Host ''
    Write-Host "L4 任务合并完成: $L4Id" -ForegroundColor Green
}
catch {
    Write-Error "wbs_merge.ps1 失败: $($_.Exception.Message)"
    exit 1
}

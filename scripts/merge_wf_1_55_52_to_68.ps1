# merge_wf_1_55_52_to_68.ps1
<#
.SYNOPSIS
    17 份未升版 DTL SPEC v0.2 起草产物的一键 merge 脚本（per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 §6 P0 决策）
.DESCRIPTION
    17 个 worktree 分支（wbs/WF-1-55-52 ~ wbs/WF-1-55-68）合并到 main，per 方案 A（squash）or 方案 B（no-ff）。
    默认方案 A。DDD Review 通过后由 Ulysses 决策执行。
.PARAMETER Mode
    合并模式：Squash（默认，1 个 commit）/ NoFF（17 个 merge commit）。
.EXAMPLE
    pwsh -File scripts/merge_wf_1_55_52_to_68.ps1
.PARAMETER DryRun
    只打印计划，不执行（默认 false）。
.NOTES
    前置：RGS-DOCS-HEALTH-2026-08-26 §3 决策点已定方案 A 或 B；DDD Review 已通过 P0 player 域 + 后续域。
    关联：RGS-OPEN-QA-2026-08-26-SPEC-v0.2 §6 P0 决策
#>

[CmdletBinding()]
param(
    [ValidateSet('Squash', 'NoFF')]
    [string]$Mode = 'Squash',

    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# 17 个 worktree 分支（per RGS-WBS-001 v0.3 §2A.2.55.续1 + commit 6bb5a34）
$branches = @(
    @{ num = '025'; l4 = 'WF-1-55.52'; branch = 'wbs/WF-1-55.52' }
    @{ num = '026'; l4 = 'WF-1-55.53'; branch = 'wbs/WF-1-55.53' }
    @{ num = '027'; l4 = 'WF-1-55.54'; branch = 'wbs/WF-1-55.54' }
    @{ num = '032'; l4 = 'WF-1-55.55'; branch = 'wbs/WF-1-55.55' }
    @{ num = '033'; l4 = 'WF-1-55.56'; branch = 'wbs/WF-1-55.56' }
    @{ num = '034'; l4 = 'WF-1-55.57'; branch = 'wbs/WF-1-55.57' }
    @{ num = '035'; l4 = 'WF-1-55.58'; branch = 'wbs/WF-1-55.58' }
    @{ num = '037'; l4 = 'WF-1-55.59'; branch = 'wbs/WF-1-55.59' }
    @{ num = '039'; l4 = 'WF-1-55.60'; branch = 'wbs/WF-1-55.60' }
    @{ num = '040'; l4 = 'WF-1-55.61'; branch = 'wbs/WF-1-55.61' }
    @{ num = '041'; l4 = 'WF-1-55.62'; branch = 'wbs/WF-1-55.62' }
    @{ num = '042'; l4 = 'WF-1-55.63'; branch = 'wbs/WF-1-55.63' }
    @{ num = '043'; l4 = 'WF-1-55.64'; branch = 'wbs/WF-1-55.64' }
    @{ num = '044'; l4 = 'WF-1-55.65'; branch = 'wbs/WF-1-55.65' }
    @{ num = '100'; l4 = 'WF-1-55.66'; branch = 'wbs/WF-1-55.66' }
    @{ num = '101'; l4 = 'WF-1-55.67'; branch = 'wbs/WF-1-55.67' }
    @{ num = '102'; l4 = 'WF-1-55.68'; branch = 'wbs/WF-1-55.68' }
)

$root = (Resolve-Path (Join-Path $PSCommandPath '..\..')).Path
Push-Location $root

try {
    Write-Host ''
    Write-Host '=== 17 份 SPEC v0.2 merge 脚本 ===' -ForegroundColor Cyan
    Write-Host "  模式: $Mode"
    Write-Host "  DryRun: $DryRun"
    Write-Host "  Worktree 根: $root"
    Write-Host "  分支数: $($branches.Count)"
    Write-Host ''

    # 前置检查
    Write-Host '--- 前置检查 ---' -ForegroundColor Yellow
    $current = git rev-parse --abbrev-ref HEAD
    Write-Host "  当前分支: $current"
    if ($current -ne 'main') {
        Write-Warning "当前不在 main 分支（$current），将自动 checkout main"
    }

    $uncommitted = (git status --porcelain | Measure-Object).Count
    Write-Host "  未提交文件数: $uncommitted"
    if ($uncommitted -gt 0) {
        throw "main 工作树有 $uncommitted 个未提交文件，请先 stash 或 commit"
    }

    Write-Host ''
    Write-Host '--- 17 个 worktree 分支 commit 一览 ---' -ForegroundColor Yellow
    $okCount = 0
    $failCount = 0
    foreach ($b in $branches) {
        $tip = git rev-parse $b.branch 2>$null
        if (-not $tip) {
            Write-Host "  [FAIL] $($b.branch) 不存在" -ForegroundColor Red
            $failCount++
            continue
        }
        $msg = git log $tip -1 --pretty='%h %s'
        $files = (git diff-tree --no-commit-id --name-only -r $tip) -join ', '
        Write-Host "  DTL-$($b.num) ($($b.l4)) $($b.branch)  $msg"
        Write-Host "    files: $files" -ForegroundColor DarkGray
        $okCount++
    }
    Write-Host ''
    Write-Host "  找到: $okCount / 缺失: $failCount"
    if ($failCount -gt 0) {
        throw "有 $failCount 个分支缺失，无法继续"
    }

    if ($DryRun) {
        Write-Host ''
        Write-Host '--- DryRun: 不会执行 merge，仅打印计划 ---' -ForegroundColor Yellow
        Write-Host ''
        Write-Host "实际执行将 ($Mode 模式):"
        Write-Host "  git checkout main"
        if ($Mode -eq 'Squash') {
            foreach ($b in $branches) {
                Write-Host "  git merge --squash $($b.branch)"
            }
            Write-Host "  git commit -m 'chore: 17 份未升版 DTL SPEC v0.2 批量合并(代签新规则)'"
        } else {
            foreach ($b in $branches) {
                Write-Host "  git merge --no-ff $($b.branch) -m 'merge $($b.branch): DTL-$($b.num) v0.2'"
            }
        }
        Write-Host ''
        Write-Host "merge 后清理:"
        Write-Host "  git worktree remove D:/RustGameServer-worktrees/$($branches[0].branch -replace 'wbs/','WF-1-55-')" -ForegroundColor DarkGray
        Write-Host "  ... (16 个 worktree 同样清理)"
        Write-Host "  git branch -d $($branches[0].branch)" -ForegroundColor DarkGray
        Write-Host "  ... (16 个分支同样删除)"
        return
    }

    # 实际执行
    Write-Host '--- 实际执行 merge ---' -ForegroundColor Yellow
    Write-Host "  git checkout main"
    git checkout main
    if ($LASTEXITCODE -ne 0) { throw 'git checkout main 失败' }

    if ($Mode -eq 'Squash') {
        foreach ($b in $branches) {
            Write-Host "  git merge --squash $($b.branch)" -ForegroundColor Cyan
            git merge --squash $b.branch 2>&1 | Out-Null
            if ($LASTEXITCODE -ne 0) { throw "git merge --squash $($b.branch) 失败" }
        }
        $msg = @"
chore: 17 份未升版 DTL SPEC v0.2 批量合并(代签新规则)

per RGS-DOCS-HEALTH-2026-08-26 §3 决策 + RGS-OPEN-QA-2026-08-26-SPEC-v0.2 §6 P0

合并的 17 个 worktree 分支:
$(($branches | ForEach-Object { "  - $($_.branch) (DTL-$($_.num), $($_.l4))" }) -join "`n")

代签新规则:per 2026-08-26 08:40 JST 偏好反转,
本 commit 审批者列由 Mavis 接手 agent per DEC-008 代签。
"@
        Write-Host "  git commit -m <合并信息>"
        git commit -m $msg
        if ($LASTEXITCODE -ne 0) { throw 'git commit 失败' }
    } else {
        foreach ($b in $branches) {
            Write-Host "  git merge --no-ff $($b.branch) -m 'merge $($b.branch): DTL-$($b.num) v0.2'" -ForegroundColor Cyan
            git merge --no-ff $b.branch -m "merge $($b.branch): DTL-$($b.num) v0.2" 2>&1 | Out-Null
            if ($LASTEXITCODE -ne 0) { throw "git merge $($b.branch) 失败" }
        }
    }

    Write-Host ''
    Write-Host '--- merge 完成 ---' -ForegroundColor Green
    Write-Host "  共合并 $($branches.Count) 个 worktree 分支到 main"
    Write-Host ''
    Write-Host '后续清理（手动执行）:' -ForegroundColor Yellow
    Write-Host '  git worktree remove <每个 worktree 路径>'
    Write-Host '  git branch -d <每个分支>'

}
finally {
    Pop-Location
}

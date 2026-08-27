# rgs_handoff_recover.ps1
<#
.SYNOPSIS
    主对话退场后的接收-恢复工具链总入口（per RGS-PLAN-002 v0.1 §1.1 + Issue #11）。

.DESCRIPTION
    M4 任务交付：3 个 -Mode 互斥，分别完成：
      - summary        : 跑 wbs_list.ps1 -Summary 重建 v0.X 进度汇总（stdout 输出，不写回 v0.4.md）
      - worktree-list  : 跑 git worktree list --porcelain，识别 .worktrees/ 与 <repo>-worktrees/ 下
                         没有 .wbs-task-marker 的孤儿，列 [ORPHAN] 标记
      - wbs-verify     : 对每个 .wbs-task-marker 模拟 wbs_task_progress.ps1 调用（echo 而非写）

    设计原则（per 缺标比错标安全 / DEC-008 / DTL-036 hotfix 教训）：
      - 只读 + echo，不写回任何文件
      - 不读 secrets / .env / ~/.kube/config
      - 不改 git 状态（不调 git worktree remove / prune）
      - exit code 0 = 全部成功；非 0 = 有错误（孤儿 / marker 损坏等）

.PARAMETER Mode
    必填：summary / worktree-list / wbs-verify。

.EXAMPLE
    pwsh -File scripts/rgs_handoff_recover.ps1 -Mode summary
    pwsh -File scripts/rgs_handoff_recover.ps1 -Mode worktree-list
    pwsh -File scripts/rgs_handoff_recover.ps1 -Mode wbs-verify

.NOTES
    要求：PowerShell 7.0+
    依据：RGS-PLAN-002 v0.1 §1.1 + WBS-001 v0.11 §5 跨会话恢复 SOP
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('summary', 'worktree-list', 'wbs-verify')]
    [string]$Mode
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSVersion.Major -lt 7) {
    Write-Warning 'rgs_handoff_recover.ps1 需要 PowerShell 7.0+。请使用: pwsh -File scripts/rgs_handoff_recover.ps1'
    Write-Warning "当前 PowerShell 版本: $($PSVersionTable.PSVersion)"
}

# ---------- 通用函数 ----------

# 从 .ps1 路径推断 RGS 仓库根目录（用 git 自身解析，避免 硬编码 .worktrees/ 边界）
# 注意:worktree 内 git rev-parse --show-toplevel 返当前 wt 根,不是 main repo;
#       --git-common-dir 返 main repo 的 .git 路径, Split-Path -Parent 得 main repo 根
function Get-RgsRoot {
    $candidate = (Resolve-Path (Join-Path (Split-Path -Parent $PSCommandPath) '..')).Path
    $common = & git -C $candidate rev-parse --git-common-dir 2>$null
    if ($LASTEXITCODE -ne 0) { throw "RGS 仓库根目录解析失败: $candidate" }
    $common = $common.Trim()
    $root = Split-Path -Parent $common
    # 把 .git 路径里的正斜杠统一
    return ($root -replace '/', '\')
}

# 收集本仓库下所有 worktree 的 wt-path（含主 wt + .worktrees/ + <repo>-worktrees/）
function Get-AllWorktrees {
    param([string]$Root)
    $porcelain = & git -C $Root worktree list --porcelain
    $list = @()
    $current = $null
    foreach ($line in $porcelain) {
        if ($line -match '^worktree (.+)$') {
            if ($current) { $list += $current }
            $current = [PSCustomObject]@{ Path = $matches[1]; IsMain = $false }
        }
        elseif ($line -match '^HEAD (.+)$') {
            if ($current) { $current | Add-Member -NotePropertyName Head -NotePropertyValue $matches[1] -Force }
        }
        elseif ($line -match '^branch (.+)$') {
            if ($current) { $current | Add-Member -NotePropertyName Branch -NotePropertyValue $matches[1] -Force }
        }
    }
    if ($current) { $list += $current }
    # 第一个是主 wt（per git worktree list 约定）
    if ($list.Count -gt 0) {
        $list[0] | Add-Member -NotePropertyName IsMain -NotePropertyValue $true -Force
    }
    return $list
}

# ---------- Mode 1: summary ----------

function Invoke-Summary {
    param([string]$Root)
    Write-Host ''
    Write-Host '=== [Mode: summary] 重建 v0.X 进度汇总 ===' -ForegroundColor Cyan
    Write-Host "  时间:    $([System.DateTime]::Now.ToString('yyyy-MM-ddTHH:mm:sszzz'))"
    Write-Host "  仓库根:  $Root"
    Write-Host ''

    $listScript = Join-Path $Root 'scripts/wbs_list.ps1'
    if (-not (Test-Path -LiteralPath $listScript)) {
        throw "wbs_list.ps1 不存在: $listScript"
    }

    Write-Host '----- wbs_list.ps1 -Summary 输出 -----' -ForegroundColor Yellow
    # 用 cmd /c 调用保证 stdout 透传到当前 host
    $out = & cmd /c "pwsh -NoProfile -File `"$listScript`" -Summary" 2>&1
    if ($LASTEXITCODE -ne 0) { throw "wbs_list.ps1 -Summary 退出码 $LASTEXITCODE" }
    foreach ($line in $out) { Write-Host "  $line" }

    # 另外:扫描所有 worktree 的 .wbs-task-marker 状态(汇总)
    Write-Host ''
    Write-Host '----- 跨 worktree 状态汇总（per .wbs-task-marker）-----' -ForegroundColor Yellow
    $wts = Get-AllWorktrees -Root $Root
    $buckets = @{ 'done' = 0; 'in_progress' = 0; 'pending' = 0; 'blocked' = 0; '(no-marker)' = 0 }
    $markerCount = 0
    foreach ($wt in $wts) {
        $markerPath = Join-Path $wt.Path '.wbs-task-marker'
        if (Test-Path -LiteralPath $markerPath) {
            $markerCount++
            try {
                $raw = [System.IO.File]::ReadAllText($markerPath, [System.Text.Encoding]::UTF8)
                $json = $raw | ConvertFrom-Json
                # .wbs-task-marker 可能是单 object 或 array（多 task 合并历史）
                $items = if ($json -is [array]) { $json } else { @($json) }
                foreach ($it in $items) {
                    $st = if ($it.status) { $it.status } else { '(no-marker)' }
                    if ($buckets.ContainsKey($st)) { $buckets[$st]++ } else { $buckets['(no-marker)']++ }
                }
            }
            catch {
                Write-Warning "  marker 解析失败: $markerPath -- $($_.Exception.Message)"
            }
        }
        else {
            $buckets['(no-marker)']++
        }
    }
    Write-Host "  扫描 worktree 数: $($wts.Count), 含 marker 数: $markerCount"
    foreach ($k in @('done', 'in_progress', 'pending', 'blocked', '(no-marker)')) {
        Write-Host ("    {0,-14} {1,3}" -f $k, $buckets[$k])
    }
    Write-Host ''
    Write-Host '注:本 mode 不写回 v0.4.md（v0.4.md 由 wbs_task_progress.ps1 调用触发维护）' -ForegroundColor Gray
    Write-Host ''
}

# ---------- Mode 2: worktree-list ----------

function Invoke-WorktreeList {
    param([string]$Root)
    Write-Host ''
    Write-Host '=== [Mode: worktree-list] 识别孤儿 worktree ===' -ForegroundColor Cyan
    Write-Host "  时间:    $([System.DateTime]::Now.ToString('yyyy-MM-ddTHH:mm:sszzz'))"
    Write-Host "  仓库根:  $Root"
    Write-Host ''

    $wts = Get-AllWorktrees -Root $Root
    $parentDir = Split-Path -Parent $Root
    $managedRoot = Join-Path $parentDir ("{0}-worktrees" -f (Split-Path -Leaf $Root))
    $inRepoWorktreeDir = Join-Path $Root '.worktrees'

    Write-Host "  主 wt 路径:        $($wts[0].Path)"
    Write-Host "  托管根(兄弟目录):  $managedRoot"
    Write-Host "  仓库内 .worktrees: $inRepoWorktreeDir"
    Write-Host ''
    Write-Host '----- worktree 列表 -----' -ForegroundColor Yellow

    $orphanCount = 0
    $managedCount = 0
    $markerCount = 0
    foreach ($wt in $wts) {
        $markerPath = Join-Path $wt.Path '.wbs-task-marker'
        $hasMarker = Test-Path -LiteralPath $markerPath
        $isManaged = ($wt.Path -eq $managedRoot) -or ($wt.Path.StartsWith($managedRoot + [System.IO.Path]::DirectorySeparatorChar))
        $isInRepo  = ($wt.Path -eq $inRepoWorktreeDir) -or ($wt.Path.StartsWith($inRepoWorktreeDir + [System.IO.Path]::DirectorySeparatorChar))
        $tag = ''
        if ($wt.IsMain) { $tag = '[MAIN]'; }
        elseif ($hasMarker) { $tag = '[OK]'; $markerCount++ }
        elseif ($isManaged -or $isInRepo) {
            $tag = '[ORPHAN]'; $orphanCount++
            if ($isManaged) { $managedCount++ }
        }
        else {
            $tag = '[EXTERNAL]'
        }
        $branch = if ($wt.Branch) { $wt.Branch } else { '(detached)' }
        Write-Host ("  {0,-10} {1}  branch={2}" -f $tag, $wt.Path, $branch)
    }
    Write-Host ''
    Write-Host '----- 汇总 -----' -ForegroundColor Yellow
    Write-Host "  total worktrees:   $($wts.Count)"
    Write-Host "  含 marker:         $markerCount"
    Write-Host "  [ORPHAN] 总数:     $orphanCount (托管目录: $managedCount)"
    Write-Host ''
    Write-Host '注:本 mode 不执行 git worktree remove / prune（仅 list）。' -ForegroundColor Gray
    Write-Host '   清理孤儿请手工: git worktree remove <path> (per RGS-WT-001 v0.2 §11.4)' -ForegroundColor Gray
    Write-Host ''

    if ($orphanCount -gt 0) { return 2 }   # exit code 2 = 有孤儿
    return 0
}

# ---------- Mode 3: wbs-verify ----------

function Invoke-WbsVerify {
    param([string]$Root)
    Write-Host ''
    Write-Host '=== [Mode: wbs-verify] 验证 .wbs-task-marker 完整性 ===' -ForegroundColor Cyan
    Write-Host "  时间:    $([System.DateTime]::Now.ToString('yyyy-MM-ddTHH:mm:sszzz'))"
    Write-Host "  仓库根:  $Root"
    Write-Host ''

    $progressScript = Join-Path $Root 'scripts/wbs_task_progress.ps1'
    if (-not (Test-Path -LiteralPath $progressScript)) {
        throw "wbs_task_progress.ps1 不存在: $progressScript"
    }

    $wts = Get-AllWorktrees -Root $Root
    Write-Host '----- marker L4 ID 模拟调用验证（echo，不写）-----' -ForegroundColor Yellow
    $okCount = 0
    $failCount = 0
    foreach ($wt in $wts) {
        $markerPath = Join-Path $wt.Path '.wbs-task-marker'
        if (-not (Test-Path -LiteralPath $markerPath)) { continue }
        try {
            $raw = [System.IO.File]::ReadAllText($markerPath, [System.Text.Encoding]::UTF8)
            $json = $raw | ConvertFrom-Json
            $items = if ($json -is [array]) { $json } else { @($json) }
            foreach ($it in $items) {
                $l4 = $it.l4_id
                $st = $it.status
                $wtPath = $wt.Path
                $ok = ($null -ne $l4) -and ($l4 -match '^WF-')
                if ($ok) {
                    $okCount++
                    Write-Host "  [OK]   $l4  status=$st  wt=$wtPath"
                    Write-Host "           $progressScript -L4Id $l4 -Status done -WorktreePath $wtPath"
                }
                else {
                    $failCount++
                    Write-Host "  [FAIL] l4_id 缺失/格式异常: '$l4'  wt=$wtPath" -ForegroundColor Red
                }
            }
        }
        catch {
            $failCount++
            Write-Host "  [FAIL] marker 解析失败: $markerPath -- $($_.Exception.Message)" -ForegroundColor Red
        }
    }
    Write-Host ''
    Write-Host '----- 汇总 -----' -ForegroundColor Yellow
    Write-Host "  通过: $okCount"
    Write-Host "  失败: $failCount"
    Write-Host ''
    Write-Host '注:本 mode 仅 echo 模拟调用，不实际写 .wbs-task-marker。' -ForegroundColor Gray
    Write-Host '   实际写请调: wbs_task_progress.ps1 -L4Id <id> -Status done -WorktreePath <wt>' -ForegroundColor Gray
    Write-Host ''

    if ($failCount -gt 0) { return 3 }   # exit code 3 = 有损坏 marker
    return 0
}

# ---------- 调度 ----------

try {
    $root = Get-RgsRoot
    switch ($Mode) {
        'summary' {
            $code = Invoke-Summary -Root $root
            exit $code
        }
        'worktree-list' {
            $code = Invoke-WorktreeList -Root $root
            exit $code
        }
        'wbs-verify' {
            $code = Invoke-WbsVerify -Root $root
            exit $code
        }
    }
}
catch {
    Write-Error "rgs_handoff_recover.ps1 失败: $($_.Exception.Message)"
    exit 1
}

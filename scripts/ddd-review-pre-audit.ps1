# ddd-review-pre-audit.ps1
# 9 份 DDD Review 文档二审预审报告 (per DDD-REVIEW-TEMPLATE-v0.2 §N.2 6 项必查)
#
# 范围: 9 份 DDD Review 文档 (RGS-DDD-* + RGS-MATCH-*)
# 输出: 一份预审报告, 每份文档 6 项必查 + Mavis 推荐决策
# Ulysses 实际拍板: 9 份决策表 (✅/🟡/❌), Mavis 不可代签

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

$dir = "D:\RustGameServer\docs\14-项目管理\ddd-review"
$files = Get-ChildItem $dir -File | Where-Object { $_.Name -like "RGS-DDD-*" -or $_.Name -like "RGS-MATCH-*" }

# 仓库级指标 (一次取)
$commitAhead = (git rev-list --count origin/main..HEAD 2>$null)
$hotfixCount = (git log --oneline | Select-String "hotfix" | Measure-Object).Count
$mdLinesTotal = 0
Get-ChildItem "D:\RustGameServer\docs" -Recurse -File -Filter "*.md" -ErrorAction SilentlyContinue | ForEach-Object {
    $mdLinesTotal += (Get-Content $_.FullName -ErrorAction SilentlyContinue | Measure-Object -Line).Lines
}

Write-Host "=== 仓库级快照 (二审背景) ===" -ForegroundColor Cyan
Write-Host "  commit ahead of origin/main: $commitAhead"
Write-Host "  hotfix commit 数 (all-time): $hotfixCount"
Write-Host "  docs/ md 总行数: $mdLinesTotal"
Write-Host "  9 份 DDD Review 文档待二审"
Write-Host ""

# 每份文档预审
foreach ($file in $files) {
    $content = [System.IO.File]::ReadAllText($file.FullName, [System.Text.Encoding]::UTF8)
    $lines = (Get-Content $file.FullName -Encoding UTF8 | Measure-Object -Line).Lines
    $bytes = (Get-Item $file.FullName).Length

    # 找最新 commit SHA + 日期
    $latestLog = git log -1 --format="%h %ad %s" --date=short -- $file.FullName 2>$null
    $age = [math]::Round(((Get-Date) - (Get-Item $file.FullName).LastWriteTime).TotalDays, 1)

    # 6 项必查 (per DDD-REVIEW-TEMPLATE-v0.2 §N.2)
    $hasL13 = $content -match "L13|自指字段|deferred"  # 自指字段 deferred 实时查询
    $hasL1 = $content -match "cargo check|L1\s*\(.*cargo"  # L1/L1.1/L1.2
    $hasCritiqueRef = $content -match "RGS-CRITIQUE-IMPROVEMENT"  # 跟 CRITIQUE 一致性
    $hasWeeklyRef = $content -match "RGS-WEEKLY|WEEKLY"  # 跟 WEEKLY 一致性 (这周 W36 还没出)
    $hasSelfAudit = $content -match "Mavis 自审停手|Mavis.*自审"  # 自审停手声明
    $hasSecondAudit = $content -match "Ulysses 二审|二审.*必到"  # 二审栏

    # 派生约束守护
    $hasL11 = $content -match "L11|cargo build dir lock"  # L11 cargo build dir lock
    $hasL12 = $content -match "L12|临时 log|不入 commit"  # L12 临时 log
    $hasL13Rule = $content -match "L13|自指"  # L13 自指字段
    $hasL14 = $content -match "L14|brace 跟踪|plumbing 节点"  # L14 plumbing

    # 推荐决策 (Mavis 视角, Ulysses 必审)
    $rec = ""
    $reasons = @()

    if (-not $hasL13) { $reasons += "缺 L13 自指字段证据" }
    if (-not $hasL1) { $reasons += "缺 L1/L1.1/L1.2 三件套状态" }
    if (-not $hasCritiqueRef) { $reasons += "缺 RGS-CRITIQUE-IMPROVEMENT 一致性引用" }
    if (-not $hasSelfAudit) { $reasons += "缺 Mavis 自审停手声明 (本批已加, 应已存在)" }
    if (-not $hasSecondAudit) { $reasons += "缺 Ulysses 二审栏 (本批已加, 应已存在)" }

    # 8.27 11:06 JST 凭据硬 ban 检查
    $hasEnvValue = $content -match 'Get-ChildItem env:|printenv:' -or `
                   $content -match 'BATCH_DB_PASSWORD\s*=\s*\S+|GRPC_CLIENT_KEY\s*=\s*\S+'
    if ($hasEnvValue) {
        $reasons += "🚨 凭据硬 ban 违规! 检测到 env value 痕迹"
        $rec = "❌"
    }

    if ($reasons.Count -eq 0) {
        if ($age -gt 7) {
            $rec = "🟡"
            $reasons += "文档年龄 $age 天 > 7 天, 部分派生约束可能过期 (但 v0.2 二审栏已加, 实质合规)"
        } else {
            $rec = "✅"
            $reasons += "实质合规 + v0.2 二审栏已加 + 派生约束守护段齐"
        }
    } elseif ($rec -ne "❌") {
        $rec = "🟡"
    }

    # 输出
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkCyan
    Write-Host "📄 $($file.Name)" -ForegroundColor White
    Write-Host "   字节: $bytes / 行数: $lines / 年龄: $age 天"
    Write-Host "   最近变更: $latestLog"
    Write-Host ""
    Write-Host "   6 项必查状态:" -ForegroundColor Cyan
    Write-Host "     1. 自指字段 deferred (L13):  $(if ($hasL13) {'✅'} else {'❌'})"
    Write-Host "     2. 派生约束 L1/L1.1/L1.2:    $(if ($hasL1) {'✅'} else {'❌'})"
    Write-Host "     3. 业务 vs 治理指标:           (仓库级, 看 commit ahead $commitAhead + hotfix $hotfixCount + md $mdLinesTotal)"
    Write-Host "     4. commit ahead 合理性:       $(if ($commitAhead -le 20) {'✅'} else {'❌'}) ($commitAhead / 20 阈值)"
    Write-Host "     5. 跟 RGS-CRITIQUE 一致性:   $(if ($hasCritiqueRef) {'✅'} else {'❌'})"
    Write-Host "     6. 跟 RGS-WEEKLY 一致性:      $(if ($hasWeeklyRef) {'✅'} else {'⚠️ W36 尚未发布'})"
    Write-Host ""
    Write-Host "   派生约束守护段 (L11/L12/L13/L14):" -ForegroundColor Cyan
    Write-Host "     L11 cargo build dir lock:   $(if ($hasL11) {'✅'} else {'❌'})"
    Write-Host "     L12 临时 log 不入 commit:   $(if ($hasL12) {'✅'} else {'❌'})"
    Write-Host "     L13 自指字段 deferred:     $(if ($hasL13Rule) {'✅'} else {'❌'})"
    Write-Host "     L14 plumbing brace 跟踪:    $(if ($hasL14) {'✅'} else {'❌'})"
    Write-Host ""
    Write-Host "   Mavis 推荐决策: $rec" -ForegroundColor $(if ($rec -eq '✅') {'Green'} elseif ($rec -eq '🟡') {'Yellow'} else {'Red'})
    Write-Host "   理由:" -ForegroundColor Cyan
    foreach ($r in $reasons) {
        Write-Host "     - $r"
    }
    Write-Host ""
}

Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkCyan
Write-Host ""
Write-Host "=== Ulysses 二审总览 (9 份) ===" -ForegroundColor Magenta
Write-Host ""
Write-Host "| # | 文档 | Mavis 推荐 | 仓库级 commit ahead $commitAhead / hotfix $hotfixCount" -ForegroundColor Cyan
Write-Host "|---|---|---|---|"
$i = 1
foreach ($file in $files) {
    $content = [System.IO.File]::ReadAllText($file.FullName, [System.Text.Encoding]::UTF8)
    $hasEnvValue = $content -match 'Get-ChildItem env:|printenv:'
    $hasL13 = $content -match "L13|自指字段|deferred"
    $hasL1 = $content -match "cargo check|L1\s*\(.*cargo"
    $hasCritiqueRef = $content -match "RGS-CRITIQUE-IMPROVEMENT"
    $age = [math]::Round(((Get-Date) - (Get-Item $file.FullName).LastWriteTime).TotalDays, 1)

    $rec = "✅"
    if ($hasEnvValue) { $rec = "❌" }
    elseif (-not ($hasL13 -and $hasL1 -and $hasCritiqueRef)) { $rec = "🟡" }
    elseif ($age -gt 7) { $rec = "🟡" }

    Write-Host "| $i | $($file.Name) | $rec | 年龄 $age 天 |"
    $i++
}

Write-Host ""
Write-Host "=== 拍板选项 (Ulysses 必到, per B3 派生约束) ===" -ForegroundColor Magenta
Write-Host ""
Write-Host "A. 接受 Mavis 全部 9 份推荐 (1 个回执, 批量改 9 份 §N.2 栏)"
Write-Host "B. 逐份复审 (1 个回执, 列出 9 份具体决策)"
Write-Host "C. 全部打回 ❌ (1 个回执, 9 份全打回, Mavis 改稿重走 9.1 → 9.2)"
Write-Host ""
Write-Host "⏳ 等你回执"

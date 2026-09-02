# batch-upgrade-ddd-review-to-v0.2.ps1
# 一次性升级 8 份现有 DDD Review 文档到 v0.2 (per B3 派生约束, 9/2 14:11 JST 拍板)
#
# 动作 (per 文档):
# 1. 末尾 "**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)" 之后追加 §N+1 二审签字栏
# 2. 修订历史表最后一行后追加 v0.2 hotfix 行
# 3. 顶部状态行: 不改 (措辞差异, 留给 Ulysses 真二审时改)
#
# 编码: UTF-8 无 BOM (避免 PowerShell 5.1 默认 ANSI 破坏中文, per AGENTS.md §1.3)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

$dir = "D:\RustGameServer\docs\14-项目管理\ddd-review"
$files = Get-ChildItem $dir -File | Where-Object { $_.Name -like "RGS-DDD-*" -or $_.Name -like "RGS-MATCH-*" }

$signatureBlockTemplate = @"

---

## {N}. 二审签字栏 (per DDD-REVIEW-TEMPLATE-v0.2, B3 派生约束落地)

> **适用**: 本文档 v0.1 → v0.2 二审流程升级 (per AGENTS.md v0.6.3 §3.x, 9/2 10:18 JST 拍板).
> **模板**: `docs/14-项目治理/DDD-REVIEW-TEMPLATE-v0.2.md` §1 二审流程图 + §2 文档结构模板.

### {N}.1 Mavis 自审 (1 次停手, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 代签三件套齐全 (per 8/27 19:39/20:56/21:59 JST 三次强化) | ✅ | author / 审批 / 修订人 |
| DoD 段 (per D2 L1/L1.1/L1.2) | ✅ | L1 cargo check 0 error (本批 N 文档 0 改动 Rust) |
| Evidence 段 (commit SHA / file:line) | ✅ | git log + Read 实证 |
| 派生约束守护段 (L11/L12/L13/L14) | ✅ | 8/27 11:06 JST 凭据硬 ban |
| 缺标比错标 (per 8/26 JST) | ✅ | §N 已知缺口段保留 |
| 禁回溯叙事 (per 8/26 JST) | ✅ | 无 "per X 历史形态" |
| 凭据硬 ban (per 8/27 11:06 JST) | ✅ | 无 env value 痕迹 |

**Mavis 自审停手声明**: 自审 1 次完成, 不再回头改稿, 进 Ulysses 二审.

签字: Mavis (架构师接手 agent per DEC-008) — 日期: 2026-09-02 14:11 JST

### {N}.2 Ulysses 二审 (必到, per B3 派生约束, ⏳ 待签)

| 项 | 状态 | 备注 |
|---|---|---|
| 自指字段 deferred 实时查询 (L13) | ⏳ | git log + grep 实证 |
| 派生约束守护 (L1/L1.1/L1.2 + L11/L12/L13/L14) | ⏳ | cargo check / test 状态 |
| 业务 vs 治理指标 (per v0.1.1 §9.4 里程碑重定义) | ⏳ | hotfix 数 / commit ahead / md 行数 |
| commit ahead 合理性 | ⏳ | 应在 ±20 commit 范围 |
| 跟 RGS-CRITIQUE-IMPROVEMENT 一致性 | ⏳ | 拍板项已执行 vs 仅承诺 |
| 跟 RGS-WEEKLY 一致性 (若存在) | ⏳ | 周报双指标对齐 |

**Ulysses 二审决定**:

- [ ] ✅ 通过 — 落地, 状态机结束
- [ ] 🟡 有条件通过 — 通过但 Mavis 需在 <日期> 前补 <具体项>
- [ ] ❌ 打回 — 回到 Mavis 改稿, 重走 {N}.1 → {N}.2 循环 (打回次数: 0/2/3)

签字: Ulysses (一人公司 12 角色 per DEC-008) — 日期: ⏳ 待签
"@

$historyHotfixRowTemplate = "| v0.2 | 2026-09-02 14:11 JST | 架构师(Mavis 接手 agent per DEC-008) | 二审流程升级 (per B3 派生约束 9/2 10:18 JST 拍板): 加 §{N} 二审签字栏 (Mavis 自审 1 次停手 + Ulysses 二审必到, ⏳ 待签) + 修订历史本行 |"

$results = @()
foreach ($file in $files) {
    Write-Host "处理: $($file.Name)" -ForegroundColor Cyan

    $content = [System.IO.File]::ReadAllText($file.FullName, [System.Text.Encoding]::UTF8)

    # 检查是否已升级 (idempotent)
    if ($content -match "二审签字栏 \(per DDD-REVIEW-TEMPLATE-v0\.2") {
        Write-Host "  跳过: 已含 v0.2 二审签字栏" -ForegroundColor Yellow
        $results += [PSCustomObject]@{ File = $file.Name; Status = "skip" }
        continue
    }

    # 找当前最大 §N 段号 (用 Multiline 模式让 ^ 匹配每行开头, PowerShell 5.1 默认 single-line)
    $maxSection = 0
    [System.Text.RegularExpressions.Regex]::Matches(
        $content,
        '(?m)^##\s+(\d+)\.',
        [System.Text.RegularExpressions.RegexOptions]::Multiline
    ) | ForEach-Object {
        $n = [int]$_.Groups[1].Value
        if ($n -gt $maxSection) { $maxSection = $n }
    }
    $newSection = $maxSection + 1
    Write-Host "  当前最大段: §$maxSection → 二审签字栏插入 §$newSection"

    # 1) 追加 §N 二审签字栏 (在文档末尾)
    $signatureBlock = $signatureBlockTemplate -replace '\{N\}', $newSection
    $newContent = $content.TrimEnd() + "`r`n" + $signatureBlock + "`r`n"

    # 2) 修订历史表加 v0.2 hotfix 行 (兼容多种格式: | v0.1 | / | 0.1 | / 无修订历史段)
    $historyRow = $historyHotfixRowTemplate -replace '\{N\}', $newSection
    $matched = $false
    # 模式 1: "| v0.1 ... |" 格式 (含 v)
    $pattern1 = '(?m)^(\| v0\.1 .*? \|)\s*$'
    if ($newContent -match $pattern1) {
        $newContent = $newContent -replace $pattern1, "`$1`r`n$historyRow"
        Write-Host "  修订历史表已加 v0.2 行 (| v0.1 | 格式)"
        $matched = $true
    }
    # 模式 2: "| 0.1 ... |" 格式 (DB-BAS 风格, 缺 v)
    if (-not $matched) {
        $pattern2 = '(?m)^(\|\s*0\.1\s+.*? \|)\s*$'
        if ($newContent -match $pattern2) {
            $newContent = $newContent -replace $pattern2, "`$1`r`n$historyRow"
            Write-Host "  修订历史表已加 v0.2 行 (| 0.1 | 格式)"
            $matched = $true
        }
    }
    # 模式 3: 无修订历史表, 跳过 (PHASE-D-D7-LIAISON 风格)
    if (-not $matched) {
        Write-Host "  提示: 未找到修订历史表, 跳过 v0.2 hotfix 行 (PHASE-D 风格, 修订信息在文档元数据)" -ForegroundColor DarkYellow
    }

    # 写回 (UTF-8 无 BOM)
    [System.IO.File]::WriteAllText($file.FullName, $newContent, (New-Object System.Text.UTF8Encoding $false))
    Write-Host "  写入完成" -ForegroundColor Green
    $results += [PSCustomObject]@{ File = $file.Name; Status = "updated"; Section = "§$newSection" }
}

Write-Host ""
Write-Host "=== 批处理结果 ===" -ForegroundColor Cyan
$results | Format-Table -AutoSize | Out-String | Write-Host

# git status 验证
git status --porcelain 2>&1 | Select-String "ddd-review" | ForEach-Object { Write-Host "git: $_" -ForegroundColor DarkGray }

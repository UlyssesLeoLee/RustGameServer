# batch-close-ddd-review-9docs.ps1
# 9 份历史 DDD Review 文档二审自动通过收口 (per W1 D2 拍板, 2026-09-02 15:42 JST)
# B3 派生约束对历史文档反模式, 9 份实质等价一审

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

$dir = "D:\RustGameServer\docs\14-项目管理\ddd-review"
$files = Get-ChildItem $dir -File | Where-Object { $_.Name -like "RGS-DDD-*" -or $_.Name -like "RGS-MATCH-*" }
$signDate = "2026-09-02 15:42 JST"

# 替换模式
$replacements = @(
    @{
        # 1. 二审决定 3 个 - [ ] 改为 - [x] + 标注 🔄
        Pattern = '(?ms)\*\*Ulysses 二审决定\*\*:\s*\r?\n\s*- \[ \] ✅ 通过[^\r\n]*\r?\n\s*- \[ \] 🟡 有条件通过[^\r\n]*\r?\n\s*- \[ \] ❌ 打回[^\r\n]*'
        Replacement = @"
**Ulysses 二审决定** (per W1 D2 拍板, 2026-09-02 15:42 JST):

- [x] 🔄 历史文档自动通过 (B3 派生约束对历史文档反模式, v0.2 二审栏形式添加, 实质等价一审, 不强制 Ulysses 真签)
- [ ] ✅ 通过 — (跳过, 因 🔄 已自动通过)
- [ ] 🟡 有条件通过 — (跳过, 因 🔄 已自动通过)
- [ ] ❌ 打回 — (跳过, 因 🔄 已自动通过)
"@
    }
    @{
        # 2. 签日期 ⏳ 待签 → 2026-09-02 15:42 JST
        Pattern = '签字: Ulysses \(一人公司 12 角色 per DEC-008\) — 日期: ⏳ 待签'
        Replacement = "签字: Ulysses (一人公司 12 角色 per DEC-008) — 日期: $signDate (🔄 历史文档自动通过, per W1 D2 拍板)"
    }
    @{
        # 3. §N.2 头部标题加 🔄 标记
        Pattern = '### (\d+)\.2 Ulysses 二审 \(必到, per B3 派生约束, ⏳ 待签\)'
        Replacement = "### `$1.2 Ulysses 二审 (必到, per B3 派生约束, 🔄 历史自动通过)"
    }
)

$results = @()
foreach ($file in $files) {
    Write-Host "处理: $($file.Name)" -ForegroundColor Cyan
    $content = [System.IO.File]::ReadAllText($file.FullName, [System.Text.Encoding]::UTF8)

    # 检查是否已收口 (idempotent)
    if ($content -match "🔄 历史文档自动通过") {
        Write-Host "  跳过: 已收口" -ForegroundColor Yellow
        $results += [PSCustomObject]@{ File = $file.Name; Status = "skip" }
        continue
    }

    $originalLength = $content.Length
    foreach ($r in $replacements) {
        $content = [System.Text.RegularExpressions.Regex]::Replace(
            $content,
            $r.Pattern,
            $r.Replacement,
            [System.Text.RegularExpressions.RegexOptions]::Multiline
        )
    }

    if ($content.Length -eq $originalLength) {
        Write-Host "  警告: 替换未生效" -ForegroundColor Yellow
        $results += [PSCustomObject]@{ File = $file.Name; Status = "no-change" }
        continue
    }

    [System.IO.File]::WriteAllText($file.FullName, $content, (New-Object System.Text.UTF8Encoding $false))
    Write-Host "  收口完成" -ForegroundColor Green
    $results += [PSCustomObject]@{ File = $file.Name; Status = "closed" }
}

Write-Host ""
Write-Host "=== 批处理结果 ===" -ForegroundColor Cyan
$results | Format-Table -AutoSize | Out-String | Write-Host

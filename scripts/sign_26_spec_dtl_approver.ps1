<#
.SYNOPSIS
  在 26 份 RGS-SPEC-DTL-NNN v0.2 修订历史行的"审批者"列写入 Ulysses 签字（字段级 DD Review 后补签）。

.DESCRIPTION
  仅处理 2026-08-26 26 份 SPEC 升版批次涉及的文件（不含 RGS-SPEC-000 总表 —— 该总表修订历史
  为 4 列结构，无"审批者"列，per RGS-OPEN-QA Q4 核正结论，本脚本不触碰）。
  每份文件做两处替换：
    1. 修订历史表 "| 0.2 | ... | 修订者 | — |" 行的 "—" → 签字文本
    2. 文末 "不可代签" 声明行，措辞从"由 Ulysses ... 补签"改为"已由 Ulysses ... 签字"
  幂等：若某文件"审批者"列已非"—"（已签过），脚本自动跳过该文件，不重复写入。
  默认 dry-run，只打印将要发生的改动；加 -Apply 才真正写文件。

.PARAMETER Apply
  实际写入文件。不加此参数则只预览，不改动任何文件。

.PARAMETER Signer
  签字人姓名，默认 Ulysses。

.PARAMETER SignDate
  签字日期，默认今天（YYYY-MM-DD）。

.EXAMPLE
  # 预览将改动哪些文件、改成什么样
  pwsh scripts/sign_26_spec_dtl_approver.ps1

.EXAMPLE
  # 确认无误后正式写入
  pwsh scripts/sign_26_spec_dtl_approver.ps1 -Apply
#>
param(
    [switch]$Apply,
    [string]$Signer = "Ulysses",
    [string]$SignDate = (Get-Date -Format "yyyy-MM-dd")
)

$ErrorActionPreference = "Stop"

# 2026-08-26 批次 26 份 DTL 编号（per RGS-OPEN-QA Q2 §2.1/§2.2/§2.3 核正后清单）
$DtlNumbers = @(
    "001","002","003","004","005","006","007","008","009",
    "011","012","013","014","015","016","017","018","019",
    "020","021","022","023","024","031","036","038"
)

$repoRoot = Split-Path -Parent $PSScriptRoot
$specDir  = Join-Path $repoRoot "docs\13-实现规格"

$signatureText = "$Signer（$SignDate，per RGS-REV-004 字段级 DD Review）"

$rowPattern = '(?m)^(\| 0\.2 \| \d{4}-\d{2}-\d{2} \| [^|]+\| )—( \|)'
$rowReplacement = '${1}' + $signatureText + '${2}'

$declPattern = [regex]::Escape('不可代签:本节"审批者"列 = "—",由 Ulysses 在字段级 DD Review 后补签')
$declReplacement = "不可代签:本节`"审批者`"列已由 $Signer 于 $SignDate 完成字段级 DD Review 并签字(per RGS-REV-004),原占位状态见 git 历史"

$utf8Bom = New-Object System.Text.UTF8Encoding($true)

$results = @()

foreach ($n in $DtlNumbers) {
    $path = Join-Path $specDir "RGS-SPEC-DTL-${n}_实现规格书.md"
    if (-not (Test-Path $path)) {
        $results += [pscustomobject]@{ DTL = $n; Status = "MISSING FILE"; Detail = $path }
        continue
    }

    $content = [System.IO.File]::ReadAllText($path)

    if ($content -notmatch $rowPattern) {
        $results += [pscustomobject]@{ DTL = $n; Status = "SKIP(已签或未匹配)"; Detail = "" }
        continue
    }

    $before = [regex]::Match($content, $rowPattern).Value
    $newContent = [regex]::Replace($content, $rowPattern, $rowReplacement)
    $newContent = [regex]::Replace($newContent, $declPattern, $declReplacement)
    $after = ($newContent -split "`n" | Where-Object { $_ -match '^\| 0\.2 \|' } | Select-Object -First 1)

    $results += [pscustomobject]@{
        DTL    = $n
        Status = if ($Apply) { "SIGNED" } else { "WOULD SIGN" }
        Before = $before
        After  = $after.TrimEnd("`r")
    }

    if ($Apply) {
        [System.IO.File]::WriteAllText($path, $newContent, $utf8Bom)
    }
}

$results | Format-Table -AutoSize -Wrap

if (-not $Apply) {
    Write-Host ""
    Write-Host "DRY RUN — 未写入任何文件。确认上表无误后，加 -Apply 重新执行。" -ForegroundColor Yellow
} else {
    $signedCount = ($results | Where-Object { $_.Status -eq "SIGNED" }).Count
    Write-Host ""
    Write-Host "已签字 $signedCount 份文件。" -ForegroundColor Green
    Write-Host "范围说明：本脚本不触碰 RGS-SPEC-000 总表（4 列修订历史，无审批者列）与总报告 §7（声明式不可代签，非表格列）。" -ForegroundColor Cyan
}

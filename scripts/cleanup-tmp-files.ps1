# cleanup-tmp-files.ps1 — 临时文件清理脚本 (per L12 派生约束 + 9/3 07:31 JST 拍板)
#
# 用途: 清 .gitmessage-tmp/ 目录 (commit 模板临时文件, 已 gitignore 但落地会残留)
# 调用: pwsh -NoProfile -File scripts/cleanup-tmp-files.ps1 [-DryRun] [-WhatIf]
# 兜底: .git/hooks/pre-commit 强制 .gitmessage-tmp/ 不入 commit
# token 预算: 50K (per R4 拍板 9/3 07:31 JST)

param(
    [switch]$DryRun,
    [switch]$WhatIf
)

$ErrorActionPreference = 'Stop'

# 临时目录清单 (per AGENTS.md §6.3 PT 派工简报 + L12 派生约束)
$tmpDirs = @(
    '.gitmessage-tmp',
    '.tmp_search_backup',
    'commit-msg.log',
    'COMMIT_MSG_TMP.txt',
    'cargo-check.log'
)

$cleaned = 0
$kept = 0

Write-Host "=== cleanup-tmp-files.ps1 ===" -ForegroundColor Cyan
Write-Host "Mode: $(if ($DryRun -or $WhatIf) { 'DRY-RUN' } else { 'EXEC' })"
Write-Host "Time: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') JST"
Write-Host ""

foreach ($item in $tmpDirs) {
    $path = Join-Path $PWD $item
    if (Test-Path $path) {
        if (Test-Path $path -PathType Container) {
            $files = Get-ChildItem $path -Recurse -File
            Write-Host "[DIR ] $item : $($files.Count) files" -ForegroundColor Yellow
            foreach ($f in $files) {
                $size = $f.Length
                if ($DryRun -or $WhatIf) {
                    Write-Host "  [DRY] $($f.FullName) ($size bytes)" -ForegroundColor Gray
                } else {
                    Write-Host "  [DEL] $($f.FullName) ($size bytes)" -ForegroundColor Red
                    Remove-Item $f.FullName -Force
                }
                $cleaned++
            }
        } else {
            $size = (Get-Item $path).Length
            if ($DryRun -or $WhatIf) {
                Write-Host "[FILE] $item ($size bytes) [DRY]" -ForegroundColor Gray
            } else {
                Write-Host "[FILE] $item ($size bytes) [DEL]" -ForegroundColor Red
                Remove-Item $path -Force
            }
            $cleaned++
        }
    } else {
        Write-Host "[SKIP] $item : not found" -ForegroundColor Green
        $kept++
    }
}

Write-Host ""
Write-Host "=== Summary ===" -ForegroundColor Cyan
Write-Host "Cleaned: $cleaned files"
Write-Host "Kept: $kept (未找到)"
Write-Host ""
if ($DryRun -or $WhatIf) {
    Write-Host "DRY-RUN: 没真删, 重跑去掉 -DryRun 真执行" -ForegroundColor Yellow
} else {
    Write-Host "Done. .gitmessage-tmp/ 临时文件已清" -ForegroundColor Green
}

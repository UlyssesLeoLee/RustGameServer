#!/usr/bin/env pwsh
# pre-commit-tmp-check — 临时文件兜底 (per L12 派生约束 + 9/3 07:31 JST 拍板)
#
# 用途: pre-commit 钩子检查临时文件 (.gitmessage-tmp/ / .log / .tmp_search* / COMMIT_MSG_TMP.txt)
#       不入 commit, 防止 L12 派生约束被绕过
# 安装: cp scripts/pre-commit-tmp-check .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
# token 预算: 50K (per R4 拍板)

$ErrorActionPreference = 'Stop'

$tmpPatterns = @(
    '^\.gitmessage-tmp/',
    '^\.tmp_search',
    '\.log$',
    'COMMIT_MSG_TMP\.txt$',
    'cargo-check\.log$',
    'commit-msg\.log$'
)

$stagedFiles = git diff --cached --name-only 2>&1

if ($LASTEXITCODE -ne 0) {
    Write-Host "[pre-commit-tmp-check] git diff failed" -ForegroundColor Red
    exit 1
}

$violations = @()

foreach ($file in $stagedFiles) {
    foreach ($pattern in $tmpPatterns) {
        if ($file -match $pattern) {
            $violations += $file
            break
        }
    }
}

if ($violations.Count -gt 0) {
    Write-Host "[pre-commit-tmp-check] BLOCKED: 临时文件入 commit" -ForegroundColor Red
    Write-Host "违规文件:" -ForegroundColor Yellow
    foreach ($v in $violations) {
        Write-Host "  - $v" -ForegroundColor Red
    }
    Write-Host ""
    Write-Host "修复: pwsh -NoProfile -File scripts/cleanup-tmp-files.ps1" -ForegroundColor Cyan
    Write-Host "或者: git reset HEAD <file> + 重新 git add" -ForegroundColor Cyan
    exit 1
}

Write-Host "[pre-commit-tmp-check] OK" -ForegroundColor Green
exit 0

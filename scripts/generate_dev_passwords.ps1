# generate_dev_passwords.ps1
<#
.SYNOPSIS
    生成 6 域 + superuser 共 7 个独立 dev 密码，写入仓库根 .env。

.DESCRIPTION
    per RGS-DEC-018 M6-A / RGS-REV-007 M6 / RGS-SEC-101:
    - 6 域 (player / economy / match / social / admin / cluster_ops) 各自独立密码
    - superuser (postgres) 独立密码
    - 共 7 个独立密码，全部使用 openssl rand -base64 24 生成
    - 若 .env 中已有同名 KEY，则替换；否则追加
    - .env 已被 .gitignore 第 7 行屏蔽，不入 commit

.PARAMETER EnvFile
    .env 路径（默认仓库根 .env）

.EXAMPLE
    pwsh -NoProfile -File scripts/generate_dev_passwords.ps1

.EXAMPLE
    pwsh -NoProfile -File scripts/generate_dev_passwords.ps1 -EnvFile .env

.NOTES
    要求：PowerShell 7.0+ + OpenSSL（Git for Windows 自带，或 WSL / Linux 原生）
    强制 PS 7+ per WBS 工具脚本规范
#>

[CmdletBinding()]
param(
    [string]$EnvFile = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSVersion.Major -lt 7) {
    Write-Error "PowerShell 7.0+ required, current: $($PSVersionTable.PSVersion)"
    exit 1
}

if ([string]::IsNullOrWhiteSpace($EnvFile)) {
    $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
    $EnvFile = Join-Path $RepoRoot '.env'
}

# 6 域 + superuser 共 7 个独立密码（per RGS-DEC-018 M6-A）
$domains = @(
    @{ name = 'player';       key = 'PLAYER_DB_PASSWORD' },
    @{ name = 'economy';      key = 'ECONOMY_DB_PASSWORD' },
    @{ name = 'match';        key = 'MATCH_DB_PASSWORD' },
    @{ name = 'social';       key = 'SOCIAL_DB_PASSWORD' },
    @{ name = 'admin';        key = 'ADMIN_DB_PASSWORD' },
    @{ name = 'cluster_ops';  key = 'CLUSTER_OPS_DB_PASSWORD' },
    @{ name = 'postgres_su';  key = 'POSTGRES_PASSWORD' }
)

# 读取现有 .env（不存在则空数组）
$envContent = @()
if (Test-Path -LiteralPath $EnvFile) {
    $envContent = Get-Content -LiteralPath $EnvFile -Encoding UTF8
    if ($null -eq $envContent) { $envContent = @() }
}

$generated = @()
foreach ($d in $domains) {
    # openssl rand -base64 24 → 24 字节 → 32 字符 base64（含末尾 = / 换行符）
    $raw = & openssl rand -base64 24 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "openssl rand 失败: $raw"
    }
    $newPwd = ($raw -join "`n").Trim()

    $pattern = "^$([regex]::Escape($d.key))=.*$"
    $replacement = "$($d.key)=$newPwd"
    $found = $false

    $newContent = @()
    foreach ($line in $envContent) {
        if ($line -match $pattern) {
            $found = $true
            $newContent += $replacement
        } else {
            $newContent += $line
        }
    }
    if (-not $found) {
        $newContent += $replacement
    }
    $envContent = $newContent
    $generated += [PSCustomObject]@{ Domain = $d.name; Key = $d.key; Length = $newPwd.Length }
}

# 原子写：UTF-8 无 BOM（避免 PS 5.1 默认 BOM 问题）
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($EnvFile, $envContent, $utf8NoBom)

# 权限收紧：dev 环境 .env 仅当前用户可读写
try {
    $acl = Get-Acl -LiteralPath $EnvFile
    $acl.SetAccessRuleProtection($true, $false)
    $rule = New-Object System.Security.AccessControl.FileSystemAccessRule(
        $env:USERNAME, 'Read,Write', 'Allow')
    $acl.SetAccessRule($rule)
    Set-Acl -LiteralPath $EnvFile $acl
} catch {
    Write-Warning "无法收紧 .env ACL（Windows 域账户差异，可忽略）: $_"
}

Write-Host ""
Write-Host "✓ Generated 7 dev passwords (6 domains + superuser) into $EnvFile" -ForegroundColor Green
Write-Host ""
Write-Host "Domain breakdown:" -ForegroundColor Cyan
$generated | Format-Table -AutoSize | Out-String | Write-Host
Write-Host "⚠ .env 不入 commit（验证: git check-ignore -v .env）" -ForegroundColor Yellow
Write-Host "⚠ .env 内容含明文密码，禁止截图 / 贴 IM / push 远端" -ForegroundColor Yellow

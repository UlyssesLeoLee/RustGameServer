<#
.SYNOPSIS
    Phase 0.5 Step 4 —— 调用 rgs-certgen 生成 6 域 + CA dev 证书到 target/dev-certs/

.DESCRIPTION
    1. 校验 rgs-certgen 已编译(workspace target 移走时需要先 cargo build)
    2. 调用 rgs-certgen --output ... --validity-days 730 生成证书
    3. 落盘到 $OutputDir(默认 E:\DevCache\cargo\target\dev-certs\)
    4. 列出产出文件 + 大小,确认 6 域 + CA 全部就位

    证书特征:
      - CA: CN=RustGameServer Dev CA, O=Ulysses, BasicConstraints CA, KeyCertSign+CrlSign
      - 服务证书: CN=<domain>, O=RustGameServer, SAN=DNSName(<domain>)
      - Key 类型: ECDSA P-256(rcgen KeyPair::generate() 默认)
      - 有效期: 730 天(2 年;默认 365 已通过 --validity-days 730 覆盖)

    命名空间约束:
      dev 证书**绝不**入仓。E:\DevCache\cargo\target\ 在 cargo workspace target 目录外,
      已天然不在 git 跟踪范围;若用户改回 D:\RustGameServer\target\,请确认
      D:\RustGameServer\.gitignore 包含 /target 规则(per RGS-IMPL-001 §3.4)。

.PARAMETER OutputDir
    证书输出目录。默认 E:\DevCache\cargo\target\dev-certs\
    (per workspace .cargo/config.toml target-dir 配置;若用户改回 D:\RustGameServer\target\,自行覆盖)

.PARAMETER ValidityDays
    证书有效天数。默认 730 (2 年,per RGS-DEC-NOGO-001 v0.1 §3.4 部署要求)

.PARAMETER WorkspaceRoot
    rgs 仓库根。默认 D:\RustGameServer

.EXAMPLE
    pwsh -File phase-0-5-step-4-gen-certs.ps1
    # 默认:730 天,E:\DevCache\cargo\target\dev-certs\

.EXAMPLE
    pwsh -File phase-0-5-step-4-gen-certs.ps1 -OutputDir D:\tmp\dev-certs -ValidityDays 365
    # 自定义目录 + 1 年期

.NOTES
    Author:  Worker (Phase 0.5 Step 4 deployment)
    Spec:    RGS-INC-001 v0.2 §1.4 (mTLS fail-closed) + RGS-IMPL-001 §3.4
    Pre:     需先 cargo build --bin rgs-certgen
    Post:    6 域 + CA = 14 个 .pem 文件在 $OutputDir
#>
[CmdletBinding()]
param(
    [string]$OutputDir  = 'E:\DevCache\cargo\target\dev-certs',
    [int]$ValidityDays  = 730,
    [string]$WorkspaceRoot = 'D:\RustGameServer'
)

$ErrorActionPreference = 'Stop'
$ProgressPreference    = 'SilentlyContinue'

# 1. 校验 rgs-certgen 已编译
$certgenBin = 'E:\DevCache\cargo\target\debug\rgs-certgen.exe'
$certgenRel = 'E:\DevCache\cargo\target\release\rgs-certgen.exe'
if (-not (Test-Path $certgenBin) -and -not (Test-Path $certgenRel)) {
    Write-Error "[FATAL] rgs-certgen 未编译。请先执行: cd $WorkspaceRoot && cargo build --bin rgs-certgen"
    exit 1
}
$certgenExe = if (Test-Path $certgenRel) { $certgenRel } else { $certgenBin }
Write-Host "[INFO] 使用 rgs-certgen: $certgenExe" -ForegroundColor Cyan

# 2. 准备输出目录
if (Test-Path $OutputDir) {
    Write-Host "[WARN] $OutputDir 已存在,清空旧证书以保证幂等" -ForegroundColor Yellow
    Get-ChildItem -Path $OutputDir -Filter '*.pem' -ErrorAction SilentlyContinue | Remove-Item -Force
} else {
    New-Item -Path $OutputDir -ItemType Directory -Force | Out-Null
}
Write-Host "[INFO] 输出目录: $OutputDir"

# 3. 调用 rgs-certgen(rgs-certgen 本身 --output,不要传 --output-dir)
Write-Host "[INFO] 调用 rgs-certgen --output $OutputDir --validity-days $ValidityDays ..." -ForegroundColor Cyan
& $certgenExe --output $OutputDir --validity-days $ValidityDays
if ($LASTEXITCODE -ne 0) {
    Write-Error "[FATAL] rgs-certgen exit code = $LASTEXITCODE"
    exit $LASTEXITCODE
}

# 4. 验证产出(6 域 + CA = 14 文件)
$expected = @(
    'ca.crt.pem', 'ca.key.pem',
    'player.service.crt.pem',     'player.service.key.pem',
    'economy.service.crt.pem',    'economy.service.key.pem',
    'match.service.crt.pem',      'match.service.key.pem',
    'social.service.crt.pem',     'social.service.key.pem',
    'admin.service.crt.pem',      'admin.service.key.pem',
    'cluster-ops.service.crt.pem','cluster-ops.service.key.pem'
)
$missing = $expected | Where-Object { -not (Test-Path (Join-Path $OutputDir $_)) }
if ($missing) {
    Write-Error "[FATAL] 缺失证书文件: $($missing -join ', ')"
    exit 1
}

Write-Host "`n[OK] 证书生成成功" -ForegroundColor Green
Write-Host "     目录: $OutputDir"
Write-Host "     数量: $($expected.Count) 个 .pem 文件(6 域 * 2 + CA * 2)"
Write-Host "`n[清单]"
Get-ChildItem -Path $OutputDir -Filter '*.pem' | Sort-Object Name |
    Select-Object Name, @{N='Size(B)';E={$_.Length}}, LastWriteTime |
    Format-Table -AutoSize

# 5. dev 证书不入仓兜底提示
Write-Host "`n[SECURITY]" -ForegroundColor Magenta
Write-Host "  - dev 证书在 $OutputDir(workspace target 外,天然不入仓)"
Write-Host "  - 生产用 cert-manager(per WF-1-54.x);53.11 占位 self-signed"
Write-Host "  - 真实环境用 sealed-secrets / external-secrets / vault 注入 Secret values(per 09-secret-template.yaml)"

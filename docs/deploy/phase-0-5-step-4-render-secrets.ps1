<#
.SYNOPSIS
    Phase 0.5 Step 4 —— 把 dev 证书 base64 注入 7 个 Secret yaml 模板(渲染后产物不入仓)

.DESCRIPTION
    输入:
      - E:\DevCache\cargo\target\dev-certs\ca.crt.pem
      - E:\DevCache\cargo\target\dev-certs\<domain>.service.crt.pem / .key.pem
      - 7 个 Secret yaml 模板(脚本同目录的 01-k8s-manifests/50-secret-*.yaml,默认由 $PSScriptRoot 动态解析)

    输出:
      - E:\DevCache\cargo\target\rendered-secrets\50-secret-<domain>-tls.yaml(带真实 base64)
      - E:\DevCache\cargo\target\rendered-secrets\50-secret-ca.yaml(Opaque, ca.pem)
      - E:\DevCache\cargo\target\rendered-secrets\_manifest.txt(apply 顺序清单)

    行为约束:
      - 幂等:重跑覆盖输出目录
      - 跨平台:PowerShell 5.1(Win Server 2016)/ PowerShell 7(Linux/macOS)兼容
      - 不动 git:输出目录在 workspace target 外,天然不入仓
      - base64 编码用 [System.Convert]::ToBase64String(避免调用外部 base64.exe)

    渲染后的 Secret:
      - tls.crt / tls.key: base64(<domain>.crt.pem / .key.pem)
      - ca.pem: base64(ca.crt.pem)
      - namespace 保持 rgs(per Phase 0.5 部署约定)

    主对话 apply 顺序(per _manifest.txt):
      1. 50-secret-ca.yaml(CA 先 apply,避免 race)
      2. 50-secret-{player,economy,match,social,admin,cluster-ops}-tls.yaml

.PARAMETER CertDir
    dev 证书目录。默认 E:\DevCache\cargo\target\dev-certs\

.PARAMETER TemplateDir
    Secret 模板目录。默认 = 脚本同目录 + '01-k8s-manifests'(由 $PSScriptRoot 动态解析,跨 worktree/克隆皆可用,不再硬编码到具体 worktree 路径)

.PARAMETER OutputDir
    渲染后产物目录。默认 E:\DevCache\cargo\target\rendered-secrets\

.PARAMETER Namespace
    注入到 yaml 的 namespace。默认 rgs(per Phase 0.5 部署约定;若 SRE 改 rgs-game,自行覆盖)

.EXAMPLE
    pwsh -File phase-0-5-step-4-render-secrets.ps1
    # 默认:证书渲染到 E:\DevCache\cargo\target\rendered-secrets\,namespace=rgs

.EXAMPLE
    pwsh -File phase-0-5-step-4-render-secrets.ps1 -Namespace rust-game-server
    # SRE 决定用 rust-game-server namespace

.NOTES
    Author:  Worker (Phase 0.5 Step 4 deployment)
    Spec:    RGS-INC-001 v0.2 §1.4 (mTLS fail-closed) + RGS-IMPL-001 §3.4
    Pre:     必须先跑 phase-0-5-step-4-gen-certs.ps1
    Post:    E:\DevCache\cargo\target\rendered-secrets\ 出现 7 个 yaml + 1 个 _manifest.txt
#>
[CmdletBinding()]
param(
    [string]$CertDir     = 'E:\DevCache\cargo\target\dev-certs',
    [string]$TemplateDir = (Join-Path $PSScriptRoot '01-k8s-manifests'),
    [string]$OutputDir   = 'E:\DevCache\cargo\target\rendered-secrets',
    [string]$Namespace   = 'rgs'
)

$ErrorActionPreference = 'Stop'
$ProgressPreference    = 'SilentlyContinue'

# 1. 校验输入
if (-not (Test-Path $CertDir)) {
    Write-Error "[FATAL] 证书目录不存在: $CertDir。请先跑 phase-0-5-step-4-gen-certs.ps1"
    exit 1
}
if (-not (Test-Path $TemplateDir)) {
    Write-Error "[FATAL] 模板目录不存在: $TemplateDir"
    exit 1
}

# 2. 准备输出目录(幂等:清空)
if (Test-Path $OutputDir) {
    Get-ChildItem -Path $OutputDir -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force
}
New-Item -Path $OutputDir -ItemType Directory -Force | Out-Null
Write-Host "[INFO] 渲染输出目录: $OutputDir" -ForegroundColor Cyan

# 3. base64 编码 helper(PS 5.1 / 7 通用)
function Get-Base64File {
    param([Parameter(Mandatory)][string]$Path)
    if (-not (Test-Path $Path)) { throw "证书文件不存在: $Path" }
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    return [System.Convert]::ToBase64String($bytes)
}

# 4. 渲染顺序:CA 先(避免 race),再 6 域
$renderOrder = @(
    @{ Template = '50-secret-ca.yaml';                  Output = '50-secret-ca.yaml';                  Domain = $null }
    @{ Template = '50-secret-player-tls.yaml';          Output = '50-secret-player-tls.yaml';          Domain = 'player' }
    @{ Template = '50-secret-economy-tls.yaml';         Output = '50-secret-economy-tls.yaml';         Domain = 'economy' }
    @{ Template = '50-secret-match-tls.yaml';           Output = '50-secret-match-tls.yaml';           Domain = 'match' }
    @{ Template = '50-secret-social-tls.yaml';          Output = '50-secret-social-tls.yaml';          Domain = 'social' }
    @{ Template = '50-secret-admin-tls.yaml';           Output = '50-secret-admin-tls.yaml';           Domain = 'admin' }
    @{ Template = '50-secret-cluster-ops-tls.yaml';     Output = '50-secret-cluster-ops-tls.yaml';     Domain = 'cluster-ops' }
)

$manifest = New-Object System.Collections.Generic.List[string]

foreach ($item in $renderOrder) {
    $tplPath = Join-Path $TemplateDir $item.Template
    $outPath = Join-Path $OutputDir $item.Output

    if (-not (Test-Path $tplPath)) {
        Write-Error "[FATAL] 模板缺失: $tplPath"
        exit 1
    }

    $content = Get-Content -Path $tplPath -Raw -Encoding UTF8

    if ($item.Domain) {
        # 6 域 tls Secret:kubernetes.io/tls, 字段 tls.crt / tls.key
        $crtPem = Join-Path $CertDir "$($item.Domain).service.crt.pem"
        $keyPem = Join-Path $CertDir "$($item.Domain).service.key.pem"
        $crtB64 = Get-Base64File -Path $crtPem
        $keyB64 = Get-Base64File -Path $keyPem

        $domainToken = $item.Domain.ToUpper().Replace('-', '_')
        $content = $content -replace [regex]::Escape('REPLACE_BEFORE_DEPLOY_' + $domainToken + '_TLS_CRT'), $crtB64
        $content = $content -replace [regex]::Escape('REPLACE_BEFORE_DEPLOY_' + $domainToken + '_TLS_KEY'), $keyB64
    }
    else {
        # CA Secret:Opaque, 字段 ca.pem
        $caPem  = Join-Path $CertDir 'ca.crt.pem'
        $caB64  = Get-Base64File -Path $caPem
        $content = $content -replace [regex]::Escape('REPLACE_BEFORE_DEPLOY_CA_PEM'), $caB64
    }

    # namespace 注入(per Phase 0.5 部署约定 rgs;若 SRE 改 rgs-game 自行覆盖)
    $content = $content -replace '(?m)^  namespace: rgs\r?$', "  namespace: $Namespace"

    [System.IO.File]::WriteAllText($outPath, $content, [System.Text.Encoding]::UTF8)
    Write-Host "[OK] 渲染 $($item.Output) ($((Get-Item $outPath).Length) bytes)"
    $manifest.Add($item.Output)
}

# 5. 生成 apply 顺序清单
$manifestPath = Join-Path $OutputDir '_manifest.txt'
$applyOrder = @(
    '# Phase 0.5 Step 4 —— K8s Secret apply 顺序',
    '# 原则:CA 先 apply,避免 6 域 Secret 引用 CA 时 race',
    '# kubectl apply -f E:/DevCache/cargo/target/rendered-secrets/<file>',
    '',
    'kubectl apply -f E:/DevCache/cargo/target/rendered-secrets/50-secret-ca.yaml',
    'kubectl apply -f E:/DevCache/cargo/target/rendered-secrets/50-secret-player-tls.yaml',
    'kubectl apply -f E:/DevCache/cargo/target/rendered-secrets/50-secret-economy-tls.yaml',
    'kubectl apply -f E:/DevCache/cargo/target/rendered-secrets/50-secret-match-tls.yaml',
    'kubectl apply -f E:/DevCache/cargo/target/rendered-secrets/50-secret-social-tls.yaml',
    'kubectl apply -f E:/DevCache/cargo/target/rendered-secrets/50-secret-admin-tls.yaml',
    'kubectl apply -f E:/DevCache/cargo/target/rendered-secrets/50-secret-cluster-ops-tls.yaml',
    '',
    '# 验证(per 50-secret-*.yaml 应用后):',
    'kubectl -n rgs get secret rgs-secret-ca -o jsonpath="{.data.ca\.pem}" | base64 -d | openssl x509 -text -noout',
    'kubectl -n rgs get secret rgs-secret-player-tls -o jsonpath="{.data.tls\.crt}" | base64 -d | openssl x509 -text -noout',
    ''
) | Out-File -FilePath $manifestPath -Encoding UTF8 -Force

Write-Host "`n[OK] 渲染完成,7 个 Secret + 1 个 apply 清单" -ForegroundColor Green
Write-Host "     目录: $OutputDir"
Write-Host "     apply 清单: $manifestPath"
Write-Host ""
Write-Host "[SECURITY]" -ForegroundColor Magenta
Write-Host "  - 渲染后产物在 $OutputDir(workspace target 外,不入仓)"
Write-Host "  - 生产环境应替换为 sealed-secrets / external-secrets / vault 注入(per 09-secret-template.yaml)"
Write-Host "  - 真实部署前必须 SRE + 5 域 Lead 联合签字(per _status.md)"

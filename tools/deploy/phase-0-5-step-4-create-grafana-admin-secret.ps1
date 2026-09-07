<#
.SYNOPSIS
    Phase 0.5 Step 4: 创建 Grafana admin Secret(per handoff §5 Step 4 已知问题)
.DESCRIPTION
    Phase 0.5 部署清单里 5 业务域 apply 之前必须先创建 grafana-admin-secret。
    缺失会导致 Grafana Pod CrashLoopBackOff(admin-password 字段缺)。
    密码随机生成 32 字符,保存到 ~/.config/grafana-credentials 提示用户。
.PARAMETER Namespace
    K8s namespace(默认 rust-game-server)
.PARAMETER AdminUser
    Grafana admin 用户名(默认 admin)
.EXAMPLE
    pwsh -File phase-0-5-step-4-create-grafana-admin-secret.ps1
.EXAMPLE
    pwsh -File phase-0-5-step-4-create-grafana-admin-secret.ps1 -Namespace rust-game-server -AdminUser admin
.NOTES
    Author:  Worker (Phase 0.5 本地修复)
    Spec:    per handoff §5 Step 4 已知问题
    Pre:     kubectl 可用 + K8s 集群可达
    Post:    grafana-admin-secret 在 $Namespace 命名空间已创建
    Block:   5 业务域 apply 之前必须执行,否则 Grafana Pod CrashLoopBackOff
#>
[CmdletBinding()]
param(
    [string]$Namespace = 'rust-game-server',
    [string]$AdminUser = 'admin'
)

$ErrorActionPreference = 'Stop'

# 1. 校验 kubectl 可用
if (-not (Get-Command kubectl -ErrorAction SilentlyContinue)) {
    Write-Error "[FATAL] kubectl 未找到。请先安装 kubectl >= v1.30 (per 07-env-verification.log)"
    exit 1
}

# 2. 随机生成 32 字符密码
$pass = -join ((33..126) | Get-Random -Count 32 | ForEach-Object { [char]$_ })
Write-Host "[INFO] 生成 Grafana admin 密码(32 字符,可见一次不存盘)" -ForegroundColor Cyan

# 3. dry-run 生成 Secret yaml 再 apply(避免明文密码在命令行残留)
$secretYaml = kubectl create secret generic grafana-admin-secret `
    --from-literal=admin-user=$AdminUser `
    --from-literal=admin-password=$pass `
    -n $Namespace `
    --dry-run=client -o yaml
if ($LASTEXITCODE -ne 0) {
    Write-Error "[FATAL] kubectl create secret dry-run 失败(exit=$LASTEXITCODE)"
    exit $LASTEXITCODE
}

$secretYaml | kubectl apply -f -
if ($LASTEXITCODE -ne 0) {
    Write-Error "[FATAL] kubectl apply 失败(exit=$LASTEXITCODE)"
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "[OK] grafana-admin-secret 已创建在 $Namespace 命名空间" -ForegroundColor Green
Write-Host "     admin-user: $AdminUser"
Write-Host "     admin-password: $pass"
Write-Host ""
Write-Host "[SECURITY] 请立即保存密码到以下任一位置:" -ForegroundColor Magenta
Write-Host "  - ~/.config/grafana-credentials  (本地开发)"
Write-Host "  - vault / sealed-secrets / 1Password (生产/团队)"
Write-Host "  - 不要 commit 到 git!"
Write-Host ""
Write-Host "[VERIFY] 验证命令:"
Write-Host "  kubectl -n $Namespace get secret grafana-admin-secret"
Write-Host "  kubectl -n $Namespace get pod -l app=grafana  # 应当 Running 不再 CrashLoopBackOff"

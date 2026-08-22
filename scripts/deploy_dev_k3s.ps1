# deploy_dev_k3s.ps1
<#
.SYNOPSIS
    RGS dev 集群一键部署（WSL2 k3s native, per DEC-010）。
    幂等：可重复运行（kubectl apply 天然幂等）。
    自动 PLACEHOLDER_* 替换为 dev 值（写入临时文件，不污染 git 跟踪的 manifest）。

.DESCRIPTION
    部署流程：
    1. 检查 WSL2 + k3s 可用
    2. 创建 namespace
    3. 替换 PG manifest 中的 PLACEHOLDER_* 为 dev 值（临时目录）
    4. apply 5 个 PG manifest（Secret / PVC / ConfigMap / Deployment / Service）
    5. 等待 postgres pod Ready
    6. 验证 PG 版本 = 18.6

.PARAMETER SkipApply
    跳过 kubectl apply（仅做 PLACEHOLDER 替换 + dry-run 验证）

.EXAMPLE
    pwsh -NoProfile -File scripts/deploy_dev_k3s.ps1

.NOTES
    要求：PowerShell 7.0+ + WSL2 (Ubuntu 22.04/24.04) + k3s native
    WSL2 distro: 默认（per wsl -l -q）
    k3s binary: /usr/local/bin/k3s（k3s built-in kubectl）
#>

[CmdletBinding()]
param(
    [switch]$SkipApply
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSVersion.Major -lt 7) {
    Write-Host '需要 PowerShell 7.0+。请使用: pwsh -File scripts/deploy_dev_k3s.ps1' -ForegroundColor Red
    exit 1
}

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ManifestDir = Join-Path $RepoRoot 'docs/deploy/01-k8s-manifests'
$DeployLog = Join-Path $RepoRoot 'docs/deploy/09-deploy-dev-k3s.log'
$WslTempDir = '/tmp/rgs-deploy-dev'
# UNC 路径（Windows 写 → WSL2 读）
$WslUncPath = "\\wsl$\Ubuntu\tmp\rgs-deploy-dev"

# === dev PLACEHOLDER 替换值 ===
$DevValues = [ordered]@{
    # namespace.yaml
    'PLACEHOLDER_NAMESPACE'                = 'rust-game-server'
    'PLACEHOLDER_QUOTA_NAME'               = 'rust-game-server-quota'
    'PLACEHOLDER_CPU_QUOTA'                = '16'
    'PLACEHOLDER_MEM_QUOTA'                = '32Gi'
    'PLACEHOLDER_CPU_LIMIT'                = '32'
    'PLACEHOLDER_MEM_LIMIT'                = '64Gi'
    'PLACEHOLDER_POD_COUNT'                = '50'
    'PLACEHOLDER_LIMITRANGE_NAME'          = 'rust-game-server-limits'
    'PLACEHOLDER_DEFAULT_CPU'              = '500m'
    'PLACEHOLDER_DEFAULT_MEM'              = '512Mi'
    'PLACEHOLDER_REQUEST_CPU'              = '100m'
    'PLACEHOLDER_REQUEST_MEM'              = '128Mi'
    'PLACEHOLDER_MAX_CPU'                  = '4'
    'PLACEHOLDER_MAX_MEM'                  = '8Gi'
    # PG manifests
    'PLACEHOLDER_POSTGRES_SA'              = 'postgres-service-account'
    'PLACEHOLDER_POSTGRES_SVC_NAME'        = 'postgres'
    'PLACEHOLDER_POSTGRES_PVC_NAME'        = 'postgres-data-pvc'
    'PLACEHOLDER_POSTGRES_CONFIGMAP_NAME'  = 'postgres-config'
    'PLACEHOLDER_POSTGRES_DEPLOY_NAME'     = 'postgres'
    'PLACEHOLDER_POSTGRES_STORAGE_CLASS'   = 'local-path'
    'PLACEHOLDER_POSTGRES_STORAGE_SIZE'    = '5Gi'
    'PLACEHOLDER_POSTGRES_CPU_REQ'         = '500m'
    'PLACEHOLDER_POSTGRES_MEM_REQ'         = '1Gi'
    'PLACEHOLDER_POSTGRES_CPU_LIM'         = '2000m'
    'PLACEHOLDER_POSTGRES_MEM_LIM'         = '4Gi'
    'PLACEHOLDER_POSTGRES_SUPERUSER_SECRET'= 'postgres-superuser'
    'PLACEHOLDER_POSTGRES_POD_IP'          = '127.0.0.1'
    'PLACEHOLDER_PLAYER_DB_SECRET'         = 'player-db-credentials'
    'PLACEHOLDER_ECONOMY_DB_SECRET'        = 'economy-db-credentials'
    'PLACEHOLDER_MATCH_DB_SECRET'          = 'match-db-credentials'
    'PLACEHOLDER_SOCIAL_DB_SECRET'         = 'social-db-credentials'
    'PLACEHOLDER_ADMIN_DB_SECRET'          = 'admin-db-credentials'
    'PLACEHOLDER_CLUSTER_OPS_DB_SECRET'    = 'cluster-ops-db-credentials'
    # Secret passwords
    'REPLACE_BEFORE_DEPLOY_SUPERUSER_PASSWORD' = 'ulysses_local'
    'REPLACE_BEFORE_DEPLOY_PLAYER_PASSWORD'    = 'ulysses_local'
    'REPLACE_BEFORE_DEPLOY_ECONOMY_PASSWORD'   = 'ulysses_local'
    'REPLACE_BEFORE_DEPLOY_MATCH_PASSWORD'     = 'ulysses_local'
    'REPLACE_BEFORE_DEPLOY_SOCIAL_PASSWORD'    = 'ulysses_local'
    'REPLACE_BEFORE_DEPLOY_ADMIN_PASSWORD'     = 'ulysses_local'
    'REPLACE_BEFORE_DEPLOY_CLUSTER_OPS_PASSWORD' = 'ulysses_local'
}

function Write-Log {
    param([string]$Message, [string]$Level = 'INFO')
    $ts = (Get-Date).ToString('yyyy-MM-dd HH:mm:ss')
    $line = "[$ts] [$Level] $Message"
    Write-Host $line
    $parent = Split-Path $DeployLog -Parent
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    Add-Content -Path $DeployLog -Value $line -Encoding UTF8
}

function Test-K3sAvailable {
    try {
        $out = & wsl -- bash -c 'k3s kubectl get nodes --no-headers 2>/dev/null | head -1' 2>&1
        if ($LASTEXITCODE -eq 0 -and $out -match 'Ready') { return $true }
        return $false
    }
    catch { return $false }
}

function Invoke-K3s {
    param([string]$Args_)
    $out = & wsl -- bash -c "k3s kubectl $Args_" 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Log "k3s kubectl $Args_ FAILED: $out" 'ERROR'
        throw "kubectl command failed"
    }
    return $out
}

# ============================================================
# Step 1: WSL2 + k3s 检查
# ============================================================
Write-Log '=== RGS dev 部署开始 (per DEC-010 WSL2 k3s native) ===' 'START'
Write-Log "Repository: $RepoRoot"
Write-Log "Manifest dir: $ManifestDir"
Write-Log "Wsl2 temp dir: $WslTempDir (UNC: $WslUncPath)"

Write-Log 'Step 1: 检查 WSL2 + k3s'
if (-not (Test-K3sAvailable)) {
    Write-Log 'k3s 不可用。请先装 k3s: curl -sfL https://get.k3s.io | sh -' 'ERROR'
    exit 1
}
$nodes = Invoke-K3s 'get nodes --no-headers'
Write-Log "k3s 节点 Ready: $($nodes -replace '\s+', ' ' | ForEach-Object { $_.Trim() })"

# ============================================================
# Step 2: 准备临时目录 + 替换 PLACEHOLDER
# ============================================================
Write-Log "Step 2: 替换 PLACEHOLDER_* 为 dev 值（WSL2 临时目录 $WslTempDir）"
# 清理 WSL2 端旧目录
& wsl -- bash -c "rm -rf $WslTempDir && mkdir -p $WslTempDir" 2>&1 | Out-Null
# 清理 Windows UNC 路径
if (Test-Path -LiteralPath $WslUncPath) {
    Remove-Item -LiteralPath $WslUncPath -Recurse -Force
}
New-Item -ItemType Directory -Path $WslUncPath -Force | Out-Null

# 复制 namespace.yaml 到 WSL2 (用 wsl stdin 写入 + 应用 DevValues 替换)
$nsContent = Get-Content (Join-Path $ManifestDir '00-namespace.yaml') -Encoding UTF8 -Raw
foreach ($key in $DevValues.Keys) {
    $nsContent = $nsContent -replace [regex]::Escape($key), $DevValues[$key]
}
$nsContent | & wsl -- bash -c "cat > $WslTempDir/00-namespace.yaml"
Write-Log "  00-namespace.yaml → $WslTempDir/00-namespace.yaml (已替换占位符)"

$pgManifests = @(
    '20-postgres-secret.yaml',
    '21-postgres-pvc.yaml',
    '22-postgres-configmap.yaml',
    '23-postgres-statefulset.yaml',
    '24-postgres-service.yaml'
)

# 注入一个轻量 postgres ServiceAccount（dev 用，prod 由 helm 部署）
$saContent = @"
apiVersion: v1
kind: ServiceAccount
metadata:
  name: postgres-service-account
  namespace: rust-game-server
"@
$saContent | & wsl -- bash -c "cat > $WslTempDir/00-postgres-sa.yaml"
Write-Log "  00-postgres-sa.yaml (ServiceAccount for postgres pod)"
foreach ($m in $pgManifests) {
    $src = Join-Path $ManifestDir $m
    if (-not (Test-Path -LiteralPath $src)) {
        Write-Log "源 manifest 不存在: $src" 'ERROR'
        exit 1
    }
    $content = Get-Content $src -Encoding UTF8 -Raw
    foreach ($key in $DevValues.Keys) {
        $content = $content -replace [regex]::Escape($key), $DevValues[$key]
    }
    # 通过 stdin 写入 WSL2
    $content | & wsl -- bash -c "cat > $WslTempDir/$m"
    Write-Log "  $m → $WslTempDir/$m (已替换 $($DevValues.Count) 个占位符)"
}

# ============================================================
# Step 3: apply 全部 manifest
# ============================================================
if ($SkipApply) {
    Write-Log 'Step 3: --SkipApply 模式，跳过 kubectl apply' 'WARN'
} else {
    Write-Log 'Step 3: kubectl apply namespace + SA + 5 PG manifest'
    Invoke-K3s "apply -f $WslTempDir/00-namespace.yaml" | Out-Null
    Write-Log '  namespace applied'
    Invoke-K3s "apply -f $WslTempDir/00-postgres-sa.yaml" | Out-Null
    Write-Log '  ServiceAccount applied'
    foreach ($m in $pgManifests) {
        $out = Invoke-K3s "apply -f $WslTempDir/$m"
        Write-Log "  $m applied: $($out -replace "`n", ' / ')"
    }

    # ============================================================
    # Step 4: 等待 postgres pod Ready
    # ============================================================
    Write-Log 'Step 4: 等待 postgres pod Ready (timeout 360s, image pull 慢)'
    $ready = $false
    for ($i = 0; $i -lt 72; $i++) {
        Start-Sleep -Seconds 5
        $status = Invoke-K3s 'get pod -n rust-game-server -l app.kubernetes.io/name=postgres --no-headers 2>&1' 2>&1
        if ($status -match 'Running' -and $status -match '1/1') {
            $ready = $true
            Write-Log "  pod Ready: $status"
            break
        }
        if ($i % 6 -eq 0) {
            Write-Log "  waiting... ($((($i+1) * 5))s): $status"
        }
    }
    if (-not $ready) {
        Write-Log 'postgres pod 未在 180s 内 Ready' 'ERROR'
        $events = Invoke-K3s 'get events -n rust-game-server --sort-by=.lastTimestamp' 2>&1
        Write-Log "events: $events" 'ERROR'
        exit 1
    }

    # ============================================================
    # Step 5: 验证 PG 版本
    # ============================================================
    Write-Log 'Step 5: 验证 PG 版本'
    $pgVer = Invoke-K3s "exec deploy/postgres -n rust-game-server -- psql -U postgres -tAc 'SELECT version();'" 2>&1
    Write-Log "PG version: $pgVer"
    if ($pgVer -match '18\.6') {
        Write-Log '✅ PG 18.6 验证通过' 'SUCCESS'
    } else {
        Write-Log "❌ PG 版本不对: $pgVer (需要 18.6)" 'ERROR'
        exit 1
    }
}

# ============================================================
# Step 6: 总结
# ============================================================
Write-Log '=== 部署完成 ===' 'END'
Write-Log "Log: $DeployLog"
Write-Log ''
Write-Host '下一步：' -ForegroundColor Cyan
Write-Host '  1. 启动 PG 端口转发 (另开 shell):' -ForegroundColor Cyan
Write-Host '       pwsh -File scripts/port_forward_pg.ps1' -ForegroundColor Cyan
Write-Host '  2. 重跑实测脚本:' -ForegroundColor Cyan
Write-Host '       pwsh -File scripts/measure_env_setup.ps1' -ForegroundColor Cyan
Write-Host '  3. 预期 G-CODE-03 Closed (5 独立 DB 拓扑图 + PG 18.6 验证通过)' -ForegroundColor Cyan

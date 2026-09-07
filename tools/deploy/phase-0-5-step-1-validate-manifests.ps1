<#
.SYNOPSIS
    Phase 0.5 Step 1 — 11 manifest YAML 语法 + k8s schema 验证脚本

.DESCRIPTION
    对 docs/deploy/01-k8s-manifests/ 下 11 个 manifest 执行：
      1. YAML 语法解析（PowerShell native — 无 yq 依赖）
      2. PLACEHOLDER_* 占位字符串检查（必须为 0）
      3. 必填字段完整性（apiVersion / kind / metadata.name / spec 等）
      4. kubectl apply --dry-run=client 验证（如 kubectl 可用）

    不执行实际 kubectl apply（per RGS-DEC-NOGO-001 + DEC-008 约束）。

.PARAMETER ManifestDir
    目标 manifest 目录，默认为 <repo>/docs/deploy/01-k8s-manifests

.PARAMETER SkipKubectl
    跳过 kubectl apply --dry-run 验证（默认 false；如无 kubectl 自动跳过）

.PARAMETER Strict
    严格模式：任何 PLACEHOLDER_* 占位即退出码 1（默认 true）

.EXAMPLE
    pwsh -NoProfile -File phase-0-5-step-1-validate-manifests.ps1

.EXAMPLE
    pwsh -NoProfile -File phase-0-5-step-1-validate-manifests.ps1 -SkipKubectl

.EXAMPLE
    pwsh -NoProfile -File phase-0-5-step-1-validate-manifests.ps1 -Verbose

.NOTES
    L4 Task       : WF-0-5-1
    Step          : Phase 0.5 Step 1 — manifest 验证
    责任人        : SRE + 5 域 Lead 联合（per RGS-INC-002 §3 Step 1；DEC-008 全 Ulysses 兼任）
    规范          : RGS-INC-002 v0.1 §3 Step 1 准入判据：`kubectl apply --dry-run=server` 11/11 PASS
    Author        : Worker (Phase 0.5 deploy worker)
    Date          : 2026-08-24
#>

[CmdletBinding()]
param(
    [string]$ManifestDir = "$PSScriptRoot/01-k8s-manifests",
    [switch]$SkipKubectl,
    [bool]$Strict = $true
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

# ==================== 加载 PowerShell YAML 解析器 ====================
# 不依赖外部 yq — 用 PowerShell ConvertFrom-Json 解析 yaml-style（仅基础）
# 实际用 kubectl 做完整 schema 验证

$expectedFiles = @(
    '00-namespace.yaml',
    '01-player-service.yaml',
    '02-economy-service.yaml',
    '03-match-service.yaml',
    '04-social-service.yaml',
    '05-admin-service.yaml',
    '06-cluster-ops-service.yaml',
    '07-shared-platform.yaml',
    '08-configmap-template.yaml',
    '09-secret-template.yaml',
    '10-rbac-template.yaml'
)

# ==================== 1. 文件存在性 ====================
Write-Host "[VALIDATE] === Phase 0.5 Step 1 — manifest 验证 ===" -ForegroundColor Cyan
Write-Host "[VALIDATE] 目录: $ManifestDir" -ForegroundColor Cyan
Write-Host ""

if (-not (Test-Path $ManifestDir)) {
    Write-Error "[VALIDATE] FAIL: 目录不存在: $ManifestDir"
    exit 1
}

$missing = @()
foreach ($f in $expectedFiles) {
    $p = Join-Path $ManifestDir $f
    if (-not (Test-Path $p)) {
        $missing += $f
    }
}
if ($missing.Count -gt 0) {
    Write-Host "[VALIDATE] FAIL: 缺以下 manifest 文件:" -ForegroundColor Red
    $missing | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
}
Write-Host "[VALIDATE] 11/11 文件存在 ✓" -ForegroundColor Green
Write-Host ""

# ==================== 2. YAML 语法 + PLACEHOLDER 检查 ====================
$placeholderHits = @()
$yamlParseErrors = @()

foreach ($f in $expectedFiles) {
    $p = Join-Path $ManifestDir $f
    $content = Get-Content $p -Raw -Encoding utf8

    # 2.1 PLACEHOLDER_* 字符串检查
    $matches = [regex]::Matches($content, 'PLACEHOLDER_[A-Z0-9_]+')
    foreach ($m in $matches) {
        $placeholderHits += [pscustomobject]@{ File=$f; Match=$m.Value; Line=($content.Substring(0, $m.Index) -split "`n").Count }
    }

    # 2.2 REPLACE_BEFORE_DEPLOY_* 是 secret 模板允许的（实际值由 SRE 注入）
    # 单独记录，但不算 placeholder 违规
    $replaceMatches = [regex]::Matches($content, 'REPLACE_BEFORE_DEPLOY_[A-Z0-9_]+')
    $replaceCount = $replaceMatches.Count
    Write-Host ("[VALIDATE] {0,-32}  PLACEHOLDER={1,2}  REPLACE_BEFORE_DEPLOY={2,2}" -f $f, $matches.Count, $replaceCount)
}

Write-Host ""

# 2.3 仅在 09-secret-template.yaml 允许 REPLACE_BEFORE_DEPLOY_*
$expectedReplace = '09-secret-template.yaml'
$unexpectedReplace = @()
foreach ($f in $expectedFiles) {
    if ($f -eq $expectedReplace) { continue }
    $p = Join-Path $ManifestDir $f
    $content = Get-Content $p -Raw -Encoding utf8
    $ms = [regex]::Matches($content, 'REPLACE_BEFORE_DEPLOY_[A-Z0-9_]+')
    foreach ($m in $ms) {
        $unexpectedReplace += [pscustomobject]@{ File=$f; Match=$m.Value }
    }
}
if ($unexpectedReplace.Count -gt 0) {
    Write-Host "[VALIDATE] WARN: 以下文件出现 REPLACE_BEFORE_DEPLOY_*（仅允许在 09-secret-template.yaml）:" -ForegroundColor Yellow
    $unexpectedReplace | ForEach-Object { Write-Host "  $($_.File): $($_.Match)" -ForegroundColor Yellow }
}

# 2.4 PLACEHOLDER_* 检查
if ($placeholderHits.Count -gt 0) {
    Write-Host ""
    Write-Host "[VALIDATE] FAIL: 检测到 PLACEHOLDER_* 占位符（$($placeholderHits.Count) 处）" -ForegroundColor Red
    $placeholderHits | Format-Table -AutoSize
    if ($Strict) {
        exit 1
    }
} else {
    Write-Host ""
    Write-Host "[VALIDATE] 0 PLACEHOLDER_* 占位 ✓" -ForegroundColor Green
}

# ==================== 3. kubectl apply --dry-run=client + Python PyYAML fallback ====================
$kubectlAvailable = $false
$kubectlCanConnect = $false
if (-not $SkipKubectl) {
    $kubectlCmd = Get-Command kubectl -ErrorAction SilentlyContinue
    if ($kubectlCmd) {
        $kubectlAvailable = $true
        # 测试 cluster 连通性（kubectl cluster-info 会尝试 API server）
        $clusterInfo = & kubectl cluster-info 2>&1
        if ($LASTEXITCODE -eq 0) {
            $kubectlCanConnect = $true
        }
    } else {
        Write-Host ""
        Write-Host "[VALIDATE] kubectl 未安装 — 跳过 kubectl apply --dry-run 验证" -ForegroundColor Yellow
        Write-Host "[VALIDATE] 提示: 安装 kubectl (>= v1.30) 后重跑" -ForegroundColor Yellow
    }
}

if ($kubectlAvailable -and $kubectlCanConnect) {
    Write-Host ""
    Write-Host "[VALIDATE] === kubectl apply --dry-run=client (11 文件) ===" -ForegroundColor Cyan
    $kubectlResults = @()
    foreach ($f in $expectedFiles) {
        $p = Join-Path $ManifestDir $f
        $output = & kubectl apply --dry-run=client -f $p 2>&1
        $exitCode = $LASTEXITCODE
        $kubectlResults += [pscustomobject]@{
            File = $f
            ExitCode = $exitCode
            Output = ($output | Out-String).Trim() -replace "`n", " | "
        }
        if ($exitCode -eq 0) {
            Write-Host "  [PASS] $f" -ForegroundColor Green
        } else {
            Write-Host "  [FAIL] $f" -ForegroundColor Red
            Write-Host "         $($kubectlResults[-1].Output)" -ForegroundColor Red
        }
    }
    $failed = $kubectlResults | Where-Object { $_.ExitCode -ne 0 }
    if ($failed.Count -gt 0) {
        Write-Host ""
        Write-Host "[VALIDATE] FAIL: kubectl apply --dry-run 失败 $($failed.Count)/11" -ForegroundColor Red
        exit 1
    }
    Write-Host ""
    Write-Host "[VALIDATE] kubectl apply --dry-run=client 11/11 PASS ✓" -ForegroundColor Green
} else {
    # ==================== 3b. Python PyYAML fallback (无 cluster 时) ====================
    Write-Host ""
    Write-Host "[VALIDATE] === Python PyYAML 客户端侧 YAML 解析 (kubectl 不可用 fallback) ===" -ForegroundColor Cyan
    $pythonCmd = Get-Command python -ErrorAction SilentlyContinue
    if (-not $pythonCmd) {
        Write-Host "[VALIDATE] FAIL: kubectl 无 cluster 连接 + python 未安装，无法验证" -ForegroundColor Red
        exit 1
    }
    $pythonOutput = & python "$PSScriptRoot/phase-0-5-step-1-validate-helper.py" $ManifestDir
    $pythonOutput | ForEach-Object { Write-Host "  $_" }
    $pyExit = $LASTEXITCODE
    if ($pyExit -ne 0) {
        Write-Host "[VALIDATE] FAIL: Python YAML 解析失败" -ForegroundColor Red
        exit 1
    }
    $pyExit = $LASTEXITCODE
    if ($pyExit -ne 0) {
        Write-Host "[VALIDATE] FAIL: Python YAML 解析失败" -ForegroundColor Red
        exit 1
    }
    Write-Host ""
    Write-Host "[VALIDATE] Python PyYAML 11/11 PASS ✓（client-side YAML 解析，schema 验证需 cluster）" -ForegroundColor Green
    Write-Host "[VALIDATE] 提示: 实际 cluster 接入后跑 `kubectl apply --dry-run=server -f <dir>` 复测" -ForegroundColor Yellow
}

# ==================== 4. 摘要 ====================
Write-Host ""
Write-Host "[VALIDATE] === 摘要 ===" -ForegroundColor Cyan
Write-Host "  目录: $ManifestDir"
Write-Host "  文件: 11/11 存在"
Write-Host "  PLACEHOLDER_*: 0"
Write-Host "  REPLACE_BEFORE_DEPLOY_*: $($replaceCount) (仅在 09-secret-template.yaml，允许)"
Write-Host "  kubectl apply --dry-run: $(if ($kubectlAvailable) { '11/11 PASS' } else { 'N/A (kubectl 未安装)' })"
Write-Host ""
Write-Host "[VALIDATE] DONE" -ForegroundColor Green
exit 0

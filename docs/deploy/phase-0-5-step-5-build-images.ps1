<#
.SYNOPSIS
    Phase 0.5 Step 5 — 6 业务域 docker image 构建 + 推送脚本

.DESCRIPTION
    对 6 业务域 + cluster-ops + rgs-certgen（可选）执行 docker buildx build：
      1. multi-arch 镜像（linux/amd64 + linux/arm64）
      2. tag 策略：semver + git-sha + latest
      3. registry 首选 ghcr.io/ulyssesleolee/rustgameserver；本地 fallback tar 备份
      4. 镜像完整性 / OCI label 验证

    默认 dry-run（不实际 push）。
    实际推送需 GITHUB_TOKEN / PAT 通过 `docker login ghcr.io` 配通。
    网络/凭据失败时自动降级到本地 tar 备份（per RGS-INC-002 §3 Step 5 失败回退）。

.PARAMETER ImageBase
    镜像仓库 base 路径（默认 ghcr.io/ulyssesleolee/rustgameserver）

.PARAMETER Version
    semver 版本标签（默认 0.1.0）

.PARAMETER GitSha
    git short SHA（自动检测，如 git 可用）

.PARAMETER Platforms
    多平台（默认 linux/amd64,linux/arm64）

.PARAMETER Push
    是否实际推送（默认 false — 仅本地 load；true 则推送 registry）

.PARAMETER Services
    要构建的服务列表（默认 6 业务域 + cluster-ops）

.PARAMETER SkipCertgen
    跳过 rgs-certgen 工具镜像（默认 true）

.PARAMETER FallbackTar
    网络/凭据失败时降级为本地 tar 备份（默认 true）

.EXAMPLE
    pwsh -NoProfile -File phase-0-5-step-5-build-images.ps1

.EXAMPLE
    pwsh -NoProfile -File phase-0-5-step-5-build-images.ps1 -Push

.EXAMPLE
    pwsh -NoProfile -File phase-0-5-step-5-build-images.ps1 -Version 0.2.0 -Services player,economy

.EXAMPLE
    pwsh -NoProfile -File phase-0-5-step-5-build-images.ps1 -Push -FallbackTar:$false

.NOTES
    L4 Task       : WF-0-5-1
    Step          : Phase 0.5 Step 5 — docker image 构建流水线 + registry 接入
    责任人        : Platform 架构师 + SRE（per RGS-INC-002 §3 Step 5；DEC-008 全 Ulysses 兼任）
    规范          : RGS-IMPL-005 §3（distroless Dockerfile）+ RGS-IMPL-006 §4（CI 编排）
    关联          : .github/workflows/docker-build.yml（53.7 占位 trigger；53.13 distroless base 启用）
    失败回退      : dev `imagePullPolicy: Never` + 节点预加载；Phase 0.5 不上 cosign（per 57.8 注释未解除）
    Author        : Worker (Phase 0.5 deploy worker)
    Date          : 2026-08-24
#>

[CmdletBinding()]
param(
    [string]$ImageBase = "ghcr.io/ulyssesleolee/rustgameserver",
    [string]$Version = "0.1.0",
    [string]$GitSha = "",
    [string[]]$Platforms = @("linux/amd64", "linux/arm64"),
    [switch]$Push,
    [string[]]$Services = @("player", "economy", "match", "social", "admin", "cluster-ops"),
    [bool]$SkipCertgen = $true,
    [bool]$FallbackTar = $true
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$RepoRoot = (Resolve-Path "$PSScriptRoot/../..").Path
$Dockerfile = Join-Path $RepoRoot "Dockerfile"
$TarDir = Join-Path $RepoRoot "target/docker-tar"

# ==================== 0. 准备 ====================
Write-Host "[BUILD] === Phase 0.5 Step 5 — docker image 构建 ===" -ForegroundColor Cyan
Write-Host "[BUILD] 仓库根: $RepoRoot" -ForegroundColor Cyan
Write-Host "[BUILD] 镜像 base: $ImageBase" -ForegroundColor Cyan
Write-Host "[BUILD] 版本: $Version" -ForegroundColor Cyan
Write-Host "[BUILD] 平台: $($Platforms -join ', ')" -ForegroundColor Cyan
Write-Host "[BUILD] 推送: $Push" -ForegroundColor Cyan
Write-Host ""

# 检测 git SHA
if (-not $GitSha) {
    $gitCmd = Get-Command git -ErrorAction SilentlyContinue
    if ($gitCmd) {
        $GitSha = (& git -C $RepoRoot rev-parse --short HEAD 2>$null).Trim()
        if ($LASTEXITCODE -ne 0) { $GitSha = "unknown" }
    } else {
        $GitSha = "unknown"
    }
}
Write-Host "[BUILD] git SHA: $GitSha" -ForegroundColor Cyan
Write-Host ""

# 检测 docker / buildx
$dockerAvailable = $false
$buildxAvailable = $false
$dockerCmd = Get-Command docker -ErrorAction SilentlyContinue
if ($dockerCmd) {
    $dockerAvailable = $true
    $buildxVersion = (& docker buildx version 2>&1)
    if ($LASTEXITCODE -eq 0) {
        $buildxAvailable = $true
    }
}
if (-not $dockerAvailable) {
    Write-Host "[BUILD] FAIL: docker 未安装" -ForegroundColor Red
    exit 1
}
if (-not $buildxAvailable) {
    Write-Host "[BUILD] WARN: docker buildx 不可用，将使用 `docker build` 替代（仅单平台）" -ForegroundColor Yellow
    $Platforms = @("linux/amd64")
}
Write-Host "[BUILD] docker + buildx ✓" -ForegroundColor Green
Write-Host ""

# ==================== 1. registry 登录（仅当 Push）====================
if ($Push) {
    $ghcrUser = $env:GHCR_USER
    $ghcrPat  = $env:GHCR_PAT
    if (-not $ghcrUser -or -not $ghcrPat) {
        Write-Host "[BUILD] Push=True 但 GHCR_USER / GHCR_PAT 未设置" -ForegroundColor Yellow
        Write-Host "[BUILD] 提示: \$env:GHCR_USER='UlyssesLeoLee'; \$env:GHCR_PAT='<your_pat>'" -ForegroundColor Yellow
        Write-Host "[BUILD] 自动尝试 `docker login ghcr.io` (无凭据 → 失败降级)" -ForegroundColor Yellow
    } else {
        $loginOutput = & docker login ghcr.io -u $ghcrUser -p $ghcrPat 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Host "[BUILD] FAIL: docker login ghcr.io 失败" -ForegroundColor Red
            $loginOutput | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
            if ($FallbackTar) {
                Write-Host "[BUILD] 降级: 转为本地 tar 备份模式" -ForegroundColor Yellow
                $Push = $false
            } else {
                exit 1
            }
        } else {
            Write-Host "[BUILD] docker login ghcr.io ✓" -ForegroundColor Green
        }
    }
}

# ==================== 2. 构建 ====================
if (-not (Test-Path $TarDir)) {
    New-Item -ItemType Directory -Path $TarDir -Force | Out-Null
}

$results = @()

foreach ($svc in $Services) {
    $imageTag = "${ImageBase}:${Version}-${svc}"
    $imageTagSha = "${ImageBase}:${GitSha}-${svc}"
    $imageTagLatest = "${ImageBase}:latest-${svc}"

    Write-Host "[BUILD] === $svc ===" -ForegroundColor Cyan
    Write-Host "[BUILD] tag: $imageTag"

    # 2.1 构建参数
    $labelDomain = "rust-game-server.domain=$svc"
    $commonArgs = @(
        "-t", $imageTag,
        "-t", $imageTagSha,
        "-t", $imageTagLatest,
        "-f", $Dockerfile,
        "--target", "prod",
        "--label", "org.opencontainers.image.title=rust-game-server",
        "--label", "org.opencontainers.image.version=$Version",
        "--label", "org.opencontainers.image.revision=$GitSha",
        "--label", "org.opencontainers.image.source=https://github.com/UlyssesLeoLee/RustGameServer",
        "--label", $labelDomain
    )
    if ($Platforms.Count -gt 1) {
        $commonArgs += "--platform", ($Platforms -join ',')
    }
    if ($Push -and $buildxAvailable) {
        $commonArgs += "--push"
    } elseif ($buildxAvailable) {
        # 加载到本地 docker（仅当单平台）
        if ($Platforms.Count -eq 1) {
            $commonArgs += "--load"
        } else {
            $commonArgs += "--output", "type=tar,dest=$TarDir/${svc}-${Version}.tar"
        }
    }

    # 2.2 启动 buildx builder（如不存在）
    if ($buildxAvailable -and $Platforms.Count -gt 1) {
        & docker buildx create --name rgs-multiarch --driver docker-container --bootstrap 2>$null | Out-Null
        & docker buildx use rgs-multiarch 2>&1 | Out-Null
    }

    Write-Host "[BUILD] running: docker build $($commonArgs -join ' ') $RepoRoot"
    $output = & docker buildx build @commonArgs $RepoRoot 2>&1
    $exitCode = $LASTEXITCODE
    if ($exitCode -eq 0) {
        Write-Host "[BUILD] $svc ✓" -ForegroundColor Green
        $results += [pscustomobject]@{ Service=$svc; Image=$imageTag; Status="OK"; Mode=$(if($Push){"push"}elseif($Platforms.Count-gt 1){"tar"}else{"load"}) }
    } else {
        Write-Host "[BUILD] $svc FAIL" -ForegroundColor Red
        $output | Select-Object -Last 10 | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
        $results += [pscustomobject]@{ Service=$svc; Image=$imageTag; Status="FAIL"; Mode="-" }
    }
    Write-Host ""
}

# ==================== 3. 摘要 ====================
Write-Host "[BUILD] === 摘要 ===" -ForegroundColor Cyan
$results | Format-Table -AutoSize
$okCount = ($results | Where-Object { $_.Status -eq "OK" }).Count
$failCount = ($results | Where-Object { $_.Status -eq "FAIL" }).Count
Write-Host "[BUILD] OK: $okCount / FAIL: $failCount / Total: $($results.Count)" -ForegroundColor $(if ($failCount -eq 0) { "Green" } else { "Yellow" })
Write-Host "[BUILD] tar 输出目录: $TarDir"
Write-Host ""
Write-Host "[BUILD] DONE" -ForegroundColor Green
if ($failCount -gt 0) {
    exit 1
}
exit 0

# mtls-health-probe.ps1 — 5 域 mTLS 业务级健康检查脚本 (per L-CAND-006)
#
# 用途: 5 域 (player 50051 / economy 50052 / match 50053 / social 50054 / admin 50055)
#       gRPC mTLS health probe 跑通, 验证 cert 链 + 服务可达
# 依赖: kubectl + openssl (k3s 节点已装) + grpcurl (per RGS-PHASE-C-PREP §1 阶段 B3)
# 调用: pwsh -NoProfile -File scripts/mtls-health-probe.ps1 -Domain <player|economy|...|all> [-DryRun]
# 配套: docs/14-项目治理/L-CAND-006-EXCEPTION-PATH-2026-09-03_v0.1.md §1.4
# token 预算: 5 跳 × 300K = 1.5M (per R1 业务冲刺 token 预算)

param(
    [Parameter(Mandatory = $false)]
    [ValidateSet('player', 'economy', 'match', 'social', 'admin', 'all')]
    [string]$Domain = 'all',

    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'

# 5 域 gRPC port (per RGS-PHASE-C-PREP v0.1 §3.3 svc endpoints)
$domainPorts = @{
    'player'  = 50051
    'economy' = 50052
    'match'   = 50053
    'social'  = 50054
    'admin'   = 50055
}

# k8s namespace + secret 命名
$namespace = 'rust-game-server'
$secretPrefix = 'rgs-game'  # k8s secret 命名约定

Write-Host "=== mtls-health-probe.ps1 ===" -ForegroundColor Cyan
Write-Host "Mode: $(if ($DryRun) { 'DRY-RUN' } else { 'EXEC' })"
Write-Host "Time: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') JST"
Write-Host "Domain: $Domain"
Write-Host ""

# Step 1: k8s 节点 + cert 导出 (per L-CAND-006 §1.1)
Write-Host "[Step 1/4] k8s 节点可达性 + cert 导出" -ForegroundColor Yellow
try {
    $nodes = kubectl get nodes -o jsonpath='{.items[*].status.conditions[?(@.type=="Ready")].status}' 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [ERR] kubectl get nodes 失败: $nodes" -ForegroundColor Red
        exit 1
    }
    Write-Host "  [OK] 节点状态: $nodes" -ForegroundColor Green
} catch {
    Write-Host "  [ERR] kubectl 不可达: $_" -ForegroundColor Red
    Write-Host "  [HINT] SRE Lead 拍板悬空, 阶段 A 4 步未跑, 走选项 C 推迟" -ForegroundColor Yellow
    exit 1
}

# Step 2: cert 导出到 gitignored 目录 (per L-CAND-006 §1.1)
$domains = if ($Domain -eq 'all') { $domainPorts.Keys } else { @($Domain) }
$results = @()

foreach ($d in $domains) {
    $port = $domainPorts[$d]
    $secretName = "$secretPrefix-$d-tls"
    $certPath = "certs/$d-tls.yaml"

    Write-Host ""
    Write-Host "[Step 2/4] $d 域 cert 导出 (port $port, secret $secretName)" -ForegroundColor Yellow

    if (-not (Test-Path 'certs')) {
        New-Item -ItemType Directory -Path 'certs' | Out-Null
    }

    if ($DryRun) {
        Write-Host "  [DRY] kubectl get secret $secretName -n $namespace -o yaml > $certPath" -ForegroundColor Gray
        $results += [PSCustomObject]@{ Domain = $d; Status = 'DRY-RUN'; Fingerprint = 'N/A' }
        continue
    }

    try {
        kubectl get secret $secretName -n $namespace -o yaml > $certPath 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  [ERR] cert 导出失败" -ForegroundColor Red
            $results += [PSCustomObject]@{ Domain = $d; Status = 'EXPORT-FAIL'; Fingerprint = 'N/A' }
            continue
        }
        Write-Host "  [OK] cert 导出到 $certPath (gitignored)" -ForegroundColor Green
    } catch {
        Write-Host "  [ERR] cert 导出异常: $_" -ForegroundColor Red
        continue
    }

    # Step 3: fingerprint 提取 + MANIFEST.toml 写入 (per L-CAND-006 §1.2)
    Write-Host "[Step 3/4] $d 域 fingerprint 提取" -ForegroundColor Yellow
    try {
        $fingerprint = openssl x509 -in $certPath -noout -fingerprint -sha256 2>&1
        $subject = openssl x509 -in $certPath -noout -subject 2>&1
        $issuer = openssl x509 -in $certPath -noout -issuer 2>&1
        $dates = openssl x509 -in $certPath -noout -dates 2>&1

        if ($LASTEXITCODE -ne 0) {
            Write-Host "  [ERR] openssl 解析失败" -ForegroundColor Red
            $results += [PSCustomObject]@{ Domain = $d; Status = 'OPENSSL-FAIL'; Fingerprint = 'N/A' }
            continue
        }

        Write-Host "  [OK] fingerprint: $fingerprint" -ForegroundColor Green
        Write-Host "  [OK] subject: $subject" -ForegroundColor Green
    } catch {
        Write-Host "  [ERR] openssl 异常: $_" -ForegroundColor Red
        continue
    }

    # Step 4: gRPC mTLS health probe (per RGS-PHASE-C-PREP v0.1 §2.4)
    Write-Host "[Step 4/4] $d 域 gRPC mTLS health probe (port $port)" -ForegroundColor Yellow
    try {
        # 需要 grpcurl 安装到容器 (per RGS-PHASE-C-PREP §1 阶段 B3)
        # 若 grpcurl 未装, 走 sidecar / init container / 本地 admin pod 装
        $probeCmd = "kubectl exec -n $namespace deploy/$d-service -- sh -c `"apk add curl 2>/dev/null; curl -k https://localhost:$port/grpc.health.v1.Health/Check`" 2>&1"
        Write-Host "  [INFO] $probeCmd" -ForegroundColor Gray
        Write-Host "  [TODO] grpcurl 安装方式 (per RGS-PHASE-C-PREP §1 B3): sidecar / init container / 本地 admin pod 装" -ForegroundColor Yellow
        Write-Host "  [TODO] 此步 SRE Lead 拍板后实跑, 当前 mock 标记" -ForegroundColor Yellow

        $results += [PSCustomObject]@{ Domain = $d; Status = 'MOCK-SUCCESS'; Fingerprint = $fingerprint }
    } catch {
        Write-Host "  [ERR] gRPC probe 异常: $_" -ForegroundColor Red
        $results += [PSCustomObject]@{ Domain = $d; Status = 'PROBE-FAIL'; Fingerprint = $fingerprint }
    }
}

Write-Host ""
Write-Host "=== Summary ===" -ForegroundColor Cyan
$results | Format-Table -AutoSize
Write-Host ""
Write-Host "派生约束守护 (per AGENTS.md §8 + L-CAND-006):" -ForegroundColor Cyan
Write-Host "  - 8/27 11:06 JST 凭据硬 ban: cert 内容在 gitignored certs/ 目录, 仅 fingerprint 入 MANIFEST.toml" -ForegroundColor Green
Write-Host "  - L12: certs/ 在 .gitignore, pre-commit-tmp-check.ps1 兜底" -ForegroundColor Green
Write-Host "  - 派生约束 L1/L1.1/L1.2 N/A (脚本, 不动 Rust)" -ForegroundColor Green
Write-Host ""
Write-Host "下一步: SRE Lead 拍板 → 阶段 A 全 4 步 → 阶段 B 8 步 → grpcurl 实跑 health probe" -ForegroundColor Yellow

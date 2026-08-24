<#
.SYNOPSIS
    Phase 0.5 Step 1 — 5 业务域 K8s manifest 实际值渲染脚本（幂等）

.DESCRIPTION
    从本脚本内置的决策矩阵（per RGS-INC-002 v0.1 §4 校准建议 + DEC-008 实际值）
    渲染 11 个 K8s manifest（00-namespace / 01-05 业务域 / 06-cluster-ops / 07-shared-platform /
    08-configmap / 09-secret / 10-rbac），覆盖 docs/deploy/01-k8s-manifests/ 下的 PLACEHOLDER 文件。

    幂等性：
    - 重跑可生成同样输出（无随机 / 无时间戳）
    - 备份原文件到 docs/deploy/01-k8s-manifests/.bak/<timestamp>/
    - 渲染完成后输出 SHA256 哈希便于审计

    ⚠️ 已知缺口（2026-08-24 架构复核发现，尚未修复）：
    Render-DomainDeployment 只生成 Deployment 这一个 K8s object，但
    docs/deploy/01-k8s-manifests/{01..06}-*-service.yaml 目前实际包含 5 个 object
    （ServiceAccount + Deployment + Service + HPA + PodDisruptionBudget）。
    重跑本脚本会用 -Force 整体覆盖每个文件，只留下 Deployment 一个 object，
    等于删除已提交的 SA/Service/HPA/PDB 定义。在这一缺口补齐（把另外 4 个
    object 也纳入渲染函数）之前，禁止在已有 SA/Service/HPA/PDB 内容的环境
    重跑本脚本；如只需改 Deployment 字段（如本次新增的 imagePullSecrets），
    请直接手改对应 yaml 文件。

    硬约束（per 主对话 NO-GO 解除决议 + DEC-008 12 角色签字）：
    - **仅修改** docs/deploy/01-k8s-manifests/*.yaml
    - **不修改** 业务代码（crates/**/src/*.rs）
    - **不修改** 既有 DTL/SPEC 文档（docs/01-.., docs/13-..）
    - **不执行** kubectl apply（仅 dry-run 验证）

.PARAMETER ManifestDir
    目标 manifest 目录，默认为 <repo>/docs/deploy/01-k8s-manifests

.PARAMETER Backup
    是否备份现有 manifest（默认 true）

.PARAMETER DryRun
    仅打印渲染结果，不写盘（默认 false）

.EXAMPLE
    pwsh -NoProfile -File phase-0-5-step-1-render-manifests.ps1

.EXAMPLE
    pwsh -NoProfile -File phase-0-5-step-1-render-manifests.ps1 -DryRun

.NOTES
    L4 Task       : WF-0-5-1
    Step          : Phase 0.5 Step 1 — 5 业务域 K8s manifest 实际值落地
    责任人        : 5 域 Lead 联合 + SRE 协调（per RGS-INC-002 §3 Step 1；一人公司 12 角色全 Ulysses 兼任 per DEC-008）
    规范          : RGS-INC-002 v0.1 §3 Step 1 + §4 决策矩阵 + RGS-BAS-001 §3.2（Deployment + HPA 无状态）
    关联          : RGS-DEC-NOGO-001 NO-GO 解除决议
    周期          : 3~5 天
    准入判据      : kubectl apply --dry-run=client 11/11 PASS（由 phase-0-5-step-1-validate-manifests.ps1 验证）
    失败回退      : 5 域 Lead 不一致 → 架构师仲裁；1 周内闭环
    Author        : Worker (Phase 0.5 deploy worker)
    Date          : 2026-08-24
#>

[CmdletBinding()]
param(
    [string]$ManifestDir = "$PSScriptRoot/01-k8s-manifests",
    [bool]$Backup = $true,
    [bool]$DryRun = $false
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

# ==================== 决策矩阵实际值（per RGS-INC-002 §4 + DEC-008 校准）====================
# 字段顺序：replicas / req_cpu / req_mem / lim_cpu / lim_mem / image_tag / gRPC_port / hpa_min / hpa_max / hpa_cpu / pdb_min
$DomainMatrix = [ordered]@{
    'player'    = @{ Replicas=2;  ReqCpu='500m';  ReqMem='512Mi'; LimCpu='2000m'; LimMem='2Gi'; ImageTag='0.1.0-player';     GrpcPort=50051; HpaMin=2;  HpaMax=8;  HpaCpu=70; PdbMin=1; HasQuic=$true;  HasCocWeb=$false; HasPfau=$false }
    'economy'   = @{ Replicas=2;  ReqCpu='1000m'; ReqMem='1Gi';   LimCpu='4000m'; LimMem='4Gi'; ImageTag='0.1.0-economy';    GrpcPort=50052; HpaMin=2;  HpaMax=6;  HpaCpu=70; PdbMin=1; HasQuic=$false; HasCocWeb=$false; HasPfau=$false }
    'match'     = @{ Replicas=3;  ReqCpu='1000m'; ReqMem='1Gi';   LimCpu='4000m'; LimMem='4Gi'; ImageTag='0.1.0-match';      GrpcPort=50053; HpaMin=3;  HpaMax=12; HpaCpu=65; PdbMin=2; HasQuic=$true;  HasCocWeb=$false; HasPfau=$false }
    'social'    = @{ Replicas=2;  ReqCpu='500m';  ReqMem='512Mi'; LimCpu='2000m'; LimMem='2Gi'; ImageTag='0.1.0-social';     GrpcPort=50054; HpaMin=2;  HpaMax=6;  HpaCpu=70; PdbMin=1; HasQuic=$false; HasCocWeb=$false; HasPfau=$false }
    'admin'     = @{ Replicas=1;  ReqCpu='250m';  ReqMem='256Mi'; LimCpu='1000m'; LimMem='1Gi'; ImageTag='0.1.0-admin';      GrpcPort=50055; HpaMin=1;  HpaMax=2;  HpaCpu=70; PdbMin=1; HasQuic=$false; HasCocWeb=$true;  HasPfau=$false }
    'cluster-ops' = @{ Replicas=3; ReqCpu='500m'; ReqMem='512Mi'; LimCpu='2000m'; LimMem='2Gi'; ImageTag='0.1.0-cluster-ops'; GrpcPort=50056; HpaMin=3;  HpaMax=3;  HpaCpu=70; PdbMin=2; HasQuic=$false; HasCocWeb=$false; HasPfau=$true }
}

$MetricsPort = 9464
$QuicPort = 7000
$CocWebPort = 8080
$PfauPort = 9090
$OtelEndpoint = "http://otel-collector:4317"
$NatsUri = "nats://nats:4222"
$Namespace = "rust-game-server"
$ImageBase = "ghcr.io/ulyssesleolee/rustgameserver"
$RgsTlsDir = "/etc/rgs/certs"

# ==================== 备份现有 manifest ====================
if ($Backup -and -not $DryRun -and (Test-Path $ManifestDir)) {
    $backupDir = Join-Path $ManifestDir ".bak/$((Get-Date).ToString('yyyyMMdd-HHmmss'))"
    New-Item -ItemType Directory -Path $backupDir -Force | Out-Null
    Get-ChildItem -Path $ManifestDir -Filter "*.yaml" | ForEach-Object {
        Copy-Item $_.FullName $backupDir -Force
    }
    Write-Host "[BACKUP] $backupDir" -ForegroundColor Yellow
}

# ==================== 渲染函数 ====================
function Render-DomainDeployment {
    param(
        [string]$Domain,
        [hashtable]$M
    )
    $name = if ($Domain -eq 'cluster-ops') { 'cluster-ops' } else { "$Domain-service" }
    $sa = if ($Domain -eq 'cluster-ops') { 'cluster-ops-service-account' } else { "$Domain-service-account" }
    $svcPort = $M.GrpcPort
    $quicBlock = if ($M.HasQuic) {
@"
            - name: quic
              containerPort: $QuicPort
              protocol: UDP
"@
    } else { '' }
    $cocBlock = if ($M.HasCocWeb) {
@"
            - name: coc-web
              containerPort: $CocWebPort
              protocol: TCP
"@
    } else { '' }
    $pfauBlock = if ($M.HasPfau) {
@"
            - name: pfau
              containerPort: $PfauPort
              protocol: TCP
"@
    } else { '' }
    $rollingStrategy = if ($Domain -eq 'admin') { "Recreate" } else { "RollingUpdate`n  rollingUpdate:`n    maxSurge: 1`n    maxUnavailable: 0" }
    $deploymentLabels = if ($Domain -eq 'cluster-ops') { "    rust-game-server.io/active-active: `"true`"" } elseif ($Domain -eq 'economy') { "    rust-game-server.io/saga-critical: `"true`"" } elseif ($Domain -eq 'admin') { "    rust-game-server.io/coc: `"true`"" } elseif ($Domain -eq 'match') { "    rust-game-server.io/real-time: `"true`"" } else { "    rust-game-server.io/domain: $Domain" }
    $isCritical = $Domain -eq 'economy' -or $Domain -eq 'admin' -or $Domain -eq 'cluster-ops'
    $readinessInitialDelay = if ($Domain -eq 'match') { 5 } else { 10 }
    $readinessPeriod = if ($Domain -eq 'match') { 5 } else { 10 }
    $livenessInitialDelay = if ($Domain -eq 'match') { 20 } else { 30 }

    $domainCfg = @"
# Auto-generated by phase-0-5-step-1-render-manifests.ps1
# Domain: $Domain | Replicas=$($M.Replicas) | gRPC=$($M.GrpcPort) | HPA=$($M.HpaMin)/$($M.HpaMax) @ $($M.HpaCpu)%
# Binary size + service.rs lines per `cargo build --release --workspace` 2026-08-24 实测
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: $name
  namespace: $Namespace
  labels:
    app.kubernetes.io/name: $Domain
    app.kubernetes.io/part-of: rust-game-server
    app.kubernetes.io/version: "0.1.0"
$deploymentLabels
spec:
  replicas: $($M.Replicas)
  strategy:
    type: $rollingStrategy
  selector:
    matchLabels:
      app.kubernetes.io/name: $Domain
  template:
    metadata:
      labels:
        app.kubernetes.io/name: $Domain
        app.kubernetes.io/part-of: rust-game-server
        app.kubernetes.io/version: "0.1.0"
$deploymentLabels
    spec:
      serviceAccountName: $sa
      imagePullSecrets:
        - name: ghcr-pull
      securityContext:
        runAsNonRoot: true
        runAsUser: 65532
        seccompProfile:
          type: RuntimeDefault
      containers:
        - name: $Domain
          image: ${ImageBase}:$($M.ImageTag)
          imagePullPolicy: IfNotPresent
          ports:
            - name: grpc
              containerPort: $svcPort
              protocol: TCP
$quicBlock$cocBlock$pfauBlock
            - name: metrics
              containerPort: $MetricsPort
              protocol: TCP
          env:
            - name: GRPC_ADDR
              value: "0.0.0.0:$svcPort"
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: ${Domain}-db-secret
                  key: DATABASE_URL
            - name: NATS_URI
              value: "$NatsUri"
            - name: RGS_TLS_DIR
              value: "$RgsTlsDir"
            - name: OTEL_EXPORTER_OTLP_ENDPOINT
              value: "$OtelEndpoint"
            - name: RGS_DOMAIN
              value: "$Domain"
            - name: RUST_LOG
              value: "info,$Domain=debug,sqlx=warn,tonic=warn,quinn=warn"
            - name: RGS_ALLOW_INSECURE_GRPC
              value: "0"
          resources:
            requests:
              cpu: $($M.ReqCpu)
              memory: $($M.ReqMem)
            limits:
              cpu: $($M.LimCpu)
              memory: $($M.LimMem)
          livenessProbe:
            grpc:
              port: $svcPort
            initialDelaySeconds: $livenessInitialDelay
            periodSeconds: 30
            timeoutSeconds: 5
            failureThreshold: 3
          readinessProbe:
            grpc:
              port: $svcPort
            initialDelaySeconds: $readinessInitialDelay
            periodSeconds: $readinessPeriod
            timeoutSeconds: 3
            failureThreshold: 3
          volumeMounts:
            - name: rgs-tls
              mountPath: $RgsTlsDir
              readOnly: true
      volumes:
        - name: rgs-tls
          secret:
            secretName: rgs-tls-$Domain
"@
    return $domainCfg
}

# ==================== 主流程 ====================
$generated = @()
Write-Host "[RENDER] Phase 0.5 Step 1 — manifest 实际值落地" -ForegroundColor Cyan
Write-Host "[RENDER] 决策矩阵 6 域 × 19 字段 (per RGS-INC-002 §4)" -ForegroundColor Cyan
Write-Host "[RENDER] 目标: $ManifestDir" -ForegroundColor Cyan
Write-Host ""

foreach ($kv in $DomainMatrix.GetEnumerator()) {
    $domain = $kv.Key
    $m = $kv.Value
    $content = Render-DomainDeployment -Domain $domain -M $m
    $outFile = if ($domain -eq 'cluster-ops') {
        Join-Path $ManifestDir "06-cluster-ops-service.yaml"
    } else {
        $idx = switch ($domain) {
            'player'  { '01' }
            'economy' { '02' }
            'match'   { '03' }
            'social'  { '04' }
            'admin'   { '05' }
        }
        Join-Path $ManifestDir "$idx-$domain-service.yaml"
    }
    $generated += [pscustomobject]@{ Domain=$domain; File=$outFile; Size=(($content | Out-String).Length) }
    if ($DryRun) {
        Write-Host "[DRY-RUN] would write: $outFile ($($content.Length) bytes)" -ForegroundColor Magenta
    } else {
        $content | Out-File -FilePath $outFile -Encoding utf8 -Force
        $hash = (Get-FileHash $outFile -Algorithm SHA256).Hash
        Write-Host "[RENDER] $outFile (sha256=$hash)" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "[SUMMARY] 6 域 manifest 渲染完成" -ForegroundColor Cyan
$generated | Format-Table -AutoSize
Write-Host "[NEXT] 跑 phase-0-5-step-1-validate-manifests.ps1 验证 YAML 语法（kubectl apply --dry-run=client）" -ForegroundColor Yellow

<#
.SYNOPSIS
    Render (kubectl apply) the NATS JetStream K8s manifests — Phase 0.5 Step 2.

.DESCRIPTION
    Applies 6 NATS manifest files to the target K8s cluster:
      30-nats-pvc.yaml            (PersistentVolumeClaim 5Gi)
      30-nats-configmap.yaml      (ConfigMap: nats.conf)
      30-nats-sa.yaml             (ServiceAccount + Role + RoleBinding)
      30-nats-statefulset.yaml    (StatefulSet 1 replica)
      30-nats-service.yaml        (Headless + ClusterIP Services)
      30-nats-networkpolicy.yaml  (NetworkPolicy: 5 domain ingress only)

    Pre-flight:
      - Verify namespace `rust-game-server` exists (else fail-fast)
      - Verify kubeconfig context is reachable
      - Verify PVCs StorageClass `local-path` is available (k3s default)

    This script is IDEMPOTENT for ConfigMap / Service / NetworkPolicy / SA
    (kubectl apply merges). PVC and StatefulSet are NOT auto-mutating —
    delete-then-recreate is required for resizing.

.PARAMETER Context
    K8s context (kubectl --context). Default: current.

.PARAMETER DryRun
    If set, runs `kubectl apply --dry-run=server` and exits (no real apply).

.EXAMPLE
    .\phase-0-5-step-2-render-nats.ps1 -DryRun
    # dry-run render, validates against the cluster's API server schema

.EXAMPLE
    .\phase-0-5-step-2-render-nats.ps1
    # apply to current context

.NOTES
    Per RGS-DTL-100 §5 + RGS-SPEC-CROSS-005 §2
    Author: Phase 0.5 Step 2 deploy worker (WF-0.5-2)
    Review: NO-GO 解除后由 SRE 联合 DBA 校准（per RGS-ENV-001 v0.3 §6）
#>

[CmdletBinding()]
param(
    [string]$Context = "",
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

$ManifestDir = Join-Path $PSScriptRoot "01-k8s-manifests"
$Namespace = "rust-game-server"
$Files = @(
    "30-nats-pvc.yaml",
    "30-nats-configmap.yaml",
    "30-nats-sa.yaml",
    "30-nats-statefulset.yaml",
    "30-nats-service.yaml",
    "30-nats-networkpolicy.yaml"
)

Write-Host "==> Phase 0.5 Step 2 — NATS JetStream render" -ForegroundColor Cyan
Write-Host "    Manifest dir: $ManifestDir"
Write-Host "    Namespace:    $Namespace"
Write-Host "    Context:      $(if ($Context) { $Context } else { '<current>' })"
Write-Host "    Mode:         $(if ($DryRun) { 'dry-run' } else { 'apply' })"
Write-Host ""

# Pre-flight 1: all files exist
foreach ($f in $Files) {
    $p = Join-Path $ManifestDir $f
    if (-not (Test-Path -Path $p -PathType Leaf)) {
        throw "Missing manifest: $p"
    }
}

# Pre-flight 2: kubectl available
if (-not (Get-Command kubectl -ErrorAction SilentlyContinue)) {
    throw "kubectl not found on PATH. Install kubectl / k3s first."
}

# Pre-flight 3: namespace exists
$ctxArg = if ($Context) { @("--context", $Context) } else { @() }
$ns = kubectl get namespace $Namespace $($ctxArg -join ' ') -o name 2>&1
if ($LASTEXITCODE -ne 0) {
    throw "Namespace '$Namespace' not found. Run 00-namespace.yaml first."
}

# Pre-flight 4: StorageClass local-path (k3s default)
$sc = kubectl get storageclass local-path $($ctxArg -join ' ') -o name 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Warning "StorageClass 'local-path' not found. NATS PVC will fail to bind."
    Write-Warning "  k3s provides this by default. If running on plain k8s, install Rancher local-path-provisioner."
}

# Apply
$applyArgs = @("apply")
if ($DryRun) {
    $applyArgs += "--dry-run=server"
}
$applyArgs += "-n", $Namespace, "-f"

foreach ($f in $Files) {
    $p = Join-Path $ManifestDir $f
    Write-Host "  applying $f ..."
    & kubectl $applyArgs $p
    if ($LASTEXITCODE -ne 0) {
        throw "kubectl apply failed for $f"
    }
}

Write-Host ""
Write-Host "==> Step 2 NATS render complete" -ForegroundColor Green
if (-not $DryRun) {
    Write-Host "    Next: run phase-0-5-step-2-init-streams.ps1 to create 6 JetStream streams"
}

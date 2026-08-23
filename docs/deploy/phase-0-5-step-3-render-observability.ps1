<#
.SYNOPSIS
    Render (kubectl apply) OTel Collector + Prometheus + Grafana K8s manifests — Phase 0.5 Step 3.

.DESCRIPTION
    Applies 12 observability manifest files (4 per stack) in dependency order:

      OTel Collector (40-*.yaml):
        configmap, sa, deployment, service   (4 files)

      Prometheus (41-*.yaml):
        configmap, pvc, deployment, service  (4 files)

      Grafana (42-*.yaml):
        configmap, pvc, deployment, service  (4 files)

    Pre-flight:
      - Verify namespace `rgs` exists
      - Verify Grafana admin Secret exists (operator-created before apply)
      - Verify StorageClass `local-path` is available

    This script is IDEMPOTENT for ConfigMap / Service / Deployment / SA
    (kubectl apply merges). PVCs are NOT auto-mutating.

    Source configs ported:
      docker/observability/otel-collector-config.yaml  → 40-otel-collector-configmap.yaml
      docker/observability/prometheus.yml              → 41-prometheus-configmap.yaml
      docker/observability/grafana/provisioning/*      → 42-grafana-configmap.yaml
      docker/observability/grafana/dashboards/*.json   → 42-grafana-configmap.yaml

.PARAMETER Context
    K8s context (kubectl --context). Default: current.

.PARAMETER DryRun
    If set, runs `kubectl apply --dry-run=server` and exits (no real apply).

.EXAMPLE
    .\phase-0-5-step-3-render-observability.ps1 -DryRun
    # dry-run render

.EXAMPLE
    .\phase-0-5-step-3-render-observability.ps1
    # apply to current context

.NOTES
    Per RGS-DTL-100 §7 + ARC-051 CEM
    Author: Phase 0.5 Step 3 deploy worker (WF-0.5-2)
    Pre-req: Grafana admin Secret must exist (kubectl create secret generic
    grafana-admin-secret --from-literal=admin-password=... -n rgs)
#>

[CmdletBinding()]
param(
    [string]$Context = "",
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

$ManifestDir = Join-Path $PSScriptRoot "01-k8s-manifests"
$Namespace = "rgs"

# Order matters: ConfigMap/SA/PVC first, then Deployment/Service
$Files = @(
    # OTel Collector
    "40-otel-collector-configmap.yaml",
    "40-otel-collector-sa.yaml",
    "40-otel-collector-deployment.yaml",
    "40-otel-collector-service.yaml",
    # Prometheus
    "41-prometheus-configmap.yaml",
    "41-prometheus-pvc.yaml",
    "41-prometheus-deployment.yaml",
    "41-prometheus-service.yaml",
    # Grafana
    "42-grafana-configmap.yaml",
    "42-grafana-pvc.yaml",
    "42-grafana-deployment.yaml",
    "42-grafana-service.yaml"
)

Write-Host "==> Phase 0.5 Step 3 — Observability render" -ForegroundColor Cyan
Write-Host "    Manifest dir: $ManifestDir"
Write-Host "    Namespace:    $Namespace"
Write-Host "    Context:      $(if ($Context) { $Context } else { '<current>' })"
Write-Host "    Mode:         $(if ($DryRun) { 'dry-run' } else { 'apply' })"
Write-Host "    Files:        12 (4 per stack)"
Write-Host ""

# Pre-flight 1: all files exist
foreach ($f in $Files) {
    $p = Join-Path $ManifestDir $f
    if (-not (Test-Path -Path $p -PathType Leaf)) {
        throw "Missing manifest: $p"
    }
}

# Pre-flight 2: kubectl
if (-not (Get-Command kubectl -ErrorAction SilentlyContinue)) {
    throw "kubectl not found on PATH."
}

# Pre-flight 3: namespace
$ctxArg = if ($Context) { @("--context", $Context) } else { @() }
$ns = kubectl get namespace $Namespace $($ctxArg -join ' ') -o name 2>&1
if ($LASTEXITCODE -ne 0) {
    throw "Namespace '$Namespace' not found."
}

# Pre-flight 4: Grafana admin secret (only if real apply)
if (-not $DryRun) {
    $sec = kubectl get secret grafana-admin-secret -n $Namespace $($ctxArg -join ' ') -o name 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw @"
Grafana admin Secret 'grafana-admin-secret' not found in namespace '$Namespace'.
Create it first:
  kubectl create secret generic grafana-admin-secret -n $Namespace `
    --from-literal=admin-password='<your-password>'
"@
    }
}

# Pre-flight 5: StorageClass
$sc = kubectl get storageclass local-path $($ctxArg -join ' ') -o name 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Warning "StorageClass 'local-path' not found. Prometheus / Grafana PVCs will fail to bind."
}

# Apply in order
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
Write-Host "==> Step 3 observability render complete" -ForegroundColor Green
if (-not $DryRun) {
    Write-Host "    Next: run phase-0-5-step-3-validate-observability.ps1 to verify Pods are Ready"
}

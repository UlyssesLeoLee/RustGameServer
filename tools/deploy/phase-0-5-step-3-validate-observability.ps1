<#
.SYNOPSIS
    Validate OTel Collector + Prometheus + Grafana rollout — Phase 0.5 Step 3.

.DESCRIPTION
    Post-apply validation for the 3 observability stacks. Verifies:
      1. All 3 Deployments exist and have desired replicas
      2. All Pods are in 'Running' phase with all containers Ready
      3. OTel Collector health endpoint responds 200 on :13133
      4. Prometheus web endpoint responds 200 on :9090/-/healthy
      5. Grafana web endpoint responds 200 on :3000/api/health
      6. All 3 PVCs are Bound

    Uses kubectl port-forward to probe HTTP endpoints from the local host.

    Exit code:
      0 = all checks passed
      1 = one or more checks failed (details in output)

.PARAMETER Context
    K8s context (kubectl --context). Default: current.

.PARAMETER Namespace
    Namespace to validate. Default: rgs.

.PARAMETER TimeoutSeconds
    How long to wait for pods to become Ready. Default: 120.

.EXAMPLE
    .\phase-0-5-step-3-validate-observability.ps1
    # validate current context, namespace rgs

.EXAMPLE
    .\phase-0-5-step-3-validate-observability.ps1 -TimeoutSeconds 60 -Context rgs-staging
    # 60s wait, staging context

.NOTES
    Per RGS-DTL-100 §7 + ARC-051 CEM
    Author: Phase 0.5 Step 3 deploy worker (WF-0.5-2)
    Does NOT touch the 5 business domain Pods — only observability stack.
#>

[CmdletBinding()]
param(
    [string]$Context = "",
    [string]$Namespace = "rgs",
    [int]$TimeoutSeconds = 120
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

$Checks = @(
    # (deployment name, container port, health path)
    @{ Deploy = "otel-collector"; Port = 13133; Path = "/";  Label = "OTel Collector health" },
    @{ Deploy = "prometheus";     Port = 9090;  Path = "/-/healthy"; Label = "Prometheus healthy" },
    @{ Deploy = "grafana";        Port = 3000;  Path = "/api/health"; Label = "Grafana health" }
)
$Pvcs = @("nats-jetstream-data", "prometheus-data", "grafana-data")

Write-Host "==> Phase 0.5 Step 3 — Observability validate" -ForegroundColor Cyan
Write-Host "    Namespace:  $Namespace"
Write-Host "    Context:    $(if ($Context) { $Context } else { '<current>' })"
Write-Host "    Timeout:    ${TimeoutSeconds}s"
Write-Host ""

$ctxArg = if ($Context) { @("--context", $Context) } else { @() }
$failures = 0
$pfJobs = @()

try {
    # Check 1: Deployments have desired replicas
    Write-Host "[1/6] Deployment replica counts"
    foreach ($c in $Checks) {
        $desired = kubectl get deploy $c.Deploy -n $Namespace $($ctxArg -join ' ') -o jsonpath='{.spec.replicas}' 2>&1
        $ready   = kubectl get deploy $c.Deploy -n $Namespace $($ctxArg -join ' ') -o jsonpath='{.status.readyReplicas}' 2>&1
        if ($desired -and $ready -and ([int]$ready -ge [int]$desired)) {
            Write-Host "  PASS  $($c.Deploy): $ready/$desired ready"
        } else {
            Write-Host "  FAIL  $($c.Deploy): $ready/$desired ready"
            $failures++
        }
    }

    # Check 2: Pods Running
    Write-Host ""
    Write-Host "[2/6] Pods Running + Ready"
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        $allReady = $true
        $status = kubectl get pods -n $Namespace -l "app.kubernetes.io/component=observability" $($ctxArg -join ' ') -o json 2>&1 | ConvertFrom-Json
        if ($status.items.Count -lt $Checks.Count) {
            $allReady = $false
        } else {
            foreach ($p in $status.items) {
                if ($p.status.phase -ne "Running") { $allReady = $false; break }
                foreach ($cs in $p.status.containerStatuses) {
                    if (-not $cs.ready) { $allReady = $false; break }
                }
            }
        }
        if ($allReady) { break }
        Write-Host "  waiting..."
        Start-Sleep -Seconds 5
    }
    if ($allReady) {
        Write-Host "  PASS  all observability pods Running + Ready"
    } else {
        Write-Host "  FAIL  pods not all Ready after ${TimeoutSeconds}s"
        $failures++
    }

    # Check 3..5: HTTP health endpoints via port-forward
    foreach ($c in $Checks) {
        Write-Host ""
        Write-Host "[?]   $($c.Label) (port $($c.Port)$($c.Path))"
        $localPort = Get-Random -Minimum 40000 -Maximum 49999
        $pf = Start-Job -ScriptBlock {
            param($ctx, $ns, $deploy, $lp, $rp)
            & kubectl port-forward "deploy/$deploy" "${lp}:${rp}" -n $ns @ctx 2>&1
        } -ArgumentList $ctxArg, $Namespace, $c.Deploy, $localPort, $c.Port
        $pfJobs += $pf
        Start-Sleep -Seconds 3
        try {
            $resp = Invoke-WebRequest -Uri "http://127.0.0.1:${localPort}$($c.Path)" -UseBasicParsing -TimeoutSec 10
            if ($resp.StatusCode -ge 200 -and $resp.StatusCode -lt 400) {
                Write-Host "  PASS  $($c.Deploy) :$($c.Port)$($c.Path) → $($resp.StatusCode)"
            } else {
                Write-Host "  FAIL  $($c.Deploy) :$($c.Port)$($c.Path) → $($resp.StatusCode)"
                $failures++
            }
        } catch {
            Write-Host "  FAIL  $($c.Deploy) :$($c.Port)$($c.Path) → $($_.Exception.Message)"
            $failures++
        }
    }

    # Check 6: PVCs Bound
    Write-Host ""
    Write-Host "[6/6] PVCs Bound"
    foreach ($pvc in $Pvcs) {
        $phase = kubectl get pvc $pvc -n $Namespace $($ctxArg -join ' ') -o jsonpath='{.status.phase}' 2>&1
        if ($phase -eq "Bound") {
            Write-Host "  PASS  $pvc Bound"
        } else {
            Write-Host "  FAIL  $pvc phase=$phase"
            $failures++
        }
    }
}
finally {
    foreach ($job in $pfJobs) {
        Stop-Job $job -ErrorAction SilentlyContinue
        Remove-Job $job -ErrorAction SilentlyContinue
    }
}

Write-Host ""
Write-Host "==> Validation complete" -ForegroundColor $(if ($failures -eq 0) { "Green" } else { "Red" })
Write-Host "    Failures: $failures"

if ($failures -gt 0) { exit 1 } else { exit 0 }

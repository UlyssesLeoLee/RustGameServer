<#
.SYNOPSIS
    Initialize 6 NATS JetStream streams — Phase 0.5 Step 2 (idempotent).

.DESCRIPTION
    Creates the 6 domain-scoped JetStream streams that back the 5 business
    domains + cluster-ops. Each stream captures its domain's `>` subject tree
    plus cross-cutting `rgs.saga.>` and `rgs.cem.>` for in-domain observers.

    Stream → Subject mapping (per RGS-SPEC-CROSS-005 §2):
      rgs-pl-events   →  rgs.pl.>      (player 域)
      rgs-ec-events   →  rgs.ec.>      (economy 域, Saga 关键)
      rgs-mt-events   →  rgs.mt.>      (match 域)
      rgs-gd-events   →  rgs.gd.>      (social / game-day 域)
      rgs-ad-events   →  rgs.ad.>      (admin 域 / COC 控制面)
      rgs-co-events   →  rgs.co.>      (cluster-ops 域, Active-Active)

    Subject naming convention (per crates/shared-platform/src/subject.rs):
      rgs.<domain>.<event_type>.<version>     — domain events
      rgs.saga.<saga_type>.<event>            — saga events
      rgs.cem.<event_type>                    — CEM events
      rgs.dlq.<source>                        — dead-letter

    The script uses `nats` CLI via a kubectl exec sidecar (assumes kubectl
    and a NATS pod are reachable). It is IDEMPOTENT: re-running on an
    already-initialized cluster returns `stream name already in use`
    which the script treats as success.

    Retention policy (dev):
      - Limits-based retention
      - MaxAge: 7 days
      - MaxMsgs: 1,000,000 per stream
      - MaxBytes: 1 GiB per stream
      - Storage: File
      - Replicas: 1 (dev; HA 阶段 3)
      - Discard: Old

    Production tuning by SRE per RGS-ENV-CALIB-001 v0.1.

.PARAMETER Context
    K8s context (kubectl --context). Default: current.

.PARAMETER NatsPod
    NATS pod name (default: nats-0 in StatefulSet nats).

.PARAMETER Namespace
    Namespace where NATS is deployed. Default: rgs.

.EXAMPLE
    .\phase-0-5-step-2-init-streams.ps1
    # init streams on current context, default pod nats-0

.EXAMPLE
    .\phase-0-5-step-2-init-streams.ps1 -NatsPod "nats-1" -Context "rgs-prod"
    # init on specific pod and context

.NOTES
    Per RGS-DTL-100 §5 + RGS-SPEC-CROSS-005 §2
    Author: Phase 0.5 Step 2 deploy worker (WF-0.5-2)
    Re-run is safe: nats stream add is idempotent (returns error if exists,
    which the script treats as success).
#>

[CmdletBinding()]
param(
    [string]$Context = "",
    [string]$NatsPod = "nats-0",
    [string]$Namespace = "rgs"
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

# 6 streams: name, subjects, maxAge, maxMsgs, maxBytes
$Streams = @(
    @{ Name = "rgs-pl-events"; Subjects = "rgs.pl.>";     MaxAge = "168h"; MaxMsgs = 1000000; MaxBytes = 1073741824 },
    @{ Name = "rgs-ec-events"; Subjects = "rgs.ec.>";     MaxAge = "168h"; MaxMsgs = 1000000; MaxBytes = 1073741824 },
    @{ Name = "rgs-mt-events"; Subjects = "rgs.mt.>";     MaxAge = "168h"; MaxMsgs = 1000000; MaxBytes = 1073741824 },
    @{ Name = "rgs-gd-events"; Subjects = "rgs.gd.>";     MaxAge = "168h"; MaxMsgs = 1000000; MaxBytes = 1073741824 },
    @{ Name = "rgs-ad-events"; Subjects = "rgs.ad.>";     MaxAge = "168h"; MaxMsgs = 1000000; MaxBytes = 1073741824 },
    @{ Name = "rgs-co-events"; Subjects = "rgs.co.>";     MaxAge = "168h"; MaxMsgs = 1000000; MaxBytes = 1073741824 }
)

Write-Host "==> Phase 0.5 Step 2 — NATS JetStream stream init" -ForegroundColor Cyan
Write-Host "    Namespace:  $Namespace"
Write-Host "    Pod:        $NatsPod"
Write-Host "    Context:    $(if ($Context) { $Context } else { '<current>' })"
Write-Host "    Streams:    6 (idempotent)"
Write-Host ""

# Pre-flight: kubectl + pod reachable
if (-not (Get-Command kubectl -ErrorAction SilentlyContinue)) {
    throw "kubectl not found on PATH."
}
$ctxArg = if ($Context) { @("--context", $Context) } else { @() }
$podReady = kubectl get pod $NatsPod -n $Namespace $($ctxArg -join ' ') -o jsonpath='{.status.phase}' 2>&1
if ($podReady -ne "Running") {
    throw "Pod $NatsPod in $Namespace is not Running (got '$podReady'). Run phase-0-5-step-2-render-nats.ps1 first."
}

$created = 0
$skipped = 0
$failed  = 0

foreach ($s in $Streams) {
    $name = $s.Name
    Write-Host "  stream: $name" -NoNewline
    $json = @{
        name = $name
        subjects = @($s.Subjects)
        retention = "limits"
        max_age = (([int64]($s.MaxAge.TrimEnd('h'))) * 3600 * 1000000000)  # ns
        max_msgs = $s.MaxMsgs
        max_bytes = $s.MaxBytes
        storage = "file"
        num_replicas = 1
        discard = "old"
    } | ConvertTo-Json -Compress

    $jsonEscaped = $json -replace '"','\"'
    $cmd = "nats stream add $name --config `"$jsonEscaped`""
    $out = kubectl exec -n $Namespace $NatsPod $($ctxArg -join ' ') -- sh -c $cmd 2>&1

    if ($LASTEXITCODE -eq 0) {
        Write-Host "  [created]" -ForegroundColor Green
        $created++
    } elseif ($out -match "already in use|stream name already in use|already exists") {
        Write-Host "  [skipped, exists]" -ForegroundColor Yellow
        $skipped++
    } else {
        Write-Host "  [FAILED]" -ForegroundColor Red
        Write-Host "    $out"
        $failed++
    }
}

Write-Host ""
Write-Host "==> Step 2 stream init complete" -ForegroundColor Green
Write-Host "    Created: $created"
Write-Host "    Skipped: $skipped (already existed)"
Write-Host "    Failed:  $failed"

if ($failed -gt 0) {
    throw "$failed stream(s) failed to initialize. See errors above."
}

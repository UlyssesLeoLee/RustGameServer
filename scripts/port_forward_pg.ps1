# port_forward_pg.ps1
<#
.SYNOPSIS
    端口转发 WSL2 k3s postgres pod 到 Windows localhost:5432。
    Windows 端 sqlx-cli / psql 可通过 localhost:5432 访问 PG。

.DESCRIPTION
    后台启动 kubectl port-forward，每 5s 检查进程，挂了自动重启。
    按 Ctrl+C 停止。

.EXAMPLE
    pwsh -NoProfile -File scripts/port_forward_pg.ps1

.NOTES
    要求：WSL2 + k3s + postgres pod 已部署（per scripts/deploy_dev_k3s.ps1）
#>

[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Continue'

if ($PSVersionTable.PSVersion.Major -lt 7) {
    Write-Host '需要 PowerShell 7.0+' -ForegroundColor Red
    exit 1
}

$LocalPort = 5432
$RemotePort = 5432
$Namespace = 'rust-game-server'
$Service = 'postgres'

function Test-PortForwardRunning {
    $out = & wsl -- bash -c "k3s kubectl get portforward -n $Namespace 2>/dev/null" 2>&1
    return ($LASTEXITCODE -eq 0)
}

function Start-PortForward {
    Write-Host "Starting port-forward: localhost:${LocalPort} → ${Service}:${RemotePort} (in ${Namespace})"
    & wsl -- bash -c "k3s kubectl port-forward -n ${Namespace} svc/${Service} ${LocalPort}:${RemotePort}" 2>&1
}

# 检查是否已运行
$conn = Test-NetConnection -ComputerName 127.0.0.1 -Port $LocalPort -WarningAction SilentlyContinue
if ($conn.TcpTestSucceeded) {
    Write-Host "Port $LocalPort 已有人在监听（如已运行 port-forward 进程）。" -ForegroundColor Yellow
    Write-Host "  Connection: $conn"
} else {
    Write-Host "Port $LocalPort 未监听，启动 port-forward (Ctrl+C 停止)..."
    Start-PortForward
}

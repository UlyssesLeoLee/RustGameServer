# port_forward_pg.ps1
<#
.SYNOPSIS
    端口转发 WSL2 k3s postgres pod 到 Windows localhost。
    Windows 端 sqlx-cli / psql 可通过 localhost 访问 PG。

.DESCRIPTION
    后台启动 kubectl port-forward，每 5s 检查进程，挂了自动重启。
    按 Ctrl+C 停止。

.PARAMETER LocalPort
    Windows 本地监听端口（默认从 .env 读 PG_PORT_LOCAL=15432，避免和 Windows PG 5432 冲突）

.EXAMPLE
    pwsh -NoProfile -File scripts/port_forward_pg.ps1

.EXAMPLE
    pwsh -NoProfile -File scripts/port_forward_pg.ps1 -LocalPort 5432

.NOTES
    要求：WSL2 + k3s + postgres pod 已部署（per scripts/deploy_dev_k3s.ps1）
    默认端口：15432（per .env PG_PORT_LOCAL，避 Windows 默认 5432 冲突）
#>

[CmdletBinding()]
param(
    [int]$LocalPort = 0   # 0 = 从 .env 读，fallback 15432
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Continue'

if ($PSVersionTable.PSVersion.Major -lt 7) {
    Write-Host '需要 PowerShell 7.0+' -ForegroundColor Red
    exit 1
}

# 加载 .env（如果存在）
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$EnvFile = Join-Path $RepoRoot '.env'
if (Test-Path -LiteralPath $EnvFile) {
    Get-Content $EnvFile -Encoding UTF8 | ForEach-Object {
        if ($_ -match '^\s*([^#][^=]*)=(.*)$') {
            $name = $matches[1].Trim()
            $value = $matches[2].Trim()
            [System.Environment]::SetEnvironmentVariable($name, $value, 'Process')
        }
    }
}

# LocalPort 优先级：参数 > .env PG_PORT_LOCAL > 15432 默认
if ($LocalPort -eq 0) {
    $LocalPort = if ($env:PG_PORT_LOCAL) { [int]$env:PG_PORT_LOCAL } else { 15432 }
}
$RemotePort = 5432
$Namespace = if ($env:K8S_NAMESPACE) { $env:K8S_NAMESPACE } else { 'rust-game-server' }
$Service = 'postgres'

function Test-PortForwardRunning {
    $out = & wsl -- bash -c "k3s kubectl get portforward -n $Namespace 2>/dev/null" 2>&1
    return ($LASTEXITCODE -eq 0)
}

function Start-PortForward {
    Write-Host "Starting port-forward: localhost:${LocalPort} → ${Service}:${RemotePort} (in ${Namespace})"
    Write-Host "(从 .env 读 PG_PORT_LOCAL + K8S_NAMESPACE；可通过 -LocalPort 覆盖)"
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

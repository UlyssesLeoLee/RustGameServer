# debug-add-52551-portproxy.ps1 — 单独跑 netsh portproxy 52551→6443 (admin)
# (debug 用途, 排错 sync-ports-env.ps1 admin 跑失败问题)
$ErrorActionPreference = 'Stop'

Write-Host "=== 当前 PowerShell admin 状态 ==="
$cur = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
Write-Host "  IsAdmin = $($cur.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))"

Write-Host "`n=== WSL2 当前 IP (取第一个 = eth0) ==="
$wslIpRaw = (wsl -e bash -c 'hostname -I' 2>&1) -split '\s+' | Where-Object { $_ -match '^\d+\.\d+\.\d+\.\d+$' }
$wslIp = $wslIpRaw | Select-Object -First 1
Write-Host "  WSL2_IP = $wslIp"

Write-Host "`n=== 删旧规则 (如有) ==="
$delResult = netsh interface portproxy delete v4tov4 listenaddress=0.0.0.0 listenport=52551 2>&1
Write-Host "  delete: $delResult (exit=$LASTEXITCODE)"

Write-Host "`n=== 加 52551 → $wslIp:6443 ==="
$addResult = netsh interface portproxy add v4tov4 `
    listenaddress=0.0.0.0 listenport=52551 `
    connectaddress=$wslIp connectport=6443 2>&1
Write-Host "  add: $addResult (exit=$LASTEXITCODE)"

Write-Host "`n=== 验证 portproxy 全规则 ==="
netsh interface portproxy show all 2>&1

Write-Host "`n=== 验证 52551 连通性 ==="
Test-NetConnection -ComputerName 127.0.0.1 -Port 52551 -WarningAction SilentlyContinue 2>&1 | Select-Object TcpTestSucceeded, RemotePort | Format-Table -AutoSize

# run-sync-ports-as-admin.ps1 — 以 admin 提权跑 sync-ports-env.ps1
# (per 2026-09-10 07:50 JST Ulysses 拍板: admin PowerShell 跑 sync-ports-env.ps1)
#
# 用途: 解决 PowerShell 非 admin 跑 netsh portproxy 失败的问题
# 行为: Start-Process -Verb RunAs 弹 UAC → Ulysses 点 yes → 新 admin PowerShell 窗口跑 sync-ports-env.ps1
# 安全: 凭据永不打印 (per 8/27 11:06 JST 守门 #5)
#
# 用法 (主会话跑):
#   pwsh scripts/run-sync-ports-as-admin.ps1
#
# 等价手动步骤 (Ulysses 嫌 UAC 烦可自己跑):
#   1. 开始菜单 → 搜 "PowerShell" → 右键 "以管理员身份运行"
#   2. cd D:\RustGameServer
#   3. pwsh scripts/sync-ports-env.ps1
#   4. 等 5s, kubectl get nodes 应通

$ErrorActionPreference = 'Stop'

# 验证 sync-ports-env.ps1 存在
$syncScript = Join-Path $PSScriptRoot "sync-ports-env.ps1"
if (-not (Test-Path $syncScript)) {
    throw "[ERR] 缺 $syncScript"
}

# 提权 + 跑 sync-ports-env.ps1
Write-Host "[INFO] 提权 admin PowerShell 弹 UAC, 请点 yes..." -ForegroundColor Yellow
Write-Host "[INFO] 等价手动步骤: 开始菜单 → PowerShell → 右键管理员 → cd D:\RustGameServer → pwsh scripts/sync-ports-env.ps1" -ForegroundColor Cyan

$argList = @(
    "-NoProfile",
    "-ExecutionPolicy", "Bypass",
    "-File", "`"$syncScript`""
)
$proc = Start-Process -FilePath "pwsh" -ArgumentList $argList -Verb RunAs -PassThru -WindowStyle Normal
Write-Host "[INFO] admin PowerShell 已启动 (PID=$($proc.Id)), 跑 sync-ports-env.ps1" -ForegroundColor Green
Write-Host "[INFO] 等 admin 窗口关闭 (5-10s)..." -ForegroundColor Cyan

# 等待 admin 进程结束
$proc.WaitForExit()
$exitCode = $proc.ExitCode
Write-Host ""
if ($exitCode -eq 0) {
    Write-Host "[OK] sync-ports-env.ps1 跑成功 (exit=$exitCode)" -ForegroundColor Green
    Write-Host "[NEXT] 跑 kubectl get nodes --request-timeout=10s 验证 52551 通" -ForegroundColor Cyan
} else {
    Write-Host "[ERR] sync-ports-env.ps1 失败 (exit=$exitCode)" -ForegroundColor Red
    Write-Host "[HINT] 看 admin 窗口的输出排错" -ForegroundColor Yellow
}

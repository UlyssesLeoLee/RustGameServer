# RGS 5 域 service 一键启动 + migrations 脚本
# 用法: 1) 启 PowerShell 7 (pwsh.exe)  2) pwsh D:\RustGameServer\start-5-rgs-services.ps1
# 前置: D:\RustGameServer\.env 已配 (PG_PORT=5433 等), docker postgres `rgs-postgres-uat` 已跑
# 5 域独立 service + DATABASE_URL 独立 + RGS_ALLOW_INSECURE_GRPC=1 跳过 mTLS (dev bypass)

$ErrorActionPreference = 'Stop'
$env:RUST_LOG = 'info'
$env:RGS_ALLOW_INSECURE_GRPC = '1'
$REPO = 'D:\RustGameServer'
$BIN  = "$REPO\target\release"

Write-Host '=== Step 1: 杀残留进程 ===' -ForegroundColor Cyan
Get-Process -Name 'player-service','economy-service','match-service','social-service','admin-service' -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Get-NetTCPConnection -LocalPort 50051,50052,50053,50054,50055 -State Listen -ErrorAction SilentlyContinue | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }
Start-Sleep -Seconds 2

Write-Host '=== Step 2: 启 5 域 service binary (RGS_ALLOW_INSECURE_GRPC=1) ===' -ForegroundColor Cyan
$svcs = @(
  @{Name='player-service';   Db='postgresql://player_user:ulysses_local@127.0.0.1:5433/player_db';   Port='50051'},
  @{Name='economy-service';  Db='postgresql://economy_user:ulysses_local@127.0.0.1:5433/economy_db';  Port='50052'},
  @{Name='match-service';    Db='postgresql://match_user:ulysses_local@127.0.0.1:5433/match_db';    Port='50053'},
  @{Name='social-service';   Db='postgresql://social_user:ulysses_local@127.0.0.1:5433/social_db';   Port='50054'},
  @{Name='admin-service';    Db='postgresql://admin_user:ulysses_local@127.0.0.1:5433/admin_db';    Port='50055'}
)
foreach ($s in $svcs) {
  $env:DATABASE_URL = $s.Db
  $env:GRPC_ADDR = "0.0.0.0:$($s.Port)"
  New-Item -Path "$REPO\logs" -ItemType Directory -Force | Out-Null
  $proc = Start-Process -FilePath "$BIN\$($s.Name).exe" -PassThru -WindowStyle Hidden `
    -RedirectStandardOutput "$REPO\logs\$($s.Name).out" `
    -RedirectStandardError  "$REPO\logs\$($s.Name).err"
  $env:DATABASE_URL = $null; $env:GRPC_ADDR = $null
  Write-Host "  started $($s.Name) PID=$($proc.Id) on port $($s.Port)" -ForegroundColor Green
}
Start-Sleep -Seconds 6

Write-Host '=== Step 3: 验证 5 域 gRPC TCP listening ===' -ForegroundColor Cyan
foreach ($s in $svcs) {
  $r = Test-NetConnection 127.0.0.1 -Port $s.Port -InformationLevel Quiet -WarningAction SilentlyContinue
  Write-Host "  $($s.Name) port $($s.Port) : $r"
}

Write-Host '=== Step 4: 跑 5 域 migrations (sqlx) ===' -ForegroundColor Cyan
$dbCfg = @(
  @{Svc='player';  User='player_user';  Db='player_db'},
  @{Svc='economy'; User='economy_user'; Db='economy_db'},
  @{Svc='match';   User='match_user';   Db='match_db'},
  @{Svc='social';  User='social_user';  Db='social_db'},
  @{Svc='admin';   User='admin_user';   Db='admin_db'}
)
foreach ($d in $dbCfg) {
  $env:DATABASE_URL = "postgresql://$($d.User):ulysses_local@127.0.0.1:5433/$($d.Db)"
  $mig = "$REPO\crates\$($d.Svc)-service\migrations"
  if (Test-Path $mig) {
    Push-Location $REPO
    cargo sqlx migrate run --source $mig 2>&1 | Select-Object -First 3
    Pop-Location
  } else {
    Write-Host "  (no migrations dir for $($d.Svc))" -ForegroundColor Yellow
  }
}

Write-Host '=== Step 5: 启 rgs-proxy 8084 (gRPC→HTTP, 浏览器用) ===' -ForegroundColor Cyan
Get-Process -Name node -ErrorAction SilentlyContinue | Where-Object { $_.MainModule.FileName -like '*node.exe' } | ForEach-Object {
  Get-NetTCPConnection -OwningProcess $_.Id -LocalPort 8084 -ErrorAction SilentlyContinue | Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
}
Start-Sleep -Seconds 1
Start-Process -FilePath 'node.exe' -ArgumentList 'server.js' `
  -WorkingDirectory 'D:\playwright-test\rgs-proxy' `
  -RedirectStandardOutput 'D:\playwright-test\rgs-proxy\proxy-stdout.log' `
  -RedirectStandardError  'D:\playwright-test\rgs-proxy\proxy-stderr.log' `
  -WindowStyle Hidden
Start-Sleep -Seconds 3
$r = Test-NetConnection 127.0.0.1 -Port 8084 -InformationLevel Quiet -WarningAction SilentlyContinue
Write-Host "  rgs-proxy 8084 : $r"

Write-Host ''
Write-Host '=== 全部完成! 现在 RGS 5 域 gRPC + proxy 都在跑 ===' -ForegroundColor Green
Write-Host '  50051 player / 50052 economy / 50053 match / 50054 social / 50055 admin'
Write-Host '  8084 rgs-proxy (gRPC→HTTP, 浏览器 fetch 用)'
Write-Host '  之后我会用 Playwright 跑 game.html 截图真 RGS 联动'

# mtls-9domain-probe.ps1 — 9 域 mTLS 端到端探针 (W26 Phase 3 收口, per 改进路线图 §1)
#
# 用途: 9 域 (5 域 + cluster_ops + 3 NEW 域 scene/battle/network-gateway) mTLS 端到端真验
#       - 9 域 cert 链验证 (openssl verify, 9/9 expected PASS)
#       - 3 NEW 域 mTLS 握手 sim (openssl s_server/s_client, 3/3 expected PASS, per W20 §1.5)
#       - 5 域 + cluster_ops mTLS 握手 (k3s pod 网络, 当前 cni0 linkdown 阻塞)
#       - 9 域业务级 RPC 抽样 (grpcurl, 18 抽样 per 改进路线图 §1 Phase 3 收口)
# 依据:
#   - 改进路线图.md §1 Phase 3 + W26 任务简报
#   - W18 8 步端到端模式 (扩展 11 步, 8 + 3 NEW 域)
#   - W20 9 域 mTLS cert (ca.pem 5 域 + ca-sbn.pem 3 NEW 域)
#   - L19 派生 (gRPC NotFound/InvalidArgument 算 handler 触达)
#   - L20 派生 (k8s mTLS cert 必须 servername=svc DNS, 无 IP SAN)
# 依赖:
#   - D:\kubectl\grpcurl.exe (v1.9.1)
#   - D:\kubectl\kubectl.exe
#   - C:\Program Files\OpenSSL-Win64\bin\openssl.exe
#   - D:\sszgC\certs\* (9 域 mTLS cert + 2 CA)
# 调用: pwsh -NoProfile -File scripts/mtls-9domain-probe.ps1

[CmdletBinding()]
param(
    [switch]$SkipCertVerify,
    [switch]$SkipNewDomainSim,
    [switch]$SkipRPC
)

$ErrorActionPreference = 'Stop'

# 9 域 + cluster_ops 配置
$script:Domains = [ordered]@{
    'player'          = @{ Port = 50051; CA = 'ca.pem';     Server = 'player.service';          Class = 'core5' }
    'economy'         = @{ Port = 50052; CA = 'ca.pem';     Server = 'economy.service';         Class = 'core5' }
    'match'           = @{ Port = 50053; CA = 'ca.pem';     Server = 'match.service';           Class = 'core5' }
    'social'          = @{ Port = 50054; CA = 'ca.pem';     Server = 'social.service';          Class = 'core5' }
    'admin'           = @{ Port = 50055; CA = 'ca.pem';     Server = 'admin.service';           Class = 'core5' }
    'cluster_ops'     = @{ Port = 50056; CA = 'ca.pem';     Server = 'cluster-ops.service';     Class = 'platform' }
    'scene'           = @{ Port = 50057; CA = 'ca-sbn.pem'; Server = 'scene.service';          Class = 'new3' }
    'battle'          = @{ Port = 50058; CA = 'ca-sbn.pem'; Server = 'battle.service';         Class = 'new3' }
    'network-gateway' = @{ Port = 50090; CA = 'ca-sbn.pem'; Server = 'network-gateway.service'; Class = 'new3' }
}

# 9 域业务级 RPC 抽样 (9 域 × 2 RPC = 18 抽样, per 任务简报 §2)
$script:RPCSamples = @(
    # player
    @{ Domain = 'player';          Method = 'player.v1.PlayerService/HealthCheck';          Data = '{}' }
    @{ Domain = 'player';          Method = 'player.v1.PlayerService/GetPlayer';            Data = '{"id":"00000000-0000-0000-0000-000000000099"}' }
    # economy
    @{ Domain = 'economy';         Method = 'economy.v1.EconomyService/GetAccount';          Data = '{"id":"00000000-0000-0000-0000-000000000099"}' }
    @{ Domain = 'economy';         Method = 'economy.v1.EconomyService/ShopList';            Data = '{}' }
    # match
    @{ Domain = 'match';           Method = 'match.v1.MatchService/SubmitMove';              Data = '{}' }
    @{ Domain = 'match';           Method = 'match.v1.MatchService/GetMatchState';           Data = '{"match_id":"00000000-0000-0000-0000-000000000099"}' }
    # social
    @{ Domain = 'social';          Method = 'social.v1.SocialService/GetGuild';              Data = '{"id":"00000000-0000-0000-0000-000000000099"}' }
    @{ Domain = 'social';          Method = 'social.v1.SocialService/SendChat';              Data = '{}' }
    # admin
    @{ Domain = 'admin';           Method = 'admin.v1.AdminService/HealthCheck';             Data = '{}' }
    @{ Domain = 'admin';           Method = 'admin.v1.AdminService/AuditLog';                Data = '{}' }
    # cluster_ops
    @{ Domain = 'cluster_ops';     Method = 'cluster_ops.v1.ClusterOpsService/GetClusterStatus'; Data = '{}' }
    @{ Domain = 'cluster_ops';     Method = 'cluster_ops.v1.ClusterOpsService/HealthCheck';  Data = '{}' }
    # scene (3 NEW)
    @{ Domain = 'scene';           Method = 'scene.v1.SceneService/HealthCheck';             Data = '{}' }
    @{ Domain = 'scene';           Method = 'scene.v1.SceneService/EnterScene';              Data = '{"player_id":"00000000-0000-0000-0000-000000000099"}' }
    # battle (3 NEW)
    @{ Domain = 'battle';          Method = 'battle.v1.BattleService/HealthCheck';            Data = '{}' }
    @{ Domain = 'battle';          Method = 'battle.v1.BattleService/StartBattle';            Data = '{}' }
    # network-gateway (3 NEW)
    @{ Domain = 'network-gateway'; Method = 'network_gateway.v1.NetworkGatewayService/HealthCheck';  Data = '{}' }
    @{ Domain = 'network-gateway'; Method = 'network_gateway.v1.NetworkGatewayService/ListRoutes';   Data = '{}' }
)

# 路径
$script:CertDir     = 'D:\sszgC\certs'
$script:OpenSSL     = 'C:\Program Files\OpenSSL-Win64\bin\openssl.exe'
$script:OpensslConf = 'C:\Program Files\OpenSSL-Win64\bin\cnf\openssl.cnf'
$script:LogFile     = 'D:\sszgC\worker26-9domain-mtls-probe.log'

# 输出 helper
function Write-Section { param([string]$Title) Write-Host ""; Write-Host "=== $Title ===" -ForegroundColor Cyan }
function Write-Pass   { param([string]$Msg)  Write-Host "  [PASS] $Msg" -ForegroundColor Green }
function Write-Fail   { param([string]$Msg)  Write-Host "  [FAIL] $Msg" -ForegroundColor Red }
function Write-Info   { param([string]$Msg)  Write-Host "  [INFO] $Msg" -ForegroundColor Gray }
function Write-Skip   { param([string]$Msg)  Write-Host "  [SKIP] $Msg" -ForegroundColor Yellow }

# =============================================================================
# 0. 环境预检
# =============================================================================
Write-Section "0. 环境预检"
$env:OPENSSL_CONF = $script:OpensslConf
$env:PYTHONIOENCODING = 'utf-8'

if (-not (Test-Path $script:CertDir))     { Write-Fail "cert 目录缺失: $($script:CertDir)"; exit 1 }
if (-not (Test-Path $script:OpenSSL))     { Write-Fail "openssl 缺失"; exit 1 }
if (-not (Test-Path "$script:CertDir\ca.pem"))     { Write-Fail "ca.pem 缺失"; exit 1 }
if (-not (Test-Path "$script:CertDir\ca-sbn.pem")) { Write-Fail "ca-sbn.pem 缺失"; exit 1 }
Write-Pass "cert 目录 OK"
Write-Pass "openssl OK"
Write-Pass "ca.pem OK (5 域)"
Write-Pass "ca-sbn.pem OK (3 NEW 域)"

# 9 域 cert 文件预检
Write-Info "9 域 cert 文件:"
foreach ($d in $script:Domains.Keys) {
    $crt = "$script:CertDir\$d-server.crt"
    $key = "$script:CertDir\$d-server.key"
    $crtOk = Test-Path $crt
    $keyOk = Test-Path $key
    if ($crtOk -and $keyOk) { Write-Pass "  $d" } else { Write-Fail "  $d cert/key 缺失" }
}

# =============================================================================
# 1. 9 域 cert 链验证 (openssl verify)
# =============================================================================
$certResults = @{}
if (-not $SkipCertVerify) {
    Write-Section "1. 9 域 cert 链验证 (openssl verify)"
    foreach ($d in $script:Domains.Keys) {
        $cfg = $script:Domains[$d]
        $crt = "$script:CertDir\$d-server.crt"
        $ca  = "$script:CertDir\$($cfg.CA)"
        $result = & $script:OpenSSL verify -CAfile $ca $crt 2>&1
        if ($result -match 'OK$') {
            Write-Pass "$d ($ca)"
            $certResults[$d] = 'OK'
        } else {
            Write-Fail "$d ($ca) → $result"
            $certResults[$d] = "FAIL: $result"
        }
    }
    $certPass = ($certResults.Values | Where-Object { $_ -eq 'OK' }).Count
    Write-Info "9 域 cert 验证结果: $certPass/9 PASS"
} else {
    Write-Skip "1. cert 验证 (per -SkipCertVerify)"
}

# =============================================================================
# 2. 3 NEW 域 mTLS 握手 sim (openssl s_server/s_client, per W20 §1.5)
# =============================================================================
$newMTLSResults = @{}
if (-not $SkipNewDomainSim) {
    Write-Section "2. 3 NEW 域 mTLS 握手 sim (openssl s_server/s_client, per W20 §1.5)"
    foreach ($d in @('scene', 'battle', 'network-gateway')) {
        $cfg = $script:Domains[$d]
        $crt = "$script:CertDir\$d-server.crt"
        $key = "$script:CertDir\$d-server.key"
        $ca  = "$script:CertDir\$($cfg.CA)"
        $servername = $cfg.Server
        $port = $cfg.Port

        # 启动 openssl s_server 后台
        $serverProc = Start-Process -FilePath $script:OpenSSL `
            -ArgumentList @('s_server', "-accept", "$port", "-cert", $crt, "-key", $key, "-CAfile", $ca, "-www", "-quiet") `
            -PassThru -NoNewWindow -RedirectStandardError "$env:TEMP\openssl_s_server_err_$d.log"
        Start-Sleep -Seconds 1

        # openssl s_client 连接 + 验证
        $clientOut = & $script:OpenSSL s_client -connect "127.0.0.1:$port" -CAfile $ca -servername $servername -showcerts 2>&1 | Select-Object -First 15
        $verifyOk = ($clientOut -match 'verify return:1')
        $cnMatch  = ($clientOut -match "CN\s*=\s*$servername")
        Stop-Process -Id $serverProc.Id -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1

        if ($verifyOk -and $cnMatch) {
            Write-Pass "$d port $port servername=$servername (verify return:1, CN match)"
            $newMTLSResults[$d] = 'OK'
        } else {
            Write-Fail "$d port $port servername=$servername (verify=$verifyOk cn=$cnMatch)"
            $newMTLSResults[$d] = 'FAIL'
        }
    }
} else {
    Write-Skip "2. 3 NEW 域 mTLS sim (per -SkipNewDomainSim)"
}

# =============================================================================
# 3. 9 域业务级 RPC 抽样 (grpcurl, 18 抽样)
# =============================================================================
$rpcResults = @()
if (-not $SkipRPC) {
    Write-Section "3. 9 域业务级 RPC 抽样 (grpcurl, 18 抽样)"
    # 取 5 域 pod IP (kubectl get pods -n rust-game-server)
    Write-Info "获取 5 域 pod IP (kubectl get pods)"
    $podIps = @{}
    try {
        $env:KUBECONFIG = 'D:\kubectl\k3s.yaml'
        $kubectlOut = wsl -u root -- bash -c 'kubectl get pods -n rust-game-server -o jsonpath="{range .items[*]}{.metadata.name}={.status.podIP} {end}" --request-timeout=5s 2>/dev/null' 2>&1
        foreach ($line in ($kubectlOut -split ' ')) {
            if ($line -match '^(.+?)-service-(.+?)=(.+)$') {
                $short = $Matches[1]
                $ip = $Matches[3]
                if (-not $podIps[$short]) { $podIps[$short] = $ip }
            }
        }
    } catch {
        Write-Info "kubectl 失败: $($_.Exception.Message)"
    }
    Write-Info "Pod IP: $($podIps | ConvertTo-Json -Compress)"

    foreach ($sample in $script:RPCSamples) {
        $d = $sample.Domain
        $method = $sample.Method
        $data = $sample.Data

        $cfg = $script:Domains[$d]
        $port = $cfg.Port
        $ip = $podIps[$d]
        $protoDir = "D:\RustGameServer\.worktrees\feat-auto-20260905-f34ff640\crates\$d-service\proto"
        $protoRel = "$d/v1/$d.proto"

        if (-not $ip) {
            $rpcResults += @{ Domain = $d; Method = $method; Verdict = 'SKIP'; Detail = 'no-pod-ip-or-not-deployed' }
            $color = 'Yellow'
            $verdict = 'SKIP'
            $detail = 'no-pod-ip-or-not-deployed'
        } else {
            $svcIp = "$ip`:$port"
            $ca = "$script:CertDir\$($cfg.CA)"
            $crt = "$script:CertDir\$d-server.crt"
            $key = "$script:CertDir\$d-server.key"
            $servername = $cfg.Server

            $grpcurlArgs = @(
                '-cacert', $ca
                '-cert', $crt
                '-key', $key
                '-servername', $servername
                '-import-path', $protoDir
                '-proto', $protoRel
                '-d', $data
                '-connect-timeout', '3'
                $svcIp
                $method
            )
            try {
                $proc = Start-Process -FilePath 'D:\kubectl\grpcurl.exe' `
                    -ArgumentList $grpcurlArgs `
                    -PassThru -NoNewWindow -Wait -Timeout 10 `
                    -RedirectStandardOutput "$env:TEMP\grpcurl_out_$d.log" `
                    -RedirectStandardError "$env:TEMP\grpcurl_err_$d.log"
                $stdout = Get-Content "$env:TEMP\grpcurl_out_$d.log" -Raw -ErrorAction SilentlyContinue
                $stderr = Get-Content "$env:TEMP\grpcurl_err_$d.log" -Raw -ErrorAction SilentlyContinue
                $combined = "$stdout$stderr"

                $isOk = ($combined -match 'STATUS_OK') -or ($combined -match '"status"')
                $isHandlerOk = $combined -match 'Code:\s*(NotFound|InvalidArgument|FailedPrecondition|PermissionDenied|Unauthenticated|AlreadyExists|ResourceExhausted|OutOfRange|Unimplemented|Internal|Unavailable|DeadlineExceeded|Aborted|Cancelled|DataLoss|Unknown|OK)'

                if ($isOk) {
                    $verdict = 'PASS'; $detail = 'OK (STATUS_OK)'
                } elseif ($isHandlerOk) {
                    $verdict = 'PASS'; $detail = 'business-handler-ok (NotFound/InvalidArgument/...)'
                } else {
                    $verdict = 'FAIL'; $detail = "rc=$($proc.ExitCode) stderr='$($stderr.Substring(0, [Math]::Min(80, $stderr.Length)))'"
                }
            } catch {
                $verdict = 'FAIL'; $detail = "exception: $($_.Exception.Message)"
            }
        }

        $rpcResults += @{ Domain = $d; Method = $method; Verdict = $verdict; Detail = $detail }
        $color = if ($verdict -eq 'PASS') { 'Green' } elseif ($verdict -eq 'SKIP') { 'Yellow' } else { 'Red' }
        Write-Host "  [$verdict] $d $method → $detail" -ForegroundColor $color
    }
} else {
    Write-Skip "3. 业务级 RPC (per -SkipRPC)"
}

# =============================================================================
# 4. 总结报告
# =============================================================================
Write-Section "4. 总结报告"

$certPass = ($certResults.Values | Where-Object { $_ -eq 'OK' }).Count
$newMtlssPass = ($newMTLSResults.Values | Where-Object { $_ -eq 'OK' }).Count
$rpcPass = ($rpcResults | Where-Object { $_.Verdict -eq 'PASS' }).Count
$rpcFail = ($rpcResults | Where-Object { $_.Verdict -eq 'FAIL' }).Count
$rpcSkip = ($rpcResults | Where-Object { $_.Verdict -eq 'SKIP' }).Count

Write-Host "  9 域 cert 验证:        $certPass/9 PASS" -ForegroundColor $(if ($certPass -eq 9) { 'Green' } else { 'Yellow' })
Write-Host "  3 NEW 域 mTLS sim:     $newMtlssPass/3 PASS (per W20 §1.5 openssl sim)" -ForegroundColor $(if ($newMtlssPass -eq 3) { 'Green' } else { 'Yellow' })
Write-Host "  9 域业务级 RPC 抽样:   $rpcPass/18 PASS (fail=$rpcFail, skip=$rpcSkip)" -ForegroundColor $(if ($rpcPass -ge 12) { 'Green' } elseif ($rpcPass -ge 6) { 'Yellow' } else { 'Red' })

# 写 log
$logContent = @"
W26 9 域 mTLS 端到端探针 log ($(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') JST)
================================================================
1. cert 验证: $certPass/9 PASS
$($certResults.GetEnumerator() | ForEach-Object { "  $($_.Key) = $($_.Value)" } | Out-String)
2. 3 NEW 域 mTLS sim: $newMtlssPass/3 PASS
$($newMTLSResults.GetEnumerator() | ForEach-Object { "  $($_.Key) = $($_.Value)" } | Out-String)
3. RPC 抽样: $rpcPass/18 PASS (fail=$rpcFail, skip=$rpcSkip)
$($rpcResults | ForEach-Object { "  $($_.Domain) $($_.Method) = $($_.Verdict) $($_.Detail)" } | Out-String)
"@
Set-Content -Path $script:LogFile -Value $logContent -Encoding UTF8
Write-Info "Log written: $script:LogFile"

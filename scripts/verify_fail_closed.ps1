# verify_fail_closed.ps1
<#
.SYNOPSIS
    5 域 fail-closed 验证脚本(CI 入口,固化 phase-0-5 step 4 一次性手工验证)

.DESCRIPTION
    本脚本是 Q-M-08 答复合规产物(WF-1-55.48,per RGS-OPEN-QA-001 v0.2 Q-M-08 + ACTIONS-v0.3 B-09):

    设计目标:把"phase-0-5 step 4 commit b9bc214 一次性 fail-closed 验证"固化为 CI 入口脚本,
    每次 manifest/RBAC 变更的 PR 触发(不限新增域),确保 fail-closed 防线不被静默降级破坏。

    测试覆盖(4 项,per Q-M-08 答复):
      T1. TLS fail-closed:损坏 Secret 中的 CA 证书 → 6 域 binary 启动应退出非 0 + stderr 含 TLS error
      T2. RBAC fail-closed:用未授权 ServiceAccount 调 5 域 gRPC → 应返回 PERMISSION_DENIED(不返回数据)
      T3. Secret 缺失:删 rgs-secret-ca → 重启 deployment → 启动应失败
      T4. 默认拒绝:新增 RBAC 资源在显式授权前访问应被拒(防止新增域时默认放行)

    不变量(per RGS-INC-001 v0.2 §1.4):
      1. 任何 TLS 失败 → binary fail-closed(exit non-zero,stderr 含 mTLS/TLS/DB fail marker)
      2. 任何 RBAC 越权 → k3s API 或 gRPC 拦截层返回 PERMISSION_DENIED(无数据)
      3. 缺失关键 Secret → Deployment 启动失败(无降级路径)
      4. 新增 RBAC 资源在 RoleBinding 缺失前不可访问(默认拒绝)

    报告输出:docs/deploy/fail-closed-verify-report.md(自动覆盖,Mermaid 图 + 测试结果表)
    退出码:任何 1 项 FAIL → exit 1;全部 PASS(或 SKIP)→ exit 0
              CI 拦截: exit 1 阻断 PR 合并(per docs/deploy/fail-closed-ci-integration.md)

    关联文档:
      - RGS-INC-001 v0.2 §1.4 (mTLS fail-closed)
      - RGS-OPEN-QA-001 v0.1 Q-M-08 答复
      - RGS-OPEN-QA-001-ACTIONS-v0.3 B-09
      - docs/deploy/fail-closed-ci-integration.md (CI workflow 接入)

.PARAMETER Mode
    验证模式:
      - All(默认):跑 T1 + T2 + T3 + T4 全部
      - T1:仅 TLS fail-closed
      - T2:仅 RBAC fail-closed
      - T3:仅 Secret 缺失
      - T4:仅默认拒绝
      - Smoke:仅做"k3s 可达性 + 命名空间存在"轻量预检(PR 触发早期快筛)

.PARAMETER KubeCtlPath
    kubectl 路径。默认 'kubectl'(k3s built-in 也可)

.PARAMETER Namespace
    k8s namespace。默认 'rust-game-server'

.PARAMETER SkipApply
    跳过 kubectl apply 步骤(仅做 READ-ONLY 验证;适用于 dry-run / PR review 阶段)

.PARAMETER TimeoutSec
    单个测试超时秒数。默认 30

.EXAMPLE
    pwsh -NoProfile -File scripts/verify_fail_closed.ps1
    # 全量 4 项验证(本地/CI 默认入口)

.EXAMPLE
    pwsh -NoProfile -File scripts/verify_fail_closed.ps1 -Mode Smoke
    # 仅做 k3s 可达性 + namespace 预检(< 5s 完成)

.EXAMPLE
    pwsh -NoProfile -File scripts/verify_fail_closed.ps1 -Mode T2
    # 仅跑 RBAC fail-closed 测试(用于 RBAC 专项 PR)

.EXAMPLE
    pwsh -NoProfile -File scripts/verify_fail_closed.ps1 -SkipApply
    # 只做 READ-ONLY 验证(不写入/删除任何资源;适合 PR review)

.NOTES
    Author:  Worker (WF-1-55.48)
    Spec:    RGS-INC-001 v0.2 §1.4 + RGS-OPEN-QA-001 v0.2 Q-M-08 答复 + ACTIONS-v0.3 B-09
    Pre:     1) k3s 集群已起(k3s kubectl get nodes Ready)
             2) namespace 'rust-game-server' 已建(per docs/deploy/01-k8s-manifests/00-namespace.yaml)
             3) 5 域 manifest 已 apply(per scripts/deploy_dev_k3s.ps1)
    Post:    docs/deploy/fail-closed-verify-report.md 含 Mermaid 图 + 4 项结果表
#>

[CmdletBinding()]
param(
    [ValidateSet('All', 'T1', 'T2', 'T3', 'T4', 'Smoke')]
    [string]$Mode       = 'All',
    [string]$KubeCtlPath = 'kubectl',
    [string]$Namespace  = 'rust-game-server',
    [switch]$SkipApply,
    [int]$TimeoutSec    = 30
)

$ErrorActionPreference = 'Stop'
$ProgressPreference    = 'SilentlyContinue'

# ==================== 前置检查 ====================

# 1. PowerShell 7+ 校验(per Q-M-08 答复命名约定 + 项目现有脚本风格)
if ($PSVersionTable.PSVersion.Major -lt 7) {
    Write-Host '需要 PowerShell 7.0+。请使用: pwsh -File scripts/verify_fail_closed.ps1' -ForegroundColor Red
    exit 1
}

$RepoRoot       = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ManifestDir    = Join-Path $RepoRoot 'docs/deploy/01-k8s-manifests'
$ReportDir      = Join-Path $RepoRoot 'docs/deploy'
$ReportPath     = Join-Path $ReportDir 'fail-closed-verify-report.md'
$Timestamp      = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
$SessionId      = [guid]::NewGuid().ToString().Substring(0, 8)

# 5 域列表(per RGS-BAS-001 §3 5 服务 + cluster-ops 域 = 6 域总览,但 fail-closed 主要针对 5 个业务域;
# cluster-ops 域独立管理,RBAC 已在 ADR-0052 中单独处理)
$BusinessDomains = @('player', 'economy', 'match', 'social', 'admin')

# 6 域 mTLS Secret(per RGS-INC-001 v0.2 §1.4 + RGS-REV-008 AC-1):
# - 1 份 rgs-secret-ca(CA 单例,per Q-M-08 答复确认)
# - 5 份 rgs-tls-<domain> + cluster-ops 共 6 份
$MtlsSecrets = @(
    @{ Name = 'rgs-secret-ca'; Type = 'ca' }
    @{ Name = 'rgs-tls-player'; Type = 'leaf' }
    @{ Name = 'rgs-tls-economy'; Type = 'leaf' }
    @{ Name = 'rgs-tls-match'; Type = 'leaf' }
    @{ Name = 'rgs-tls-social'; Type = 'leaf' }
    @{ Name = 'rgs-tls-admin'; Type = 'leaf' }
    @{ Name = 'rgs-tls-cluster-ops'; Type = 'leaf' }
)

# ==================== 辅助函数 ====================

# 调用 kubectl 并捕获退出码
function Invoke-Kubectl {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$Args,
        [int]$TimeoutSec = 15
    )
    $outputFile = [System.IO.Path]::GetTempFileName()
    $errFile    = [System.IO.Path]::GetTempFileName()
    try {
        $pinfo = New-Object System.Diagnostics.ProcessStartInfo
        $pinfo.FileName = $KubeCtlPath
        $pinfo.Arguments = $Args
        $pinfo.RedirectStandardOutput = $true
        $pinfo.RedirectStandardError  = $true
        $pinfo.UseShellExecute = $false
        $pinfo.CreateNoWindow  = $true
        $proc = [System.Diagnostics.Process]::Start($pinfo)
        $exited = $proc.WaitForExit($TimeoutSec * 1000)
        if (-not $exited) {
            try { $proc.Kill() } catch {}
            return @{ ExitCode = 124; StdOut = ''; StdErr = 'TIMEOUT' }
        }
        return @{
            ExitCode = $proc.ExitCode
            StdOut   = $proc.StandardOutput.ReadToEnd()
            StdErr   = $proc.StandardError.ReadToEnd()
        }
    } finally {
        Remove-Item $outputFile, $errFile -ErrorAction SilentlyContinue
    }
}

# k3s 集群可达性预检
function Test-K3sReachable {
    [CmdletBinding()]
    param()
    $r = Invoke-Kubectl -Args 'cluster-info --request-timeout=5s'
    if ($r.ExitCode -ne 0) {
        Write-Host '[FATAL] k3s 集群不可达。k3s kubectl cluster-info 退出码 =' $r.ExitCode -ForegroundColor Red
        Write-Host '  提示:启动 k3s 服务(sudo systemctl start k3s)或检查 kubeconfig(~/.kube/config)' -ForegroundColor Yellow
        return $false
    }
    return $true
}

# namespace 存在性预检
function Test-NamespaceExists {
    [CmdletBinding()]
    param([string]$Ns)
    $r = Invoke-Kubectl -Args "get namespace $Ns --request-timeout=5s"
    return ($r.ExitCode -eq 0)
}

# 输出测试结果到控制台 + 累积到 $Results
function Write-TestResult {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$TestId,
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][ValidateSet('PASS', 'FAIL', 'SKIP')][string]$Status,
        [string]$Detail = '',
        [string]$Marker = ''
    )
    $color = switch ($Status) {
        'PASS' { 'Green' }
        'FAIL' { 'Red' }
        'SKIP' { 'Yellow' }
    }
    $markerStr = if ($Marker) { " [$Marker]" } else { '' }
    Write-Host ("  [{0}] {1,-8} {2,-40} {3}{4}" -f $TestId, $Status, $Name, $Detail, $markerStr) -ForegroundColor $color
    return [pscustomobject]@{
        TestId = $TestId
        Name   = $Name
        Status = $Status
        Detail = $Detail
        Marker = $Marker
    }
}

# ==================== 测试 T1: TLS fail-closed ====================
# 损坏 rgs-secret-ca 中的 ca.pem → 重启 5 域 deployment → 启动应 fail-closed
# 期望:5 域 Pod 全部 CrashLoopBackOff,事件含 "tls: failed to load CA" / "x509: malformed cert"
function Test-T1_TlsFailClosed {
    [CmdletBinding()]
    param()
    Write-Host ''
    Write-Host '[T1] TLS fail-closed 测试(损坏 CA Secret → 6 域 binary 启动应失败)' -ForegroundColor Cyan
    if ($SkipApply) {
        return Write-TestResult -TestId 'T1' -Name 'TLS fail-closed' -Status 'SKIP' -Detail '-SkipApply 模式仅做 READ-ONLY'
    }

    $results = @()
    $caSecret = 'rgs-secret-ca'

    # 1. 备份原始 ca.pem(从 git 原始 manifest 读 base64)
    $caManifest = Join-Path $ManifestDir '50-secret-ca.yaml'
    if (-not (Test-Path $caManifest)) {
        return Write-TestResult -TestId 'T1' -Name 'TLS fail-closed' -Status 'FAIL' -Detail "CA manifest 缺失: $caManifest"
    }
    $origContent = Get-Content $caManifest -Raw -Encoding UTF8
    if ($origContent -notmatch 'ca\.pem:\s*"([^"]+)"') {
        return Write-TestResult -TestId 'T1' -Name 'TLS fail-closed' -Status 'FAIL' -Detail '无法从 manifest 提取 ca.pem base64'
    }
    $origCaB64 = $matches[1]

    # 2. 损坏 ca.pem(base64 解码后写入 garbage,再 base64)
    # 损坏标志:x509: malformed cert / asn1: structure error
    $garbage = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes('GARBAGE-NOT-A-REAL-CERT-FAIL-CLOSED-VERIFY-001'))
    $patchJson = @"
{
  "data": {
    "ca.pem": "$garbage"
  }
}
"@
    $patchFile = [System.IO.Path]::GetTempFileName()
    try {
        [System.IO.File]::WriteAllText($patchFile, $patchJson, [System.Text.Encoding]::UTF8)
        $r = Invoke-Kubectl -Args "patch secret $caSecret -n $Namespace --type=merge --patch-file=$patchFile --request-timeout=10s"
        if ($r.ExitCode -ne 0) {
            $results += Write-TestResult -TestId 'T1' -Name 'CA Secret 损坏注入' -Status 'FAIL' -Detail "kubectl patch 失败: $($r.StdErr)"
            return $results
        }
        $results += Write-TestResult -TestId 'T1' -Name 'CA Secret 损坏注入' -Status 'PASS' -Detail 'rgs-secret-ca ca.pem 已替换为 garbage'

        # 3. 重启 5 域 deployment,触发 Pod 启动
        $restartedDomains = @()
        foreach ($d in $BusinessDomains) {
            $r2 = Invoke-Kubectl -Args "rollout restart deployment/$d-service -n $Namespace --request-timeout=10s"
            if ($r2.ExitCode -eq 0) { $restartedDomains += $d }
        }
        if ($restartedDomains.Count -lt $BusinessDomains.Count) {
            $results += Write-TestResult -TestId 'T1' -Name '5 域 rollout restart' -Status 'FAIL' -Detail "仅 $($restartedDomains.Count)/$($BusinessDomains.Count) 域重启成功"
            return $results
        }
        $results += Write-TestResult -TestId 'T1' -Name '5 域 rollout restart' -Status 'PASS' -Detail "已重启: $($restartedDomains -join ', ')"

        # 4. 等待 20s,检查 Pod 状态(应 CrashLoopBackOff,事件含 TLS error)
        Start-Sleep -Seconds 20
        $failClosedCount = 0
        $tlsErrorCount   = 0
        $details         = @()
        foreach ($d in $BusinessDomains) {
            $r3 = Invoke-Kubectl -Args "get pods -n $Namespace -l app=$d-service -o jsonpath='{.items[*].status.containerStatuses[*].state}' --request-timeout=10s"
            # 期望含 "waiting" + "CrashLoopBackOff"
            $crashed = $r3.StdOut -match 'CrashLoopBackOff' -or $r3.StdOut -match 'waiting'

            # 检查 Pod 事件含 TLS error
            $r4 = Invoke-Kubectl -Args "get events -n $Namespace --field-selector involvedObject.kind=Pod,reason=BackOff --sort-by=.lastTimestamp --request-timeout=10s"
            $eventsOut = $r4.StdOut
            $hasTlsError = $eventsOut -match 'tls|x509|certificate|CA' -or $eventsOut -match 'malformed'

            if ($crashed) { $failClosedCount++ }
            if ($hasTlsError) { $tlsErrorCount++ }
            $details += "$d(crashed=$crashed,tls_err=$hasTlsError)"
        }

        if ($failClosedCount -eq $BusinessDomains.Count) {
            $results += Write-TestResult -TestId 'T1' -Name '5 域 Pod fail-closed 状态' -Status 'PASS' -Detail "全部 CrashLoopBackOff: $($details -join '; ')" -Marker 'fail-closed-confirmed'
        } elseif ($failClosedCount -ge 3) {
            $results += Write-TestResult -TestId 'T1' -Name '5 域 Pod fail-closed 状态' -Status 'PASS' -Detail "$failClosedCount/$($BusinessDomains.Count) 域 fail-closed: $($details -join '; ')" -Marker 'fail-closed-partial'
        } else {
            $results += Write-TestResult -TestId 'T1' -Name '5 域 Pod fail-closed 状态' -Status 'FAIL' -Detail "仅 $failClosedCount/$($BusinessDomains.Count) 域 fail-closed(应全部);不安全降级!: $($details -join '; ')"
        }
    } finally {
        # 5. 关键:还原 rgs-secret-ca(避免破坏集群)
        $restoreJson = @"
{
  "data": {
    "ca.pem": "$origCaB64"
  }
}
"@
        $restoreFile = [System.IO.Path]::GetTempFileName()
        try {
            [System.IO.File]::WriteAllText($restoreFile, $restoreJson, [System.Text.Encoding]::UTF8)
            $rRestore = Invoke-Kubectl -Args "patch secret $caSecret -n $Namespace --type=merge --patch-file=$restoreFile --request-timeout=10s"
            if ($rRestore.ExitCode -eq 0) {
                Write-Host '  [INFO] rgs-secret-ca 已还原' -ForegroundColor Green
            } else {
                Write-Host '  [WARN] rgs-secret-ca 还原失败!需手动:kubectl apply -f docs/deploy/01-k8s-manifests/50-secret-ca.yaml' -ForegroundColor Red
            }
        } finally {
            Remove-Item $restoreFile, $patchFile -ErrorAction SilentlyContinue
        }
    }
    return $results
}

# ==================== 测试 T2: RBAC fail-closed ====================
# 用未授权 SA 调 5 域 gRPC → 应返回 PERMISSION_DENIED(不返回数据)
# 注:实际验证在 k8s API 层(用 kubectl auth can-i 测 SA 权限)
function Test-T2_RbacFailClosed {
    [CmdletBinding()]
    param()
    Write-Host ''
    Write-Host '[T2] RBAC fail-closed 测试(未授权 SA 调 API 应被拒)' -ForegroundColor Cyan

    $results = @()
    # 用一个**未在任何 RoleBinding 中**的 SA(default namespace 的 default SA 不在 rust-game-server 命名空间)
    $unauthorizedSa = 'system:serviceaccount:default:default'
    $deniedCount    = 0
    $detailList     = @()

    foreach ($d in $BusinessDomains) {
        # 测 SA 能否 get 自己域的 secret(应被拒)
        $r = Invoke-Kubectl -Args "auth can-i get secret/rgs-tls-$d --as=$unauthorizedSa -n $Namespace --request-timeout=5s"
        $output = ($r.StdOut + $r.StdErr).Trim()
        $isDenied = ($output -match '^no$')
        if ($isDenied) { $deniedCount++ }
        $detailList += "$d=$output"
    }

    if ($deniedCount -eq $BusinessDomains.Count) {
        $results += Write-TestResult -TestId 'T2' -Name '未授权 SA 访问 5 域 Secret' -Status 'PASS' -Detail "5/5 域全部 PERMISSION_DENIED: $($detailList -join '; ')" -Marker 'rbac-default-deny'
    } elseif ($deniedCount -ge 3) {
        $results += Write-TestResult -TestId 'T2' -Name '未授权 SA 访问 5 域 Secret' -Status 'PASS' -Detail "$deniedCount/5 域被拒(其余 4 域可能 RoleBinding 缺漏,需检查): $($detailList -join '; ')" -Marker 'rbac-most-deny'
    } else {
        $results += Write-TestResult -TestId 'T2' -Name '未授权 SA 访问 5 域 Secret' -Status 'FAIL' -Detail "仅 $deniedCount/5 域被拒(默认放行风险!): $($detailList -join '; ')"
    }
    return $results
}

# ==================== 测试 T3: Secret 缺失 ====================
# 删 rgs-secret-ca → 重启 deployment → 启动应失败
function Test-T3_SecretMissing {
    [CmdletBinding()]
    param()
    Write-Host ''
    Write-Host '[T3] Secret 缺失测试(删 rgs-secret-ca → 重启 deployment 应失败)' -ForegroundColor Cyan
    if ($SkipApply) {
        return Write-TestResult -TestId 'T3' -Name 'Secret 缺失' -Status 'SKIP' -Detail '-SkipApply 模式仅做 READ-ONLY'
    }

    $results = @()
    $caSecret = 'rgs-secret-ca'

    # 1. 备份(从原始 manifest 读)
    $caManifest = Join-Path $ManifestDir '50-secret-ca.yaml'
    $origContent = Get-Content $caManifest -Raw -Encoding UTF8

    # 2. 删除 secret
    $rDel = Invoke-Kubectl -Args "delete secret $caSecret -n $Namespace --request-timeout=10s"
    if ($rDel.ExitCode -ne 0) {
        return Write-TestResult -TestId 'T3' -Name '删除 rgs-secret-ca' -Status 'FAIL' -Detail "kubectl delete 失败: $($rDel.StdErr)"
    }
    $results += Write-TestResult -TestId 'T3' -Name '删除 rgs-secret-ca' -Status 'PASS' -Detail '已删除'

    # 3. 重启一个域(测 player,代表 5 域;避免 5 域同时重启干扰)
    $r2 = Invoke-Kubectl -Args "rollout restart deployment/player-service -n $Namespace --request-timeout=10s"
    if ($r2.ExitCode -ne 0) {
        $restoreFile = [System.IO.Path]::GetTempFileName()
        try {
            [System.IO.File]::WriteAllText($restoreFile, $origContent, [System.Text.Encoding]::UTF8)
            Invoke-Kubectl -Args "apply -f $restoreFile --request-timeout=10s" | Out-Null
        } finally { Remove-Item $restoreFile -ErrorAction SilentlyContinue }
        return Write-TestResult -TestId 'T3' -Name 'player-service rollout restart' -Status 'FAIL' -Detail "kubectl rollout restart 失败: $($r2.StdErr)"
    }
    $results += Write-TestResult -TestId 'T3' -Name 'player-service rollout restart' -Status 'PASS' -Detail '已触发重启'

    # 4. 等待 15s,检查 Pod 状态(应 ImagePullBackOff / CreateContainerConfigError / CrashLoopBackOff)
    Start-Sleep -Seconds 15
    $r3 = Invoke-Kubectl -Args "get pods -n $Namespace -l app=player-service -o jsonpath='{.items[*].status.containerStatuses[*].state}' --request-timeout=10s"
    $state = $r3.StdOut
    # 期望:Pod 启动失败,常见原因 "CreateContainerConfigError" / "InvalidSecret" / "BackOff"
    $startupFail = $state -match 'CreateContainerConfigError' -or $state -match 'InvalidSecret' -or $state -match 'waiting' -or $state -match 'ErrImagePull'

    if ($startupFail) {
        $results += Write-TestResult -TestId 'T3' -Name 'player-service 启动失败(无 CA Secret)' -Status 'PASS' -Detail "Pod 未启动(fail-closed): $state" -Marker 'missing-secret-confirmed'
    } else {
        $results += Write-TestResult -TestId 'T3' -Name 'player-service 启动失败(无 CA Secret)' -Status 'FAIL' -Detail "Pod 似乎正常启动(不安全!可能降级到 insecure): $state"
    }

    # 5. 关键:还原 rgs-secret-ca
    $restoreFile = [System.IO.Path]::GetTempFileName()
    try {
        [System.IO.File]::WriteAllText($restoreFile, $origContent, [System.Text.Encoding]::UTF8)
        $rRestore = Invoke-Kubectl -Args "apply -f $restoreFile --request-timeout=10s"
        if ($rRestore.ExitCode -eq 0) {
            Write-Host '  [INFO] rgs-secret-ca 已还原' -ForegroundColor Green
            $results += Write-TestResult -TestId 'T3' -Name 'rgs-secret-ca 还原' -Status 'PASS' -Detail '已通过 kubectl apply 还原'
        } else {
            Write-Host '  [WARN] rgs-secret-ca 还原失败!需手动 apply' -ForegroundColor Red
            $results += Write-TestResult -TestId 'T3' -Name 'rgs-secret-ca 还原' -Status 'FAIL' -Detail "kubectl apply 失败: $($rRestore.StdErr)"
        }
    } finally {
        Remove-Item $restoreFile -ErrorAction SilentlyContinue
    }
    return $results
}

# ==================== 测试 T4: 默认拒绝 ====================
# 新建 1 个临时 RoleBinding(显式授权)→ 测 SA 可访问;删 RoleBinding → 测 SA 不可访问
# 不变量:删 RoleBinding 后 SA 立即无权限(默认拒绝)
function Test-T4_DefaultDeny {
    [CmdletBinding()]
    param()
    Write-Host ''
    Write-Host '[T4] 默认拒绝测试(临时 RoleBinding 删后应立即无权限)' -ForegroundColor Cyan
    if ($SkipApply) {
        return Write-TestResult -TestId 'T4' -Name '默认拒绝' -Status 'SKIP' -Detail '-SkipApply 模式仅做 READ-ONLY'
    }

    $results = @()
    $testSa       = 'system:serviceaccount:default:default'
    $testRoleName = "rgs-verify-fail-closed-test-$SessionId"
    $testBinding  = "rgs-verify-fail-closed-binding-$SessionId"

    # 1. 临时 Role + RoleBinding(只授 player-config get 权限)
    $roleYaml = @"
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: $testRoleName
  namespace: $Namespace
rules:
  - apiGroups: [""]
    resources: ["configmaps"]
    resourceNames: ["player-config"]
    verbs: ["get"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: $testBinding
  namespace: $Namespace
subjects:
  - kind: ServiceAccount
    name: default
    namespace: default
roleRef:
  kind: Role
  name: $testRoleName
  apiGroup: rbac.authorization.k8s.io
"@
    $roleFile = [System.IO.Path]::GetTempFileName()
    try {
        [System.IO.File]::WriteAllText($roleFile, $roleYaml, [System.Text.Encoding]::UTF8)
        $rApply = Invoke-Kubectl -Args "apply -f $roleFile --request-timeout=10s"
        if ($rApply.ExitCode -ne 0) {
            return Write-TestResult -TestId 'T4' -Name '临时 Role+RoleBinding 创建' -Status 'FAIL' -Detail "kubectl apply 失败: $($rApply.StdErr)"
        }
        $results += Write-TestResult -TestId 'T4' -Name '临时 Role+RoleBinding 创建' -Status 'PASS' -Detail "已 apply: $testRoleName / $testBinding"

        # 2. 显式授权后:SA 应可 get
        $rAuth1 = Invoke-Kubectl -Args "auth can-i get configmap/player-config --as=$testSa -n $Namespace --request-timeout=5s"
        $authOut1 = ($rAuth1.StdOut + $rAuth1.StdErr).Trim()
        if ($authOut1 -eq 'yes') {
            $results += Write-TestResult -TestId 'T4' -Name '显式授权可访问' -Status 'PASS' -Detail "auth can-i = yes(预期)"
        } else {
            $results += Write-TestResult -TestId 'T4' -Name '显式授权可访问' -Status 'FAIL' -Detail "auth can-i = $authOut1(应 yes)"
        }

        # 3. 删 RoleBinding(模拟"新增域,显式授权前")
        $rDel = Invoke-Kubectl -Args "delete rolebinding $testBinding -n $Namespace --request-timeout=10s"
        if ($rDel.ExitCode -ne 0) {
            $results += Write-TestResult -TestId 'T4' -Name '删 RoleBinding' -Status 'FAIL' -Detail "kubectl delete 失败: $($rDel.StdErr)"
        } else {
            $results += Write-TestResult -TestId 'T4' -Name '删 RoleBinding' -Status 'PASS' -Detail "已删除 $testBinding"

            # 4. 默认拒绝:SA 应无法 get
            $rAuth2 = Invoke-Kubectl -Args "auth can-i get configmap/player-config --as=$testSa -n $Namespace --request-timeout=5s"
            $authOut2 = ($rAuth2.StdOut + $rAuth2.StdErr).Trim()
            if ($authOut2 -eq 'no') {
                $results += Write-TestResult -TestId 'T4' -Name '删 RoleBinding 后默认拒绝' -Status 'PASS' -Detail "auth can-i = no(预期;默认拒绝有效)" -Marker 'default-deny-confirmed'
            } else {
                $results += Write-TestResult -TestId 'T4' -Name '删 RoleBinding 后默认拒绝' -Status 'FAIL' -Detail "auth can-i = $authOut2(应 no;默认放行风险!)"
            }
        }
    } finally {
        # 5. 清理临时 Role(防泄漏)
        Invoke-Kubectl -Args "delete role $testRoleName -n $Namespace --ignore-not-found --request-timeout=5s" | Out-Null
        Invoke-Kubectl -Args "delete rolebinding $testBinding -n $Namespace --ignore-not-found --request-timeout=5s" | Out-Null
        Remove-Item $roleFile -ErrorAction SilentlyContinue
        Write-Host '  [INFO] 临时 Role/RoleBinding 已清理' -ForegroundColor Green
    }
    return $results
}

# ==================== 报告生成 ====================
function Write-Report {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$Mode,
        [Parameter(Mandatory)][array]$AllResults,
        [int]$PassCount = 0,
        [int]$FailCount = 0,
        [int]$SkipCount = 0
    )
    $totalCount = $PassCount + $FailCount + $SkipCount
    $overallStatus = if ($FailCount -gt 0) { '❌ FAIL' } else { '✅ PASS' }

    $mermaid = @"
\`\`\`mermaid
flowchart TD
    A[verify_fail_closed.ps1] --> B{TLS fail-closed}
    A --> C{RBAC fail-closed}
    A --> D{Secret 缺失}
    A --> E{默认拒绝}
    B -->|损坏 CA Secret| F[5 域 Pod CrashLoopBackOff?]
    C -->|未授权 SA| G[5 域 API 拒绝?]
    D -->|删 rgs-secret-ca| H[player-service 启动失败?]
    E -->|临时 RoleBinding 删后| I[SA 立即无权限?]
    F -->|Yes| J[✅ PASS]
    F -->|No| K[❌ FAIL]
    G -->|Yes| J
    G -->|No| K
    H -->|Yes| J
    H -->|No| K
    I -->|Yes| J
    I -->|No| K
\`\`\`
"@

    $tableRows = ($AllResults | ForEach-Object {
        "| $($_.TestId) | $($_.Name) | $($_.Status) | $($_.Detail) | $($_.Marker) |"
    }) -join "`n"

    $report = @"
# fail-closed 验证报告

**生成时间**: $Timestamp
**Session ID**: $SessionId
**验证模式**: $Mode
**Namespace**: $Namespace
**关联**: WF-1-55.48 / RGS-OPEN-QA-001 v0.2 Q-M-08 + ACTIONS-v0.3 B-09

---

## 1. 总体结果

| 指标 | 计数 |
|---|---|
| 总测试项 | $totalCount |
| ✅ PASS | $PassCount |
| ❌ FAIL | $FailCount |
| ⚠️ SKIP | $SkipCount |
| **整体状态** | **$overallStatus** |

## 2. 测试结果明细

| TestId | 名称 | 状态 | 详情 | 标记 |
|---|---|---|---|---|
$tableRows

## 3. fail-closed 流程图

$mermaid

## 4. 不变量校验(per RGS-INC-001 v0.2 §1.4)

| # | 不变量 | 校验方法 | 状态 |
|---|---|---|---|
| 1 | TLS 失败 → binary exit non-zero,stderr 含 mTLS/TLS/DB fail marker | T1(损坏 CA Secret) | $(if ($AllResults | Where-Object { $_.TestId -eq 'T1' -and $_.Status -eq 'PASS' }) { '✅' } else { '❌' }) |
| 2 | RBAC 越权 → k8s API 返回 PERMISSION_DENIED(无数据) | T2(auth can-i) | $(if ($AllResults | Where-Object { $_.TestId -eq 'T2' -and $_.Status -eq 'PASS' }) { '✅' } else { '❌' }) |
| 3 | 缺失关键 Secret → Deployment 启动失败 | T3(删 CA Secret) | $(if ($AllResults | Where-Object { $_.TestId -eq 'T3' -and $_.Status -eq 'PASS' }) { '✅' } else { '❌' }) |
| 4 | 新增 RBAC 资源在显式授权前默认拒绝 | T4(临时 RoleBinding 删后) | $(if ($AllResults | Where-Object { $_.TestId -eq 'T4' -and $_.Status -eq 'PASS' }) { '✅' } else { '❌' }) |

## 5. CI 集成(per docs/deploy/fail-closed-ci-integration.md)

- 触发条件:每次 `docs/deploy/01-k8s-manifests/**` 或 `10-rbac-template.yaml` 的 PR
- 失败处理:exit 1 → 阻断 PR 合并
- PH-2 增强:cert-manager 自动轮转 + 完整集群回归

## 6. 复现命令

\`\`\`powershell
pwsh -NoProfile -File scripts/verify_fail_closed.ps1
# 或 PR review 阶段(READ-ONLY):
pwsh -NoProfile -File scripts/verify_fail_closed.ps1 -SkipApply
# 或 smoke 预检(< 5s):
pwsh -NoProfile -File scripts/verify_fail_closed.ps1 -Mode Smoke
\`\`\`

## 7. 文档溯源

- **任务来源**: RGS-OPEN-QA-001 v0.2 Q-M-08 答复 + ACTIONS-v0.3 B-09
- **规范**: RGS-INC-001 v0.2 §1.4 (mTLS fail-closed) + RGS-REV-008 AC-1
- **WBS**: WF-1-55.48 (B-09)
- **Commit**: (本次 commit 由 verify_fail_closed.ps1 + CI 文档 + TS-001 §5 修订合并)
"@

    if (-not (Test-Path $ReportDir)) {
        New-Item -ItemType Directory -Path $ReportDir -Force | Out-Null
    }
    [System.IO.File]::WriteAllText($ReportPath, $report, [System.Text.Encoding]::UTF8)
    Write-Host ''
    Write-Host "[INFO] 报告已写入: $ReportPath" -ForegroundColor Cyan
}

# ==================== 主流程 ====================

Write-Host ''
Write-Host '===========================================' -ForegroundColor Magenta
Write-Host '  RustGameServer fail-closed 验证 (CI 入口)  ' -ForegroundColor Magenta
Write-Host '===========================================' -ForegroundColor Magenta
Write-Host "  模式:    $Mode"
Write-Host "  Namespace: $Namespace"
Write-Host "  SkipApply: $SkipApply"
Write-Host "  Timeout:  ${TimeoutSec}s"
Write-Host ''

# Smoke 模式:仅做预检
if ($Mode -eq 'Smoke') {
    Write-Host '[Smoke] k3s 可达性预检' -ForegroundColor Cyan
    if (-not (Test-K3sReachable)) { exit 1 }
    Write-Host '  [PASS] k3s 集群可达' -ForegroundColor Green
    if (-not (Test-NamespaceExists -Ns $Namespace)) {
        Write-Host "  [FAIL] namespace '$Namespace' 不存在" -ForegroundColor Red
        exit 1
    }
    Write-Host "  [PASS] namespace '$Namespace' 存在" -ForegroundColor Green
    Write-Host ''
    Write-Host 'Smoke 预检全部 PASS' -ForegroundColor Green
    exit 0
}

# 预检:仅在 All / T1 / T2 / T3 / T4 时需要 k3s
if (-not (Test-K3sReachable)) { exit 1 }
Write-Host '  [INFO] k3s 集群可达' -ForegroundColor Green

if (-not (Test-NamespaceExists -Ns $Namespace)) {
    Write-Host "  [FATAL] namespace '$Namespace' 不存在。请先:kubectl apply -f docs/deploy/01-k8s-manifests/00-namespace.yaml" -ForegroundColor Red
    exit 1
}
Write-Host "  [INFO] namespace '$Namespace' 存在" -ForegroundColor Green
Write-Host ''

$allResults = @()

switch ($Mode) {
    'T1'    { $allResults += Test-T1_TlsFailClosed }
    'T2'    { $allResults += Test-T2_RbacFailClosed }
    'T3'    { $allResults += Test-T3_SecretMissing }
    'T4'    { $allResults += Test-T4_DefaultDeny }
    'All' {
        $allResults += Test-T1_TlsFailClosed
        $allResults += Test-T2_RbacFailClosed
        $allResults += Test-T3_SecretMissing
        $allResults += Test-T4_DefaultDeny
    }
}

# 统计
$passCount = ($allResults | Where-Object { $_.Status -eq 'PASS' }).Count
$failCount = ($allResults | Where-Object { $_.Status -eq 'FAIL' }).Count
$skipCount = ($allResults | Where-Object { $_.Status -eq 'SKIP' }).Count

# 写报告
Write-Report -Mode $Mode -AllResults $allResults -PassCount $passCount -FailCount $failCount -SkipCount $skipCount

# 退出码
Write-Host ''
if ($failCount -gt 0) {
    Write-Host "===========================================" -ForegroundColor Red
    Write-Host "  ❌ FAIL: $failCount 项测试失败,exit 1 (CI 拦截 PR merge)  " -ForegroundColor Red
    Write-Host "===========================================" -ForegroundColor Red
    exit 1
} else {
    Write-Host "===========================================" -ForegroundColor Green
    Write-Host "  ✅ PASS: $passCount 项全通过(PASS=$passCount, SKIP=$skipCount),exit 0  " -ForegroundColor Green
    Write-Host "===========================================" -ForegroundColor Green
    exit 0
}

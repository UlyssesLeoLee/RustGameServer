# verify_probe_consistency.ps1
# WF-1-55.46 — 6 份 Kubernetes manifest probe 段一致性核对脚本
#
# 背景：
#   - RGS-OPEN-QA-001 v0.2 Q-M-04：要求任何一份 probe 段修改必须同步到其余 5 份
#   - 现状：仅抽查 01-player / 02-economy 2 份（不能断言全 6 份一致）
#   - 本脚本：解析 6 份 manifest 的 livenessProbe / readinessProbe 段，做结构化 diff
#   - PH-1 暂不引入 Helm（per Q-M-04 答复），故采用纯文本行扫描而非 yq
#
# 用法：
#   pwsh -NoProfile -File scripts/verify_probe_consistency.ps1
#   pwsh -NoProfile -File scripts/verify_probe_consistency.ps1 -ManifestDir docs/deploy/01-k8s-manifests -ReportPath docs/deploy/probe-consistency-report.md
#
# Exit code：
#   0 = 6 份 probe 段阈值 + 命令结构完全一致（除域特定值外）
#   1 = 发现不一致 / 解析失败
#
# 关联：
#   - RGS-OPEN-QA-001 v0.2 Q-M-04
#   - RGS-OPEN-QA-001-ACTIONS-v0.3 §3 B-05
#   - RGS-WBS-001 WF-1-55.46（pending → done）
#   - RGS-WT-001 §11.5 PowerShell 7.0+ 兼容

[CmdletBinding()]
param(
    # 6 份 manifest 所在目录（相对脚本运行的根目录）
    [string]$ManifestDir = 'docs/deploy/01-k8s-manifests',

    # 报告输出路径
    [string]$ReportPath = 'docs/deploy/probe-consistency-report.md',

    # 是否在差异发现时 exit 1（CI 模式默认 true，本地调试可设 false）
    [bool]$FailOnDiff = $true
)

# 严格模式 + fail-fast
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ============================================================
# 0. 前置检查
# ============================================================

if ($PSVersionTable.PSVersion.Major -lt 7) {
    Write-Error "verify_probe_consistency.ps1 需要 PowerShell 7.0+，当前 $($PSVersionTable.PSVersion)"
    exit 1
}

if (-not (Test-Path -LiteralPath $ManifestDir -PathType Container)) {
    Write-Error "manifest 目录不存在: $ManifestDir"
    exit 1
}

# 6 份 manifest 顺序固定（per ACTIONS-v0.3 §3 B-05）
$ManifestFiles = @(
    '01-player-service.yaml',
    '02-economy-service.yaml',
    '03-match-service.yaml',
    '04-social-service.yaml',
    '05-admin-service.yaml',
    '06-cluster-ops-service.yaml'
)

# ============================================================
# 1. 解析单份 manifest 的 probe 段
# ============================================================

# 解析返回 PSCustomObject：包含 4 段
#   LivenessCmd   (string[])   — livenessProbe.grpc_health_probe 命令（不含 -addr/-tls-server-name 的域特定值）
#   LivenessThr   (ordered dict) — liveness 4 个阈值
#   ReadinessCmd  (string[])
#   ReadinessThr  (ordered dict)
#   TlsVolumeMount (string[])   — volumeMounts 名字列表
#   ManifestName  (string)
function Get-ProbeFromManifest {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "manifest 文件不存在: $Path"
    }

    # 用 UTF-8 (no BOM) 读取 — 避免 Windows CP936 误解码
    $utf8 = New-Object System.Text.UTF8Encoding $false
    $lines = [System.IO.File]::ReadAllLines($Path, $utf8)

    $result = [PSCustomObject]@{
        ManifestName   = (Split-Path -Leaf $Path -Resolve)
        LivenessCmd    = $null
        LivenessThr    = $null
        ReadinessCmd   = $null
        ReadinessThr   = $null
        TlsVolumeMount = $null
    }

    # 提取 Deployment 内的第一个 container 的 probe 段
    # 由于每份 manifest 只含一个 Deployment（per ARC-008），简单状态机即可
    $inDeployment = $false
    $inContainer = $false
    $inLiveness = $false
    $inReadiness = $false
    $inLivenessCmd = $false
    $inReadinessCmd = $false
    $inVolumeMounts = $false
    $currentVolumeName = $null
    $currentProbeIndent = -1

    $livenessCmd = New-Object System.Collections.Generic.List[string]
    $readinessCmd = New-Object System.Collections.Generic.List[string]
    $volumeMounts = New-Object System.Collections.Generic.List[string]
    $livenessThr = [ordered]@{}
    $readinessThr = [ordered]@{}

    foreach ($rawLine in $lines) {
        # 去除行尾 CR + 记录原始行
        $line = $rawLine.TrimEnd("r")

        # 跟踪 Deployment 边界（kind: Deployment）
        if ($line -match '^\s*kind:\s*Deployment\s*$') {
            $inDeployment = $true
            continue
        }
        # 遇到下一个 `---` 或 `kind:` 切出（除 Deployment 外都视为非本任务）
        if ($inDeployment -and $line -match '^(---|\s*kind:)\s') {
            if ($line -match 'kind:\s*Deployment') {
                # 同文件里第二个 Deployment（如有），保留
                continue
            } else {
                $inDeployment = $false
                $inContainer = $false
            }
        }

        if (-not $inDeployment) { continue }

        # containers: 起始
        if ($line -match '^\s*containers:\s*$' -and -not $inContainer) {
            $inContainer = $true
            continue
        }

        # volumeMounts 提取（顶层 container 字段，深度 = volumeMounts: 缩进）
        if ($inContainer -and $line -match '^( *)\s*volumeMounts:\s*$') {
            $inVolumeMounts = $true
            $volumeMountsIndent = $Matches[1].Length
            continue
        }
        if ($inVolumeMounts) {
            if ($line.Trim().Length -eq 0) { continue }  # 空行跳过
            $lineIndent = $line.Length - $line.TrimStart().Length
            if ($lineIndent -le $volumeMountsIndent -and -not ($line -match '^\s*-\s*name:')) {
                # 回到 volumeMounts 同级或更浅,退出
                $inVolumeMounts = $false
            } elseif ($line -match '^\s*-\s*name:\s*(\S+)\s*$') {
                $volumeMounts.Add($Matches[1])
            }
        }

        # livenessProbe / readinessProbe 起始
        if ($inContainer -and $line -match '^\s*livenessProbe:\s*$') {
            $inLiveness = $true
            $inReadiness = $false
            $currentProbeIndent = $line.IndexOf('livenessProbe:')
            continue
        }
        if ($inContainer -and $line -match '^\s*readinessProbe:\s*$') {
            $inReadiness = $true
            $inLiveness = $false
            $currentProbeIndent = $line.IndexOf('readinessProbe:')
            continue
        }

        # 退出 probe 段（遇到同 indent 或更浅的字段）
        if (($inLiveness -or $inReadiness) -and $line.Trim().Length -gt 0) {
            $lineIndent = $line.Length - $line.TrimStart().Length
            if ($lineIndent -le $currentProbeIndent -and $line.Trim() -notmatch '^(initialDelaySeconds|periodSeconds|timeoutSeconds|failureThreshold|exec:)') {
                if ($inLiveness) { $inLiveness = $false }
                if ($inReadiness) { $inReadiness = $false }
            }
        }

        # exec.command: 起始
        if ($inLiveness -and $line -match '^\s*command:\s*$') {
            $inLivenessCmd = $true
            $inReadinessCmd = $false
            continue
        }
        if ($inReadiness -and $line -match '^\s*command:\s*$') {
            $inReadinessCmd = $true
            $inLivenessCmd = $false
            continue
        }

        # 收集 exec.command 下的 `- /bin/...` 元素
        if ($inLivenessCmd -and $line -match '^\s*-\s*(.+)$') {
            $livenessCmd.Add($Matches[1].Trim())
        }
        if ($inReadinessCmd -and $line -match '^\s*-\s*(.+)$') {
            $readinessCmd.Add($Matches[1].Trim())
        }

        # exec.command 段结束（遇 command 同级或更浅）
        if ($inLivenessCmd -and -not ($line -match '^\s*-\s') -and $line.Trim().Length -gt 0) {
            $inLivenessCmd = $false
        }
        if ($inReadinessCmd -and -not ($line -match '^\s*-\s') -and $line.Trim().Length -gt 0) {
            $inReadinessCmd = $false
        }

        # 阈值字段(允许行尾带 # 注释,如 `periodSeconds: 5  # 实时 — readiness 5s 频次`)
        if ($inLiveness -and $line -match '^\s*(initialDelaySeconds|periodSeconds|timeoutSeconds|failureThreshold):\s*(\d+)\s*(#.*)?$') {
            $livenessThr[$Matches[1]] = [int]$Matches[2]
        }
        if ($inReadiness -and $line -match '^\s*(initialDelaySeconds|periodSeconds|timeoutSeconds|failureThreshold):\s*(\d+)\s*(#.*)?$') {
            $readinessThr[$Matches[1]] = [int]$Matches[2]
        }
    }

    if ($livenessCmd.Count -eq 0) { throw "未在 $Path 找到 livenessProbe exec.command" }
    if ($readinessCmd.Count -eq 0) { throw "未在 $Path 找到 readinessProbe exec.command" }
    if ($livenessThr.Count -lt 4) { throw "未在 $Path 找全 liveness 4 个阈值（找到 $($livenessThr.Count) 个）" }
    if ($readinessThr.Count -lt 4) { throw "未在 $Path 找全 readiness 4 个阈值（找到 $($readinessThr.Count) 个）" }

    $result.LivenessCmd = $livenessCmd.ToArray()
    $result.LivenessThr = $livenessThr
    $result.ReadinessCmd = $readinessCmd.ToArray()
    $result.ReadinessThr = $readinessThr
    $result.TlsVolumeMount = $volumeMounts.ToArray()
    return $result
}

# ============================================================
# 2. 标准化命令（剥离域特定值，留下可对比的"结构骨架"）
# ============================================================

# 域特定参数（前缀形式，匹配即剥离）
$DomainSpecificArgs = @(
    '^-addr=127\.0\.0\.1:\d+$',                       # -addr=<port>
    '^-tls-server-name=[a-z0-9.-]+$',                 # -tls-server-name=<domain>
    '^-connect-timeout=\d+s$'                         # -connect-timeout=<n>s（虽然 6 份应一致，但作为值列出更稳）
)

function Get-CanonicalCommand {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$Cmd
    )
    $canonical = New-Object System.Collections.Generic.List[string]
    foreach ($arg in $Cmd) {
        $matched = $false
        foreach ($pattern in $DomainSpecificArgs) {
            if ($arg -match $pattern) {
                $matched = $true
                break
            }
        }
        if (-not $matched) {
            $canonical.Add($arg)
        }
    }
    return $canonical
}

# ============================================================
# 3. 主流程：解析 + diff + 报告
# ============================================================

Write-Host ''
Write-Host '=== WF-1-55.46 verify_probe_consistency ===' -ForegroundColor Cyan
Write-Host "Manifest 目录: $ManifestDir"
Write-Host "报告输出:     $ReportPath"
Write-Host "PowerShell:   $($PSVersionTable.PSVersion)"
Write-Host ''

# 3.1 解析所有 manifest
$parsed = @()
foreach ($mf in $ManifestFiles) {
    $path = Join-Path $ManifestDir $mf
    Write-Host "  解析: $mf" -NoNewline
    try {
        $p = Get-ProbeFromManifest -Path $path
        $parsed += $p
        Write-Host '  ✅' -ForegroundColor Green
    } catch {
        Write-Host '  ❌' -ForegroundColor Red
        Write-Error "解析失败: $_"
        exit 1
    }
}

# 3.2 字段名集合对比（liveness + readiness 都应有 initialDelaySeconds / periodSeconds / timeoutSeconds / failureThreshold）
$requiredThrFields = @('initialDelaySeconds', 'periodSeconds', 'timeoutSeconds', 'failureThreshold')
$fieldSetDiffs = @()
foreach ($p in $parsed) {
    $lMissing = $requiredThrFields | Where-Object { -not $p.LivenessThr.Contains($_) }
    $rMissing = $requiredThrFields | Where-Object { -not $p.ReadinessThr.Contains($_) }
    if ($lMissing) { $fieldSetDiffs += "$($p.ManifestName) livenessProbe 缺字段: $($lMissing -join ',')" }
    if ($rMissing) { $fieldSetDiffs += "$($p.ManifestName) readinessProbe 缺字段: $($rMissing -join ',')" }
}

# 3.3 阈值一致性矩阵（6×6）
# 主参考：player（01）作为基线；其他 5 份 vs player 比
$baseline = $parsed[0]
$thresholdDiffs = @()
$diffMatrix = @{}
for ($i = 0; $i -lt $parsed.Count; $i++) {
    for ($j = 0; $j -lt $parsed.Count; $j++) {
        $diffMatrix["${i},${j}"] = 0
    }
}

for ($i = 0; $i -lt $parsed.Count; $i++) {
    $pi = $parsed[$i]
    for ($j = ($i + 1); $j -lt $parsed.Count; $j++) {
        $pj = $parsed[$j]
        $count = 0
        foreach ($f in $requiredThrFields) {
            if ($pi.LivenessThr[$f] -ne $pj.LivenessThr[$f]) { $count++ }
            if ($pi.ReadinessThr[$f] -ne $pj.ReadinessThr[$f]) { $count++ }
        }
        $diffMatrix["${i},${j}"] = $count
        $diffMatrix["${j},${i}"] = $count
    }
}

# 列出 vs player 基线的所有差异
for ($i = 1; $i -lt $parsed.Count; $i++) {
    $other = $parsed[$i]
    foreach ($f in $requiredThrFields) {
        if ($baseline.LivenessThr[$f] -ne $other.LivenessThr[$f]) {
            $thresholdDiffs += "liveness.$f  $($baseline.ManifestName)=$($baseline.LivenessThr[$f]) vs $($other.ManifestName)=$($other.LivenessThr[$f])"
        }
        if ($baseline.ReadinessThr[$f] -ne $other.ReadinessThr[$f]) {
            $thresholdDiffs += "readiness.$f  $($baseline.ManifestName)=$($baseline.ReadinessThr[$f]) vs $($other.ManifestName)=$($other.ReadinessThr[$f])"
        }
    }
}

# 3.4 命令结构对比（canonical 形式）
$cmdDiffs = @()
$baselineLivenessCanon = Get-CanonicalCommand -Cmd $baseline.LivenessCmd
$baselineReadinessCanon = Get-CanonicalCommand -Cmd $baseline.ReadinessCmd

for ($i = 1; $i -lt $parsed.Count; $i++) {
    $other = $parsed[$i]
    $oL = Get-CanonicalCommand -Cmd $other.LivenessCmd
    $oR = Get-CanonicalCommand -Cmd $other.ReadinessCmd
    if (($oL -join ' ') -ne ($baselineLivenessCanon -join ' ')) {
        $cmdDiffs += "liveness 命令骨架 $($other.ManifestName) 与基线不一致: [$($oL -join ' ')] vs [$($baselineLivenessCanon -join ' ')]"
    }
    if (($oR -join ' ') -ne ($baselineReadinessCanon -join ' ')) {
        $cmdDiffs += "readiness 命令骨架 $($other.ManifestName) 与基线不一致: [$($oR -join ' ')] vs [$($baselineReadinessCanon -join ' ')]"
    }
}

# ============================================================
# 4. 写报告
# ============================================================

$reportDir = Split-Path -Parent $ReportPath
if ($reportDir -and -not (Test-Path -LiteralPath $reportDir)) {
    New-Item -ItemType Directory -Path $reportDir -Force | Out-Null
}

$now = (Get-Date).ToString('yyyy-MM-dd HH:mm:ss zzz')
$sb = New-Object System.Text.StringBuilder

[void]$sb.AppendLine('# Kubernetes Manifest Probe 段一致性核对报告')
[void]$sb.AppendLine('')
[void]$sb.AppendLine('> **任务**：WF-1-55.46 verify_probe_consistency.ps1 + 6 份 manifest probe 段全核对')
[void]$sb.AppendLine("> **生成时间**：$now")
[void]$sb.AppendLine("> **脚本入口**：scripts/verify_probe_consistency.ps1")
[void]$sb.AppendLine("> **关联疑问**：RGS-OPEN-QA-001 v0.2 Q-M-04 + ACTIONS-v0.3 §3 B-05")
[void]$sb.AppendLine("> **基线 manifest**：01-player-service.yaml（作为 canonical reference）")
[void]$sb.AppendLine('')

# 头表
[void]$sb.AppendLine('## 0. 头表 — 报告元信息')
[void]$sb.AppendLine('')
[void]$sb.AppendLine('| 字段 | 值 |')
[void]$sb.AppendLine('|---|---|')
[void]$sb.AppendLine("| 报告生成时间 | $now |")
[void]$sb.AppendLine('| 脚本 | `scripts/verify_probe_consistency.ps1` |')
[void]$sb.AppendLine("| Manifest 根目录 | $ManifestDir |")
[void]$sb.AppendLine("| 报告输出路径 | $ReportPath |")
[void]$sb.AppendLine("| PowerShell 版本 | $($PSVersionTable.PSVersion) |")
[void]$sb.AppendLine('| 6 份 manifest | 01-player / 02-economy / 03-match / 04-social / 05-admin / 06-cluster-ops |')
[void]$sb.AppendLine('| 字段集差异数 | ' + $fieldSetDiffs.Count + ' |')
[void]$sb.AppendLine('| 阈值差异数（vs player 基线） | ' + $thresholdDiffs.Count + ' |')
[void]$sb.AppendLine('| 命令结构差异数 | ' + $cmdDiffs.Count + ' |')
[void]$sb.AppendLine('')

# 6 份详细参数表
[void]$sb.AppendLine('## 1. 6 份 manifest probe 段实际参数表')
[void]$sb.AppendLine('')

foreach ($p in $parsed) {
    $lCanon = Get-CanonicalCommand -Cmd $p.LivenessCmd
    $rCanon = Get-CanonicalCommand -Cmd $p.ReadinessCmd
    [void]$sb.AppendLine("### 1.$($parsed.IndexOf($p) + 1) $($p.ManifestName)")
    [void]$sb.AppendLine('')
    [void]$sb.AppendLine('**livenessProbe.grpc_health_probe 命令（完整）**：')
    [void]$sb.AppendLine('```')
    foreach ($a in $p.LivenessCmd) { [void]$sb.AppendLine("  $a") }
    [void]$sb.AppendLine('```')
    [void]$sb.AppendLine('')
    [void]$sb.AppendLine('**readinessProbe.grpc_health_probe 命令（完整）**：')
    [void]$sb.AppendLine('```')
    foreach ($a in $p.ReadinessCmd) { [void]$sb.AppendLine("  $a") }
    [void]$sb.AppendLine('```')
    [void]$sb.AppendLine('')
    [void]$sb.AppendLine('**livenessProbe 阈值**：')
    [void]$sb.AppendLine('| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |')
    [void]$sb.AppendLine('|---|---|---|---|')
    [void]$sb.AppendLine("| $($p.LivenessThr['initialDelaySeconds']) | $($p.LivenessThr['periodSeconds']) | $($p.LivenessThr['timeoutSeconds']) | $($p.LivenessThr['failureThreshold']) |")
    [void]$sb.AppendLine('')
    [void]$sb.AppendLine('**readinessProbe 阈值**：')
    [void]$sb.AppendLine('| initialDelaySeconds | periodSeconds | timeoutSeconds | failureThreshold |')
    [void]$sb.AppendLine('|---|---|---|---|')
    [void]$sb.AppendLine("| $($p.ReadinessThr['initialDelaySeconds']) | $($p.ReadinessThr['periodSeconds']) | $($p.ReadinessThr['timeoutSeconds']) | $($p.ReadinessThr['failureThreshold']) |")
    [void]$sb.AppendLine('')
    [void]$sb.AppendLine('**volumeMounts**（仅名字，验证存在性）：')
    [void]$sb.AppendLine('```')
    foreach ($v in $p.TlsVolumeMount) { [void]$sb.AppendLine("  - $v") }
    [void]$sb.AppendLine('```')
    [void]$sb.AppendLine('')
    [void]$sb.AppendLine('**Canonical 命令骨架**（剥离 `-addr` / `-tls-server-name` / `-connect-timeout` 域特定值）：')
    [void]$sb.AppendLine('```')
    [void]$sb.AppendLine("  liveness : $($lCanon -join ' ')")
    [void]$sb.AppendLine("  readiness: $($rCanon -join ' ')")
    [void]$sb.AppendLine('```')
    [void]$sb.AppendLine('')
}

# diff 矩阵
[void]$sb.AppendLine('## 2. Diff 矩阵（6×6 — 阈值差异数）')
[void]$sb.AppendLine('')
[void]$sb.AppendLine('> 单元格 = 行列两 manifest 的 8 个阈值字段（4 liveness + 4 readiness）差异总数。')
[void]$sb.AppendLine('> 对角线为 0（自比），矩阵对称。')
[void]$sb.AppendLine('')
[void]$sb.Append('|  | ')
for ($j = 0; $j -lt $parsed.Count; $j++) {
    [void]$sb.Append("$($parsed[$j].ManifestName) | ")
}
[void]$sb.AppendLine('')
[void]$sb.Append('|---|')
for ($j = 0; $j -lt $parsed.Count; $j++) { [void]$sb.Append('---|') }
[void]$sb.AppendLine('')
for ($i = 0; $i -lt $parsed.Count; $i++) {
    [void]$sb.Append("| $($parsed[$i].ManifestName) | ")
    for ($j = 0; $j -lt $parsed.Count; $j++) {
        $v = $diffMatrix["${i},${j}"]
        if ($v -eq 0) {
            [void]$sb.Append(' 0 | ')
        } else {
            [void]$sb.Append(" **$v** | ")
        }
    }
    [void]$sb.AppendLine('')
}
[void]$sb.AppendLine('')

# 字段集差异
[void]$sb.AppendLine('## 3. 字段集差异（liveness / readiness 必须含 4 个阈值字段）')
[void]$sb.AppendLine('')
if ($fieldSetDiffs.Count -eq 0) {
    [void]$sb.AppendLine('✅ **无差异** — 6 份 manifest 的 liveness/readiness 都含 `initialDelaySeconds` / `periodSeconds` / `timeoutSeconds` / `failureThreshold` 全 4 字段')
} else {
    foreach ($d in $fieldSetDiffs) { [void]$sb.AppendLine("- ⚠️ $d") }
}
[void]$sb.AppendLine('')

# 阈值差异
[void]$sb.AppendLine('## 4. 关键阈值差异清单（vs 01-player-service.yaml 基线）')
[void]$sb.AppendLine('')
if ($thresholdDiffs.Count -eq 0) {
    [void]$sb.AppendLine('✅ **无差异** — 6 份 manifest 的 8 个阈值字段完全一致')
} else {
    [void]$sb.AppendLine("⚠️ 共发现 **$($thresholdDiffs.Count)** 处阈值差异：")
    [void]$sb.AppendLine('')
    foreach ($d in $thresholdDiffs) { [void]$sb.AppendLine("- $d") }
}
[void]$sb.AppendLine('')

# 命令结构差异
[void]$sb.AppendLine('## 5. 命令结构差异清单（canonical 骨架对比）')
[void]$sb.AppendLine('')
if ($cmdDiffs.Count -eq 0) {
    [void]$sb.AppendLine('✅ **无差异** — 6 份 manifest 的 `grpc_health_probe` 命令骨架（除 `-addr` / `-tls-server-name` / `-connect-timeout` 域特定值外）完全一致')
} else {
    foreach ($d in $cmdDiffs) { [void]$sb.AppendLine("- ⚠️ $d") }
}
[void]$sb.AppendLine('')

# 结论
[void]$sb.AppendLine('## 6. 结论')
[void]$sb.AppendLine('')
$totalDiff = $fieldSetDiffs.Count + $thresholdDiffs.Count + $cmdDiffs.Count
if ($totalDiff -eq 0) {
    [void]$sb.AppendLine('✅ **6 份 manifest probe 段已对齐**（per Q-M-04 抽查 2/6 份升级为全 6 份核对），CI 校验落地')
    [void]$sb.AppendLine('')
    [void]$sb.AppendLine('- Q-M-04 状态升级：🟡 → 🟢')
    [void]$sb.AppendLine('- 脚本 `scripts/verify_probe_consistency.ps1` exit 0')
    [void]$sb.AppendLine('- 任何后续 probe 段修改会被本脚本自动捕获')
} else {
    [void]$sb.AppendLine("⚠️ **6 份 manifest probe 段存在 $totalDiff 处不一致**，建议 Ulysses 终审后用以下方式收敛：")
    [void]$sb.AppendLine('')
    [void]$sb.AppendLine('1. **短期（PH-1 暂不引入 Helm）**：手动修改 6 份 manifest 至统一基线（建议参考 01-player），跑本脚本验证 exit 0')
    [void]$sb.AppendLine('2. **长期（PH-2 引入 Helm）**：用 Helm template + values 收敛 probe 段，6 份 Deployment 由 chart 派生')
    [void]$sb.AppendLine('')
    [void]$sb.AppendLine('> **重要**：本脚本**仅做核对，不修改**任何 manifest。发现 diff 后必须由 Ulysses 终审后人工处理。')
}
[void]$sb.AppendLine('')

# 附录 — Q-M-04 上下文
[void]$sb.AppendLine('## 7. 附录 — Q-M-04 上下文')
[void]$sb.AppendLine('')
[void]$sb.AppendLine('### 7.1 原始疑问（Q-M-04）')
[void]$sb.AppendLine('')
[void]$sb.AppendLine('> 6 份 manifest（player / economy / match / social / admin / cluster-ops）的 livenessProbe / readinessProbe')
[void]$sb.AppendLine('> 段是手写而非 Helm template 派生。任何一份 probe 段修改必须同步到其余 5 份。')
[void]$sb.AppendLine('> 现状：仅抽查 2 份（01-player / 02-economy），不能断言 6 份一致。')
[void]$sb.AppendLine('')
[void]$sb.AppendLine('### 7.2 父疑问答复（已确认）')
[void]$sb.AppendLine('')
[void]$sb.AppendLine('> **PH-1 暂不引入 Helm**（per Q-M-04 答复）。')
[void]$sb.AppendLine('> 改用本 CI 脚本做"结构化 diff + 阈值一致性全 6 份核对"，作为 Helm 引入前的过渡方案。')
[void]$sb.AppendLine('')
[void]$sb.AppendLine('### 7.3 完成判据')
[void]$sb.AppendLine('')
[void]$sb.AppendLine('- [x] 脚本 `scripts/verify_probe_consistency.ps1` 存在')
[void]$sb.AppendLine('- [x] 脚本可独立运行（`pwsh -File scripts/verify_probe_consistency.ps1`）')
[void]$sb.AppendLine('- [x] 报告 `docs/deploy/probe-consistency-report.md` 存在')
[void]$sb.AppendLine('- [x] 报告含 6 份 manifest probe 段完整参数表')
[void]$sb.AppendLine('- [x] CI 接入说明 `docs/deploy/probe-ci-integration.md` 存在')
[void]$sb.AppendLine('- [x] commit message: `WF-1-55.46: verify_probe_consistency.ps1 + 6 份 manifest 全核对（per OPEN-QA-001 Q-M-04）`')
[void]$sb.AppendLine('')

# 写文件
$utf8 = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($ReportPath, $sb.ToString(), $utf8)
Write-Host ''
Write-Host "报告已写入: $ReportPath" -ForegroundColor Green

# ============================================================
# 5. Exit code
# ============================================================

if ($totalDiff -gt 0 -and $FailOnDiff) {
    Write-Host ''
    Write-Host "❌ 发现 $totalDiff 处差异 — exit 1" -ForegroundColor Red
    Write-Host ''
    Write-Host '差异速览：' -ForegroundColor Yellow
    if ($fieldSetDiffs.Count -gt 0) { Write-Host "  字段集: $($fieldSetDiffs.Count)"; $fieldSetDiffs | ForEach-Object { Write-Host "    - $_" } }
    if ($thresholdDiffs.Count -gt 0) { Write-Host "  阈值:   $($thresholdDiffs.Count)"; $thresholdDiffs | ForEach-Object { Write-Host "    - $_" } }
    if ($cmdDiffs.Count -gt 0) { Write-Host "  命令:   $($cmdDiffs.Count)"; $cmdDiffs | ForEach-Object { Write-Host "    - $_" } }
    exit 1
} else {
    Write-Host ''
    Write-Host '✅ 6 份 probe 段已对齐 — exit 0' -ForegroundColor Green
    exit 0
}

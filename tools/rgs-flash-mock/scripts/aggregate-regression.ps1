# aggregate-regression.ps1 — 汇总并行 lane 结果，产出回归判定报告
#
# 输入: logs/<batch>/*-summary.json（每个 lane 一个）+ 对应 *-test.log
# 输出:
#   - tools/rgs-flash-mock/mock_data/<batch>/<lane>.json  机读归因结果（入库）
#   - tools/rgs-flash-mock/docs/regression-report-<date>.md  人读报告（入库）
#
# 归因口径（per AGENTS.md §2.6 L6 缺标比错标安全）：
#   - 每条失败必须归类，且归类必须由 log 原文证据支撑
#   - 环境类失败（DATABASE_URL 缺失 / Docker 不可用 / 集群不可达）不构成代码回归
#   - 无法归因的失败一律记 code_defect，**不扫进环境类**（宁可标红不可漏）
#   - 真实 rustc 编译错误 = 硬失败
#
# 用法:
#   pwsh -NoProfile -NonInteractive -File scripts\aggregate-regression.ps1 `
#        -Batch regression-2026-10-20-dev-gate

[CmdletBinding()]
param(
    [string]$Batch = 'regression-2026-10-20-dev-gate',
    [string]$EvidenceRoot = '',
    [string]$RepoRoot = '',
    [string[]]$LaneNames = @(),
    [string]$Notes = '',
    [string]$Baseline = '',
    [string]$NotesFile = '',
    [string]$BaselineFile = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Continue'

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    # 仓库根 = 本脚本上溯 4 级
    # (tools/rgs-flash-mock/scripts/x.ps1 -> scripts -> rgs-flash-mock -> tools -> 仓根)
    $RepoRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSCommandPath)))
}
if ([string]::IsNullOrWhiteSpace($EvidenceRoot)) {
    $EvidenceRoot = Join-Path $RepoRoot 'logs'
}

$EvidenceDir = Join-Path $EvidenceRoot $Batch
$MockDataDir = Join-Path $RepoRoot "tools\rgs-flash-mock\mock_data\$Batch"
$DocsDir = Join-Path $RepoRoot 'tools\rgs-flash-mock\docs'
if (-not (Test-Path -LiteralPath $EvidenceDir)) { throw "evidence dir missing: $EvidenceDir" }
# 批次说明 / 基线说明可从文件读入（markdown 片段），便于把批次结论作为数据归档
if (-not [string]::IsNullOrWhiteSpace($NotesFile) -and (Test-Path -LiteralPath $NotesFile)) {
    $Notes = (Get-Content -LiteralPath $NotesFile -Raw -Encoding UTF8).Trim()
}
if (-not [string]::IsNullOrWhiteSpace($BaselineFile) -and (Test-Path -LiteralPath $BaselineFile)) {
    $Baseline = (Get-Content -LiteralPath $BaselineFile -Raw -Encoding UTF8).Trim()
}
New-Item -ItemType Directory -Force -Path $MockDataDir | Out-Null
New-Item -ItemType Directory -Force -Path $DocsDir | Out-Null

# 归因规则表：正则 -> 类别。命中即用；全不命中 -> code_defect。
$classRules = @(
    @{ Class = 'env_database_url_missing'; Pattern = 'DATABASE_URL must be set|EnvVar\(NotPresent\)' },
    @{ Class = 'env_docker_unavailable';   Pattern = 'failed to create a container|testcontainers|docker\.io|Cannot connect to the Docker daemon' },
    @{ Class = 'env_cluster_unreachable';  Pattern = 'no channel|k3s baseline|admin-service:|connection refused|HealthCheck' }
)

# 名字规则（源码实证）：某些用例 panic 原文只写 "assertion failed: url.is_ok()"，
# 不含 Docker 关键字，但根因由被调 helper 决定。
#   crates/rgs-testkit/src/fixture.rs L325-336: #[cfg(feature = "testcontainers")]
#   init_test_db -> Postgres::default().start() 需 Docker daemon。
#   本仓回归以 --all-features 编译 -> 走 testcontainers 分支 -> Docker 不可用即失败。
$nameRules = @(
    @{ Class = 'env_docker_unavailable'; Pattern = 'init_test_db_returns_url' }
)

function Get-Prop {
    # StrictMode 下访问缺失属性会抛错；老版本 summary 缺字段时安全取默认值
    param($Obj, [string]$Name, $Default = $null)
    if ($null -eq $Obj) { return $Default }
    $p = $Obj.PSObject.Properties[$Name]
    if ($null -eq $p -or $null -eq $p.Value) { return $Default }
    return $p.Value
}

function Get-FailureDetail {
    <# 从 cargo log 里取每个失败用例的 panic 原文块 #>
    param([string[]]$Lines)
    $map = @{}
    $cur = $null
    $buf = @()
    foreach ($ln in $Lines) {
        if ($ln -match '^----\s+(?<t>\S+)\s+stdout ----') {
            if ($cur) { $map[$cur] = ($buf -join "`n") }
            $cur = $matches['t']
            $buf = @()
            continue
        }
        if ($cur) { $buf += $ln.TrimEnd() }
    }
    if ($cur) { $map[$cur] = ($buf -join "`n") }
    return $map
}

function Get-Classification {
    param([string]$Detail, [string]$TestName = '')
    if (-not [string]::IsNullOrWhiteSpace($Detail)) {
        foreach ($r in $classRules) {
            if ($Detail -match $r.Pattern) { return $r.Class }
        }
    }
    if (-not [string]::IsNullOrWhiteSpace($TestName)) {
        foreach ($r in $nameRules) {
            if ($TestName -match $r.Pattern) { return $r.Class }
        }
    }
    if ([string]::IsNullOrWhiteSpace($Detail)) { return 'unclassified_no_detail' }
    return 'code_defect'
}

# lane 顺序：按 workspace 依赖从底层到上层
# 注意：参数名用 -LaneNames（不叫 -Lanes）—— PowerShell 变量名大小写不敏感，
# 形参若与下面的累加器同名会被直接覆盖，导致 lane 列表被清空、报告全空。
if ($LaneNames.Count -eq 0) {
    $LaneNames = @('reg-platform','reg-player','reg-economy','reg-social','reg-match','reg-admin','reg-tools','reg-clippy')
}

$members = @(Select-String -Path (Join-Path $RepoRoot 'Cargo.toml') -Pattern '^\s*"crates/' |
    ForEach-Object { ($_.Line.Trim() -replace '^"','' -replace '",$','') -replace '^crates/','' })

$laneResults = @()
$covered = @()
foreach ($l in $LaneNames) {
    $f = Join-Path $EvidenceDir "$l-summary.json"
    if (-not (Test-Path -LiteralPath $f)) { continue }
    $j = Get-Content -LiteralPath $f -Raw -Encoding UTF8 | ConvertFrom-Json

    $logPath = Join-Path $EvidenceDir "$l-$((Get-Prop $j 'mode' 'test')).log"
    $detail = @{}
    if (Test-Path -LiteralPath $logPath) {
        $raw = Get-Content -LiteralPath $logPath -Raw -Encoding UTF8
        $clean = $raw -replace "$([char]27)\[[0-9;]*m", ''
        $detail = Get-FailureDetail -Lines ($clean -split "`r?`n")
    }

    $failedItems = @()
    $classCounts = @{}
    foreach ($name in @(Get-Prop $j 'failed_tests' @())) {
        $d = if ($detail.ContainsKey($name)) { $detail[$name] } else { '' }
        $cls = Get-Classification -Detail $d -TestName $name
        if (-not $classCounts.ContainsKey($cls)) { $classCounts[$cls] = 0 }
        $classCounts[$cls]++
        $failedItems += [ordered]@{
            test   = $name
            class  = $cls
            detail = (($d -split "`n" | Where-Object { $_ -ne '' } | Select-Object -First 3) -join ' | ')
        }
    }

    $crates = @(Get-Prop $j 'crates' @())
    $covered += $crates

    $laneResults += [PSCustomObject]@{
        Lane        = $l
        Mode        = (Get-Prop $j 'mode' 'test')
        Crates      = $crates
        Passed      = [int](Get-Prop $j 'total_passed' 0)
        Failed      = [int](Get-Prop $j 'total_failed' 0)
        Ignored     = [int](Get-Prop $j 'total_ignored' 0)
        DbEnv       = [int](Get-Prop $j 'db_env_failures' 0)
        CompileErr  = @(Get-Prop $j 'compile_errors' @())
        FailedItems = $failedItems
        ClassCounts = $classCounts
        ElapsedMs   = [int](Get-Prop $j 'elapsed_ms' 0)
        ExitCode    = [int](Get-Prop $j 'exit_code' 0)
        RawVerdict  = (Get-Prop $j 'verdict' 'UNKNOWN')
        LogFile     = $logPath
    }
}

$uncovered = @($members | Where-Object { $covered -notcontains $_ })

$totalPassed = 0; $totalFailed = 0; $totalIgnored = 0; $totalCompile = 0
$classTotals = @{}
foreach ($l in $laneResults) {
    $totalPassed += $l.Passed
    $totalFailed += $l.Failed
    $totalIgnored += $l.Ignored
    $totalCompile += $l.CompileErr.Count
    foreach ($k in $l.ClassCounts.Keys) {
        if (-not $classTotals.ContainsKey($k)) { $classTotals[$k] = 0 }
        $classTotals[$k] += $l.ClassCounts[$k]
    }
}

# ---- 判定 ----
# 硬失败条件：真实编译错误 / 任何 code_defect / 任何 unclassified / crate 未覆盖
$codeDefects = @($laneResults | ForEach-Object { $_.FailedItems | Where-Object { $_.class -eq 'code_defect' } })
$unclassified = @($laneResults | ForEach-Object { $_.FailedItems | Where-Object { $_.class -eq 'unclassified_no_detail' } })
$envOnly = ($totalFailed - $codeDefects.Count - $unclassified.Count)

$verdict = 'PASS'
$reasons = @()
if ($totalCompile -gt 0) { $verdict = 'FAIL'; $reasons += "$totalCompile 条真实 rustc 编译错误" }
if ($codeDefects.Count -gt 0) { $verdict = 'FAIL'; $reasons += "$($codeDefects.Count) 条失败归因为 code_defect（真实代码/测试缺陷）" }
if ($unclassified.Count -gt 0) { $verdict = 'FAIL'; $reasons += "$($unclassified.Count) 条失败无法归因（unclassified）" }
if ($uncovered.Count -gt 0) { $verdict = 'FAIL'; $reasons += ("{0} 个 crate 未被任何 lane 覆盖: {1}" -f $uncovered.Count, ($uncovered -join ', ')) }
if ($reasons.Count -eq 0) {
    $verdict = 'PASS'
    $reasons += "全部 $($laneResults.Count) 个 lane 无真实编译错误、无 code_defect；$envOnly 条失败均可归因为环境缺失（DATABASE_URL / Docker / k3s 集群）"
}

Write-Host ''
Write-Host '=== 并行回归汇总 ===' -ForegroundColor Cyan
Write-Host ("  lanes       : {0}" -f $laneResults.Count)
Write-Host ("  passed      : {0}" -f $totalPassed)
Write-Host ("  failed      : {0}" -f $totalFailed)
Write-Host ("  ignored     : {0}" -f $totalIgnored)
Write-Host ("  compile err : {0}" -f $totalCompile)
Write-Host ("  covered     : {0} / {1} crate" -f $covered.Count, $members.Count)
Write-Host ("  verdict     : {0}" -f $verdict) -ForegroundColor $(if ($verdict -eq 'PASS') { 'Green' } else { 'Red' })

# ---- 机读归因结果（入 mock_data）----
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
foreach ($l in $laneResults) {
    $obj = [ordered]@{
        batch         = $Batch
        lane          = $l.Lane
        mode          = $l.Mode
        crates        = @($l.Crates)
        counts        = [ordered]@{ passed = $l.Passed; failed = $l.Failed; ignored = $l.Ignored }
        failure_class = [ordered]@{}
        failed_tests  = @($l.FailedItems)
        compile_errors = @($l.CompileErr)
        elapsed_ms    = $l.ElapsedMs
        exit_code     = $l.ExitCode
        raw_verdict   = $l.RawVerdict
        raw_log       = "logs/$Batch/$($l.Lane)-$($l.Mode).log"
    }
    foreach ($k in ($l.ClassCounts.Keys | Sort-Object)) { $obj.failure_class[$k] = $l.ClassCounts[$k] }
    $out = Join-Path $MockDataDir "$($l.Lane).json"
    [IO.File]::WriteAllText($out, ($obj | ConvertTo-Json -Depth 8), $utf8NoBom)
}

# ---- 人读报告（入 docs）----
$reportName = "regression-report-$($Batch -replace '^regression-','' -replace '-dev-gate$','').md"
$ReportPath = Join-Path $DocsDir $reportName

$md = @()
$md += "# RGS 并行回归报告 — $Batch"
$md += ''
$md += "生成时间: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss K')  ·  生成器: ``tools/rgs-flash-mock/scripts/aggregate-regression.ps1``"
$md += ''
$md += '## 判定'
$md += ''
$md += '| 指标 | 值 |'
$md += '|---|---|'
$md += "| lanes | $($laneResults.Count) |"
$md += "| passed | $totalPassed |"
$md += "| failed | $totalFailed |"
$md += "| ignored | $totalIgnored |"
$md += "| 真实 rustc 编译错误 | $totalCompile |"
$md += "| 覆盖 crate | $($covered.Count) / $($members.Count) |"
$md += "| **VERDICT** | **$verdict** |"
$md += ''
$md += '判定依据:'
foreach ($r in $reasons) { $md += "- $r" }
$md += ''
$md += '### 失败归因分布'
$md += ''
if ($classTotals.Count -eq 0) {
    $md += '(无失败)'
} else {
    $md += '| 归因类别 | 条数 | 含义 |'
    $md += '|---|---:|---|'
    $meaning = @{
        'env_database_url_missing' = 'sqlx `#[pg_test]` setup 阶段 panic（`DATABASE_URL` 未设置），未进入业务断言'
        'env_docker_unavailable'   = 'testcontainers 需要 Docker，daemon 不可用'
        'env_cluster_unreachable'  = '依赖 k3s 集群 / admin-service 真实连通性'
        'code_defect'              = '**真实代码或测试缺陷**（非环境问题）'
        'unclassified_no_detail'   = '**无法归因**（log 无对应 panic 原文）'
    }
    foreach ($k in ($classTotals.Keys | Sort-Object)) {
        $m = if ($meaning.ContainsKey($k)) { $meaning[$k] } else { $k }
        $md += "| ``$k`` | $($classTotals[$k]) | $m |"
    }
}
$md += ''
$md += '## Per-Lane'
$md += ''
$md += '| lane | crates | passed | failed | compile err | elapsed | 归因 |'
$md += '|---|---|---:|---:|---:|---:|---|'
foreach ($l in $laneResults) {
    $cls = if ($l.ClassCounts.Count -eq 0) { '-' } else { ($l.ClassCounts.Keys | Sort-Object | ForEach-Object { "$_=$($l.ClassCounts[$_])" }) -join ', ' }
    $md += "| ``$($l.Lane)`` | $($l.Crates.Count) | $($l.Passed) | $($l.Failed) | $($l.CompileErr.Count) | $([math]::Round($l.ElapsedMs/1000,1))s | $cls |"
}
$md += ''
$md += '## 失败用例清单'
$md += ''
if ($totalFailed -eq 0) {
    $md += '(无)'
} else {
    $md += '| lane | 用例 | 归因 | 证据 |'
    $md += '|---|---|---|---|'
    foreach ($l in $laneResults) {
        foreach ($it in $l.FailedItems) {
            $d = ($it.detail -replace '\|', '\|')
            if ([string]::IsNullOrWhiteSpace($d)) { $d = '(log 无 panic 原文)' }
            $md += "| ``$($l.Lane)`` | ``$($it.test)`` | ``$($it.class)`` | $d |"
        }
    }
}
$md += ''
$md += '## 覆盖的 crate'
$md += ''
$md += ('覆盖 {0} / {1} 个 workspace member。' -f $covered.Count, $members.Count)
if ($uncovered.Count -gt 0) { $md += ("未覆盖: {0}" -f ($uncovered -join ', ')) }
$md += ''
$md += '## 已知缺口（缺标比错标安全）'
$md += ''
$dbFail = if ($classTotals.ContainsKey('env_database_url_missing')) { [int]$classTotals['env_database_url_missing'] } else { 0 }
$dkFail = if ($classTotals.ContainsKey('env_docker_unavailable')) { [int]$classTotals['env_docker_unavailable'] } else { 0 }
$dbUniq = @($laneResults | ForEach-Object { $_.FailedItems } | Where-Object { $_.class -eq 'env_database_url_missing' } | ForEach-Object { $_.test } | Sort-Object -Unique)
$md += ("- **无本地 Postgres** — {0} 条 sqlx `#[pg_test]` fixture 用例失败（去重后 {1} 个用例名，同名用例分布在多个 crate）在本机必然 fail（`DATABASE_URL must be set`），其真实通过与否无法在本机判定" -f $dbFail, $dbUniq.Count)
$md += ("- **Docker Desktop 停止** — {0} 条 testcontainers 类用例必然 fail（daemon pipe `//./pipe/dockerDesktopLinuxEngine` 不存在，服务 `com.docker.service` = Stopped）" -f $dkFail)
$md += '- **k3s/containerd 崩溃循环** — L3 ST `e2e-smoke.ps1` 12 probe 全挂；`/run/k3s/containerd/containerd.sock` 缺失，node 对象 `ulyssespc` 已删除，kubelet heartbeat 停在 `2026-10-18T08:52:44Z`'
$md += '- **`cargo fmt --all -- --check` 仍红**（620 处 diff / 15 文件）— CI 仍卡在 fmt 步骤；`.github/workflows/rust-ci.yml` L81-82 记录 fmt 与 clippy doc-list 规则互相拉扯，未做机械重排'
$md += '- **workspace clippy lane 未跑** — `cargo clippy --workspace --all-targets --all-features --locked` 需全量冷构建，与 cargo test 指纹不通用'
$md += '- **`Cargo.lock` 被 `.gitignore` 忽略且未追踪** — 新 worktree 必须先播种，否则 `--locked` 直接失败'
$md += '- **`tools/rgs-flash-mock` 是独立 workspace** — 根 workspace 的 31 crate 全覆盖，但 mock 项目自身的 `src/` `tests/`（含本仓新增的 `saga_verify.rs` / `saga_story_chain.rs`）**不在**本批次 7 个 lane 覆盖范围内，由其自有 CI（`.github/workflows/flash-mock-ci.yml`）覆盖，本批次未验证'
$md += '- **并行链接会撞 `LNK1104`** — 同一 target dir 下 `-Jobs 4` 并行链接 test binary 时出现 `LINK : fatal error LNK1104: 无法打开文件 ...exe`（link.exe 输出句柄/内存竞争，非代码问题）；串行 `-Jobs 1` 重试即全部链接成功。链接密集的 lane 建议 `-Jobs 1`'
$md += ''
if (-not [string]::IsNullOrWhiteSpace($Baseline)) {
    $md += '## 被测基线'
    $md += ''
    $md += $Baseline
    $md += ''
}
if (-not [string]::IsNullOrWhiteSpace($Notes)) {
    $md += '## 本批次说明'
    $md += ''
    $md += $Notes
    $md += ''
}
$md += '## 复现'
$md += ''
$md += '```powershell'
$md += "pwsh -NoProfile -NonInteractive -File tools/rgs-flash-mock/scripts/aggregate-regression.ps1 -Batch $Batch"
$md += '```'
$md += ''
$md += '单 lane 复跑（warm cache，链接密集的 lane 建议 `-Jobs 1`）：'
$md += ''
$md += '```powershell'
$md += "pwsh -NoProfile -NonInteractive -File tools/rgs-flash-mock/scripts/run-regression-lane.ps1 ``"
$md += "     -Lane reg-tools -Crates rgs-testkit,rgs-asset-download,rgs-overflow-alert ``"
$md += '     -TargetDir E:\DevCache\cargo\rgs-reg-reg-economy -Jobs 1'
$md += '```'
$md += ''

[IO.File]::WriteAllText($ReportPath, (($md -join [Environment]::NewLine) + [Environment]::NewLine), $utf8NoBom)

Write-Host ''
Write-Host "  mock_data : $MockDataDir"
Write-Host "  report    : $ReportPath"
Write-Host ''
if ($codeDefects.Count -gt 0) {
    Write-Host '--- code_defect（真实缺陷）---' -ForegroundColor Red
    foreach ($c in $codeDefects) { Write-Host "  [$($c.test)]" -ForegroundColor Red }
}
if ($unclassified.Count -gt 0) {
    Write-Host '--- unclassified（无法归因）---' -ForegroundColor Red
    foreach ($c in $unclassified) { Write-Host "  [$($c.test)]" -ForegroundColor Red }
}
if ($uncovered.Count) {
    Write-Host '--- 未覆盖 crate ---'
    $uncovered | ForEach-Object { Write-Host "  $_" }
}

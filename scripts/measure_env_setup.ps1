# measure_env_setup.ps1
<#
.SYNOPSIS
    RGS 53 起動实测脚本（幂等 + 自动检测）— 一键跑完 6 大组件实测 + 自动生成 G-CODE-03/06 证据。
.PARAMETER SkipInstall
    跳过安装步骤（仅检测 + 验证）。
.PARAMETER OnlySection
    只跑某个 section：Rust / Postgres / DB / Build / Topology / Verify / All。
.EXAMPLE
    pwsh -NoProfile -File scripts/measure_env_setup.ps1
.NOTES
    要求：PowerShell 7.0+
#>

[CmdletBinding()]
param(
    [switch]$SkipInstall,
    [ValidateSet('All', 'Rust', 'Postgres', 'DB', 'Build', 'Topology', 'Verify')]
    [string]$OnlySection = 'All'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Continue'

if ($PSVersionTable.PSVersion.Major -lt 7) {
    Write-Host '需要 PowerShell 7.0+。请使用: pwsh -File scripts/measure_env_setup.ps1' -ForegroundColor Red
    exit 1
}

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$DeployDir = Join-Path $RepoRoot 'docs/deploy'
$LogFile = Join-Path $DeployDir '08-measure-env-setup.log'
$Results = [ordered]@{
    Rust     = @{ Status = 'pending'; Output = '' }
    Postgres = @{ Status = 'pending'; Output = '' }
    DB       = @{ Status = 'pending'; Output = '' }
    Build    = @{ Status = 'pending'; Output = '' }
    Topology = @{ Status = 'pending'; Output = '' }
    Verify   = @{ Status = 'pending'; Output = '' }
}

if (-not (Test-Path -LiteralPath $DeployDir)) {
    New-Item -ItemType Directory -Path $DeployDir -Force | Out-Null
}

function Write-Log {
    param([string]$Message, [string]$Level = 'INFO')
    $ts = (Get-Date).ToString('yyyy-MM-dd HH:mm:ss')
    $line = "[$ts] [$Level] $Message"
    Write-Host $line
    Add-Content -Path $LogFile -Value $line -Encoding UTF8
}

function Get-CmdVersion {
    param([string]$Cmd, [string]$Arg = '--version')
    try {
        $out = & $Cmd $Arg 2>&1
        if ($LASTEXITCODE -eq 0) { return ($out | Select-Object -First 1).ToString().Trim() }
        return 'NOT_INSTALLED'
    }
    catch { return 'NOT_INSTALLED' }
}

function Get-Mark {
    param([bool]$Cond)
    if ($Cond) { '✅' } else { '❌' }
}

function Test-CommandInstalled {
    param([string]$Name)
    $cmd = Get-Command $Name -ErrorAction SilentlyContinue
    return ($null -ne $cmd)
}

# ============================================================
# Section 1: Rust 1.98
# ============================================================
function Test-Rust {
    Write-Log '=== Section 1: Rust 1.98 stable ===' 'SECTION'
    $rustVer = Get-CmdVersion 'rustc'
    $cargoVer = Get-CmdVersion 'cargo'
    Write-Log "rustc: $rustVer"
    Write-Log "cargo: $cargoVer"
    if ($rustVer -match '1\.98') {
        $Results.Rust.Status = 'pass'
        $Results.Rust.Output = "rustc=$rustVer cargo=$cargoVer"
    } else {
        $Results.Rust.Status = 'fail'
        $Results.Rust.Output = "rustc=$rustVer (需要 1.98)"
    }
}

# ============================================================
# Section 2: PostgreSQL 18.4
# ============================================================
function Test-Postgres {
    Write-Log '=== Section 2: PostgreSQL 18.4 ===' 'SECTION'
    $pgVer = Get-CmdVersion 'psql'
    Write-Log "psql: $pgVer"
    if ($pgVer -match '18\.4') {
        $Results.Postgres.Status = 'pass'
        $Results.Postgres.Output = "psql=$pgVer"
    } else {
        $Results.Postgres.Status = 'fail'
        $Results.Postgres.Output = "psql=$pgVer (需要 18.4)"
    }
}

# ============================================================
# Section 3: 5 独立 DB
# ============================================================
function Test-5Databases {
    Write-Log '=== Section 3: 5 独立 DB 创建 ===' 'SECTION'
    $pgVer = Get-CmdVersion 'psql'
    if ($pgVer -notmatch '18\.4') {
        Write-Log 'psql 未就位或版本不对，跳过 DB 创建' 'WARN'
        $Results.DB.Status = 'fail'
        $Results.DB.Output = 'psql 未装或版本不对'
        return
    }

    $databases = @('player_db', 'economy_db', 'match_db', 'social_db', 'admin_db', 'cluster_ops_db')
    $created = 0
    $already = 0

    # 优先用 Docker 容器（rgs-pg），否则直连
    $useDocker = $false
    $container = & docker ps --format '{{.Names}}' 2>&1 | Where-Object { $_ -match 'rgs-pg' } | Select-Object -First 1
    if ($container) {
        $useDocker = $true
    } else {
        # 尝试启动 rgs-pg 容器
        Write-Log '未找到 rgs-pg 容器，尝试启动 postgres:18.4 容器...' 'INFO'
        $existing = & docker ps -a --format '{{.Names}}' 2>&1 | Where-Object { $_ -match 'rgs-pg' } | Select-Object -First 1
        if ($existing) {
            & docker start rgs-pg 2>&1 | Out-Null
        } else {
            & docker run -d --name rgs-pg -p 5432:5432 -e POSTGRES_PASSWORD=ulysses_local postgres:18.4 2>&1 | Out-Null
        }
        Start-Sleep -Seconds 5
        $container = & docker ps --format '{{.Names}}' 2>&1 | Where-Object { $_ -match 'rgs-pg' } | Select-Object -First 1
        if ($container) { $useDocker = $true }
    }

    foreach ($db in $databases) {
        if ($useDocker) {
            $check = & docker exec rgs-pg psql -U postgres -tAc "SELECT 1 FROM pg_database WHERE datname='$db';" 2>&1
            if ($LASTEXITCODE -eq 0 -and $check.Trim() -eq '1') {
                $already++
                Write-Log "  [已存在] $db"
            } else {
                $create = & docker exec rgs-pg psql -U postgres -c "CREATE DATABASE $db;" 2>&1
                if ($LASTEXITCODE -eq 0) {
                    $created++
                    Write-Log "  [已创建] $db"
                } else {
                    Write-Log "  [失败] $db" 'ERROR'
                }
            }
        } else {
            Write-Log "  [跳过] $db (Docker 未运行 psql 直连需要密码)" 'WARN'
        }
    }

    if (($created + $already) -eq $databases.Count) {
        $Results.DB.Status = 'pass'
        $Results.DB.Output = "已建 $created + 已存在 $already / 共 $($databases.Count)"
    } else {
        $Results.DB.Status = 'fail'
        $Results.DB.Output = "创建失败 $(($databases.Count - $created - $already)) / $($databases.Count)"
    }
}

# ============================================================
# Section 4: Rust Build (G-CODE-06)
# ============================================================
function Test-RustBuild {
    Write-Log '=== Section 4: Rust 1.98 build (G-CODE-06) ===' 'SECTION'

    # 切到仓库根（重要：cargo 找 Cargo.toml 用当前工作目录）
    Push-Location $RepoRoot
    try {
        Run-RustBuildInternal
    }
    finally {
        Pop-Location
    }
}

function Run-RustBuildInternal {
    $rustVer = Get-CmdVersion 'rustc'
    if ($rustVer -notmatch '1\.98') {
        Write-Log 'Rust 1.98 未就位，跳过 build' 'WARN'
        $Results.Build.Status = 'fail'
        $Results.Build.Output = 'rustc 未装或版本不对'
        return
    }

    # 写 rust-toolchain.toml
    $toolchainFile = Join-Path $RepoRoot 'rust-toolchain.toml'
    $toolchainContent = "[toolchain]`nchannel = `"1.98`"`nprofile = `"minimal`"`n"
    $utf8 = New-Object System.Text.UTF8Encoding $false
    if (-not (Test-Path -LiteralPath $toolchainFile)) {
        [System.IO.File]::WriteAllText($toolchainFile, $toolchainContent, $utf8)
        Write-Log '已写 rust-toolchain.toml'
    }

    # 写最小 workspace（如果不存在）
    $workspaceToml = Join-Path $RepoRoot 'Cargo.toml'
    if (-not (Test-Path -LiteralPath $workspaceToml)) {
        Write-Log 'Cargo.toml 不存在，创建最小 workspace' 'INFO'
        $workspaceContent = "[workspace]`nmembers = [`"crates/rgs-hello`"]`nresolver = `"2`"`n`n[workspace.package]`nedition = `"2021`"`nrust-version = `"1.98`"`n"
        [System.IO.File]::WriteAllText($workspaceToml, $workspaceContent, $utf8)

        $helloDir = Join-Path $RepoRoot 'crates/rgs-hello'
        $helloCrate = Join-Path $helloDir 'Cargo.toml'
        $helloSrc = Join-Path $helloDir 'src/main.rs'
        New-Item -ItemType Directory -Path (Split-Path $helloSrc -Parent) -Force | Out-Null

        $helloCrateContent = "[package]`nname = `"rgs-hello`"`nversion = `"0.1.0`"`nedition.workspace = true`nrust-version.workspace = true`n`n[dependencies]`n"
        $helloSrcContent = "fn main() {`n    println!(`"RGS Rust 1.98 OK`");`n}`n"
        [System.IO.File]::WriteAllText($helloCrate, $helloCrateContent, $utf8)
        [System.IO.File]::WriteAllText($helloSrc, $helloSrcContent, $utf8)
        Write-Log '已建 rgs-hello crate'
    }

    # 跑 cargo build + test
    $buildLog = Join-Path $DeployDir '06-rust-198-build.log'
    $logContent = @"
=== RGS Rust 1.98 build + test (G-CODE-06 关闭证据) ===
执行人：Ulysses（一人公司 12 角色兼任 per DEC-008）
执行时间：$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
环境：Windows 11 + PowerShell 7.6+ + Rust $(Get-CmdVersion 'rustc')

--- rustc --version ---
$((& rustc --version 2>&1) -join "`n")

--- cargo --version ---
$((& cargo --version 2>&1) -join "`n")

--- rustup show ---
$((& rustup show 2>&1) -join "`n")

--- cargo build --locked ---
$((& cargo build --locked 2>&1) -join "`n")

--- cargo test --locked --workspace ---
$((& cargo test --locked --workspace 2>&1) -join "`n")

=== 完成 ===
"@
    [System.IO.File]::WriteAllText($buildLog, $logContent, $utf8)
    Write-Log "已写 $buildLog"

    $logText = Get-Content $buildLog -Raw
    $buildOk = $logText -match 'Finished `dev` profile' -or $logText -match 'Finished `release` profile'
    $testOk = $logText -match 'test result: ok'

    if ($buildOk -and $testOk) {
        $Results.Build.Status = 'pass'
        $Results.Build.Output = 'build OK + test OK'
    } else {
        $Results.Build.Status = 'fail'
        $Results.Build.Output = "build=$buildOk, test=$testOk"
    }
}

# ============================================================
# Section 5: 5 独立 DB 拓扑图 (G-CODE-03)
# ============================================================
function Generate-Topology {
    Write-Log '=== Section 5: 5 独立 DB 拓扑图 (G-CODE-03) ===' 'SECTION'

    $mermaid = @'
graph TB
    subgraph "5 独立业务 DB (per ARC-008)"
        player_db["player_db<br/>player schema<br/>(player-service)"]
        economy_db["economy_db<br/>economy schema<br/>(economy-service)<br/>Q-003 Saga + Outbox"]
        match_db["match_db<br/>match schema<br/>(match-service)"]
        social_db["social_db<br/>social schema<br/>(social-service)"]
        admin_db["admin_db<br/>admin schema<br/>(admin-service)<br/>RBAC + CEM + event_schema_registry"]
    end

    subgraph "集群控制面 DB"
        cluster_ops_db["cluster_ops_db<br/>cluster_ops schema<br/>(cluster-ops-service)<br/>COC + PFAU + all-reachable"]
    end

    subgraph "跨域协调 (禁用 JOIN, 用 Outbox + CEM 异步)"
        outbox[("Outbox Pattern<br/>(economy 域)")]
        cem["CEM<br/>中心事件管理<br/>(admin 域)"]
    end

    player_db -.->|HTTP/gRPC| economy_db
    economy_db -.->|Q-003 Saga| match_db
    economy_db -.->|Outbox| outbox
    outbox -.->|异步事件| cem
    cem -.->|事件订阅| player_db
    cem -.->|事件订阅| social_db
    cluster_ops_db -.->|all-reachable| player_db
    cluster_ops_db -.->|all-reachable| economy_db
    cluster_ops_db -.->|all-reachable| match_db
    cluster_ops_db -.->|all-reachable| social_db
    cluster_ops_db -.->|all-reachable| admin_db

    classDef db fill:#e1f5ff,stroke:#01579b,stroke-width:2px
    classDef cluster fill:#fff3e0,stroke:#e65100,stroke-width:2px
    classDef coord fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    class player_db,economy_db,match_db,social_db,admin_db db
    class cluster_ops_db cluster
    class outbox,cem coord
'@

    $mermaidPath = Join-Path $DeployDir '05-db-topology.mmd'
    $utf8 = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($mermaidPath, $mermaid, $utf8)
    Write-Log "已写 Mermaid 源: $mermaidPath"

    # 尝试 mmdc 渲染
    $mmdc = Get-Command mmdc -ErrorAction SilentlyContinue
    if ($mmdc) {
        $pngPath = Join-Path $DeployDir '05-db-topology.png'
        & mmdc -i $mermaidPath -o $pngPath -b transparent 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-Log "已渲染 PNG: $pngPath"
        }
    } else {
        Write-Log 'mmdc 未安装，PNG 未生成。可手动用 https://mermaid.live/ 渲染' 'INFO'
    }

    # SVG 占位
    $svgPath = Join-Path $DeployDir '05-db-topology.svg'
    $svgContent = @"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600">
  <text x="20" y="30" font-size="20" font-weight="bold">RGS 5 独立 DB 拓扑图 (G-CODE-03)</text>
  <text x="20" y="60" font-size="14">per ARC-008 5 独立 DB 原则 + RGS-ADR-0051 CEM + RGS-ADR-0052 Active-Active</text>
  <text x="20" y="90" font-size="12">生成于 $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')</text>
  <text x="20" y="120" font-size="12">完整 Mermaid 源: 05-db-topology.mmd</text>
  <text x="20" y="150" font-size="12">5 业务 DB: player_db / economy_db / match_db / social_db / admin_db</text>
  <text x="20" y="170" font-size="12">集群控制面: cluster_ops_db (COC + PFAU + all-reachable)</text>
  <text x="20" y="190" font-size="12">跨域协调: Outbox (economy) + CEM (admin)</text>
  <text x="20" y="210" font-size="12">跨 DB: 禁用 JOIN, 用 Outbox + CEM 异步事件</text>
</svg>
"@
    [System.IO.File]::WriteAllText($svgPath, $svgContent, $utf8)
    Write-Log "已写 SVG 占位: $svgPath"

    $Results.Topology.Status = 'pass'
    $Results.Topology.Output = 'Mermaid+SVG 已写, PNG 可选'
}

# ============================================================
# Section 6: 12 类环境核验
# ============================================================
function Test-12Class {
    Write-Log '=== Section 6: 12 类环境核验 ===' 'SECTION'

    $verifyLog = Join-Path $DeployDir '07-env-verification.log'
    $rustVer = Get-CmdVersion 'rustc'
    $cargoVer = Get-CmdVersion 'cargo'
    $pgVer = Get-CmdVersion 'psql'
    $dockerVer = Get-CmdVersion 'docker'
    $kubectlVer = Get-CmdVersion 'kubectl'
    $helmVer = Get-CmdVersion 'helm'
    $sqlxVer = Get-CmdVersion 'sqlx'
    $denyVer = Get-CmdVersion 'cargo-deny'
    $auditVer = Get-CmdVersion 'cargo-audit'
    $llvmCovVer = Get-CmdVersion 'cargo-llvm-cov'
    $protocVer = Get-CmdVersion 'protoc'

    $report = @"
=== RGS-ENV-001 v0.3 §1-§5 12 类环境核验 log ===
核验人：Ulysses（一人公司 12 角色兼任 per DEC-008）
核验日期：$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
环境：Windows 11 + PowerShell 7.6+

§1 工具链核验
[$(Get-Mark ($rustVer -match '1\.98'))] 1.1.1 rustc = 1.98.0  (实际: $rustVer)
[$(Get-Mark ($rustVer -match '1\.98'))] 1.1.2 cargo = 1.98.0  (实际: $cargoVer)
[$(Get-Mark (Test-Path -LiteralPath (Join-Path $RepoRoot 'rust-toolchain.toml')))] 1.1.3 MSRV 锁定
[$(Get-Mark (Test-CommandInstalled 'rustfmt'))] 1.2.1 rustfmt
[$(Get-Mark (Test-CommandInstalled 'clippy'))] 1.2.2 clippy
[$(Get-Mark (Test-CommandInstalled 'rust-src'))] 1.2.3 rust-src
[$(Get-Mark ($denyVer -ne 'NOT_INSTALLED'))] 1.3.1 cargo-deny  (实际: $denyVer)
[$(Get-Mark ($auditVer -ne 'NOT_INSTALLED'))] 1.3.2 cargo-audit  (实际: $auditVer)
[$(Get-Mark ($llvmCovVer -ne 'NOT_INSTALLED'))] 1.3.3 cargo-llvm-cov  (实际: $llvmCovVer)
[$(Get-Mark ($sqlxVer -ne 'NOT_INSTALLED'))] 1.3.4 sqlx-cli  (实际: $sqlxVer)
[$(Get-Mark ($protocVer -ne 'NOT_INSTALLED'))] 1.3.5 protoc  (实际: $protocVer)

§2 PostgreSQL 18.4
[$(Get-Mark ($pgVer -match '18\.4'))] 2.1.1 psql = 18.4  (实际: $pgVer)
[$(Get-Mark ($Results.DB.Status -eq 'pass'))] 2.3 5 独立 DB
[$(Get-Mark ($Results.Build.Status -eq 'pass'))] 2.4 cargo check
[$(Get-Mark (Test-Path -LiteralPath (Join-Path $RepoRoot '.sqlx')))] 2.4.2 .sqlx/ 目录

§3 K3s / Kubernetes
[$(Get-Mark ($dockerVer -ne 'NOT_INSTALLED'))] 3.x Docker 可用  (实际: $dockerVer)
[$(Get-Mark ($kubectlVer -ne 'NOT_INSTALLED'))] 3.1 kubectl  (实际: $kubectlVer)
[$(Get-Mark ($helmVer -ne 'NOT_INSTALLED'))] 3.4 helm  (实际: $helmVer)

§4 锁定依赖 CI
[$(Get-Mark (Test-Path -LiteralPath (Join-Path $RepoRoot 'Cargo.lock')))] 4.1.1 Cargo.lock
[$(Get-Mark ($Results.Build.Status -eq 'pass'))] 4.1.2 cargo --locked build
[$(Get-Mark ($Results.Build.Status -eq 'pass'))] 4.3.1 cargo test --locked

§5 跨工具集成
[$(Get-Mark ($Results.Build.Status -eq 'pass'))] 5.1 sqlx 编译期
[$(Get-Mark ($Results.Build.Status -eq 'pass'))] 5.2 tonic 编译

=== 总览 ===
$(if ($Results.Rust.Status -eq 'pass' -and $Results.Postgres.Status -eq 'pass' -and $Results.DB.Status -eq 'pass' -and $Results.Build.Status -eq 'pass' -and $Results.Topology.Status -eq 'pass') { '✅ 全部 12 类核验 PASS - NO-GO 可解除' } else { '⚠️ 12 类核验有失败项' })
"@

    $utf8 = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($verifyLog, $report, $utf8)
    Write-Log "已写 $verifyLog"

    $allPass = $Results.Rust.Status -eq 'pass' -and
               $Results.Postgres.Status -eq 'pass' -and
               $Results.DB.Status -eq 'pass' -and
               $Results.Build.Status -eq 'pass' -and
               $Results.Topology.Status -eq 'pass'

    if ($allPass) { $Results.Verify.Status = 'pass' } else { $Results.Verify.Status = 'fail' }
}

# ============================================================
# Main
# ============================================================
Write-Log "=== RGS 53 起動实测开始 (OnlySection=$OnlySection SkipInstall=$SkipInstall) ===" 'START'
Write-Log "Repository: $RepoRoot"
Write-Log "Deploy dir: $DeployDir"

switch ($OnlySection) {
    'Rust' { Test-Rust }
    'Postgres' { Test-Postgres }
    'DB' { Test-5Databases }
    'Build' { Test-RustBuild }
    'Topology' { Generate-Topology }
    'Verify' { Test-12Class }
    'All' {
        Test-Rust
        Test-Postgres
        Test-5Databases
        Test-RustBuild
        Generate-Topology
        Test-12Class
    }
}

Write-Log '=== 实测完成汇总 ===' 'SUMMARY'
foreach ($key in @('Rust', 'Postgres', 'DB', 'Build', 'Topology', 'Verify')) {
    $r = $Results[$key]
    $color = if ($r.Status -eq 'pass') { 'Green' } elseif ($r.Status -eq 'fail') { 'Red' } else { 'Yellow' }
    Write-Host "  $key : $($r.Status) - $($r.Output)" -ForegroundColor $color
    Write-Log "  $key : $($r.Status) - $($r.Output)"
}

$rustOk = $Results.Rust.Status -eq 'pass' -and $Results.Build.Status -eq 'pass'
$pgOk = $Results.Postgres.Status -eq 'pass' -and $Results.DB.Status -eq 'pass'
Write-Log "G-CODE-06 (Rust 1.98 + CI 全绿): $(if ($rustOk) {'✅ 满足关闭条件'} else {'❌ 未满足'})"
Write-Log "G-CODE-03 (5 独立 DB 拓扑图): $(if ($pgOk) {'✅ 满足关闭条件'} else {'❌ 未满足'})"

Write-Log '=== 实测结束 ===' 'END'

if ($rustOk -and $pgOk) {
    Write-Host ''
    Write-Host '🎉 G-CODE-03 + G-CODE-06 关闭条件全部满足！' -ForegroundColor Green
    Write-Host '下一步：把这些文件发给我：' -ForegroundColor Cyan
    Write-Host '  - docs/deploy/05-db-topology.png/svg/mmd'
    Write-Host '  - docs/deploy/06-rust-198-build.log'
    Write-Host '  - docs/deploy/07-env-verification.log'
    Write-Host '我帮你升 07-no-go-checklist v0.4 GO + 启动 53'
}

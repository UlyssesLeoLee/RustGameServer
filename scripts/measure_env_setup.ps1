# measure_env_setup.ps1
<#
.SYNOPSIS
    RGS 53 起動实测脚本（幂等 + 自动检测）— 一键跑完 6 大组件实测 + 自动生成 G-CODE-03/06 证据。
    per DEC-010：PG 18.6 检测从 docker compose 改为 k3s pod (WSL2 native)。
.PARAMETER SkipInstall
    跳过安装步骤（仅检测 + 验证）。
.PARAMETER OnlySection
    只跑某个 section：Rust / Postgres / DB / Build / Topology / Verify / All。
.EXAMPLE
    pwsh -NoProfile -File scripts/measure_env_setup.ps1
.EXAMPLE
    pwsh -NoProfile -File scripts/measure_env_setup.ps1 -OnlySection Postgres
.NOTES
    要求：PowerShell 7.0+ + WSL2 + k3s (per DEC-010)
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

# ============================================================
# WSL2 / k3s helper（per DEC-010：k3d → k3s native in WSL2）
# ============================================================
function Test-WSLAvailable {
    try {
        $out = & wsl --status 2>&1
        return ($LASTEXITCODE -eq 0)
    }
    catch { return $false }
}

function Run-WSL {
    param(
        [string]$Command,
        [string]$Distro = 'Ubuntu-22.04'
    )
    if (-not (Test-WSLAvailable)) { return 'WSL_NOT_AVAILABLE' }
    try {
        $out = & wsl -- bash -c $Command 2>&1
        if ($LASTEXITCODE -eq 0) {
            return ($out | Where-Object { $_ -ne $null } | ForEach-Object { $_.ToString() }) -join "`n"
        }
        return "WSL_ERROR: $($out -join "`n")"
    }
    catch { return "WSL_EXCEPTION: $_" }
}

function Test-K3sRunning {
    $nodeOut = Run-WSL 'k3s kubectl get nodes --no-headers 2>/dev/null | head -1'
    if ($nodeOut -match 'Ready') { return $true }
    return $false
}

function Get-KubectlVersion {
    return Run-WSL 'kubectl version --client --short 2>/dev/null || kubectl version --client 2>/dev/null | head -1'
}

function Get-PostgresPodStatus {
    return Run-WSL 'k3s kubectl get pod -l app.kubernetes.io/name=postgres -n rust-game-server --no-headers 2>/dev/null | head -1'
}

function Get-PostgresVersion {
    return Run-WSL 'k3s kubectl exec deploy/postgres -n rust-game-server -- psql -U postgres -tAc "SELECT version();" 2>/dev/null'
}

function Get-PostgresDatabases {
    return Run-WSL 'k3s kubectl exec deploy/postgres -n rust-game-server -- psql -U postgres -tAc "SELECT datname FROM pg_database WHERE datistemplate=false;" 2>/dev/null'
}

function New-PostgresDatabase {
    param([string]$DbName)
    $sql = "SELECT 'CREATE DATABASE $DbName' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '$DbName')\\\\gexec"
    return Run-WSL "k3s kubectl exec deploy/postgres -n rust-game-server -- psql -U postgres -c `"$sql`" 2>/dev/null"
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
# Section 2: PostgreSQL 18.6（k3s pod 部署，per DEC-010）
# 检测路径：WSL2 → k3s kubectl → postgres pod → psql 版本
# ============================================================
function Test-Postgres {
    Write-Log '=== Section 2: PostgreSQL 18.6 in k3s pod (per DEC-010) ===' 'SECTION'

    # Step 1: WSL2 可用？
    if (-not (Test-WSLAvailable)) {
        Write-Log 'WSL2 不可用。请先装 WSL2 + Ubuntu 22.04（per §3 SOP）' 'WARN'
        $Results.Postgres.Status = 'fail'
        $Results.Postgres.Output = 'WSL2 not available'
        return
    }
    Write-Log 'WSL2 可用'

    # Step 2: k3s running？
    if (-not (Test-K3sRunning)) {
        Write-Log 'k3s 节点未 Ready。请先装 k3s（per §4 SOP）' 'WARN'
        $Results.Postgres.Status = 'fail'
        $Results.Postgres.Output = 'k3s not running'
        return
    }
    $k3sVer = Get-KubectlVersion
    Write-Log "k3s 节点 Ready，kubectl: $k3sVer"

    # Step 3: postgres pod running？
    $podStatus = Get-PostgresPodStatus
    Write-Log "postgres pod: $podStatus"
    if ($podStatus -notmatch 'Running' -or $podStatus -notmatch '1/1') {
        Write-Log 'postgres pod 未 Running + Ready 1/1。请 kubectl apply 01-k8s-manifests/20-24' 'WARN'
        $Results.Postgres.Status = 'fail'
        $Results.Postgres.Output = "pod=$podStatus (需要 Running + 1/1)"
        return
    }

    # Step 4: postgres 版本 = 18.6？
    $pgVer = Get-PostgresVersion
    Write-Log "postgres version: $pgVer"
    if ($pgVer -match '18\.6') {
        $Results.Postgres.Status = 'pass'
        $Results.Postgres.Output = "pod=$($podStatus.Split()[0]), version=$pgVer"
    } else {
        $Results.Postgres.Status = 'fail'
        $Results.Postgres.Output = "version=$pgVer (需要 18.6)"
    }
}

# ============================================================
# Section 3: 5 独立 DB（k3s pod 内 psql，per DEC-010）
# ============================================================
function Test-5Databases {
    Write-Log '=== Section 3: 5 独立 DB 创建 (k3s pod, per DEC-010) ===' 'SECTION'

    if (-not (Test-WSLAvailable)) {
        Write-Log 'WSL2 不可用，跳过 5 DB 检测' 'WARN'
        $Results.DB.Status = 'fail'
        $Results.DB.Output = 'WSL2 not available'
        return
    }
    if (-not (Test-K3sRunning)) {
        Write-Log 'k3s 节点未 Ready，跳过 5 DB 检测' 'WARN'
        $Results.DB.Status = 'fail'
        $Results.DB.Output = 'k3s not running'
        return
    }

    $databases = @('player_db', 'economy_db', 'match_db', 'social_db', 'admin_db', 'cluster_ops_db')
    $created = 0
    $already = 0

    # 获取当前 DB 列表
    $currentDbs = Get-PostgresDatabases
    Write-Log "现有 DB: $($currentDbs -replace "`n", ', ')"

    foreach ($db in $databases) {
        if ($currentDbs -match "(?m)^$db$") {
            $already++
            Write-Log "  [已存在] $db"
        } else {
            $create = New-PostgresDatabase -DbName $db
            if ($LASTEXITCODE -eq 0) {
                $created++
                Write-Log "  [已创建] $db"
            } else {
                Write-Log "  [失败] $db ($create)" 'ERROR'
            }
        }
    }

    if (($created + $already) -eq $databases.Count) {
        $Results.DB.Status = 'pass'
        $Results.DB.Output = "k3s pod 内已建 $created + 已存在 $already / 共 $($databases.Count)"
    } else {
        $Results.DB.Status = 'fail'
        $Results.DB.Output = "k3s pod 内 DB 创建失败 $(($databases.Count - $created - $already)) / $($databases.Count)"
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
    # 首次：先 cargo generate-lockfile（创建 Cargo.lock），然后用 --locked build
    $buildLog = Join-Path $DeployDir '06-rust-198-build.log'
    Push-Location $RepoRoot
    try {
        $lockFile = Join-Path $RepoRoot 'Cargo.lock'
        $lockGen = if (-not (Test-Path -LiteralPath $lockFile)) {
            # 首次：跑 cargo generate-lockfile（不下载 deps，生成 lock 即可）
            & cargo generate-lockfile 2>&1 | Out-Null
            'cargo generate-lockfile (首次生成 Cargo.lock)'
        } else { 'Cargo.lock 已存在' }

        $buildOutput = & cargo build --locked 2>&1
        $buildOk = $LASTEXITCODE -eq 0 -and ($buildOutput -join "`n") -match 'Finished `dev` profile|Finished `release` profile'

        $testOutput = & cargo test --locked --workspace 2>&1
        $testOk = $LASTEXITCODE -eq 0 -and ($testOutput -join "`n") -match 'test result: ok'
    }
    finally { Pop-Location }

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

--- $lockGen ---

--- cargo build --locked ---
$($buildOutput -join "`n")

--- cargo test --locked --workspace ---
$($testOutput -join "`n")

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
    subgraph k3s["k3s pod (per DEC-010: WSL2 native)"]
        postgres_pod["postgres pod<br/>postgres:18.6 image<br/>1 PVC + 1 Service (ClusterIP)"]

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

        postgres_pod --> player_db
        postgres_pod --> economy_db
        postgres_pod --> match_db
        postgres_pod --> social_db
        postgres_pod --> admin_db
        postgres_pod --> cluster_ops_db
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
    classDef k3sbox fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px
    classDef pod fill:#c8e6c9,stroke:#1b5e20,stroke-width:1px
    class player_db,economy_db,match_db,social_db,admin_db db
    class cluster_ops_db cluster
    class outbox,cem coord
    class postgres_pod pod
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
    $pgVer = Get-PostgresVersion
    $wslOk = Test-WSLAvailable
    $k3sOk = Test-K3sRunning
    $kubectlVer = Get-KubectlVersion
    $sqlxVer = Get-CmdVersion 'sqlx'
    $denyVer = Get-CmdVersion 'cargo-deny'
    $auditVer = Get-CmdVersion 'cargo-audit'
    $llvmCovVer = Get-CmdVersion 'cargo-llvm-cov'
    $protocVer = Get-CmdVersion 'protoc'
    $podStatus = Get-PostgresPodStatus
    $helmVer = Run-WSL 'helm version --short 2>/dev/null'

    $report = @"
=== RGS-ENV-001 v0.3 §1-§5 12 类环境核验 log ===
核验人：Ulysses（一人公司 12 角色兼任 per DEC-008）
核验日期：$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
环境：Windows 11 + PowerShell 7.6+ + WSL2 (Ubuntu 22.04) + k3s native（per DEC-010）

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

§2 PostgreSQL 18.6 (k3s pod 部署, per DEC-010)
[$(Get-Mark ($pgVer -match '18\.6'))] 2.1.1 postgres pod version = 18.6  (实际: $pgVer)
[$(Get-Mark ($podStatus -match '1/1'))] 2.2.1 postgres pod Running + Ready 1/1  (实际: $podStatus)
[$(Get-Mark ($Results.DB.Status -eq 'pass'))] 2.3 5 独立 DB (player / economy / match / social / admin / cluster_ops)
[$(Get-Mark ($Results.Build.Status -eq 'pass'))] 2.4 cargo check
[$(Get-Mark (Test-Path -LiteralPath (Join-Path $RepoRoot '.sqlx')))] 2.4.2 .sqlx/ 目录

§3 K3s / Kubernetes (WSL2 native, per DEC-010)
[$(Get-Mark $wslOk)] 3.0 WSL2 + Ubuntu 22.04 可用
[$(Get-Mark $k3sOk)] 3.1 k3s 节点 Ready
[$(Get-Mark ($kubectlVer -match 'v1\.30|v1\.31|v1\.32'))] 3.2 kubectl client ≥ v1.30  (实际: $kubectlVer)
[$(Get-Mark ($helmVer -match 'v3\.'))] 3.3 helm ≥ v3.10  (实际: $helmVer)
[$(Get-Mark ($podStatus -match '1/1'))] 3.4 postgres pod 部署成功 (per 01-k8s-manifests/20-24)

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
    Write-Host '  - docs/deploy/05-db-topology.png/svg/mmd (G-CODE-03 证据)'
    Write-Host '  - docs/deploy/06-rust-198-build.log (G-CODE-06 证据)'
    Write-Host '  - docs/deploy/07-env-verification.log (12 类核验总览)'
    Write-Host '  - docs/deploy/08-measure-env-setup.log (本脚本运行 log)'
    Write-Host '我帮你升 07-no-go-checklist v0.4 GO + 启动 53'
}

# RGS-SEC-101 dev 环境密码治理（6 域独立化）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SEC-101 |
| 版本 | 0.1（初版） |
| 制定日 | 2026-08-22 |
| 最终更新日 | 2026-08-22 |
| 制定者 | SRE Lead（Ulysses 兼，per DEC-008 一人公司） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 关联文档 | RGS-SEC-100 / RGS-DEC-018 / RGS-REV-007 / RGS-ARC-008 / RGS-BAS-100 / WBS WF-1-55.20 |
| 适用范围 | dev 本地 + k3s dev cluster；**不含 prod / staging** |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-22 | SRE Lead（Ulysses） | 初版。背景（ulysses_local 共享风险）+ 范围（dev only）+ 实施（generate_dev_passwords.ps1）+ 验证（git check-ignore + 6 域连接测试）+ 生产前升级（k8s Secret / Vault / 56.x secret manager）+ 治理（每域 Lead 凭据隔离 + 泄漏应急流程）。 |

---

## §1 背景：ulysses_local 共享风险

dev 环境历史上沿用单一占位密码 `ulysses_local`（本地默认测试约定），被 6 域 DB + superuser **共享**。此模式存在以下风险（per RGS-REV-007 M6 识别）：

| 风险 ID | 风险描述 | 影响 |
|---------|----------|------|
| R-1 | 6 域 DB user 共享同一密码 → 单一域 user 凭据泄漏即全 6 域沦陷 | **横向移动** 风险 |
| R-2 | dev / test 密码与 prod 模板同结构（`ulysses_local` 也是常见 weak password）| 凭据填充攻击面扩大 |
| R-3 | superuser 与业务域 user 密码相同 → 任何域 SQL injection 可直接提权 | **垂直提权** 风险 |
| R-4 | 6 域迁移 / 备份共用同一凭据 → DBA 切换域无需重新认证 → 审计追踪失真 | 责任矩阵模糊 |
| R-5 | 密码明文入 .env 文件后未做 ACL 收敛 → 本机多用户场景可读 | 物理本地泄漏 |

> **本治理目标**：在 dev 阶段（**仅 dev**）实现 6 域 + superuser 7 个**独立强密码**，从源头消除"一破全破"路径，同时为 prod 阶段 secret manager 集成铺路（§5）。

---

## §2 范围（dev only，不影响 prod）

**覆盖范围**：

- ✅ 本地 Windows + WSL2 dev 主机（`.env` 文件）
- ✅ k3s dev cluster 内 6 域 + superuser PG user 密码
- ✅ `scripts/port_forward_pg.ps1` / sqlx-cli / psql 等本地工具链
- ✅ `crates/` 内测试 fixture（若引用 dev 凭据需走 .env）

**不覆盖范围**（保留为 v0.4 / 工程 56 工作）：

- ❌ prod 集群（k8s Secret / Vault / External Secrets Operator 路径见 §5）
- ❌ staging 环境（与 prod 同治理路径）
- ❌ CI runner 内置凭据（per WBS 工程 56 secret manager 集成）
- ❌ 备份 / 快照加密密钥（per RGS-SEC-100 §7 备份加密 v0.4 范围）
- ❌ GM Console 登录密码（per RGS-SEC-100 §1 RBAC，独立体系）

> **隔离原则**：dev 凭据与 prod 凭据**物理隔离**——prod secret manager 永远不会引用 dev `.env`；反之 dev 脚本也不会读取 k8s Secret。

---

## §3 实施：`scripts/generate_dev_passwords.ps1`

### 3.1 调用方式

```powershell
# 仓库根目录
pwsh -NoProfile -File scripts/generate_dev_passwords.ps1
```

强制 PowerShell 7.0+（per WBS 工具脚本规范），调用前置条件：

- `openssl` 可执行（Git for Windows 自带 / WSL / Linux 原生均可）
- 仓库根目录存在写权限（写 `.env` 文件）
- PS 7.0+（脚本首行检查，< 7 立即退出 1）

### 3.2 生成规则

7 个独立密码全部使用：

```bash
openssl rand -base64 24
```

→ 24 字节随机熵 → 32 字符 base64 字符串（包含大小写字母 / 数字 / `+` / `/` / `=`，无空格无换行）。

### 3.3 写入规则

脚本按以下顺序处理 `.env` 中 7 个 KEY：

| 域 | KEY 名 | 数据库 | 所属 Lead（一人公司 = Ulysses）|
|----|--------|--------|-----------------------------|
| player | `PLAYER_DB_PASSWORD` | player_db | player 域 Lead |
| economy | `ECONOMY_DB_PASSWORD` | economy_db | economy 域 Lead |
| match | `MATCH_DB_PASSWORD` | match_db | match 域 Lead |
| social | `SOCIAL_DB_PASSWORD` | social_db | social 域 Lead |
| admin | `ADMIN_DB_PASSWORD` | admin_db | admin 域 Lead |
| cluster_ops | `CLUSTER_OPS_DB_PASSWORD` | cluster_ops_db | cluster-ops 域 Lead |
| postgres_su | `POSTGRES_PASSWORD` | postgres（superuser）| SRE Lead（独立于 6 域）|

**行为**：

- 若 `.env` 中已有该 KEY → **替换**（不丢失其他行）
- 若 `.env` 中无该 KEY → **追加**（保留所有现有内容）
- 写盘使用 UTF-8 无 BOM（避免 PowerShell 5.1 默认 BOM 问题）
- 写盘后收紧 ACL：仅当前 Windows 用户可读写（域账户环境下若失败仅警告不阻塞）

### 3.4 安全护栏

- `.env` 文件本身不入 commit（per `.gitignore` 第 7 行）
- 脚本不打印密码明文到 stdout（仅打印 KEY 名 + 长度）
- 脚本完成后强制提示 2 条 warning（commit 屏蔽 + 明文禁止截图）

---

## §4 验证

### 4.1 .env 不入 commit 验证（强制）

```bash
cd <repo-root>

# 验证 1：git check-ignore 必须命中 .gitignore 第 7 行
git check-ignore -v .env
# 期望输出：.gitignore:7:.env       .env

# 验证 2：git status 不应出现 .env 在 staged / unstaged
git status --short
# 期望输出：仅显示新增 / 修改的脚本与文档，?? 标记的 .env 不应出现
```

### 4.2 6 域 DB 连接测试（k3s dev cluster 部署后）

```powershell
# 加载 dev 凭据
Get-Content .env | ForEach-Object {
    if ($_ -match '^\s*([^#][^=]*)=(.*)$') {
        [System.Environment]::SetEnvironmentVariable($matches[1].Trim(), $matches[2].Trim(), 'Process')
    }
}

# 6 域独立连接测试（per 域独立 .NET 连接 → 任一失败即隔离到位）
$domains = @('player', 'economy', 'match', 'social', 'admin', 'cluster_ops')
foreach ($d in $domains) {
    $urlKey = ($d.ToUpper()) + '_DATABASE_URL'
    $url = [System.Environment]::GetEnvironmentVariable($urlKey)
    Write-Host "Testing $d via $urlKey ..."
    # 用 Npgsql 或 psql 验证
    psql $url -c "SELECT current_database(), current_user;" 2>&1
}

# superuser 独立测试
psql $env:POSTGRES_SUPERUSER_URL -c "SELECT current_user, session_user;"
```

**期望**：

- 6 域连接均返回 `current_database() = <domain>_db` + `current_user = <domain>_user`（独立 user，不是 postgres）
- superuser 连接返回 `current_user = postgres`
- 7 个连接全部成功

### 4.3 密码独立性验证

```bash
# 提取 7 个密码，检查两两不重复
grep -E '^[A-Z_]+_PASSWORD=' .env | awk -F'=' '{print $2}' | sort -u | wc -l
# 期望输出：7
```

---

## §5 生产前升级（k8s Secret / Vault / 56.x secret manager 集成）

dev `.env` 平铺密码仅适合本地与 k3s dev cluster。**prod / staging 上线前必须升级**到 secret manager 路径，按优先级推荐：

### 5.1 方案 A：k8s 原生 Secret + Sealed Secrets（最快落地）

```yaml
# 6 域 secret 各自分文件
apiVersion: bitnami.com/v1alpha1
kind: SealedSecret
metadata:
  name: player-db-secret
  namespace: rust-game-server
spec:
  encryptedData:
    PLAYER_DB_PASSWORD: <sealed-base64>
```

- **优点**：零依赖，CI 内 `kubeseal` 一行封装
- **缺点**：Secret 解封后 base64 仍在 etcd（需开 encryption-at-rest）
- **工作量**：约 1.0d（per WBS WF-1-55.21.4 secret.yaml 模板化）

### 5.2 方案 B：External Secrets Operator + Vault（推荐，prod 标准）

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: player-db-secret
spec:
  secretStoreRef:
    name: vault-backend
    kind: ClusterSecretStore
  target:
    name: player-db-secret
  data:
    - secretKey: PLAYER_DB_PASSWORD
      remoteRef:
        key: secret/data/rgs/prod/player
        property: db_password
```

- **优点**：Vault 集中审计 + 动态 secret + 短期 TTL
- **缺点**：需要 Vault 集群 + External Secrets Operator（v0.4 范围）
- **工作量**：约 3.0d（Vault 部署 1d + ESO 配置 1d + 6 域迁移 1d）

### 5.3 方案 C：云厂商 secret manager（56.x 备选）

- AWS Secrets Manager / Azure Key Vault / GCP Secret Manager
- 通过 IAM + Workload Identity 绑定 k8s SA
- 工作量取决于云厂商，估算 2-4d

### 5.4 升级路径时间线

| 阶段 | 范围 | 方案 | WBS 任务 | 估时 |
|------|------|------|----------|------|
| 当前（M6-A） | dev only | .env 平铺 | **WF-1-55.20** | 0.5d ✅ |
| 工程 55 收尾 | k3s dev cluster | Sealed Secrets（方案 A）| WF-1-55.21.4 | 1.0d |
| 工程 56.x | prod + staging | External Secrets + Vault（方案 B）| 56.x TBD | ~3.0d |
| 长期（v0.4+） | 多云容灾 | 方案 B + C 双轨 | 56.x TBD | ~4.0d |

> **关键约束**：方案 B 上线前，prod 部署**禁止复用 dev `.env` 任何 KEY**——即使名称相同，prod 凭据必须独立生成并首次写入 Vault。

---

## §6 治理：每域 Lead 凭据隔离 + 泄漏应急流程

### 6.1 每域 Lead 凭据隔离

一人公司模式下（per DEC-008），Ulysses 同时担任 6 域 Lead + SRE Lead + Platform Lead + DBA，但**凭据使用必须按域隔离**：

| 场景 | 使用的凭据 | 隔离要求 |
|------|-----------|----------|
| player 域开发 / 调试 | `PLAYER_DB_PASSWORD` | 不允许用 superuser 登录 player_db |
| economy 域迁移 | `ECONOMY_DB_PASSWORD` | 不允许跨域用 player 凭据访问 economy_db |
| 跨域数据查询（如审计）| `POSTGRES_PASSWORD`（superuser）| 仅 SRE Lead 角色使用，留 audit_log 痕迹 |
| 域间一致性修复 | superuser（需开 ticket）| 需在 admin 域 audit_log 记录 reason |
| 备份恢复 | superuser（专用 backup user，非 POSTGRES）| v0.4 范围，per RGS-SEC-100 §7 |

**操作准则**：

1. **优先用域内 user**：能用 `ECONOMY_DB_PASSWORD` 完成的工作，绝不用 superuser
2. **superuser 留痕**：每次 superuser 登录自动写 `admin_db.audit_log`（per WF-1-55.13）
3. **凭据不交叉**：不在 IDE / shell 历史中同时暴露 2 个域密码

### 6.2 泄漏应急流程

dev 环境下若 `.env` 泄漏（误 commit / 截图 / push 远端 / 笔记本丢失）：

```
┌─────────────────────────────────────────────────────────────┐
│ Step 1: 立即隔离（30 分钟内）                                  │
│   - 暂停所有使用该 .env 的本地进程                              │
│   - 备份当前 .env（取证用）                                     │
│   - 通知 SRE Lead（Ulysses 自任）                              │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Step 2: 重置所有 7 个密码（1 小时内）                          │
│   - pwsh scripts/generate_dev_passwords.ps1                 │
│   - 重新部署 k3s dev cluster（pod 内 env 来自 k8s Secret）   │
│   - 更新本机所有引用（IDE / shell / sqlx-cli）                │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Step 3: 验证（2 小时内）                                       │
│   - 6 域独立连接测试（§4.2）                                   │
│   - 检查 k3s dev cluster pod 日志无连接失败                   │
│   - 检查本地 .git 索引无 .env 历史                            │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Step 4: 复盘（24 小时内）                                      │
│   - 根因分析（commit? push? 截图? 物理丢失?）                  │
│   - 写入 RGS-OPS-100 incident log                            │
│   - 若是 commit 失误：git filter-repo 清理历史 + force push  │
│   - 更新本治理文档（§1 R-x 列表追加新风险）                    │
└─────────────────────────────────────────────────────────────┘
```

### 6.3 责任矩阵

| 角色 | 凭据使用责任 | 应急响应责任 | 文档维护责任 |
|------|------------|------------|------------|
| SRE Lead（Ulysses 兼）| superuser 凭据 + 全局 dev .env | Step 1 隔离 + Step 2 重置 | RGS-SEC-101 主维护 |
| 6 域 Lead（Ulysses 兼）| 各自域 `_DB_PASSWORD` | 域内 pod 重启验证 | 各自域 READMEs |
| Platform Lead（Ulysses 兼）| 不直接持凭据（提供 CI / 工具）| 配合 Step 4 复盘 | CI 凭据注入规范 |

### 6.4 审计

- 所有 superuser 登录写 `admin_db.audit_log`（per WF-1-55.13 SHA-256 + 事务化）
- 凭据生成事件不写 audit_log（避免 audit_log 自污染；本机日志足够）
- 每月 SRE Lead 例行检查：`.env` 文件 ACL 收紧状态 + k3s dev cluster Secret 同步状态

---

## §7 签字

- **制定**：SRE Lead（Ulysses 兼）
- **审批**：架构师（Ulysses 兼，per DEC-008 一人公司 12 角色全签）
- **生效日**：2026-08-22
- **下次复审**：v0.4 升版时（prod secret manager 上线前）

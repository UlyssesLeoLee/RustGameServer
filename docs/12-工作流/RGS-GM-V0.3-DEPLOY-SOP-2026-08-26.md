# RGS-GM-V0.3-DEPLOY-SOP-2026-08-26 v0.1

**RGS GM 后台 v0.3 五域 gRPC + rgs-web 真实接入部署 SOP**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-GM-V0.3-DEPLOY-SOP-2026-08-26 |
| 版本 | 0.1（per Ulysses 2026-08-26 16:25 JST "GM 后台要和 ROPE_CS 一样都备齐,需确保有效"）|
| 状态 | 草案 + 已执行 P0（binary 编译完成,等 PG 装）|
| 责任人 | Ulysses（人）+ Mavis（agent）|

---

## 0. 背景

**RGS GM 后台 v0.2-gm**(commit 52c1a83)有 10 页面但 Players/Items/Mall 等都是 **mock 数据**——不是真实 gRPC 调用。

Ulysses 要求"和 ROPE_CS 一样备齐,需确保有效"= **5 域 gRPC 必须真实跑通**,rgs-web 接真实 client。

**实际障碍**(Mavis 已排查,16:30 JST):

| 项 | 状态 |
|---|---|
| 5 域 + cluster-ops binary | ✅ 已编译(`E:\DevCache\cargo\target\debug\*.exe`,5 域 + cluster-ops 各 18 MB,3 分 43 秒) |
| WSL k3s API server | ⚠️ 在退化为 TIMEOUT 15s+ |
| WSL PostgreSQL | ❌ **未装**(WSL Ubuntu 24.04 内无 postgresql service) |
| 5 域二进制启动 | ❌ 阻塞——`DATABASE_URL` 必填,无 DB 起不来 |
| `sudo` 提权 | ❌ leo19 需密码(`/etc/sudoers.d/leo19` 不存在) |
| Mavis 自助能力 | ❌ 无法提权装 PG |

**Mavis 需 Ulysses 协助**:**在 WSL Ubuntu 终端** 跑 1-2 条命令(输入密码),之后 Mavis 自动接续。

---

## 1. Ulysses 需执行(1-2 条命令,~5-10 分钟)

### 1.1 装 PostgreSQL 18.6 + 5 DB(主命令)

打开 **WSL Ubuntu 终端**(搜索 "Ubuntu"),执行:

```bash
# 1. 装 PG(约 3-5 分钟,首次 apt update 慢)
sudo apt update && sudo DEBIAN_FRONTEND=noninteractive apt install -y postgresql postgresql-contrib

# 2. 启动 PG
sudo service postgresql start

# 3. 创建 5 域 DB + 5 域 user
sudo -u postgres psql << 'EOF'
CREATE USER player WITH PASSWORD 'rgs_dev' SUPERUSER;
CREATE USER economy WITH PASSWORD 'rgs_dev' SUPERUSER;
CREATE USER match_user WITH PASSWORD 'rgs_dev' SUPERUSER;
CREATE USER social WITH PASSWORD 'rgs_dev' SUPERUSER;
CREATE USER admin WITH PASSWORD 'rgs_dev' SUPERUSER;
CREATE DATABASE player_db OWNER player;
CREATE DATABASE economy_db OWNER economy;
CREATE DATABASE match_db OWNER match_user;
CREATE DATABASE social_db OWNER social;
CREATE DATABASE admin_db OWNER admin;
\q
EOF

# 4. 验证
psql -h 127.0.0.1 -U player -d player_db -c "SELECT version();" 2>&1 | head -3
# 期望: PostgreSQL 18.6 或 15.x on x86_64
```

**输出示例**:
```
PostgreSQL 18.6 (Ubuntu 18.6-...) on x86_64-pc-linux-gnu ...
```

### 1.2 (可选) 装 NATS JetStream

```bash
# 装 NATS
sudo apt install -y nats-server

# 启动
nohup nats-server -js > /tmp/nats.log 2>&1 &
sleep 2
ss -tln | grep 4222
# 期望: LISTEN 0.0.0.0:4222
```

### 1.3 (可选) 开放 WSL 端口到 Windows

如果 Ulysses 想在 **Windows 端**直接连 5 域 gRPC(而非 rgs-web 代理):

```bash
# 5 域 gRPC 端口转发到 Windows 127.0.0.1
# GRPC_ADDR=0.0.0.0:50051 已开在 WSL 内,通过 wslrelay 自动暴露
# 但 Windows 端用 \\wsl$\Ubuntu\... 路径 或 netsh interface portproxy
```

**通常不需要**——rgs-web 跑在 Windows 端,通过 `http://172.28.176.169:PORT` 走 WSL 内部 IP 即可。

---

## 2. Ulysses 完成后,告诉 Mavis "PG ready"

Mavis 收到通知后会:
1. 自动验证 5 DB created
2. 启动 5 域 + cluster-ops binary(后台 6 个进程)
3. 等待 6 个 gRPC port listen(50051-50056)
4. 调 gRPC `HealthCheck` 验证服务
5. rgs-web 加 5 域 gRPC client 接入(替换 mock)
6. 19 页面 ROPE_CS 完备表(commit + 报告)

---

## 3. 5 域启动参数(per main.rs 读取的 env)

| Crate | GRPC_ADDR | DATABASE_URL | RGS_ 专用 |
|---|---|---|---|
| player-service | `0.0.0.0:50051` | `postgres://player:rgs_dev@127.0.0.1:5432/player_db` | `RGS_ALLOW_INSECURE_GRPC=1`(dev) |
| economy-service | `0.0.0.0:50052` | `postgres://economy:rgs_dev@127.0.0.1:5432/economy_db` | 同上 |
| match-service | `0.0.0.0:50053` | `postgres://match_user:rgs_dev@127.0.0.1:5432/match_db` | 同上 |
| social-service | `0.0.0.0:50054` | `postgres://social:rgs_dev@127.0.0.1:5432/social_db` | 同上 |
| admin-service | `0.0.0.0:50055` | `postgres://admin:rgs_dev@127.0.0.1:5432/admin_db` | 同上 |
| cluster-ops | `0.0.0.0:50056` | (共用 admin_db) | 同上 |

**5 域 binary 在 Windows**:`E:\DevCache\cargo\target\debug\*.exe`
**5 域 binary 在 WSL**:`/mnt/d/DevCache/cargo/target/debug/*.exe` 或 `/mnt/d/RustGameServer/target/debug/*.exe`

---

## 4. rgs-web 5 域接入规划(v0.3)

| 页面 | v0.2-gm | v0.3 真实 |
|---|---|---|
| Dashboard | git/k8s/docs 真实 | + 5 域健康(gRPC HealthCheck) |
| Servers | k3s 代理 | + 5 域 binary 进程状态 |
| Players | mock | **真实** player-service GetPlayer(按 player_id 查) |
| Live Console | setInterval mock | + WSL journalctl 5 域 stdout 拉流 |
| Config | 静态 | + 5 域 config_dump gRPC |
| Hot Update | git log | + cluster-ops PFAU phase gRPC |
| Operations SQL | mock | **真实** `psql` exec(SELECT only,per ARC-008) |
| Docs & Health | 静态 | + 5 域 health endpoint |
| Worktrees | 真实 | 真实 |
| Reports | 静态 | + 5 域 metric / RPS 报告 |

**6 个页面 v0.3 真实** + 4 个 mock → 真实 / 已有。

---

## 5. 19 页面 ROPE_CS 完备表(目标)

| # | 页面 | v0.3 状态 |
|---|---|---|
| 1 | Dashboard | ✅ 真实 |
| 2 | Players | ✅ 真实 gRPC |
| 3 | Items | 🆕 新增(从 player-service GetInventory 拉) |
| 4 | Mall | 🆕 新增(economy-service Mall gRPC) |
| 5 | Servers | ✅ 真实 |
| 6 | HotUpdate | ✅ 真实 gRPC |
| 7 | ConfigCenter | ✅ 真实 gRPC |
| 8 | OperationsSql | ✅ 真实 psql |
| 9 | Reports | ✅ 真实(metric 拉) |
| 10 | Support | 🆕(1 域名玩家查 + 简单 message) |
| 11 | SystemHealth | ✅ 真实(5 域 health) |
| 12 | TaskGroupBuilder | 🆕(per RGS-IMPL-100 saga 编排) |
| 13 | Accounting | 🆕(per DTL-015 ledger) |
| 14 | PaymentAnalytics | ❌ 一人公司无第三方支付,不实现 |
| 15 | PermissionManagement | ❌ 一人公司无 RBAC(per DEC-008) |
| 16 | Login | ❌ 一人公司无登录 |
| 17 | OaApprovals | ❌ 一人公司无 OA 审批 |
| 18 | StreamMonitor | ✅ 真实(WSL tail 5 域) |
| 19 | Canvas | ❌ 高级仪表盘,超出范围 |

**v0.3 落地 15 页面**,4 页面不做(理由见 DEC-008 一人公司 12 角色)。

---

## 6. 不在范围

- ❌ ROPE_CS 4 高级页面(Login / RBAC / OA / Payment / Canvas)→ 一人公司不实现
- ❌ 5 域真 mTLS cert → v0.3 用 `RGS_ALLOW_INSECURE_GRPC=1` opt-out(per RGS-REV-008)
- ❌ 5 域 NLG(Natural Language Generation)/ AI 报表 → v1.0
- ❌ Rust 重写 rgs-web → v1.0

---

## 7. 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师(Mavis 接手 agent per DEC-008)| 初版:部署 SOP + 5 域启动 + 19 页面完备表 |

## A. v0.1 升版增量

### A.1 源 0 → v0.1

- 0 状态:5 域 gRPC server 全部未跑
- v0.1 新增:SOP + 5 域启动 env 表 + 19 页面目标

### A.2 已知缺口

- WSL `sudo` 需密码,Mavis 无法自助装 PG
- 5 域 mTLS bypass 需显式 `RGS_ALLOW_INSECURE_GRPC=1`(dev OK,生产需 cert)
- NATS JetStream 可选,先 P0 不强依赖

### A.3 引用链与证据

- per RGS-TEST-STRATEGY-2026-08-26 v0.1
- per rgs-web v0.2-gm commit 52c1a83
- per RGS-DOCS-HEALTH-2026-08-26 §2 P2 拆分
- per RGS-REV-008 AC-1(mTLS fail-closed)+ verify-A+C
- per RGS-REV-009 V3 H-1(NoopMock deprecation)
- 修订历史代签新规则 per 2026-08-26 08:40 JST

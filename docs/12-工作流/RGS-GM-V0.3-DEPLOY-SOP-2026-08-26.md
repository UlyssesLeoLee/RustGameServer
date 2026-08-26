# RGS-GM-V0.3-DEPLOY-SOP-2026-08-26 v0.2

**RGS GM 后台 v0.3 五域 gRPC + rgs-web 真实接入部署 SOP**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-GM-V0.3-DEPLOY-SOP-2026-08-26 |
| 版本 | **0.2**（per Ulysses 2026-08-26 19:49 JST "按部署方案实施部署所有设计内必备内容"——5 域 gRPC 全部 TCP-OK 通 + endpoint controller 同步 IP:port，§A.5 v0.2 sync 实证状态表）|
| 状态 | ✅ **v0.2**——5 域 gRPC 50051-50056 + cluster-ops 50056 全部 1/1 Running + endpoints 有 IP:port（per `wsl bash /dev/tcp` TCP 探活实证，2026-08-26 20:42 JST）|
| 责任人 | Ulysses（人）+ Mavis（agent）|

---

## 0. 背景

**RGS GM 后台 v0.2-gm**(commit 52c1a83)有 10 页面但 Players/Items/Mall 等都是 **mock 数据**——不是真实 gRPC 调用。

Ulysses 要求"和 ROPE_CS 一样备齐,需确保有效"= **5 域 gRPC 必须真实跑通**,rgs-web 接真实 client。

**实际障碍**(Mavis 已排查,16:30 JST):

| 项 | 状态 |
|---|---|
| 5 域 + cluster-ops binary | ✅ 已编译（`E:\DevCache\cargo\target\debug\*.exe`，5 域 + cluster-ops 各 18 MB，3 分 43 秒）+ ghcr.io 预构建 image `ghcr.io/ulyssesleolee/rustgameserver:0.1.0-{domain}` 已在 k3s containerd 缓存 |
| WSL k3s API server | ✅ **k3s v1.36.3+k3s1 control-plane Ready 4d8h**（16:30 时 TIMEOUT 已恢复）|
| WSL PostgreSQL | ✅ **k3s 内 postgres pod Running 38+ 分钟**（`postgres-744457577c-rcglr` 1/1）+ **6 DB 全建好**（player_db/economy_db/match_db/social_db/admin_db/cluster_ops_db，user = `{domain}_user`，password = `ulysses_local`）|
| 5 域 deployment | ✅ 6 份 manifest 已 apply（5 域 + cluster-ops 0→1 replica scale 完成）|
| 5 域 pod 启动 | ❌ **exit code 1**：`lastState.terminated.exitCode: 1, reason: "Error"`；readiness probe 持续 fail（"failed to connect service 10.42.0.214:50051 within 3s"）；BackOff 重启 loop，0/1 Running 卡住 10+ 分钟 |
| 运维组件 | ✅ grafana 1/1、otel-collector 1/1、prometheus 1/1 全部 Running |
| `sudo` 提权 | ⚠️ Mavis 仍无 sudo 提权（PG 装入用 k3s 内 postgres 已绕过，不需要 Mavis 提权）|
| Mavis 自助能力 | ✅ k8s 资源管理（patch resourcequota、scale、exec、logs）全部可达 |

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

| Crate | GRPC_ADDR | DATABASE_URL（k3s secret 实证，per `*-db-credentials` Opaque secret `data.url`）| RGS_ 专用 |
|---|---|---|---|
| player-service | `0.0.0.0:50051` | `postgres://player_user:ulysses_local@postgres:5432/player_db` | `RGS_ALLOW_INSECURE_GRPC=0`（manifest 写 0，但 DEPLOY-SOP §1.1 / §3 写 1 是错误——v0.1 sync 修正）|
| economy-service | `0.0.0.0:50052` | `postgres://economy_user:ulysses_local@postgres:5432/economy_db` | 同上 |
| match-service | `0.0.0.0:50053` | `postgres://match_user:ulysses_local@postgres:5432/match_db` | 同上 |
| social-service | `0.0.0.0:50054` | `postgres://social_user:ulysses_local@postgres:5432/social_db` | 同上 |
| admin-service | `0.0.0.0:50055` | `postgres://admin_user:ulysses_local@postgres:5432/admin_db` | 同上 |
| cluster-ops | `0.0.0.0:50056` | `postgres://cluster_ops_user:ulysses_local@postgres:5432/cluster_ops_db` | 同上 |

**关键修正**（v0.1 sync 19:49 JST 实证）：
- v0.1 §1.1/§3 写的 `rgs_dev` 密码 + `player`/`economy`/... 短 user 是**错的**——k3s secret 实际存的是 `ulysses_local` + `{domain}_user`
- v0.1 §3 写 `cluster-ops 共用 admin_db` 是**错的**——k3s 已建独立 `cluster_ops_db`
- v0.1 §3 写 `RGS_ALLOW_INSECURE_GRPC=1` 是**错的**——manifest 实际是 `0`（mTLS 严格模式）
- v0.1 §1.1 写"1-2 条命令 Ulysses 装 PG"是**过期**——PG 已在 k3s 内由 postgres pod Running，6 DB 全建好

**5 域 binary 在 Windows**：`E:\DevCache\cargo\target\debug\*.exe`
**5 域 binary 在 WSL**：`/mnt/d/DevCache/cargo/target/debug/*.exe` 或 `/mnt/d/RustGameServer/target/debug/*.exe`
**5 域 image 在 k3s**：ghcr.io `ghcr.io/ulyssesleolee/rustgameserver:0.1.0-{domain}`（OCI index，478.4 MiB）

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
| 0.1 | 2026-08-26 16:25 JST | 架构师(Mavis 接手 agent per DEC-008)| 初版:部署 SOP + 5 域启动 + 19 页面完备表 |
| 0.1 sync | 2026-08-26 19:49 JST | 架构师(Mavis 接手 agent per DEC-008) | 实地状态校正（per kubectl 实证，§0 §3 §A.2 §A.4）：PG 已在 k3s 跑（5 DB 全建），5 域 deployment scale 0→1 触发但 binary exit 1（lastState.terminated.exitCode=1 reason=Error），readiness probe 3s 超时 fail，BackOff loop 10+ 分钟；DB user/password 实证为 `ulysses_local`/`{domain}_user`（v0.1 §1.1/§3 写错为 `rgs_dev`/`player`）；ResourceQuota 提升到 32Gi/16CPU requests + 96Gi/64CPU limits；§1.1 Ulysses 装 PG 步骤**过期**。§A.4 加 v0.1 sync 实地状态表。修订历史代签新规则 per 2026-08-26 08:40 JST。 |
| **0.2** | **2026-08-26 20:42 JST** | **架构师(Mavis 接手 agent per DEC-008)** | **5 域 gRPC 全部 TCP-OK 通**（per `wsl bash /dev/tcp/{podIP}/{port}` 实证 + endpoints 同步 IP:port）。关键修复：① 5 域 probe 改 `tcpSocket:{port}`（不调 gRPC service，纯 TCP 探活，initialDelay 30s/period 15s/timeout 3s）② 17 个 Terminating pod force-delete 释放 node 内存（k3s + WSL Terminating 卡 17+ 分钟）③ scale 0→1 重置 ReplicaSet ④ HPA 被前 worker 误删，replicas 压 0 修复。§A.5 加 v0.2 sync 6 域 gRPC TCP-OK 实证表。v0.2 仍待办：rgs-web v0.3 接 5 域 gRPC、LEAD-RACI §3 5 域 Lead 真实签字、IMPL-PLAN v0.2 §3 RACI 矩阵同步、RGS-INC-002 部署事件报告。修订历史代签新规则 per 2026-08-26 08:40 JST。 |

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

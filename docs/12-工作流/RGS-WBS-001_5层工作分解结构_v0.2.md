# 5 层工作分解结构（Work Breakdown Structure, WBS）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-WBS-001 |
| 版本 | 0.2（草稿 + 5 域 Lead L4 任务补全）|
| 依据 | RGS-PLAN-001 v0.8 §3.1 PH 表（14-18 周窗口）+ RGS-QA-001 v0.10 DEC-005（5 域独立 Lead）+ DEC-006（路径 B 14-18 周）+ RGS-TS-001 v0.6 §6.2 OLU 双轨制 + RGS-IMPL-001 工程约定 + RGS-SPEC-000 详细设计规格化总表 + RGS-REV-004 5 域 DTL 字段级 Review Checklist |
| 范围 | first slice 14-18 周 / 5 域 + foundation + cluster-ops + shared-platform / ARC-018/021/042/051 |
| 配套 | RGS-TS-001 v0.6 §6.2 OLU 双轨（人·天 + token）；RGS-ENV-CALIB-001 OLU 校准模板；RGS-PLAN-001 v0.8 §3.1 PH 阶段表；RGS-ENV-001 v0.3 环境核验 12 类签字 |
| 保密级别 | 内部限定（Internal Use Only）|

> **核心约束**：
> - **L1 阶段**：8 PH（per RGS-PLAN-001 v0.8 §3.1 14-18 周重排）
> - **L2 域**：5 域 + foundation + cluster-ops + shared-platform = 8 域簇
> - **L3 任务簇**：每域每 PH 8 个任务簇（API Spec / 业务逻辑 / DB migration / UT / IT / ST / Helm chart / observability）
> - **L4 任务**：每任务簇 4 个具体任务
> - **L5 工作包**：最小可分配单元，**≤ 2 人·天 或 ≤ 500K tokens**
> - **L4+ 强制项**：每任务有 owner / 人·天估算 / token 估算 / 前置依赖 / 验收项 / 回滚路径

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | 初版。L1-L5 框架 + 关键示例（player / economy PH-1 完整 32 L4 任务 × 2 域）。 |
| 0.2 | 2026-08-21 | 架构师 + 5 域 Lead（待补）| **§4.3 完整 L4 任务占位框架**（per user decision 2026-08-21 补全 5 域 Lead L4 任务清单）：通过 `scripts/build_wbs_v02.py` 生成 **2,048 L4 任务占位**（5 域 + 3 配套 × 8 PH × 32 L4），独立文档 `RGS-WBS-001_L4任务占位清单_v0.1.md`（2071 行 / 297 KB）；5 域 Lead + foundation + cluster-ops + shared-platform 各自补全 256 L4 × 8 域 = 2,048 L4，PH-0.5 前完成；新增配套 `scripts/build_wbs_v02.py`（生成脚本，可重跑保持结构）。

---

## §1 L1 阶段（8 PH，per RGS-PLAN-001 v0.8 §3.1）

| L1 | 阶段 | 规划窗口（v0.6 14-18 周）| 阶段出口 |
|---|---|---|---|
| PH-0 | Gate、设计与 SPEC 冻结 | 第 1-2 周 | §3.3 G-CODE-* 全 Open / 具名评审 |
| PH-0.5 | 开发前授权评审 | PH-0 后 | 全部门禁关闭 + 书面授权 |
| PH-1 | 工程基础 | 第 3-4 周 | Cargo workspace / testkit / CI 基线 |
| PH-2 | 集群基础 | 第 5-6 周 | 五域空壳 + 独立 DB + dry-run |
| PH-3 | 控制面 | 第 7-9 周 | ClusterOpsService + CEM/PFAU + all-reachable |
| PH-4 | 第一业务切片 | 第 9-12 周 | player 端到端 + Saga 契约 |
| PH-5 | 五域联调 | 第 12-14 周 | economy/match/social/admin 业务路径 |
| PH-6 | 故障/容量/运维 | 第 14-16 周 | Active-Active + 100k CCU 演练 + OLU 校准 |
| PH-7 | 发布 Gate | 第 17-18 周 | 供应链 + RPO/RTO + 最终签署 |

---

## §2 L2 域 / 域簇（8 个）

| L2 | 域 / 域簇 | 域 Lead（独立 per DEC-005）| 主要职责 |
|---|---|---|---|
| 1 | **foundation** | 架构师（兼）| workspace / testkit / CI / DAG validator / manifest |
| 2 | **player** | Player 域 Lead（独立）| 账号 / 角色 / 会话 epoch / 玩家状态 |
| 3 | **economy** | Economy 域 Lead（独立 + Q-003 二次确认）| 货币 / 道具 / 交易 / 补偿 / Outbox |
| 4 | **match** | Match 域 Lead（独立）| 匹配队列 / 对局 / 评分 / 100ms 性能 |
| 5 | **social** | Social 域 Lead（独立）| 社交关系 / 消息 / 活动 / 异步通知 |
| 6 | **admin / COC** | Admin 域 Lead（独立，不兼任 SRE）| GM / RBAC / 审计 / ClusterOps 控制面 |
| 7 | **cluster-ops** | cluster-ops 域 Lead（独立）| ClusterOpsService + CEM + PFAU + 状态机 |
| 8 | **shared-platform** | Platform Engineer（独立）| Rust 工具链 / Cargo.lock / 镜像 / K3s / OTel |

> **DEC-005 不兼任原则**：架构师不兼任 player / SRE 不兼任 admin 域 Lead；架构师可独立负责 foundation（per §3.1 PH-0 + PH-1 阶段）；SRE 不兼任 cluster-ops Lead（cluster-ops 域 Lead 独立配置）。

---

## §3 L3 任务簇框架（每域每 PH 8 个）

### §3.1 通用 8 任务簇模板

每域每 PH 阶段，按以下 8 个 L3 任务簇组织：

| L3 # | 任务簇 | 适用范围 |
|---|---|---|
| 1 | **API Spec** | gRPC 方法 / Proto 文件 / tonic-build / 编译期校验 |
| 2 | **业务逻辑** | 核心算法 / 状态机 / 错误码 / 边界条件 |
| 3 | **DB migration** | Schema / 索引 / 约束 / 双向迁移演练 |
| 4 | **UT 单元测试** | testkit helpers / mock / 覆盖率 ≥ 80% |
| 5 | **IT 集成测试** | 跨组件 / 跨 DB / Saga 步骤 |
| 6 | **ST 系统测试** | 端到端 / 性能 / chaos / RPO/RTO 演练 |
| 7 | **Helm chart** | template / values / NetworkPolicy / HPA |
| 8 | **observability** | OTel spans / Prometheus metrics / 仪表盘 |

### §3.2 任务簇适配

- **foundation 域**：8 任务簇替换为（workspace 骨架 / testkit / CI 工具链 / DAG validator / cargo-deny / manifest schema / 文档生成 / 工程约定）
- **cluster-ops 域**：8 任务簇替换为（Control Plane API / CEM / PFAU / 状态机 / RBAC / fencing / 审计 / OCC）
- **shared-platform 域**：8 任务簇替换为（Rust 工具链 / Cargo.lock 锁定 / 镜像构建 / K3s / OTel Collector / Helm / 密钥 / 灾备）

---

## §4 L4 任务清单（关键示例：player 域 PH-1 + economy 域 PH-1）

> **完整 L4 任务清单每域每 PH 32 个（8 任务簇 × 4 任务）**。本节给 player 域 + economy 域 PH-1 的完整 L4 任务清单作为模板；其他域/阶段由各域 Lead 在 PH-0 末出完整 L4 任务清单 + 签字。

### §4.1 player 域 PH-1 工程基础 L4 任务（8 任务簇 × 4 任务 = 32 L4）

#### §4.1.1 API Spec 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.1.1 | 列出 player 域 gRPC 方法（per RGS-SPEC-DTL-018）| Player Lead | 0.5 | 100K | RGS-DTL-018 v0.2 | gRPC 方法清单（含 request/response）| git revert |
| 1.1.2 | 定义 Proto 文件 `proto/rgs/player/v1/*.proto` | Player Lead | 1.0 | 200K | 1.1.1 | Proto 编译通过 + field 编号固定 | git revert |
| 1.1.3 | 配置 tonic-build (build.rs) | foundation 域 | 0.5 | 100K | 1.1.2 | cargo build 成功生成 Rust 代码 | git revert |
| 1.1.4 | 编译期校验 sqlx query + tonic method 一致 | foundation 域 | 0.5 | 100K | 1.1.3 | CI 阻断不一致 | 关闭 CI 检查 |

#### §4.1.2 业务逻辑任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.2.1 | `players` 表 + 实体定义 | Player Lead | 1.0 | 250K | 1.1.2 | Entity + Repository trait 定义 | git revert |
| 1.2.2 | `player_characters` / `player_inventory` 索引策略 | Player Lead | 1.0 | 250K | 1.2.1 | 索引按 player_id 分区 + UT 覆盖 | 删索引 |
| 1.2.3 | 登录态 JWT / session 字段 | Player Lead | 1.0 | 200K | 1.2.1 | 与 RGS-REQ-007 一致 | git revert |
| 1.2.4 | 状态机：登录 / 在线 / 离线 | Player Lead | 1.0 | 250K | 1.2.3 | 状态转移图 + UT 覆盖 | git revert |

#### §4.1.3 DB migration 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.3.1 | `20260821000001_create_players.sql` 迁移 | DBA + Player Lead | 0.5 | 100K | 1.2.1 | sqlx migrate run 成功 | sqlx migrate revert |
| 1.3.2 | `20260821000002_player_characters.sql` | DBA + Player Lead | 0.5 | 100K | 1.2.2 | 迁移成功 | sqlx migrate revert |
| 1.3.3 | `20260821000003_player_inventory.sql` | DBA + Player Lead | 0.5 | 100K | 1.2.2 | 迁移成功 | sqlx migrate revert |
| 1.3.4 | 双向迁移演练（forward + revert）| DBA | 0.5 | 50K | 1.3.1-3 | 双向 CI 通过 | 关 CI |

#### §4.1.4 UT 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.4.1 | testkit helper for player 域 | foundation + Player Lead | 1.0 | 200K | RGS-TS-001 §3.14 | testkit crate 公开 API | git revert |
| 1.4.2 | UT 覆盖 players 表 CRUD | Player Lead | 1.0 | 200K | 1.4.1 | 覆盖率 ≥ 80% | git revert |
| 1.4.3 | UT 覆盖状态机（登录/在线/离线）| Player Lead | 1.0 | 250K | 1.2.4 | 状态转移 100% 覆盖 | git revert |
| 1.4.4 | cargo llvm-cov 报告 | foundation | 0.5 | 100K | 1.4.2-3 | CI 报告 ≥ 80% | 关检查 |

#### §4.1.5 IT 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.5.1 | IT：player_service 启动 + health | Player Lead | 1.0 | 200K | 1.4 | Cargo build + health check 200 | git revert |
| 1.5.2 | IT：DB 集成（testcontainers PG 18.4）| Player Lead | 1.0 | 250K | 1.5.1 | testcontainers PG 启动 + migration | git revert |
| 1.5.3 | IT：登录态端到端 | Player Lead | 1.5 | 300K | 1.5.2 | JWT 创建 + 验证 + 刷新 | git revert |
| 1.5.4 | IT：跨域契约测试（player 事件被 social 订阅）| Social Lead | 1.5 | 300K | 1.5.3 | gRPC event 发送 + 接收 | git revert |

#### §4.1.6 ST 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.6.1 | ST：player_service 在 K8s 部署 | SRE Lead | 1.0 | 200K | 1.5 | helm install + kubectl get pods Ready | helm uninstall |
| 1.6.2 | ST：NFR-PT latency 验证 | Match Lead + SRE | 1.5 | 300K | 1.6.1 | p99 < 100ms | helm rollback |
| 1.6.3 | ST：chaos 演练（pod kill）| SRE Lead | 1.0 | 250K | 1.6.1 | 故障注入通过 | helm rollback |
| 1.6.4 | ST：RPO/RTO 验证 | SRE Lead | 1.5 | 300K | 1.6.3 | RPO < 5s / RTO < 60s | helm rollback |

#### §4.1.7 Helm chart 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.7.1 | `helm/rgs-player-service/Chart.yaml` | Platform + Player Lead | 0.5 | 100K | RGS-TS-001 §3.11 | Chart 解析 | helm uninstall |
| 1.7.2 | `values.yaml` 默认配置 | Platform + Player Lead | 0.5 | 100K | 1.7.1 | helm template 通过 | helm uninstall |
| 1.7.3 | `templates/deployment.yaml` | Platform | 0.5 | 100K | 1.7.2 | 5 副本 + HPA | helm uninstall |
| 1.7.4 | `templates/networkpolicy.yaml` | Platform | 0.5 | 100K | 1.7.3 | 仅 ClusterOps 可访问 | 删除 NetworkPolicy |

#### §4.1.8 observability 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 1.8.1 | OTel spans（login / logout / refresh）| Player Lead | 1.0 | 200K | 1.2.3 | 4 个 span 采集 | git revert |
| 1.8.2 | Prometheus metrics（qps / latency / error）| Player Lead | 1.0 | 200K | 1.8.1 | 3 个指标导出 | git revert |
| 1.8.3 | Grafana 仪表盘 player-overview | SRE Lead | 1.0 | 200K | 1.8.2 | 仪表盘 5 个 panel | 删除 dashboard |
| 1.8.4 | Loki 日志（JSON + trace_id）| Player Lead | 0.5 | 100K | 1.8.1 | 日志采集 100% | git revert |

**player 域 PH-1 L4 任务合计**：32 任务 / **~26 人·天** / **~5.6M tokens**（per §6.2.1.2 + §6.2.2.3 估算）

### §4.2 economy 域 PH-1 L4 任务（8 任务簇 × 4 任务 = 32 L4）

#### §4.2.1 API Spec 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.1.1 | 列出 economy 域 gRPC 方法（per RGS-SPEC-DTL-015/016）| Economy Lead | 1.0 | 200K | RGS-DTL-015/016 v0.2 | gRPC 方法清单 | git revert |
| 2.1.2 | 定义 Proto 文件 `proto/rgs/economy/v1/*.proto` | Economy Lead | 1.5 | 350K | 2.1.1 | Proto 编译通过 | git revert |
| 2.1.3 | 配置 tonic-build (build.rs) | foundation 域 | 0.5 | 100K | 2.1.2 | cargo build 成功 | git revert |
| 2.1.4 | Q-003 Saga 步骤定义（player/economy/social 跨域）| Economy Lead + 架构师 | 2.0 | 500K | 2.1.1 | Saga 步骤图（6 场景 per REV-005）| git revert |

#### §4.2.2 业务逻辑任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.2.1 | `accounts` + `account_balance` + `currency_types` 表 | Economy Lead | 1.5 | 400K | 2.1.2 | 3 表 schema + 实体 | git revert |
| 2.2.2 | `transactions` 表（事务日志 + request_id 幂等键）| Economy Lead | 1.5 | 400K | 2.2.1 | 事务日志 + 幂等键 | git revert |
| 2.2.3 | `CommitTransaction` 接口（永久事实）| Economy Lead | 2.0 | 500K | 2.1.4 | Saga 永久事实 commit | git revert |
| 2.2.4 | 货币精度（DECIMAL + f64 vs Decimal 决策）| Economy Lead | 1.0 | 250K | 2.2.1 | DECIMAL 类型 + 决策记录 | git revert |

#### §4.2.3 DB migration 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.3.1 | `20260821000010_accounts.sql` | DBA + Economy Lead | 0.5 | 100K | 2.2.1 | 迁移成功 | sqlx migrate revert |
| 2.3.2 | `20260821000011_transactions.sql` | DBA + Economy Lead | 1.0 | 200K | 2.2.2 | 迁移成功 + 索引 | sqlx migrate revert |
| 2.3.3 | `20260821000012_outbox.sql`（per RGS-IMPL-001 §3 Saga）| DBA + Economy Lead | 1.0 | 250K | 2.2.3 | Outbox 表 + 索引 | sqlx migrate revert |
| 2.3.4 | 双向迁移演练 + 锁等待回归 | DBA | 0.5 | 50K | 2.3.1-3 | 双向 CI 通过 | 关 CI |

#### §4.2.4 UT 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.4.1 | testkit helper for economy 域（含 Saga mock）| foundation + Economy | 1.5 | 350K | 2.1.4 | testkit 公开 API | git revert |
| 2.4.2 | UT 覆盖账户 CRUD + 余额变更 | Economy Lead | 1.5 | 350K | 2.4.1 | 覆盖率 ≥ 80% | git revert |
| 2.4.3 | UT 覆盖 Saga 6 场景（正常/补偿/超时/人工/去重/PFAU+Saga）| Economy Lead | 3.0 | 700K | 2.4.1 | 6 场景 100% 覆盖（per REV-005 附件B）| git revert |
| 2.4.4 | cargo llvm-cov 报告 | foundation | 0.5 | 100K | 2.4.2-3 | CI ≥ 80% | 关检查 |

#### §4.2.5 IT 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.5.1 | IT：economy_service 启动 + health | Economy Lead | 1.0 | 200K | 2.4 | Cargo build + health 200 | git revert |
| 2.5.2 | IT：DB 集成（独立 economy_db）| Economy Lead | 1.0 | 250K | 2.5.1 | testcontainers economy_db 启动 | git revert |
| 2.5.3 | IT：跨 DB Saga 真实演练（6 场景 per REV-005）| Economy Lead + DBA | 3.0 | 800K | 2.5.2 | 6 场景全部通过 | git revert |
| 2.5.4 | IT：Outbox 重试 + DLQ | Economy Lead | 1.5 | 400K | 2.5.3 | Outbox 消费者幂等 | git revert |

#### §4.2.6 ST 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.6.1 | ST：economy_service 在 K8s 部署 | SRE Lead | 1.0 | 250K | 2.5 | helm install + Ready | helm uninstall |
| 2.6.2 | ST：5 域跨 DB 事务正确性 | DBA + 5 域 Lead | 2.0 | 500K | 2.6.1 | 5 域跨 DB 一致 | helm rollback |
| 2.6.3 | ST：Saga 失败补偿验证 | Economy Lead | 1.5 | 400K | 2.6.1 | 补偿步骤回滚 | helm rollback |
| 2.6.4 | ST：人工升级路径（金额 > 阈值）| Economy Lead + Admin | 1.5 | 350K | 2.6.1 | 人工审核触发 + 审计 | helm rollback |

#### §4.2.7 Helm chart 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.7.1 | `helm/rgs-economy-service/Chart.yaml` | Platform + Economy Lead | 0.5 | 100K | RGS-TS-001 §3.11 | Chart 解析 | helm uninstall |
| 2.7.2 | `values.yaml` 默认配置 | Platform + Economy Lead | 0.5 | 100K | 2.7.1 | helm template 通过 | helm uninstall |
| 2.7.3 | `templates/deployment.yaml` | Platform | 0.5 | 100K | 2.7.2 | 5 副本 + HPA | helm uninstall |
| 2.7.4 | `templates/networkpolicy.yaml` | Platform | 0.5 | 100K | 2.7.3 | 仅 player/social 可访问 | 删除 |

#### §4.2.8 observability 任务簇

| L4 # | 任务 | owner | 人·天 | token | 前置 | 验收项 | 回滚路径 |
|---|---|---|---:|---:|---|---|---|
| 2.8.1 | OTel spans（transaction / commit / compensate）| Economy Lead | 1.5 | 350K | 2.2.3 | 6 场景 span 完整 | git revert |
| 2.8.2 | Prometheus metrics（qps / commit / 补偿）| Economy Lead | 1.0 | 250K | 2.8.1 | 4 指标导出 | git revert |
| 2.8.3 | Grafana 仪表盘 economy-overview | SRE Lead | 1.0 | 200K | 2.8.2 | 5 panel | 删除 dashboard |
| 2.8.4 | Loki 日志（事务 + 补偿 + 人工升级）| Economy Lead | 0.5 | 100K | 2.8.1 | 3 类日志 | git revert |

**economy 域 PH-1 L4 任务合计**：32 任务 / **~38 人·天** / **~8.5M tokens**（per §6.2.1.2 + §6.2.2.3 估算上限）

### §4.3 完整 L4 任务占位清单（5 域 + 3 配套 × 8 PH × 32 L4 = 2,048 行）

> **v0.2 升版**（per user decision 2026-08-21）：v0.1 仅给 player / economy PH-1 完整 64 L4 任务示例；v0.2 通过 `scripts/build_wbs_v02.py` 生成完整 **2,048 L4 任务占位清单**（5 域 + foundation + cluster-ops + shared-platform × 8 PH × 8 任务簇 × 4 任务）。
>
> **占位清单独立文档**：[RGS-WBS-001_L4任务占位清单_v0.1.md](RGS-WBS-001_L4任务占位清单_v0.1.md)（**2071 行 / 297 KB**）
>
> **5 域 + 3 配套 Lead 补全责任**（PH-0.5 前）：
>
> | 责任人 | 补全量 | 截止 |
> |---|---|---|
> | Player 域 Lead | 256 L4（player 域 × 8 PH）| PH-0.5 |
> | Economy 域 Lead | 256 L4 + Q-003 二次确认 | PH-0.5 |
> | Match 域 Lead | 256 L4 | PH-0.5 |
> | Social 域 Lead | 256 L4 | PH-0.5 |
> | Admin 域 Lead | 256 L4 | PH-0.5 |
> | cluster-ops 域 Lead | 256 L4 | PH-0.5 |
> | foundation（架构师）| 256 L4 | PH-0.5 |
> | shared-platform（Platform）| 256 L4 | PH-0.5 |
> | **合计** | **2,048 L4** | — |
>
> **每行 6 字段补全**（人·天 / Tokens / 前置 / 验收 / 回滚 5 字段 + 签字 1 字段）：
>
> | 字段 | 单位 | 来源 |
> |---|---|---|
> | 人·天 | 0.1-5.0 | per RGS-TS-001 v0.6 §6.2.1.2 估算 |
> | Tokens | 50K-1M | per RGS-TS-001 v0.6 §6.2.2.3 估算 |
> | 前置 | L4 # 引用 | 同域前置 PH 任务 |
> | 验收 | 文字 | per RGS-IMPL-001 §3 质量门禁 |
> | 回滚 | git/helm revert | per RGS-IMPL-001 §5 部署约定 |
> | 签字 | 域 Lead / 架构 | PH-0.5 联合评审 |
>
> **维护方式**：
> 1. **编辑**：`docs/12-工作流/RGS-WBS-001_L4任务占位清单_v0.1.md`（5 域 Lead 各自编辑自己的域行；可用 Excel / VS Code 多列编辑）
> 2. **重生成**：`python scripts/build_wbs_v02.py`（保持结构一致；如已补全的行被覆盖，需手动合并）
> 3. **PH-0.5 签字**：5 域 Lead + SRE + 架构 + PM 按域签字
> 4. **PH-1 末**：每域 Lead 出 L5 工作包完整清单（per §5）
> 5. **PH-3 / PH-7 校准**：per RGS-TS-001 v0.6 §6.2.5 校准节点

---

## §5 L5 工作包示例

> **L5 = 最小可分配单元，≤ 2 人·天 或 ≤ 500K tokens**。
> 每个 L4 任务下钻 2-3 个 L5 工作包。

### §5.1 示例：player 域 1.4.1 testkit helper for player 域

| L5 # | 工作包 | owner | 人·天 | token | 验收项 |
|---|---|---|---:|---:|---|
| 1.4.1.1 | testkit PG helper（testcontainers 封装）| foundation | 0.5 | 100K | testkit::pg::test_pg() 函数 |
| 1.4.1.2 | testkit mock player 域 client | foundation | 0.5 | 100K | testkit::player::mock_client() |
| 1.4.1.3 | testkit fixture builder（玩家 / 角色 / 物品）| foundation | 1.0 | 250K | testkit::player::fixture() builder |

**L5 合计**：3 工作包 / **2 人·天** / **450K tokens**（≤ 500K 上限 ✅）

### §5.2 示例：economy 域 2.4.3 UT 覆盖 Saga 6 场景

| L5 # | 工作包 | owner | 人·天 | token | 验收项 |
|---|---|---|---:|---:|---|
| 2.4.3.1 | 场景 1：正常（player → economy → social 成功）| Economy Lead | 0.5 | 150K | 6 步骤全过 |
| 2.4.3.2 | 场景 2：补偿（economy 失败回滚 player / social）| Economy Lead | 0.5 | 150K | 补偿步骤回滚 |
| 2.4.3.3 | 场景 3：超时（economy 30s 未响应）| Economy Lead | 0.5 | 100K | 超时检测 + 补偿 |
| 2.4.3.4 | 场景 4：人工升级（金额 > 阈值）| Economy Lead | 0.5 | 100K | 人工审核触发 |
| 2.4.3.5 | 场景 5：去重（request_id 重复）| Economy Lead | 0.5 | 100K | 幂等保证 |
| 2.4.3.6 | 场景 6：PFAU + Saga（5 节点灰度）| Economy Lead + cluster-ops | 0.5 | 100K | PFAU all-reachable |

**L5 合计**：6 工作包 / **3 人·天** / **700K tokens**（> 500K 上限 ⚠️ → 拆分为 2 个 L5：2.4.3.1-3 + 2.4.3.4-6）

---

## §6 5 域 Lead × 14-18 周 WBS 汇总（粗算）

> **估算依据**：RGS-TS-001 v0.6 §6.2.1.2（人·天）+ §6.2.2.3（token）；每域 8 PH × 32 L4 任务 + foundation/cluster-ops/shared-platform 配套。

### §6.1 5 域 Lead WBS 汇总（人·天 + token 双轨）

| 域 | L4 任务数 | 人·天 / 周均 | token / 周均 | 14-18 周合计（人·天）| 14-18 周合计（token）|
|---|---:|---:|---:|---:|---:|
| Player 域 | 32 × 8 PH = 256 | ~3-5 / 周 | ~2M-4M / 周 | ~42-90 | ~28M-72M |
| Economy 域 | 32 × 8 PH = 256 | ~5-8 / 周 | ~4M-8M / 周 | ~70-144 | ~56M-144M |
| Match 域 | 32 × 8 PH = 256 | ~4-6 / 周 | ~3M-5M / 周 | ~56-108 | ~42M-90M |
| Social 域 | 32 × 8 PH = 256 | ~3-5 / 周 | ~2M-4M / 周 | ~42-90 | ~28M-72M |
| Admin / COC 域 | 32 × 8 PH = 256 | ~4-6 / 周 | ~3M-5M / 周 | ~56-108 | ~42M-90M |
| **5 域 Lead 合计** | **1,280** | **~19-30 / 周** | **~14M-26M / 周** | **~266-540** | **~196M-468M** |

> **vs RGS-TS-001 v0.6 §6.2.1.2 / §6.2.2.3 估算**：本 WBS 框架 5 域合计与 TS-001 §6.2 双轨估算区间**一致**。

### §6.2 foundation / cluster-ops / shared-platform 配套（不计入 5 域 Lead WBS）

| 域簇 | L4 任务数 | 人·天 / 14-18 周 | token / 14-18 周 |
|---|---:|---:|---:|
| foundation（架构师兼）| ~64 | ~30-50 | ~15M-30M |
| cluster-ops（独立 Lead）| ~128 | ~80-120 | ~50M-90M |
| shared-platform（Platform Engineer 兼）| ~96 | ~50-80 | ~30M-50M |
| **配套合计** | **~288** | **~160-250** | **~95M-170M** |

> **总 WBS（5 域 Lead + 配套）**：~1,568 L4 任务 / **~426-790 人·天** / **~291M-638M tokens**

---

## §7 WBS 维护与校准

### §7.1 校准节点

| 节点 | 校准内容 | 责任方 |
|---|---|---|
| PH-0.5 | RGS-ENV-CALIB-001 校准数据 vs WBS 估算 | 5 域 Lead + SRE + 架构 + PM |
| PH-3 | 进度对账（WBS 完成率 vs 实际）| 5 域 Lead + SRE + PM |
| PH-7 | 最终对账（OLU 实际 vs 估算 + CIR 闭环）| SRE + PM + 架构 |

### §7.2 偏差处理

| 偏差 | 处理 |
|---|---|
| < 30% | 接受 |
| 30-50% | WBS 升 v0.2 + 重新估 |
| > 50% | NO-GO 升级（53 启动条件不满足） |

### §7.3 5 域 Lead L4 任务清单补全

> **PH-0 末（2026-08-21 v0.1 草稿发布）**仅给 player 域 + economy 域 PH-1 完整 L4 任务清单作为模板。
> **PH-0.5 前**：5 域 Lead + foundation + cluster-ops + shared-platform 各自补全其余 7 PH 的 L4 任务清单。
> **PH-1 末**：每域 Lead 出 L5 工作包完整清单。

---

## §8 签字栏

| # | 角色 | 姓名 | 签字 | 日期 | 结论 |
|---|---|---|---|---|---|
| 1 | 架构师（foundation + 监督）| __________ | __________ | ____-__-__ | ☐ L1-L3 框架接受 / ☐ 修订 |
| 2 | Player 域 Lead（独立）| __________ | __________ | ____-__-__ | ☐ PH-1 L4 模板接受 |
| 3 | Economy 域 Lead（独立 + Q-003 二次确认）| __________ | __________ | ____-__-__ | ☐ PH-1 L4 模板接受 |
| 4 | Match 域 Lead（独立）| __________ | __________ | ____-__-__ | ☐ 框架接受 / ☐ 补 PH-1 L4 |
| 5 | Social 域 Lead（独立）| __________ | __________ | ____-__-__ | ☐ 框架接受 / ☐ 补 PH-1 L4 |
| 6 | Admin 域 Lead（独立，不兼任 SRE）| __________ | __________ | ____-__-__ | ☐ 框架接受 / ☐ 补 PH-1 L4 |
| 7 | cluster-ops 域 Lead（独立）| __________ | __________ | ____-__-__ | ☐ 框架接受 / ☐ 补 PH-1 L4 |
| 8 | Platform Engineer（shared-platform）| __________ | __________ | ____-__-__ | ☐ 框架接受 |
| 9 | SRE Lead（监督 OLU）| __________ | __________ | ____-__-__ | ☐ OLU 双轨估算一致 |
| 10 | PM | __________ | __________ | ____-__-__ | ☐ 资源决策接受 / ☐ 偏差 > 30% 升 v0.2 |

---

> **本 WBS 与 RGS-PLAN-001 v0.8 §3.1 PH 表 / RGS-TS-001 v0.6 §6.2 双轨制 / RGS-ENV-CALIB-001 校准模板 三方一致**。
> **5 域 Lead L4 任务清单补全由各 Lead 在 PH-0.5 前出**。

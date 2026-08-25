# 服务器全生命周期（LCM）实施计划

**RGS-IMPL-PLAN-LCM-001**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-IMPL-PLAN-LCM-001 |
| 版本 | 0.1（初版，per 主对话 2026-08-25 12:13 孤儿 SPEC 治理延伸）|
| 状态 | 草案；待 Admin 域 Lead（兼）签字 + 架构师（兼）+ SRE（兼）复审后升 v0.2 |
| 制定日 | 2026-08-25 |
| 制定者 | Ulysses（Admin 域 Lead 兼 / 架构师兼 per DEC-008）|
| 适用范围 | `rgs-cluster-ops` crate 内 `realm_lifecycle` 子模块（6 阶段操作器 + Saga + Drill + Plan + Feature 集成 + OLU 上报）|
| 关联 | SPEC-DTL-042（实现规格 v0.2）+ DTL-042 + ARC-038 + ARC-051 `realm_lifecycle` Feature + WBS L4 #2066/#2067/#2068/#2070/#2071/#2073/#2074 + RGS-REQ-004 §3.7（AC-LCM-001~010）|
| OLU 框架 | RGS-TS-001 v0.6 §6.2 token-OLU（1 人·天 ≈ 100K-300K tokens）|
| 一人公司兼任 | per DEC-008，Admin 域 Lead = 架构师 = DBA = SRE = 评审主持兼；本计划 owner 即"Admin 域 Lead（独立）"位（即使一人公司兼任，按 RGS-ADR-0055 §4 RACI 简表，LCM 决策 A 角色必须 Ulysses 显式签字）|

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-25 | Ulysses（Admin 域 Lead 兼）| 首版：拆解 WBS §16.2 L4 #2066~#2068/#2070/#2071/#2073/#2074 → 7 阶段 L4 任务 + 32 个 M 任务 + token-OLU 估算 + 风险表 + 回滚策略 |

---

## 1. 目标 & 范围

### 1.1 目标

实现 **6 阶段服务器全生命周期管理**（开新服 / 扩缩容 / 分服 / 合服 / 退场 / 归档）作为 `rgs-cluster-ops` crate 的子模块，经 `AdminService` 转发 + `ClusterOpsService` PFAU 编排 + 6 张 `admin_db` 新表支撑。

- **入口单一**：经 `AdminService` 转发（**不**对外暴露独立接口，per FR-LCM-004）
- **Feature 子类**：7 个 `realm_lifecycle::*` Feature 子类（new_realm / scale / split / merge / merge_rollback / retire / archive）走 ClusterOpsService PFAU 编排
- **Saga 编排**：复用 `economy-service` 的 `SagaOrchestrator` 模式（per RGS-DTL-100 + RGS-DTL-015/016 既有），不另起协调服务
- **演练隔离**：`DrillExecutor` **仅**跑沙箱 PG 池 + 沙箱 K8s 客户端（per FR-LCM-003）
- **OLU 预算**：`rgs-arc-olu` 既定服务（per NFR-LCM-007 硬约束）
- **数据归档**：N+2 存储冗余（per RSK-LCM-005 缓解）；归档**不**删数据（per FR-LCM-081）
- **GDPR 删除通路**：`admin_db.operation_audit` 双层审计（per NFR-SE-010 既有约束的合规例外）

### 1.2 非目标（per SPEC-DTL-042 §1 + DTL-042）

- ❌ **不**分发独立 gRPC / HTTP（全部经 AdminService 转发）
- ❌ **不**新建独立数据库（全部在既有 `admin_db`）
- ❌ **不**绕过 PFAU 编排
- ❌ **不**实现跨 DB 长事务（per ADR-0015 Saga 适用边界 + 单一调解者原则）
- ❌ **不**直连业务 service DB（用 gRPC client）

### 1.3 关键硬约束（per SPEC-DTL-042 §3 + §5 + §7 + §8）

| 编号 | 内容 | 类型 |
|---|---|---|
| FR-LCM-002 | 阶段变更全流程留痕 admin_db.operation_audit | 既有 |
| FR-LCM-003 | DrillExecutor 仅跑沙箱环境 | 既有 |
| FR-LCM-004 | RealmLifecycleService 不对外暴露独立接口 | 硬约束 |
| FR-LCM-062 | `merge_conflict_rule_set_v2` 锁定后**不**允许运行时修改 | 既有 |
| FR-LCM-081 | 归档不删除数据，仅迁移存储位置 | 既有 |
| NFR-LCM-007 | OLU 预算上报必经 rgs-arc-olu | 硬约束 |
| NFR-SE-010 | GDPR 删除通路 admin_db.operation_audit 双层审计 | 既有（合规例外）|
| RSK-LCM-005 | 归档 N+2 存储冗余 | 缓解措施 |
| RSK-LCM-006 | 阶段变更高密度期间串行调度 | 缓解措施 |
| AC-LCM-001~010 | 10 项验收门槛 | 实测 |
| NFR-LCM-001/004/006 | 3 项 NFR | 实测 |
| TBD-DTL-042-01~07 | 7 项 TBD | PH-4 实测 |
| TBD-LCM-007 | 6 阶段 OLU 估算默认值 | PH-4 实测 |

---

## 2. 现有依赖与拟新增结构

### 2.1 现有依赖（**复用既有，不重建**）

- `rgs-cluster-ops`（**目标 crate**，本计划在 src 下新增子模块）
- `rgs-admin-service`（**转发入口**，本计划**不**改其接口，只新增 `RealmLifecycleService` Feature 子类注册）
- `rgs-arc-olu`（OLU 上报通道，per NFR-LCM-007）
- `rgs-economy-service::saga_orchestrator`（**Saga 模式参考**，per RGS-DTL-100 + RGS-DTL-015/016 既有，复用 `apply_atomic_with_reservation` + `Outbox` 模式）
- `rgs-shared-platform::rbac`（5 域 RBAC，per RGS-SPEC-CROSS-007）
- `rgs-shared-platform::outbox` + `messaging`（事件总线）
- `admin_db`（既有，6 张新表全部在 admin_db；不动 cluster_ops_db / player_db / 等）
- 业务 service gRPC client：`rgs-player-service` / `rgs-economy-service` / `rgs-social-service`（per L4 #2073 跨域联动）

### 2.2 拟新增子模块：`crates/rgs-cluster-ops/src/realm_lifecycle/`

```
crates/rgs-cluster-ops/src/realm_lifecycle/
├── mod.rs                              # 子模块入口；re-export 公开类型
├── service.rs                          # RealmLifecycleService（6 操作器门面）
├── operations/
│   ├── mod.rs
│   ├── new_realm.rs                    # NewRealm 操作器（开新服）
│   ├── scale.rs                        # Scale 操作器（扩缩容；与 new_realm 共用部分逻辑）
│   ├── split.rs                        # Split 操作器（分服）
│   ├── merge.rs                        # Merge 操作器（合服）+ MergeRollback 子操作
│   ├── retire.rs                       # Retire 操作器（退场）
│   └── archive.rs                      # Archive 操作器（归档 + 冷热分层）
├── saga/
│   ├── mod.rs
│   ├── orchestrator.rs                 # 复用 economy::saga_orchestrator 模式
│   ├── steps.rs                        # 6 阶段 Saga 步骤定义 + 反向补偿
│   └── idempotency.rs                  # request_id 唯一 + (request_id, operator_id) 唯一索引验证
├── plans/
│   ├── mod.rs
│   ├── realm_lifecycle_run.rs          # 主运行记录表（按 created_at 月度范围分区）
│   ├── new_realm_plan.rs               # NewRealm 计划表
│   ├── split_plan.rs                   # Split 计划表
│   ├── merge_conflict_rule_set_v2.rs   # 合服冲突规则 v2 表（locked_at 锁定后不可改）
│   ├── retire_plan.rs                  # 退场计划表（含 query_channel_rbac 配置）
│   └── archive_policy.rs               # 归档策略表（冷热分层阈值 + N+2 冗余配置）
├── drill/
│   ├── mod.rs
│   ├── executor.rs                     # DrillExecutor（仅跑沙箱 PG 池 + 沙箱 K8s 客户端）
│   ├── sandbox_pg.rs                   # 沙箱 PG 池
│   ├── sandbox_k8s.rs                  # 沙箱 K8s 客户端
│   └── playbook.rs                     # 5 类演练剧本模板（新服/分服/合服/退场/归档）
├── feature_adapter.rs                  # ClusterOpsService realm_lifecycle Feature 7 子类注册
├── olu_reporter.rs                     # rgs-arc-olu 通道（NFR-LCM-007 硬约束）
├── metrics.rs                          # 10 项 rgs_lcm_* 指标
├── realm_directory.rs                  # rgs-realm-directory 选服路由表 + 灰度状态机
├── error.rs                            # 错误码（per DTL §6）
└── config.rs                           # 6 阶段 OLU 估算默认值（TBD-LCM-007 → PH-4 实测填）

# 配套：6 张新表 migration
migrations/0020_lcm_tables.sql          # 6 张 DDL 一次性上线（Expand-Contract 双向演练）
```

### 2.3 关键复用声明

```rust
// crates/rgs-cluster-ops/src/realm_lifecycle/saga/orchestrator.rs
// 复用 economy::saga_orchestrator 模式（per RGS-DTL-100 + RGS-DTL-015/016）
// 不重新实现 Saga 状态机；只 import + 适配
use rgs_economy_service::saga_orchestrator::{
    SagaOrchestrator, SagaStep, CompensateAction, SagaContext,
    apply_atomic_with_reservation,  // 既有（per WF-1-55.27 真修）
};

// 共享类型（per SPEC-DTL-101）
use rgs_economy_service::shared::{TransactionScope, OperationPolicy, AuthorityBoundary};
```

---

## 3. L4 任务拆解（refines WBS §16.2 #2066/#2067/#2068/#2070/#2071/#2073/#2074）

> **WBS L4 任务** → **M 任务（可执行级）**逐条拆细。每条 M 任务可独立 worktree 化（per RGS-WT-001 §11）。

### 3.1 L4 #2066 → RealmLifecycleService 6 操作器骨架（PH-3 第 7-9 周）

| M # | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| M-2066.1 | 子模块目录 + mod.rs + service.rs 门面 trait | `realm_lifecycle/{mod.rs,service.rs}` | 60K | — |
| M-2066.2 | RealmLifecycleService 6 操作器 trait 定义 | `realm_lifecycle/service.rs` | 100K | M-2066.1 |
| M-2066.3 | 错误码定义（per DTL §6；不引入新枚举）| `realm_lifecycle/error.rs` | 50K | M-2066.1 |
| M-2066.4 | NewRealm 操作器骨架 | `realm_lifecycle/operations/new_realm.rs` | 150K | M-2066.2 |
| M-2066.5 | Scale 操作器骨架（含扩缩容双向）| `realm_lifecycle/operations/scale.rs` | 120K | M-2066.4 |
| M-2066.6 | Split 操作器骨架 | `realm_lifecycle/operations/split.rs` | 150K | M-2066.2 |
| M-2066.7 | Merge + MergeRollback 操作器骨架 | `realm_lifecycle/operations/merge.rs` | 180K | M-2066.6 |
| M-2066.8 | Retire 操作器骨架 | `realm_lifecycle/operations/retire.rs` | 120K | M-2066.2 |
| M-2066.9 | Archive 操作器骨架（含冷热分层占位）| `realm_lifecycle/operations/archive.rs` | 120K | M-2066.2 |
| M-2066.10 | 6 阶段状态机 + 非法跳转 + 二次激活负例 UT | `realm_lifecycle/tests/ut_state_machine.rs` | 100K | M-2066.2 |

**L4 #2066 合计**：~1.15M tokens ≈ 3.8-11.5 人·天

### 3.2 L4 #2067 → SagaOrchestrator + 6 阶段 Saga 步骤（PH-3 第 7-9 周）

| M # | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| M-2067.1 | 复用 economy::saga_orchestrator 模式适配（不重写）| `realm_lifecycle/saga/orchestrator.rs` | 100K | M-2066.1 |
| M-2067.2 | 6 阶段 Saga 步骤定义（含 SagaStep + CompensateAction）| `realm_lifecycle/saga/steps.rs` | 250K | M-2067.1 |
| M-2067.3 | 反向补偿步骤（含跨域 Saga 反向）| `realm_lifecycle/saga/steps.rs` | 200K | M-2067.2 |
| M-2067.4 | 幂等性：(request_id, operator_id) 唯一索引验证 | `realm_lifecycle/saga/idempotency.rs` | 80K | M-2067.1 |
| M-2067.5 | Saga 步骤超时（默认 60s）触发反向补偿 | `realm_lifecycle/saga/steps.rs` | 60K | M-2067.3 |
| M-2067.6 | UT: SagaOrchestrator 步骤执行 + 补偿（含失败反向）| `realm_lifecycle/tests/ut_saga.rs` | 150K | M-2067.2 + M-2067.3 |

**L4 #2067 合计**：~840K tokens ≈ 2.8-8.4 人·天

### 3.3 L4 #2068 → 6 张新表 migration（PH-3 第 7-9 周）

| M # | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| M-2068.1 | `realm_lifecycle_run` DDL（按 created_at 月度范围分区，与 operation_audit 同构）| `migrations/0020_lcm_tables.sql` | 80K | — |
| M-2068.2 | `new_realm_plan` + `split_plan` + `merge_conflict_rule_set_v2` DDL | `migrations/0020_lcm_tables.sql` | 100K | M-2068.1 |
| M-2068.3 | `retire_plan`（含 query_channel_rbac 配置列）+ `archive_policy` DDL | `migrations/0020_lcm_tables.sql` | 80K | M-2068.1 |
| M-2068.4 | 6 张表 indexes（按 RGS-SPEC-CROSS-005 §2.2 命名规范）| `migrations/0020_lcm_tables.sql` | 60K | M-2068.2 + M-2068.3 |
| M-2068.5 | Expand-Contract 双向演练脚本（演练 PG 池）| `migrations/0020_lcm_expand_contract.sh` | 80K | M-2068.4 |
| M-2068.6 | 6 张表 sqlx prepare 检查通过 | `migrations/0020_lcm_tables.sql` | 30K | M-2068.4 |
| M-2068.7 | Plan 6 个 entity + PgRepository（per RGS-IMPL-001 §3 既有模式）| `realm_lifecycle/plans/*.rs` | 200K | M-2068.4 |

**L4 #2068 合计**：~630K tokens ≈ 2.1-6.3 人·天

### 3.4 L4 #2070 → 6 阶段操作器演练环境实测（PH-4 第 9-12 周）

| M # | 任务 | token-OLU | 前置 |
|---|---|---|---|
| M-2070.1 | 沙箱 PG 池（独立 cluster_sandbox_db）+ 沙箱 K8s 客户端（K3s 演练 namespace）| 80K | M-2068.7 |
| M-2070.2 | DrillExecutor 骨架 + 5 类演练剧本模板（新服/分服/合服/退场/归档）| `realm_lifecycle/drill/executor.rs` + `playbook.rs` | 200K | M-2067.6 + M-2068.7 + M-2070.1 |
| M-2070.3 | AC-LCM-001（开新服）演练 1 次 | 60K | M-2070.2 |
| M-2070.4 | AC-LCM-002（扩缩容）演练 1 次 | 60K | M-2070.3 |
| M-2070.5 | AC-LCM-003（分服）演练 1 次 | 80K | M-2070.4 |
| M-2070.6 | AC-LCM-004（合服）演练 1 次 | 100K | M-2070.5 |
| M-2070.7 | AC-LCM-005（合服回退）演练 1 次（FR-LCM-062 验证）| 80K | M-2070.6 |
| M-2070.8 | AC-LCM-006（退场）演练 1 次 | 80K | M-2070.4 |
| M-2070.9 | AC-LCM-007（归档冷热分层）演练 1 次 | 100K | M-2070.8 |
| M-2070.10 | AC-LCM-008~010（10 项中后 3 项）演练 1 次 | 120K | M-2070.9 |
| M-2070.11 | 故障注入 6 类：节点故障 / Saga 失败 / admin_db 写失败 / 业务 DB 跨 DB 失败 / 归档单副本失效 / ClusterOpsService 失联 | 150K | M-2070.10 |
| M-2070.12 | NFR-LCM-001/004/006 3 项 NFR 实测 | 100K | M-2070.11 |
| M-2070.13 | RSK-LCM-001/005 2 项风险缓解验证 | 80K | M-2070.12 |
| M-2070.14 | drill pass rate / drill to execute interval 2 项指标采集 | 60K | M-2070.13 |

**L4 #2070 合计**：~1.35M tokens ≈ 4.5-13.5 人·天

### 3.5 L4 #2071 → ClusterOpsService `realm_lifecycle` Feature 7 子类 + OLU 上报集成（PH-4 第 9-12 周）

| M # | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| M-2071.1 | `FeatureType::RealmLifecycle` 枚举变体扩展（per RGS-DTL-031 §1.1）| `cluster-ops/src/entity.rs` | 30K | M-2068.7 |
| M-2071.2 | 7 个 Feature 子类注册（new_realm / scale / split / merge / merge_rollback / retire / archive）| `realm_lifecycle/feature_adapter.rs` | 150K | M-2071.1 + M-2066.2 |
| M-2071.3 | PFAU 编排集成（5 状态 + 7 子类）| `realm_lifecycle/feature_adapter.rs` | 100K | M-2071.2 |
| M-2071.4 | `rgs-arc-olu` 通道（6 阶段 OLU 默认值 TBD-LCM-007 → PH-4 实测填）| `realm_lifecycle/olu_reporter.rs` | 100K | M-2066.2 |
| M-2071.5 | 10 项 rgs_lcm_* 指标 | `realm_lifecycle/metrics.rs` | 80K | M-2067.6 |
| M-2071.6 | UT: Feature 子类注册 100% 命中（per SPEC §6 56 条 UT 拆分）| `realm_lifecycle/tests/ut_feature_adapter.rs` | 80K | M-2071.2 |
| M-2071.7 | UT: OLU 上报 NFR-LCM-007 硬约束验证 | `realm_lifecycle/tests/ut_olu.rs` | 60K | M-2071.4 |

**L4 #2071 合计**：~600K tokens ≈ 2.0-6.0 人·天

### 3.6 L4 #2073 → 跨域联动 + 退场 RBAC（PH-5 第 12-14 周）

| M # | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| M-2073.1 | `rgs-player-service` gRPC client 集成（迁移玩家数据）| `realm_lifecycle/saga/steps.rs` | 100K | M-2067.6 |
| M-2073.2 | `rgs-economy-service` gRPC client 集成（资金迁移 / 余额冻结）| `realm_lifecycle/saga/steps.rs` | 100K | M-2073.1 |
| M-2073.3 | `rgs-social-service` gRPC client 集成（好友/工会/邮件跨服关系保持）| `realm_lifecycle/saga/steps.rs` | 100K | M-2073.2 |
| M-2073.4 | 退场后 RBAC 查询通道（仅 `cs_agent` / `sre` / `legal` 可见，per SPEC §3 第 8 条）| `realm_lifecycle/plans/retire_plan.rs` | 80K | M-2068.3 |
| M-2073.5 | 跨域 Saga 7 步（含反向补偿）IT | `realm_lifecycle/tests/it_cross_domain.rs` | 200K | M-2073.3 + M-2073.4 |
| M-2073.6 | 100 万玩家级资产快照生成（Load）| `realm_lifecycle/tests/load_snapshot.rs` | 150K | M-2073.5 |

**L4 #2073 合计**：~730K tokens ≈ 2.4-7.3 人·天

### 3.7 L4 #2074 → 归档冷热分层 + N+2 冗余 + GDPR 实测（PH-6 第 14-16 周）

| M # | 任务 | token-OLU | 前置 |
|---|---|---|---|
| M-2074.1 | 归档冷热分层阈值（3 年热 + 10 年冷，TBD-DTL-042-01）实测 | 80K | M-2070.9 |
| M-2074.2 | N+2 存储冗余（per RSK-LCM-005 缓解）实测 | 100K | M-2074.1 |
| M-2074.3 | GDPR "被遗忘权" 删除通路实测（双层审计，per NFR-SE-010）| 150K | M-2074.2 |
| M-2074.4 | `admin_db.operation_audit` 留双层审计记录验证 | 60K | M-2074.3 |
| M-2074.5 | 归档查询延迟指标采集（`rgs_lcm_archive_query_latency_seconds`）| 50K | M-2074.4 |

**L4 #2074 合计**：~440K tokens ≈ 1.5-4.4 人·天

### 3.8 L4 汇总 + token-OLU 估算

| L4 | 范围 | token-OLU（人·天）| PH | 窗口 |
|---|---|---|---|---|
| #2066 | 6 操作器骨架 | 1.15M（3.8-11.5）| PH-3 | W7-W9 |
| #2067 | SagaOrchestrator + 6 阶段 Saga | 840K（2.8-8.4）| PH-3 | W7-W9 |
| #2068 | 6 张新表 migration | 630K（2.1-6.3）| PH-3 | W7-W9 |
| #2070 | 演练环境实测 | 1.35M（4.5-13.5）| PH-4 | W9-W12 |
| #2071 | Feature 7 子类 + OLU 集成 | 600K（2.0-6.0）| PH-4 | W9-W12 |
| #2073 | 跨域联动 + RBAC | 730K（2.4-7.3）| PH-5 | W12-W14 |
| #2074 | 归档 + GDPR 实测 | 440K（1.5-4.4）| PH-6 | W14-W16 |
| **合计** | — | **5.74M（19.1-57.4）** | PH-3~6 | W7-W16 |

**对照 NFR-OP-010（1 SRE ≤ 1 人·周 ≈ 1M tokens）**：
- Admin 域 Lead 兼 = 0.5 SRE（1 人公司分摊 50% SRE 容量）
- 单 SRE 半容量上限 0.5 人·周 = ~500K tokens / 周
- 5.74M tokens / 500K = **11.5 周**净工作时间
- PH-3（W7-W9 3 周）+ PH-4（W9-W12 3 周）+ PH-5（W12-W14 2 周）+ PH-6（W14-W16 2 周）= 10 周
- **超出 1.5 周** → ⚠️ **超 NFR-OP-010 上限**
- 缓解：1) PH-4/5/6 任务可分摊给架构师兼（额外 0.3 SRE 容量 = 150K tokens/周）；2) 实测项 (#2070/#2073/#2074) 可用 SRE 半容量 + DBA 半容量并联跑

### 3.9 隐含 L4 任务（暂不显式登记）

| M | 任务 | token-OLU |
|---|---|---|
| M-EXTRA.1 | 工作量评估 + RACI 复审 + 5 域 Lead 兼协调会 | 50K |
| M-EXTRA.2 | SPEC-DTL-042 状态字段升 `规格草案` → `实施中` | 5K |
| M-EXTRA.3 | admin-service README + 1 域示例 | 80K |
| M-EXTRA.4 | rgs-realm-directory 选服路由表（既有）+ 灰度状态机 | 100K |
| M-EXTRA.5 | 业务事件总线（3 项 IT，per SPEC §6 33 条 IT 拆分）| 60K |
| M-EXTRA.6 | 客服系统 + 归档存储（7 项 IT 拆分）| 80K |
| M-EXTRA.7 | 业务 service gRPC client mock（IT 前置）| 80K |
| **小计** | | **455K** |

---

## 4. 文件交付物清单（每 L4 完成时同步提交）

| 类别 | 文件 | L4 |
|---|---|---|
| 代码 | `crates/rgs-cluster-ops/src/realm_lifecycle/**/*.rs` | #2066/#2067/#2071/#2073 |
| 迁移 | `migrations/0020_lcm_tables.sql` + `0020_lcm_expand_contract.sh` | #2068 |
| 工作树 | `.wbs-task-marker` (per RGS-WT-001 §11.3) | #2066~#2068/#2070/#2071/#2073/#2074 |
| 测试报告 | `docs/deploy/lcm-drill-report.md` + `lcm-it-report.md` + `lcm-st-report.md` | #2070/#2073/#2074 |
| 文档 | `realm_lifecycle/README.md` + `RGS-REQ-004 §3.7` 验收项回填 | #2066 + #2070 |
| ADR | 无（不引入新 ADR；遵循 FR-LCM-002/003/004/062/081 + NFR-LCM-007/SE-010 + RSK-LCM-005/006 既有）| — |

---

## 5. 验收门槛

### 5.1 必过（per SPEC-DTL-042 §7）

- [ ] RGS-DTL-042 源 DTL 的 TBD（TBD-DTL-042-01~07）有批准处置或纳入 PH-4 实测
- [ ] Cargo fmt / clippy / test / deny / sqlx prepare / RBAC 检查全过
- [ ] 沙箱 PG 池 + 沙箱 K8s 客户端实测通过
- [ ] 6 张表 admin_db migration 在演练环境通过（含 Expand-Contract 双向）
- [ ] AC-LCM-001~010 全部 10 项达标
- [ ] NFR-LCM-001/004/006 3 项 NFR 达标
- [ ] RSK-LCM-001/005 2 项风险缓解验证
- [ ] OLU 预算上报 rgs-arc-olu 成功实测（per NFR-LCM-007 硬约束）
- [ ] 当前无实现文件时保持"待实现/待评审"状态（per §7 第 7 条）—— 本计划实施前 `Test-Path crates/cluster-ops/src/realm_lifecycle` = False，实施后 = True 且 ≥ 1 个非空 .rs 文件

### 5.2 实测参数回填（per SPEC §8）

| 参数 | 目标 | 实测位置 |
|---|---|---|
| 合服回退窗口期 | 7~30 天 | M-2070.7 |
| 退场后归档启动阈值 | 30~90 天 | M-2070.8 |
| 归档冷热分层阈值 | 3 年热 + 10 年冷 | M-2074.1 |
| 6 阶段 OLU 估算默认值 | TBD-LCM-007 → PH-4 实测填 | M-2071.4 |
| 演练剧本模板 | 5 类各通过 1 次 | M-2070.3~#2070.10 |
| Saga 步骤超时 | 60s | M-2067.5 |

### 5.3 必须 grep 的 5 处代码评审检查

```bash
# 1. FR-LCM-004: RealmLifecycleService 不对外暴露独立接口
# （只能通过 admin-service 转发，cluster-ops/src/lib.rs 不得 re-export 独立 gRPC service）
Select-String -Path crates/rgs-cluster-ops/src/lib.rs -Pattern "realm_lifecycle.*Service.*tonic::include_proto" -List
# 期望：空

# 2. FR-LCM-003: DrillExecutor 仅跑沙箱环境
# （drill/executor.rs 不得引用生产 PG / 生产 K8s client）
Select-String -Path crates/rgs-cluster-ops/src/realm_lifecycle/drill/executor.rs -Pattern "admin_db|player_db|cluster_ops_db" -List
# 期望：仅 sandbox_pg.rs 引用 sandbox_*

# 3. NFR-LCM-007: OLU 必经 rgs-arc-olu
Select-String -Path crates/rgs-cluster-ops/src/realm_lifecycle/olu_reporter.rs -Pattern "rgs_arc_olu|olu_reporter" -List
# 期望：≥ 1 处

# 4. FR-LCM-062: merge_conflict_rule_set_v2 锁定后不可改
Select-String -Path crates/rgs-cluster-ops/src/realm_lifecycle/plans/merge_conflict_rule_set_v2.rs -Pattern "locked_at.*Option<DateTime>|check_locked" -List
# 期望：≥ 1 处

# 5. FR-LCM-081: 归档不删除数据
Select-String -Path crates/rgs-cluster-ops/src/realm_lifecycle/operations/archive.rs -Pattern "DELETE FROM|DROP TABLE|truncate" -List
# 期望：空（不删数据，只迁移存储位置）
```

---

## 6. 风险 & 缓解

| # | 风险 | 等级 | 缓解 |
|---|---|---|---|
| R1 | 跨 DB Saga 协调（player + economy + social 同时迁移数据）触发 Q-003 长事务 | **高** | 复用 `rgs-economy-service::saga_orchestrator` 既有 `apply_atomic_with_reservation` 模式（per WF-1-55.27 真修）+ Q-003 决策包（per RGS-DEC-Q003 v0.1 计划中）|
| R2 | admin_db 6 张新表 DDL 在生产环境 migration 失败 | **高** | Expand-Contract 双向演练（per M-2068.5）；先在沙箱 admin_db 跑 1 周再上生产 |
| R3 | 演练环境（沙箱 PG 池 + 沙箱 K8s）状态与生产不一致导致演练通过但生产失败 | 中 | M-2070.1 沙箱数据每日从生产 snapshot 同步（仅结构 + 采样数据，不含 PII）|
| R4 | 阶段变更高密度期间 OLU 预算超限（per RSK-LCM-006）| 中 | olu_reporter 串行调度（避免并发击穿）；M-2071.4 实测 6 阶段 OLU 默认值 |
| R5 | 退场后 RBAC 查询通道（仅 `cs_agent`/`sre`/`legal`）被绕过 | 中 | M-2073.4 + SPEC §6 ST 覆盖 Security 100% 命中 |
| R6 | 归档冷热分层阈值 3 年热 + 10 年冷 误判（业务量变化）| 低 | M-2074.1 + RSK-LCM-005 N+2 冗余兜底 |
| R7 | GDPR "被遗忘权" 删除通路双层审计写入失败 | 中 | M-2074.4 + admin_db.operation_audit 既有双层写入（per NFR-SE-010 既有）|
| R8 | 6 阶段操作器并发导致 PFAU 编排冲突 | 中 | ClusterOpsService 既有 PFAU 5 状态机 + 7 子类注册（per M-2071.3）|
| R9 | 业务 service gRPC 失败导致 Saga 反向补偿不完整 | 中 | 复用 economy::saga_orchestrator SagaStep 失败处理（per M-2067.3）|
| R10 | one-person company 兼任下，Admin 域 Lead = 架构师 = DBA 兼任导致 RACI A 角色被自我审查 | **高** | per RGS-ADR-0055 §4.3，LCM 决策 A 必须 Ulysses 显式 `/sign`（不能 PR review 顶替）；LCM 涉及"资金迁移"+"GDPR 合规"两类，属 §4.3 关键标注，必须独立签字 |

---

## 7. 回滚策略

### 7.1 应用回滚

`RealmLifecycleService` **不**是必选路径（v0 状态 = 无 LCM，6 阶段操作器全部 disable）。如 LCM 在生产环境出现回归：

1. `ClusterOpsService` 移除 `FeatureType::RealmLifecycle` 7 子类注册（disable 所有 realm_lifecycle Feature）
2. `AdminService` 移除 LCM 转发路由
3. 已运行的 LCM run 进入 rollback Saga（per M-2067.3 反向补偿）
4. 监控：AC-LCM-001~010 任意 1 项失败 → 自动 disable + 告警

### 7.2 数据回滚

- **6 张新表 DDL 回滚** = Expand-Contract 第 2 阶段（per M-2068.5）：
  - DROP COLUMN（非破坏性，向后兼容）
  - DROP INDEX
  - DROP TABLE（最后手段；需先备份 admin_db 全量）
- **生产 admin_db 备份** = 7 天保留（per 既有 `pg_dump` 调度）
- **归档数据回滚** = 冷热分层迁移日志（per `archive_policy` 表）反向操作

### 7.3 配置回滚

- 6 阶段 OLU 默认值（TBD-LCM-007）保守估算：每个阶段 0.5 SRE 半容量 = 250K tokens
- Saga 步骤超时 60s（per M-2067.5）— 短超时有助于快速失败
- 演练剧本：5 类各通过 1 次即停止（避免演练本身消耗预算）

---

## 8. 一人公司 RACI 简表（per RGS-ADR-0055 §4 + §4.3 关键标注）

| 决策 | R（执行）| A（最终批准）| C（咨询）| I（知会）|
|---|---|---|---|---|
| **代码合并**（#2066~#2068）| AI worker 子代理 | **Ulysses（Admin 域 Lead 兼）** PR merge | CI 4 workflow + OTel | 全员 |
| **DTL 升版**（SPEC-DTL-042 v0.2→v0.3）| AI worker | **Ulysses** 显式签字 | 5 域 Lead 兼 | 全员 |
| **PH-3 数据库 migration**（#2068，admin_db 6 张新表）| AI worker | **Ulysses（DBA 兼 + Admin 域 Lead 兼）** 显式 `/sign` | SRE 兼 + QA 兼 | 5 域 Lead 兼 + 评审主持兼 |
| **PH-4 演练实测启动**（#2070）| AI worker | **Ulysses（SRE 兼）** 显式 `/sign` | Admin 域 Lead 兼 | 全员 |
| **PH-4 Feature 集成**（#2071，7 子类注册）| AI worker | **Ulysses（Admin 域 Lead 兼）** 显式签字 | 架构师兼 + SRE 兼 | 全员 |
| **PH-5 跨域联动**（#2073）| AI worker | **Ulysses（Admin 域 Lead 兼）** 显式签字 | 5 域 Lead 兼 + 评审主持兼 | 全员 |
| **PH-6 GDPR 归档**（#2074，**资金/合规相关**）| AI worker | **Ulysses（PM 兼）** 显式 `/sign` + 单独审批 + 审计 channel | DBA 兼 + 法务兼 + 评审主持兼 | 全员（审计 channel 推 Slack）|

> **§4.3 关键标注应用**：PH-6 决策属"资金/合规相关"（GDPR 删除涉及监管合规），A 必须 Ulysses 显式独立签字动作，**不能**用 PR review 合并顶替。

---

## 9. 关联文档

- 上行：
  - [SPEC-DTL-042 实现规格书 v0.2](../13-实现规格/RGS-SPEC-DTL-042_实现规格书.md)
  - [RGS-DTL-042 详细设计](../01-核心架构与设计模式/)
  - [ARC-038 服区边界原子迁移（服务器全生命周期治理扩展）](../00-基本与治理/)
  - [ARC-051 `realm_lifecycle` Feature 类型扩展](../00-基本与治理/)
  - [RGS-DTL-031 §1.1 Feature 类型扩（已有 5 类，新增 `realm_lifecycle`）](../01-核心架构与设计模式/RGS-DTL-031_集群运营管理_每功能原子升级_详细设计.md)
  - [RGS-DTL-100 Saga 业务模式设计](../01-核心架构与设计模式/)
  - [RGS-DTL-101 OperationPolicy 与 AuthorityBoundary](../01-核心架构与设计模式/)
  - [RGS-DEC-Q003 跨 DB Saga 审批包 v0.1（计划中，per WF-1-55.43）](../00-基本与治理/)
  - [RGS-ADR-0015 跨域 Saga 适用边界与单一调解者原则](../08-架构决策记录/RGS-ADR-0015_跨域Saga适用边界与单一调解者原则.md)
  - [WBS-001 §16.2 L4 #2066/#2067/#2068/#2070/#2071/#2073/#2074](RGS-WBS-001_瀑布式工作分解结构_v0.3.md)
  - [RGS-PLAN-001 v1.1 项目实施计划](RGS-PLAN-001_项目实施计划_v1.0.md)
  - [RGS-TS-001 v0.6 §6.2 token-OLU](../10-技术选型/RGS-TS-001_主要技术选型报告.md)
  - [RGS-ADR-0055 v0.1 DEC-005/008 兼容论证 + RACI 简表](../08-架构决策记录/RGS-ADR-0055_DEC-005_008_兼容论证_v0.1.md)
  - [RGS-IMPL-001 实施约定与工程边界](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md)
  - [RGS-SPEC-CROSS-007 5 域 RBAC 角色矩阵](../13-实现规格/RGS-SPEC-CROSS-007_5域RBAC角色矩阵_v0.1.md)
  - [RGS-ANTIPATTERN-001 孤儿 SPEC 自查清单 v0.1](RGS-ANTIPATTERN-001_孤儿SPEC自查清单_v0.1.md)
- 下行：
  - `crates/rgs-cluster-ops/src/realm_lifecycle/README.md`（实施时新建）
  - `docs/deploy/lcm-drill-report.md` + `lcm-it-report.md` + `lcm-st-report.md`（实测时新建）
  - `migrations/0020_lcm_tables.sql` + `0020_lcm_expand_contract.sh`（实施时新建）

---

## 10. 审批栏（per DEC-008 一人公司 12 角色兼任）

| # | 角色 | 姓名 | 审批日 | 结论/条件 |
|---|---|---|---|---|
| 1 | **Admin 域 Lead**（独立位）| **Ulysses**（Admin 域 Lead 兼 per DEC-008）| _pending_ | ✅ 草案待签字 |
| 2 | 架构负责人 | **Ulysses**（架构师兼 per DEC-008）| _pending_ | ✅ 草案待签字 |
| 3 | **DBA Lead** | **Ulysses**（DBA 兼 per DEC-008）| _pending_ | ✅ 草案待签字（admin_db 6 张新表 migration 责任）|
| 4 | SRE Lead | **Ulysses**（SRE 兼 per DEC-008）| _pending_ | ✅ 草案待签字（沙箱 PG 池 + 沙箱 K8s 客户端 + 演练）|
| 5 | QA Lead | **Ulysses**（QA 兼 per DEC-008）| _pending_ | ✅ 草案待签字（AC-LCM-001~010 全部 10 项 + NFR-LCM-001/004/006 3 项）|
| 6 | **法务/合规** | **Ulysses**（法务兼 per DEC-008）| _pending_ | ✅ 草案待签字（GDPR 删除通路 + N+2 冗余 + RSK-LCM-005 缓解）|
| 7 | 评审主持人 | **Ulysses**（评审主持兼 per DEC-008）| _pending_ | ✅ 草案待签字（per §8 RACI A 角色显式签字要求）|
| 8 | PM/项目负责人 | **Ulysses**（PM 兼 per DEC-008）| _pending_ | ✅ 草案待签字（含 token-OLU 估算批准 + §3.8 NFR-OP-010 超 1.5 周风险确认）|

> **本计划升 v0.2 条件**：8 角色签字 + 1 次 worktree 试跑（M-2066.1~M-2066.3 + M-2068.1 完成 + 编译通过） + WBS L4 #2066/#2068 状态由 pending → in_progress

---

> **本计划是 living document**。每次 M 任务完成后，在 §3 各表追加 `commit hash` + `实测数据` 链接；OLU 估算与实测偏差 > 30% 时在 §3.8 加修正行；任何 RACI A 角色决策的实际签字留痕必须回填到 §8。

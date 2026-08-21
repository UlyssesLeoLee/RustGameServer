# 项目实施计划（Implementation Plan）

**RustGameServer First Slice：五域 Atomic App + CEM/PFAU + 插件/集群联动**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-PLAN-001 |
| 版本 | 0.4 |
| 状态 | **开发前就绪计划・Gate 未闭合・NO-GO（禁止业务编码、迁移与部署）** |
| 依据 | DEC-001～004、RGS-QA-001 v0.7、RGS-ADR-0052、RGS-DTL-031 v0.2、RGS-SPEC-000、RGS-IMPL-001、RGS-REV-003、RGS-ENV-001 |
| 范围 | player / economy / match / social / admin 五域；ARC-018/021/042/051 |
| 计划窗口 | 8～12 周规划假设，须以 Gate、OLU 和演练证据校准 |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 + 项目负责人 |

> 本计划只定义依赖、交付物和验收门槛。它不替代 RGS-QA-001 的具名审批，不把 AI 估算、RPO/RTO、OLU 改善或 8～12 周窗口写成已批准承诺。v0.2 将全部 36 份 DTL 对应的 SPEC 绑定为实施输入；在 §3.3 的全部 `G-CODE-*` 门禁关闭前，本计划不是编码、数据库迁移、集群部署或排期承诺的授权。

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 |
|---|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | — | 首版草案。采用“全域契约先行、单域纵向实现”，把 workspace、cluster manifest、插件边界和 Gate 放到首周。 |
| 0.2 | 2026-08-21 | 架构师 | — | 绑定 RGS-SPEC-000 与 36 份子 SPEC；新增开发前 Go/No-Go 门禁、SPEC 变更追踪、当前工具链差距和授权证据清单。 |
| 0.3 | 2026-08-21 | 架构师 | — | 绑定 RGS-IMPL-001，收敛 Q-101～Q-405 的工程答案；将 Q-003/Q-025 从“缺少方案”改为“方案已定、待具名 Gate/证据”。Rust 1.98 stable 为用户目标，GA 前 Gate 保持 Open。 |
| 0.4 | 2026-08-21 | 架构师 + PM | — | 同步 handoff §5 Step 1-2 进展：① 升级 RGS-QA-001 v0.7 引用（Q-021 治理闭环落地 + Q-027 文档版本同步 + Q-031 WBS 主题重定义）② DTL-031 v0.1 → v0.2 ③ 新增 §3.4 RGS-REV-003 联合评审组织 + §3.5 RGS-ENV-001 环境核验 ④ 审批栏扩 5 域 Lead + SRE + DBA + Platform Engineer。**本计划不把 v0.3→v0.4 升版当作取消 53 NO-GO；NO-GO 仍由 §3.3 G-CODE 全部签字关闭后解除。** |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 结论/条件 |
|---|---|---|---|
| 架构负责人 | 待指定 | — | 确认 ARC 组合、Q-003/Q-004 与 DTL-031 |
| SRE/运维负责人 | 待指定 | — | 确认 Active-Active、all-reachable、演练和 OLU |
| 安全/DBA 负责人 | 待指定 | — | 确认 DB 隔离、Saga 补偿、凭证和审计 |
| QA 负责人 | 待指定 | — | 确认 SPEC 验收项、testkit 前置与测试证据路径 |
| 平台负责人（Platform Engineer）| 待指定 | — | 确认 Rust 1.98 / Cargo.lock / 镜像构建链路 |
| 5 域 Lead（player / economy / match / social / admin）| 待指定 | — | 确认各自 DTL 字段级 Review、testkit 责任、依赖矩阵签字 |
| 评审主持人（RGS-REV-003）| 架构师（兼任）| — | 主持联合评审流程与异议闭环 |
| 项目负责人 | 待指定 | — | 确认范围、风险接受、资源和实施授权 |

> **v0.4 扩列说明**：原 5 类签字方扩为 9 类（5 域 Lead 单列 + 平台工程师 + 评审主持人单列）。具体责任矩阵见 [RGS-REV-003 §3](../00-基准与治理/reviews/RGS-REV-003_联合评审_Q003-Q025-ADR0052-5域DTL.md) 与 [RGS-REV-006 附件C](../00-基准与治理/reviews/RGS-REV-006_附件C_责任矩阵与签字模板.md)。

---

# 1. 目标与不可变约束

## 1.1 目标

首个端到端切片必须同时证明：

1. 五个领域 App 可按 ARC-018 独立构建、部署、健康检查和回滚。
2. 集群由 ARC-042 声明式 manifest 构造，执行前完成 DAG 校验；基础设施先于业务域，同层才允许并行。
3. ARC-021 插件按宿主 App 管理，支持安全点启停/热重载/回滚，但不独立拥有 DB，不加载动态库。
4. ARC-051 的 Feature、CEM、PFAU 统一进入 ClusterOpsService 控制面。
5. ClusterOpsService 双副本可在幂等、OCC、租约 fencing 下安全处理并发命令。
6. 每一项实现工作均能从“需求/ADR → DTL → SPEC → 代码、测试与运行证据”反查；SPEC 不替代源 DTL 的字段、状态机、错误码、SQL/proto 或非目标。

## 1.2 不可变约束

- 未完成 RGS-QA-001 四类 Gate 前，只进行文档、契约、原型和测试设计；不提交业务实现。
- 五域 DTL 的接口/边界/依赖契约必须先冻结；禁止以 player 域代码反向定义全局边界。
- `ClusterOpsService` 不协调业务跨 DB 事务；Q-003 未审批前，economy 不进入跨 DB 写流程。
- COC UI 不直连 ClusterOpsService、K8s、Helm 或 DB；所有写操作经 AdminService。
- 任何 Agent 能力、RPO/RTO、OLU 减少都必须有实测证据后才能进入基线。
- 新建或修改任一实现单元前，必须登记其源 DTL、源 SPEC、owner、版本、验收项与回滚路径；源 DTL/SPEC 未同步评审的变更不得进入实现分支。
- §3.3 任一 `G-CODE-*` 未关闭时，只允许文档修订、评审、签署、环境核验与测试设计；禁止提交业务 Rust 代码、数据库 migration、Kubernetes/Helm manifest 或生产配置。

---

# 2. 交付物分层

| 层 | 交付物 | 退出条件 |
|---|---|---|
| 治理 | RGS-ADR-0052、RGS-DTL-031、RGS-PLAN-001、Q-003/Q-004/Q-015/Q-016/Q-025 审批包 | 具名审批或带条件风险接受；适用 `G-CODE-*` 均关闭 |
| 实现规格 | [RGS-SPEC-000](../13-实现规格/RGS-SPEC-000_详细设计规格化总表.md) 与全部 36 份 RGS-SPEC-DTL-* | DTL↔SPEC 一对一、源文档可追溯、SPEC DoD/未决项已纳入计划和测试证据 |
| 契约 | RGS-DTL-036～040、protobuf/event/error/ID 契约 | 五域接口、DB、插件和依赖矩阵冻结 |
| 工程骨架 | virtual Cargo workspace、按域 `rgs-contracts-*`、`rgs-testkit`、manifest validator | `cargo fmt/check/clippy/test` 与 DAG 负例通过 |
| 集群骨架 | foundation Apps、五域空壳、Helm/GitOps/NetworkPolicy、独立 DB | dry-run、部署、续跑、逆拓扑回滚通过 |
| 控制面 | AdminService 转发、ClusterOpsService、CEM/PFAU | request_id/OCC/fencing/all-reachable 集成测试通过 |
| 业务切片 | player 首条路径；economy/match/social/admin 契约接入 | 端到端业务路径、审计和回滚通过 |
| 质量与运维 | chaos、容量、OLU、RPO/RTO、供应链和发布证据 | 证据包完成，负责人签署 |

## 2.1 DTL/SPEC 绑定规则

| 实施范围 | 必须使用的规格 | 计划约束 |
|---|---|---|
| 全部详细设计 | [RGS-SPEC-000 §4](../13-实现规格/RGS-SPEC-000_详细设计规格化总表.md#4-全部详细设计映射) 与对应的 RGS-SPEC-DTL-* | 36 份 DTL 必须各有且仅有一个同号 SPEC；SPEC 不得脱离源 DTL 单独解释字段或状态机。 |
| 开发前核心 | [RGS-SPEC-DTL-004](../13-实现规格/RGS-SPEC-DTL-004_实现规格书.md)、[RGS-SPEC-DTL-005](../13-实现规格/RGS-SPEC-DTL-005_实现规格书.md)、[RGS-SPEC-DTL-031](../13-实现规格/RGS-SPEC-DTL-031_实现规格书.md) | 先确认可观测性 façade、插件生命周期和 ClusterOps/PFAU 契约；三者任一未满足 DoD 不得以业务代码绕过。 |
| First Slice 五域 | [RGS-SPEC-DTL-036～040](../13-实现规格/RGS-SPEC-000_详细设计规格化总表.md#4-全部详细设计映射) | 五域独立 App/DB/契约/插件宿主关系必须先冻结；任何跨域交互必须映射到 API、event、Outbox 或 workflow。 |

SPEC 版本、源 DTL 版本、关联 ADR/QA、实现分支、测试证据和部署制品必须同时登记到追踪矩阵。若任一源 DTL 的字段、状态机、错误码、SQL/proto 或非目标变更，则关联 SPEC 回到评审状态，已生成的实现证据失效直至重新对账。

---

# 3. 依赖关系与阶段计划

## 3.1 阶段表

| 阶段 | 规划窗口 | 主要工作 | 前置 | 阶段出口 |
|---|---:|---|---|---|
| PH-0 Gate、设计与 SPEC 冻结 | 第 1 周 | DTL-031；PLAN-001 v0.2；Q-003/Q-004/Q-015/Q-016/Q-025；ADR-0052；五域 DTL/SPEC 契约评审 | 无 | 形成 §3.3 开发前 Go/No-Go 证据包；不允许创建实现分支 |
| PH-0.5 开发前授权评审 | PH-0 后 | 核对全部 `G-CODE-*`、审批栏、环境核验记录、追踪矩阵和风险接受 | PH-0 | 全部门禁关闭后，项目负责人书面授权进入 PH-1；未授权即 NO-GO |
| PH-1 工程基础 | 第 2 周 | Cargo workspace、按域 contracts、`rgs-testkit`、CI 基线、manifest schema/DAG validator | PH-0.5 书面授权 | 负例测试全通过，五域均可登记 |
| PH-2 集群基础 | 第 3 周 | gateway/event-bus/config/observability/secrets；五域空壳；AdminService/ClusterOpsService health | PH-1 | 开发环境 dry-run 与独立 DB 开通通过 |
| PH-3 控制面 | 第 4～5 周 | Feature registry、CEM、PFAU、AdminService 转发、OCC/fencing、all-reachable | PH-2 | 单节点故障可暂停/回滚；不自动跳过 |
| PH-4 第一业务切片 | 第 5～7 周 | player 端到端；economy 仅实现已批准的 Saga 契约；其余域完成契约接入 | PH-3；Q-003 | 五域 manifest 一致，player 路径可重复部署 |
| PH-5 五域联调 | 第 7～9 周 | economy/match/social/admin 业务路径、事件、插件隔离与回滚 | PH-4 | 领域集成测试与数据隔离通过 |
| PH-6 故障/容量/运维 | 第 9～11 周 | Active-Active、跨 AZ、network partition、100k CCU 计划、OLU、灾备 | PH-5 | 证据包满足验收矩阵 |
| PH-7 发布 Gate | 第 11～12 周 | 供应链、许可证、发布/回滚、RPO/RTO、最终签署 | PH-6 | 仅在负责人签署后进入目标环境 |

以上窗口是排期假设；任一阶段出口未通过时，后续阶段不自动顺延为“已开始”。

## 3.2 关键 DAG

```text
Gate approvals
  -> DTL/PLAN/SPEC/5-domain contract freeze
  -> pre-code Go/No-Go authorization
  -> Cargo workspace + testkit + manifest validator
  -> foundation Apps
  -> admin-service + cluster-ops-service
  -> five-domain App shells + independent DBs
  -> Feature registry/CEM/PFAU integration
  -> player vertical slice
  -> economy/match/social/admin domain paths
  -> chaos/capacity/OLU/RPO-RTO evidence
  -> release approval
```

## 3.3 开发前 Go/No-Go 门禁与证据

本表是进入 PH-1 前唯一的编码授权清单。状态为 `Open`、`Blocked` 或 `N/A 未具名接受` 时，结论一律为 NO-GO；不得用计划日期、AI 候选结论或“代码先写起来”替代证据。

| ID | 必须关闭的门禁 | 当前状态（2026-08-21 v0.4） | 关闭证据 | 责任人 | 评审 checklist |
|---|---|---|---|---|---|
| G-CODE-01 | 36 份 DTL 与 36 份 SPEC 一对一，目录登记、链接和交叉引用有效 | 🟣 机械校验已通过；待 DD 具名评审 | RGS-SPEC-000 映射、`verify_docs.py`、交叉引用检查、DD 记录 | 架构负责人 + QA 负责人 | [REV-003](../00-基准与治理/reviews/RGS-REV-003_联合评审_Q003-Q025-ADR0052-5域DTL.md) §2.4 |
| G-CODE-02 | RGS-DTL-031 与 Q-025 完成字段级 DD Review | 🟠 **Open / Blocker**（DTL-031 v0.2 已存在 21 KB） | 接口、状态机、fencing、CEM/PFAU、测试映射和审批栏具名签署 | 架构负责人 + 平台负责人 + DBA | [REV-004 附件A](../00-基准与治理/reviews/RGS-REV-004_附件A_5域DTL字段级ReviewChecklist.md) §A.6 |
| G-CODE-03 | RGS-ADR-0052 的 all-reachable 与 Active-Active 规则获具名批准 | 🟠 **Open**（ADR-0052 已起草 5.7 KB） | ADR 审批栏、目标拓扑核验、故障注入计划与风险接受 | 架构负责人 + SRE 负责人 | REV-003 §2.3 + ADR-0052 联审 |
| G-CODE-04 | Q-003 跨 DB Saga 与 Q-004 原子组合完成具名决策 | 🟠 **Open / Q-003 Blocker**（技术方案已固定在 RGS-IMPL-001 §3 + RGS-QA-001 v0.7） | Saga/Outbox/补偿边界、四层原子状态机合并图、6 个业务场景验收计划 | 架构负责人 + DBA + Economy 域 Lead | [REV-005 附件B](../00-基准与治理/reviews/RGS-REV-005_附件B_Saga演练场景Checklist.md) 6 场景 |
| G-CODE-05 | RGS-DTL-036～040 及其 SPEC 的五域边界、依赖和 App/DB/Plugin 宿主关系冻结 | 🟠 **Open**：工程目录/依赖规则已定义，DD Review 未签署 | 五域 DD Review、接口/事件/DB/插件依赖矩阵、反向依赖检查 | 5 域 Lead + 架构负责人 | REV-004 附件A §A.2-A.6 |
| G-CODE-06 | 工具链与开发环境达到目标基线 | 🟠 **Open**：Rust 1.98 stable GA 已发 (2026-08-20) ✅；待"可安装 + 完整 CI 通过"实测 | Rust 1.98 实测、Actix Web 4.14.1 锁定、PostgreSQL 18.4 migration 演练、K3s 能力核验、锁定依赖 CI | 平台负责人 + DBA + SRE | [RGS-ENV-001](../00-基准与治理/reviews/RGS-ENV-001_环境核验记录模板.md) 全部签字 |
| G-CODE-07 | OLU 与测试基础前置获批准 | 🟠 **Open**：Q-015 待具名审批；Q-031 WBS 候选答案 v0.7 起草 | OLU 重算、Q-031 5 层 WBS 实施、`crates/testkit` 范围/复用指标 | SRE 负责人 + QA 负责人 + PM | REV-003 §3 + RGS-PLAN-001 v0.4 |

**当前结论：NO-GO。** v0.4 同步了 handoff §5 Step 1-2 进展（评审草稿 + 环境核验模板就绪），但 7 个 G-CODE-* 仍 **Open / Blocker**。解除 NO-GO 条件：

1. RGS-REV-003 §7.3 全部 7 类签字栏签署（架构师 + 5 域 Lead + Platform + DBA + SRE + QA + PM）
2. RGS-ENV-001 §6 全部 5 类签字栏签署（Platform + DBA + SRE + 架构师 + PM）
3. 7 个 G-CODE 全部 "🟢 Closed" 状态

**3 项全部满足后**，§3.3 状态由 NO-GO 切到 GO，PM 可按 handoff §5 Step 4 启动 53。

---

## 3.4 联合评审组织（per handoff §5 Step 1）

53 启动前置的 7 个 G-CODE 中，6 个依赖**人审签字**（G-CODE-01/02/03/04/05/07），1 个依赖**环境实测**（G-CODE-06）。联合评审是签字流程的组织载体。

### §3.4.1 评审工具集

| 文档 | 用途 | 路径 |
|---|---|---|
| [RGS-REV-003](../00-基准与治理/reviews/RGS-REV-003_联合评审_Q003-Q025-ADR0052-5域DTL.md) | 联合评审主文（agenda + 责任矩阵 + 签字栏）| `docs/00-基准与治理/reviews/` |
| [RGS-REV-004 附件A](../00-基准与治理/reviews/RGS-REV-004_附件A_5域DTL字段级ReviewChecklist.md) | 5 域 DTL 字段级 Review Checklist（14 通用 + 5 域特定 + 跨域一致性） | 同上 |
| [RGS-REV-005 附件B](../00-基准与治理/reviews/RGS-REV-005_附件B_Saga演练场景Checklist.md) | G-CODE-04 Saga 演练场景（6 场景：正常 / 补偿 / 超时 / 人工升级 / 去重 / PFAU+Saga） | 同上 |
| [RGS-REV-006 附件C](../00-基准与治理/reviews/RGS-REV-006_附件C_责任矩阵与签字模板.md) | 完整 RACI 矩阵 + 签字流程（按依赖顺序，不可跳签）| 同上 |
| [签字提案邮件](../00-基准与治理/reviews/签字提案邮件_模板.md) | Step 3 沟通：评审启动邮件 | 同上 |
| [评审会议议程通知](../00-基准与治理/reviews/评审会议议程通知_模板.md) | Step 3 沟通：阶段 2 现场会议通知 | 同上 |

### §3.4.2 评审流程（12 天硬上限）

| 阶段 | 时长 | 活动 | 责任方 |
|---|---|---|---|
| **阶段 1 预读** | 第 0-3 天 | 责任人阅读 RGS-REV-003 + 3 附件 + 关联 DTL/SPEC/ADR | 5 域 Lead + 架构 + SRE + DBA + Platform + QA + PM |
| **阶段 2 会议** | 第 5 天 14:00-16:00 | 现场/视频会议，2 小时硬上限 | 架构师主持 |
| **阶段 3 闭环** | 第 5-12 天 | 异议以文档/ADR 修订闭环 | 各责任人 |
| **签字** | 第 12 天 23:59 截止 | 按 REV-006 附件 C §C.2.2 顺序签字 | 全 9 类责任人 |

### §3.4.3 签字顺序（不可跳签）

DBA → SRE → 5 域 Lead → 架构师 → Economy 域 Lead（Q-003） → Platform → PM

### §3.4.4 异议处理

- 🔴 Blocker：评审后 3 天内闭环
- 🟠 重要：7 天内闭环
- 🟡 应当：14 天内闭环
- 🟢 Nice：Phase 1 内闭环，不阻塞 53
- 闭环方式：A 文档修订 / B ADR 修订 / C 升级 NO-GO
- 第 2 轮未闭环 → 升级 NO-GO，53 不可启动

### §3.4.5 评审失败后果

按 handoff §1 + 本计划 §3.3，**任何 G-CODE 状态不是 "🟢 Closed" 都保持 NO-GO**。评审失败不构成"带条件进入实施"的口子。

---

## 3.5 环境核验（per handoff §5 Step 2）

环境核验是 G-CODE-06 关闭的前置。

### §3.5.1 核验工具

| 文档 | 用途 | 路径 |
|---|---|---|
| [RGS-ENV-001](../00-基准与治理/reviews/RGS-ENV-001_环境核验记录模板.md) | 环境核验 checklist（工具链 / PG 18.4 / K3s / 锁定依赖 / 跨工具集成）| `docs/00-基准与治理/reviews/` |

### §3.5.2 核验范围（5 层）

1. **工具链**：rustc/cargo 1.98 + clippy + rustfmt + sqlx-cli + cargo-deny/audit/llvm-cov
2. **PostgreSQL 18.4**：psql + 服务器连接 + 5 DB 划分 + sqlx 编译期 + migration 双向演练
3. **K3s / Kubernetes**：kubectl + 节点就绪 + CoreDNS/Traefik + Helm + 镜像仓库
4. **锁定依赖 CI**：`Cargo.lock` 入仓 + `--locked` 构建 + fmt/clippy/deny/audit/llvm-cov
5. **跨工具集成**：sqlx 编译期 + tonic gRPC + tracing + distroless 容器

### §3.5.3 签字路径

Platform Engineer（§1/§4/§5） → DBA（§2） → SRE Lead（§3） → 架构师（§5） → **PM**（总签字）

### §3.5.4 时效

核验通过后 **30 天内**必须启动 53；超时重新核验。

---

## 3.6 §3.4-§3.5 与 §3.3 G-CODE 的关系

```
G-CODE-01 ~ 05, 07 ── 依赖 ──> RGS-REV-003 联合评审
   ↓
G-CODE-06 ── 依赖 ──> RGS-ENV-001 环境核验
   ↓
§3.3 NO-GO 解锁 = (RGS-REV-003 7 类签字齐) AND (RGS-ENV-001 5 类签字齐) AND (7 G-CODE 全 Closed)
```

---

# 4. 五域边界基线

| 域 | App/DB | 依赖方向 | 首版职责 | 插件边界 |
|---|---|---|---|---|
| player | `player-service` / `player_db` | foundation | 账号、角色、会话 epoch、玩家状态 | 只经宿主 API；不得写 economy_db |
| economy | `economy-service` / `economy_db` | player 事件/契约 + foundation | 货币、道具、交易提交与补偿 | 永久事实必须走 `CommitTransaction`；Q-003 前不实现跨 DB 写 |
| match | `match-service` / `match_db` | player 状态契约 + foundation | 匹配队列、对局确认、评分结算 | 匹配规则可用受限插件；不得直接读 player_db |
| social | `social-service` / `social_db` | player 事件/契约 + foundation | 社交关系、治理、消息/活动 | 脚本只能调用白名单 API |
| admin | `admin-service` / `admin_db` | foundation；转发至 ClusterOpsService | GM/COC 统一入口、RBAC、审计 | 控制面 Feature，不承载业务域数据 |

`cluster-ops-service` 是 admin 限界上下文的控制面服务，不作为五域业务 App 的替代；它只持有控制面数据和状态。

---

# 5. 工程规范与 CI 门禁

## 5.1 版本与框架基线（2026-08-21 核验）

| 组件 | 基线 | 约束 |
|---|---|---|
| Rust | 1.98 stable（用户目标；GA 前不可验证） | workspace 使用 Edition 2024、resolver 3；`rust-toolchain.toml` 与根 `Cargo.lock` 固定已验证构建，升级需通过全量 CI；不得用 beta/nightly 绕过 Gate |
| HTTP 服务框架 | Actix Web 4.14.1 | 运行于 Tokio；五域 App/AdminService 的 HTTP ingress 统一使用 Actix Web，tonic/hyper 仅用于内部 RPC/底层协议 |
| PostgreSQL | 18.4 | 五个独立 DB；开发/预发/生产统一以 18.4 为基线，后续 18.x 补丁须经灰度与回退验证，PostgreSQL 19 在 GA 前不得进入生产基线 |

本基线以官方发布资料为准：Rust release announcements、PostgreSQL release notes、Actix Web crate documentation。版本“最新版”不等于跳过锁定、迁移、回滚与兼容性验证；Rust 1.98 在 GA 前只是用户目标，不得伪造为已验证 stable。

当前开发机实测为 `rustc/cargo 1.95.0`、`psql 15.3`，且尚未建立 Cargo workspace；与目标基线不一致。此事实记录对应 `G-CODE-06`，不构成“升级已完成”的承诺。PH-1 在获得 PH-0.5 书面授权且 Rust 1.98 stable GA 后，必须先由 CI/开发环境镜像完成 Rust 1.98、Actix Web 4.14.1 与 PostgreSQL 18.4 的工具链对齐；未对齐前不得宣称环境构建完成。

获 PH-0.5 书面授权后创建 virtual workspace：

```text
Cargo.toml                         # virtual workspace / resolver = "3"
Cargo.lock                         # 唯一根锁文件，必须入仓
proto/rgs/{domain}/v1/*.proto
crates/rgs-{player,economy,match,social,admin}/
crates/rgs-cluster-ops/
crates/rgs-contracts-{domain}/
crates/rgs-testkit/
services/rgs-cluster-ops-service/
services/player-service/
services/economy-service/
services/match-service/
services/social-service/
services/admin-service/
deploy/cluster-manifest/
```

完整的目录、错误、Saga、测试、配置、密钥和制品约定见 [RGS-IMPL-001](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md)。最低 CI 阶段：

1. `cargo fmt --check`；
2. `cargo clippy --all-targets --all-features -- -D warnings`（不全局启用 `clippy::pedantic`）；
3. 单元测试、契约测试、DAG 负例和插件隔离测试；
4. migrations 向前迁移/回滚演练；
5. Helm lint/render/dry-run；
6. `cargo deny`、`cargo audit`、`cargo llvm-cov` 与许可证/供应链门禁；
7. 文档交叉引用与 manifest/Feature registry 登记一致性校验。

实际工具版本须在 workspace 建立时锁定并写入 CI，不以本计划中的命令替代版本评审。

---

# 6. 风险与决策账本

| ID | 风险/未决 | 处理时点 | 阻断条件 |
|---|---|---|---|
| Q-003 | 跨 5 DB Saga、补偿与延迟上限 | PH-0 | 技术方案已定为 Saga + Outbox + 补偿；未获具名批准则 economy 跨 DB 写禁止 |
| Q-004 | ARC-018/021/042/051 组合矩阵 | PH-0 | 未批准则 Feature/App 映射不冻结 |
| Q-015 | OLU 重新核算 | PH-0/每周 | 超过 2 SRE 上限则暂停范围扩张 |
| Q-016 | `crates/testkit` 共用骨架 | PH-1 | 未通过则五域并行开发禁止 |
| Q-025 | DTL-031 字段级 DD Review 与审批窗口 | PH-0 | 设计已完成，未完成具名 DD Review 则 ClusterOpsService 代码禁止 |
| Q-036 | 五域 DTL 同步起草与可视化 | PH-0/每周 | 任一域契约滞后阻断纵向切片扩展 |
| GATE-SPEC-001 | 源 DTL、SPEC 与实现证据发生漂移 | 持续 | 未完成 DTL/SPEC 同步评审、追踪矩阵回填和回归测试前禁止合并 |
| GATE-ENV-001 | Rust/PostgreSQL 实际版本低于目标基线，Rust 1.98 尚未 GA，且 workspace 未建立 | PH-0.5/PH-1 | `G-CODE-06` 未关闭前禁止声明开发环境或 CI 已构建完成 |
| RSK-DEP-001 | manifest 与挂载脚手架脱节 | PH-1 起 | CI 必须阻断未登记 App/Feature |
| OLU-001 | Agent 缓解收益未验证 | 每周 CIR | 不得计入可用工时预算 |

---

# 7. 每周出口检查

每周结束前由架构、平台、SRE、QA、项目负责人共同更新：

- 文档版本与审批状态；
- DTL/SPEC 版本、映射完整性、变更影响和失效的实现/测试证据；
- `G-CODE-*` 状态、具名审批、风险接受和 Go/No-Go 结论；
- 五域契约差异和 manifest 变更；
- 已通过/未通过测试证据；
- DB migration、插件制品和 Helm 制品摘要；
- OLU 实际工时与剩余容量；
- 新增 TBD/风险、回滚条件和下周唯一主路径。

任何“已完成”必须有可复核产物；“代码已存在”不等于“Gate 已通过”。

---

# 8. 计划验收标准

| ID | 验收标准 |
|---|---|
| AC-PLAN-001 | 五域 App、foundation App、依赖关系和插件宿主关系可从同一 manifest 重建 |
| AC-PLAN-002 | manifest 环依赖或缺 foundation 祖先时在执行前失败，不触发 Helm |
| AC-PLAN-003 | 同一 cluster_id 同时只有一个 RUNNING 编排；失败可续跑，成功项不重复执行 |
| AC-PLAN-004 | 单节点失联使 PFAU 暂停或按明确规则回滚，不自动跳过版本不兼容 |
| AC-PLAN-005 | 插件异常、脚本超限或禁用不影响宿主进程与其他 Feature |
| AC-PLAN-006 | 五域跨边界调用均经 gRPC/event contract，无跨 DB 直连 |
| AC-PLAN-007 | 8～12 周、OLU、RPO/RTO 结论均有真实测量和具名签署，不能由计划文字推定 |
| AC-PLAN-008 | 任一实现单元均可由 RGS-SPEC-000 反查源 DTL、SPEC、ADR/QA、owner、测试证据和回滚路径；DTL/SPEC 变更会使旧证据失效并触发复核 |
| AC-PLAN-009 | 进入 PH-1 前，§3.3 全部 `G-CODE-*` 有关闭证据和具名授权；否则计划状态保持 NO-GO |

---

# 9. 开发前授权记录

| 检查项 | 结论 | 证据/备注 |
|---|---|---|
| SPEC 规格包 | 已绑定，未获 DD 实施授权 | [RGS-SPEC-000](../13-实现规格/RGS-SPEC-000_详细设计规格化总表.md)；36 份 DTL↔SPEC 机械映射有效 |
| 架构与详细设计 | 未获具名授权 | RGS-ADR-0052、RGS-DTL-031、五域 DTL 均须按 §3.3 关闭相应门禁 |
| 事务与原子组合 | 未获具名授权 | Q-003 为 Blocker；Q-004 未决；不得以实现代码替代架构决策 |
| 工具链与环境 | 未就绪 | 当前 Rust/Cargo 1.95.0、PostgreSQL 客户端 15.3；目标为 Rust 1.98 stable（GA 前不可核验）、Actix Web 4.14.1、PostgreSQL 18.4 |
| 编码授权 | **NO-GO** | 项目负责人只能在 §3.3 门禁全部关闭后填写具名授权；本表不得由 AI 或未具名记录代签 |

本版本完成实施编程之前的计划基线：实现范围、SPEC 追踪、门禁、证据和责任归属已经明确。下一项工作是收集并核验 `G-CODE-*` 的关闭证据，而不是开始 Rust、SQL 或部署代码。

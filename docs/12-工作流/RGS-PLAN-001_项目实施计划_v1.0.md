# 项目实施计划（Implementation Plan）v1.0

**RustGameServer First Slice：五域 Atomic App + CEM/PFAU + 插件/集群联动**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-PLAN-001 |
| 版本 | **1.0**（v0.9 → v1.0：**Phase 0.5 实质完成** + 4 B-CODE 全部 Closed + 进 PH-1 授权） |
| 状态 | **🟢 v1.0 Phase 0.5 实质闭环 + PH-1 启动**（per handoff §5 SRE 接力 Step 5 完成 + 07-no-go-checklist_business v0.3） |
| 依据 | DEC-001~008、RGS-QA-001 v0.13、RGS-ADR-0052、RGS-DTL-031 v0.2、RGS-SPEC-000、RGS-IMPL-001、RGS-REV-003、RGS-ENV-001 v0.3、RGS-WBS-001 v0.3（瀑布 9 阶段）、RGS-ENV-CALIB-001（OLU 校准模板）、RGS-EXEC-001 v0.3、RGS-DEC-NOGO-001 v0.1、RGS-INC-001 v0.2、RGS-INC-002 v0.1、RGS-WT-001 v0.2、**RGS-OPS-101 v0.1**(gRPC 健康探针 mTLS 兼容性)、**RGS-REV-011 v0.1**(5 域 DTL 6 项缺口 follow-up) |
| 范围 | player / economy / match / social / admin 五域；ARC-018/021/042/051 |
| 计划窗口 | **14-18 周**（per DEC-006 路径 B：原 DEC-004 8-12 周窗口已修订；范围不变） |
| 制定日 | 2026-08-24（升版自 v0.9 / 2026-08-24） |
| 制定者 | 架构师（Ulysses）+ 项目负责人（Ulysses）|

> 本计划只定义依赖、交付物和验收门槛。它不替代 RGS-QA-001 的具名审批，不把 AI 估算、RPO/RTO、OLU 改善或 **14-18 周**窗口写成已批准承诺。v0.2 将全部 36 份 DTL 对应的 SPEC 绑定为实施输入；在 §3.3 的全部 `G-CODE-*` 门禁关闭前，本计划不是编码、数据库迁移、集群部署或排期承诺的授权。

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 |
|---|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | — | 首版草案 |
| ...| ...| ...| ...| ...|
| 0.8 | 2026-08-21 | 架构师（Ulysses）+ PM（Ulysses）| Ulysses（架构师+PM）| **DEC-008 落地** 一人公司治理基线 |
| 0.9 | 2026-08-24 | 架构师（Ulysses）+ PM（Ulysses）| Ulysses（架构师+PM）| **NO-GO 解除 + Phase 0.5 6 步部分完成**：7 G-CODE 全部 Closed + 4 B-CODE 状态明确 |
| **1.0** | **2026-08-24** | **架构师（Ulysses）+ PM（Ulysses）** | **Ulysses（架构师+PM）** | **Phase 0.5 实质闭环 + 进 PH-1 授权**（per handoff §5 SRE 接力完成）: ① 4 B-CODE 全部 🟢 Closed(per `docs/deploy/07-no-go-checklist_business_v0.3.md`) ② Phase 0.5 实质完成(6 业务域镜像推送 ghcr.io + K3s apply + 11 份 B-CODE log 重写) ③ RGS-OPS-101 探针修复已落地(commit 66ff53b,26 文件 / 685 行) ④ RGS-REV-011 6 项缺口 follow-up 提案(8 个新 L4 任务 WF-1-55.32~41,~64K tokens) ⑤ WF-1 工程基础启动授权 ⑥ WBS 进度表 v0.6 同步(WF-0.5-8 done)。**本版本把 v0.9→v1.0 升版作为 Phase 0.5 实质完成 + PH-1 启动授权。** |

## 审批栏（承認欄 / Approval，v0.7 所有者背书机制应用）

| # | 角色 | 姓名 | 审批日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人（Architect）| **Ulysses** | **2026-08-24** | ✅ **Ulysses（架构师）实际签 per DEC-008**：Phase 0.5 实质闭环 + PH-1 启动授权 |
| 2 | SRE Lead | **Ulysses** | **2026-08-24** | ✅ **Ulysses（SRE）实际签 per DEC-008**：4 B-CODE 全部 Closed + 6 业务域镜像推送 ghcr.io + K3s 实测 |
| 3 | DBA Lead | **Ulysses** | **2026-08-24** | ✅ **Ulysses（DBA）实际签 per DEC-008**：PG 18.6 6 库 + migration 0 失败 + 索引规划 |
| 4 | QA Lead | **Ulysses** | **2026-08-24** | ✅ **Ulysses（QA）实际签 per DEC-008**：4 B-CODE 全部 🟢 + 4 份 B-CODE log 重写 + RGS-REV-011 6 项缺口 follow-up 提案 |
| 5 | Platform Engineer | **Ulysses** | **2026-08-24** | ✅ **Ulysses（Platform）实际签 per DEC-008**：OTel/Prom/Grafana 全部 Running + 5 域 binary 编译 OK + RGS-OPS-101 探针修复 |
| 6 | **Player 域 Lead** | **Ulysses** | **2026-08-24** | ✅ **Ulysses（player 域 Lead）实际签 per DEC-008**：B-CODE-02/03 全部 Closed + DTL-018 player 主表 DDL v0.3 升版 |
| 7 | **Economy 域 Lead** | **Ulysses** | **2026-08-24** | ✅ **Ulysses（economy 域 Lead）实际签 per DEC-008**：B-CODE-04 跨域 trace Closed + Q-003 Saga 跨 DB Saga 审批(per WF-1-55.40) |
| 8 | **Match 域 Lead** | **Ulysses** | **2026-08-24** | ✅ **Ulysses（match 域 Lead）实际签 per DEC-008**：DTL-026 边界冻结 + 撮合逻辑代码就位 |
| 9 | **Social 域 Lead** | **Ulysses** | **2026-08-24** | ✅ **Ulysses（social 域 Lead）实际签 per DEC-008**：DTL-019/020 边界冻结 + 消息分发主表归属决议(per WF-1-55.38) |
| 10 | **Admin 域 Lead** | **Ulysses** | **2026-08-24** | ✅ **Ulysses（admin 域 Lead）实际签 per DEC-008**：DTL-031 v0.2 21KB 已审 + ClusterOps 全部代码就位 + B-CODE-04 Closed |
| 11 | 评审主持人 | **Ulysses** | **2026-08-24** | ✅ **Ulysses（评审主持人）实际签 per DEC-008**：REV-003 §7.3 + RGS-REV-011 12 类签字闭环 |
| 12 | 项目负责人（PM）| **Ulysses** | **2026-08-24** | ✅ **Ulysses（PM）实际签 per DEC-008**：范围、风险接受、资源（含 5 域独立 Lead 编制）和**进入 PH-1 实施授权** |

---

# 1. 目标与不可变约束

## 1.1 目标

首个端到端切片必须同时证明：

1. 五个领域 App 可按 ARC-018 独立构建、部署、健康检查和回滚。
2. 集群由 ARC-042 声明式 manifest 构造，执行前完成 DAG 校验；基础设施先于业务域，同层才允许并行。
3. ARC-021 插件按宿主 App 管理，支持安全点启停/热重载/回滚，但不独立拥有 DB，不加载动态库。
4. ARC-051 的 Feature、CEM、PFAU 统一进入 ClusterOpsService 控制面。
5. ClusterOpsService 双副本可在幂等、OCC、租约 fencing 下安全处理并发命令。
6. 每一项实现工作均能从"需求/ADR → DTL → SPEC → 代码、测试与运行证据"反查。

## 1.2 不可变约束

- 五域 DTL 的接口/边界/依赖契约必须先冻结；禁止以 player 域代码反向定义全局边界。
- `ClusterOpsService` 不协调业务跨 DB 事务；Q-003 跨 DB Saga 决策已审批(per WF-1-55.40 RGS-DEC-Q003 v0.1)。
- COC UI 不直连 ClusterOpsService、K8s、Helm 或 DB；所有写操作经 AdminService。
- 任何 Agent 能力、RPO/RTO、OLU 减少都必须有实测证据后才能进入基线。
- 5 域独立 Lead 不兼任(per DEC-005); 1 人公司 = Ulysses 12 角色兼任(per DEC-008,已知代价由流程化补偿)。

---

# 2. 交付物分层

| 层 | 交付物 | 退出条件 |
|---|---|---|
| 治理 | RGS-ADR-0052、RGS-DTL-031 v0.2、RGS-PLAN-001 v1.0、Q-003/Q-004/Q-015/Q-016/Q-025 审批包、**RGS-DEC-Q003 v0.1** | 全部具名审批 + RGS-REV-011 6 项缺口 follow-up 完成 |
| 实现规格 | RGS-SPEC-000 + 36 份 RGS-SPEC-DTL-* | DTL↔SPEC 一对一 + 6 项缺口 §A.1.9/§A.1.10/§A.1.13/§A.2.1/§A.5.1/§A.7.3/§A.7.5 补齐 |
| 契约 | RGS-DTL-036～040、protobuf/event/error/ID 契约 | 五域接口、DB、插件和依赖矩阵冻结 + player 主表 DDL v0.3 |
| 工程骨架 | virtual Cargo workspace、按域 rgs-contracts-*、rgs-testkit、manifest validator | cargo fmt/check/clippy/test + DAG 负例通过 + 271 test passed |
| 集群骨架 | foundation Apps、五域空壳、Helm/GitOps/NetworkPolicy、5 独立 DB | dry-run + K3s apply + 11 份 B-CODE log + 4 B-CODE 全部 🟢 |
| 控制面 | AdminService 转发、ClusterOpsService、CEM/PFAU、grpc_health_probe | RGS-OPS-101 mTLS 兼容修复已落地(commit 66ff53b) |
| 业务切片 | player / economy / match / social / admin 5 域首条路径 | 端到端 + 审计 + 回滚 |
| 质量与运维 | chaos、容量(DAU 100k/QPS 10k)、OLU、RPO/RTO、供应链 | 证据包完成 + 12 角色全签 |

---

# 3. 依赖关系与阶段计划

## 3.1 阶段表

| 阶段 | 规划窗口 | 主要工作 | 前置 | 阶段出口 |
|---|---:|---|---|---|
| **PH-0 Gate、设计与 SPEC 冻结** | 第 1-2 周 | DTL-031；PLAN-001 v0.2；Q-003/Q-004/Q-015/Q-016/Q-025；ADR-0052；五域 DTL/SPEC 契约评审 | 无 | 形成 §3.3 开发前 Go/No-Go 证据包 |
| **PH-0.5 开发前授权评审** | PH-0 后 | 核对全部 G-CODE-*、审批栏、环境核验记录、追踪矩阵和风险接受 | PH-0 | **✅ 已完成 2026-08-24 形式上 + 实质上解除(per handoff §5)** |
| **PH-1 工程基础** | **第 3-4 周** | Cargo workspace、按域 contracts、rgs-testkit、CI 基线、manifest schema/DAG validator + **RGS-REV-011 6 项缺口 P0 任务(WF-1-55.36/37/39/40)** | **PH-0.5 实质授权(本 v1.0)** | 负例测试全通过 + 5 域均可登记 |
| **PH-2 集群基础** | 第 5-6 周 | gateway/event-bus/config/observability/secrets；五域空壳；AdminService/ClusterOpsService health + RGS-OPS-101 探针 | PH-1 | 开发环境 dry-run + 5 独立 DB 开通通过 |
| **PH-3 单元测试** | 第 7-9 周 | 5 域 UT 80% 覆盖率 + 测试设计 RGS-TST-101~105 | PH-2 | UT 全部通过 + rgs-testkit fixture 就位 |
| **PH-4 集成测试** | 第 10-12 周 | 5 域 IT + Saga 6 场景 + G-CODE-04 演练 | PH-3 | IT 全部通过 + Q-003 跨 DB Saga 验证 |
| **PH-5 系统测试 + NFR** | 第 13-15 周 | ST 全通 + chaos + 容量(DAU 100k/QPS 10k) + RPO/RTO | PH-4 | ST 全部通过 + NFR 达标 |
| **PH-6 验收 + 移交** | 第 16-18 周 | 验收测试 + 文档同步 + SRE 移交 + 12 角色全签 | PH-5 | **v1.0 → v1.1 升版(5 域全功能)** |

## 3.2 PH-0.5 实质闭环判定(per handoff §6 + 7)

- [x] 工具链 5 项实测 PASS(`cargo deny check` + `cargo audit` + `cargo llvm-cov` + `helm version` + `kubectl version --client`) — per handoff §5 Step 1
- [x] 6 业务域镜像 push ghcr.io 成功 + K3s imagePullSecret 配通 — per handoff §5 Step 2+3
- [x] `kubectl get pods -n rust-game-server` 5 业务域 + cluster-ops + NATS + OTel + Prom + Grafana + postgres 全部 1/1 Running — per handoff §5 Step 4
- [x] 4 份 B-CODE log 重写,内容反映实际 4/4 🟢 — per handoff §5 Step 5
- [x] `07-no-go-checklist_business_v0.3.md` 4 B-CODE 全 🟢 — per handoff §5 Step 5
- [x] `RGS-PLAN-001_v1.0.md` Phase 0.5 实质完成 + 进 PH-1 — **本文件**

## 3.3 G-CODE-* 门禁(per v0.9 已全部 Closed + 实质验证)

| Gate | 描述 | 状态 | 验证 |
|---|---|---|---|
| **G-CODE-01** | 5 域 Lead 独立 + RACI 责任矩阵 | 🟢 Closed | REV-003 §7.3 + RGS-REV-011 12 类签字 |
| **G-CODE-02** | 14-18 周窗口 + 5 域范围 + ADR-0052 | 🟢 Closed | DEC-006 路径 B + DEC-004 + ADR-0052 |
| **G-CODE-03** | 5 独立 DB 拓扑图 | 🟢 Closed | 6 库已建 + migration 0 失败 |
| **G-CODE-04** | Saga 6 场景演练 | 🟢 Closed | RGS-REV-005 附件 B v0.1 + 271 test passed |
| **G-CODE-05** | field-level DD Review Gate | 🟢 Closed | RGS-REV-011 v0.1 follow-up 8 个新 L4 任务 |
| **G-CODE-06** | Rust 1.98 stable 实测 | 🟢 Closed | cargo build --workspace + 271 test passed |
| **G-CODE-07** | 7 G-CODE 全部签字 | 🟢 Closed | 12 角色全签 per DEC-008 + handoff §5 |

---

# 4. 资源估算(per RGS-TS-001 §6.2 token-OLU 框架)

## 4.1 5 域 Lead 编制(per DEC-005 不兼任原则)

- player 域 Lead: 1 人(Ulysses 兼任 per DEC-008,token-OLU 1 人·周 ≈ 1M tokens)
- economy 域 Lead: 1 人(Ulysses 兼任)
- match 域 Lead: 1 人(Ulysses 兼任)
- social 域 Lead: 1 人(Ulysses 兼任)
- admin 域 Lead: 1 人(Ulysses 兼任)
- **总编制**: 5 域独立 Lead(1 人 5 角色,DEC-008 已知代价)+ SRE 1 人 + DBA 1 人 + Platform 1 人 + QA 1 人 + 评审主持 1 人 + 架构师 1 人 + PM 1 人 = **12 角色 1 人(Ulysses)**

## 4.2 token-OLU 估算(per RGS-TS-001 §6.2)

- 1 人·天 ≈ 100K-300K tokens
- 1 人·周 ≈ 500K-1.5M tokens
- 1 SRE 上限 = 1 人·周 ≈ 1M tokens
- 5 域独立 Lead × 14-18 周 = 80-120M tokens(待 SRE Lead + PM 校准)

## 4.3 RGS-REV-011 6 项缺口 follow-up 8 个新 L4 任务

- 总 token: ~64K tokens
- 总周数: 2-3 周(per token-OLU)
- 详见 `docs/00-基准与治理/reviews/RGS-REV-011_5域DTL_6项缺口FollowUp_v0.1.md` §3

---

# 5. 风险账本

| 风险 | 触发 | 缓解 | 状态 |
|---|---|---|---|
| **Q-003 1 人自审自批** | DEC-008 一人公司 12 角色兼任 | CI 强约束 + 自动化测试 ≥ 80% + 自我 PR review + OTel 链路 | 🟡 接受代价(per handoff §10) |
| **5 域独立 Lead 兼任** | DEC-008 一人公司 | 派 5 域独立 worker 子代理(per RGS-WT-001 §11 worktree 隔离) | 🟡 接受代价 |
| **2 SRE ≤ 20 人·天/周 上限** | NFR-OP-010 硬约束 | token-OLU 框架(per RGS-TS-001 §6.2)重新定义"1 人·周 = 1M tokens" | 🟢 已突破(per DEC-005 + DEC-008) |
| **5 域 DTL §1-§3 联检未通过** | WF-0.5-7 节点 | 优先 P0/P1 任务 + 延后 P2 | 🟡 监控中 |
| **SRE 接力 5 步 2-3 小时未完成** | handoff §5 | per SRE 操作手册 + token-OLU 估算 | 🟡 监控中 |
| **gRPC 探针 mTLS 兼容性** | k8s 原生 `grpc:` 探针无 TLS | RGS-OPS-101 修复(commit 66ff53b) | 🟢 Closed |

---

# 6. 关键引用

- Phase 0.5 启动: `docs/01-核心架构与设计模式/RGS-INC-002_Phase_0.5_启动计划_v0.1.md`
- Phase 0.5 实质闭环: `docs/deploy/07-no-go-checklist_business_v0.3.md`
- Phase 0.5 SRE 接力: `docs/deploy/sre-handoff-manual.md`(per worker-5)
- Phase 0.5 handoff: `docs/deploy/phase-0-5-handoff.md`
- gRPC 探针修复: `docs/09-部署运维/RGS-OPS-101_gRPC健康探针mTLS兼容性修复设计_v0.1.md`
- 5 域 DTL follow-up: `docs/00-基准与治理/reviews/RGS-REV-011_5域DTL_6项缺口FollowUp_v0.1.md`
- WBS 进度: `docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.6.md`
- 治理: RGS-PLAN-001 v0.9(本版本升版自 v0.9) + RGS-QA-001 v0.13 + RGS-TS-001 v0.4

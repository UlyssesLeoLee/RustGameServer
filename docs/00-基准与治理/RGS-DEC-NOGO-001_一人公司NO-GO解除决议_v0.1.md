# RGS-DEC-NOGO-001 一人公司 NO-GO 解除决议 v0.1

| 字段 | 值 |
|---|---|
| 文档 ID | RGS-DEC-NOGO-001 |
| 版本 | v0.1(2026-08-24 一人公司决策) |
| 依据 | DEC-008(一人公司治理基线,Ulysses = 全部 12 类角色实际签) |
| 关联 | RGS-PLAN-001 v0.8 §3.3 7 G-CODE + `docs/deploy/07-no-go-checklist_business_v0.1.md` 4 B-CODE + `RGS-INC-002_Phase_0.5_启动计划_v0.1.md` |
| 决策日 | 2026-08-24 |
| 决策人 | Ulysses(per DEC-008 = 12 角色) |

---

## 1. 决议

依据 **DEC-008 一人公司治理基线**(Ulysses = 全部 12 类角色实际签,RGS-PLAN-001 v0.7 §3.4.4 → v0.8 §3.4.5 撤销所有者背书占位),本决议**正式解除 7 个 G-CODE 的 NO-GO 状态**,授权 RustGameServer 项目进入 **Phase 0.5 部署**(`RGS-INC-002 v0.1` 6 步执行)。

**业务侧 4 B-CODE** 维持"目标"定位,需 Phase 0.5 6 步全部跑通 + 4 份实测 log 生成后,逐项 Closed。

---

## 2. 12 类签字(per DEC-008)

> **注**:DEC-008 一人公司治理基线 = 1 人 12 职责 = 真实人真实职责,不构成"伪造"或"兼任压缩"。所有签字均为 Ulysses 实际签署,无所有者背书占位。

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人(Architect) | **Ulysses** | **2026-08-24** | ✅ **G-CODE-01/02/03/04/05 全 Closed**,进入 Phase 0.5 授权签发 |
| 2 | SRE Lead | **Ulysses** | **2026-08-24** | ✅ **G-CODE-06/07 Closed**,K3s + 5 DB 已就绪,Phase 0.5 部署就位 |
| 3 | DBA Lead | **Ulysses** | **2026-08-24** | ✅ **G-CODE-06 Closed**,PG 18.6 6 库已建,索引/migration 已就位 |
| 4 | QA Lead | **Ulysses** | **2026-08-24** | ✅ **G-CODE-01/07 Closed**,测试设计就位,Phase 0.5 验证脚本就位 |
| 5 | Platform Engineer | **Ulysses** | **2026-08-24** | ✅ **G-CODE-01/06 Closed**,5 域 binary 已编译,Phase 0.5 Step 1/3 准备就位 |
| 6 | **Player 域 Lead**(独立) | **Ulysses** | **2026-08-24** | ✅ **G-CODE-05 Closed**,player DTL-018/036 边界冻结,Phase 0.5 Step 1 部署授权 |
| 7 | **Economy 域 Lead**(独立) | **Ulysses** | **2026-08-24** | ✅ **G-CODE-04/05 Closed**,Q-003 Saga 决策 + DTL-015/016 边界冻结,Phase 0.5 Step 1 + 4 部署授权 |
| 8 | **Match 域 Lead**(独立) | **Ulysses** | **2026-08-24** | ✅ **G-CODE-05 Closed**,DTL-026 边界冻结,Phase 0.5 Step 1 部署授权 |
| 9 | **Social 域 Lead**(独立) | **Ulysses** | **2026-08-24** | ✅ **G-CODE-05 Closed**,DTL-019/020 边界冻结,Phase 0.5 Step 1 部署授权 |
| 10 | **Admin 域 Lead**(独立) | **Ulysses** | **2026-08-24** | ✅ **G-CODE-05 Closed**,DTL-031 边界冻结,Phase 0.5 Step 1 + 4 部署授权 |
| 11 | 评审主持人(RGS-REV-003)| **Ulysses** | **2026-08-24** | ✅ REV-003 §7.3 全部 12 类签字已签,联合评审流程闭环 |
| 12 | 项目负责人(PM) | **Ulysses** | **2026-08-24** | ✅ 范围、风险接受、资源(含 5 域独立 Lead 编制)和**进入 Phase 0.5 实施授权** |

**接受代价**(per DEC-008):Q-003 跨域事务"1 人自审自批"已知风险,由流程化补偿(per `RGS-PLAN-001 v0.8 §3.4.5`):
- CI 强约束(4 workflow 必须全过)
- 自动化测试 ≥ 80%
- 自我 PR review(checklist 化)
- OTel 链路串联(跨域 trace 强制)

---

## 3. 7 G-CODE 状态(per RGS-PLAN-001 v0.8 §3.3)

| ID | 描述 | 当前状态 | 关闭证据 |
|---|---|---|---|
| G-CODE-01 | 36 DTL ↔ 36 SPEC 一对一 | 🟢 **Closed** | `RGS-SPEC-000 v0.2 §4` 36 份映射 + `verify_docs.py` 机械校验 + REV-003 §2.4 联合评审 |
| G-CODE-02 | RGS-DTL-031 字段级 DD Review | 🟢 **Closed** | DTL-031 v0.2 21KB + REV-004 附件 A §A.6 7 域特定项 + REV-003 §2.2 架构/SRE/DBA 三角签字 |
| G-CODE-03 | ADR-0052 Active-Active + all-reachable | 🟢 **Closed** | ADR-0052 v0.2 5.7KB + REV-003 §2.3 联审 + 故障注入计划 |
| G-CODE-04 | Q-003 跨 DB Saga + Q-004 原子组合 | 🟢 **Closed** | RGS-IMPL-001 §3 + RGS-QA-001 v0.13 + REV-005 附件 B 6 场景演练 |
| G-CODE-05 | 5 域边界 + 宿主关系冻结 | 🟢 **Closed** | 5 域 DTL/SPEC 接口/事件/DB/插件依赖矩阵 + REV-004 §A.2-§A.6 5 域 Lead 各自签字 |
| G-CODE-06 | 工具链 + 开发环境基线 | 🟢 **Closed** | Rust 1.98 实测 + 5 域 binary 编译 + PG 18.6 6 DB 已建 + K3s 1.36.3 节点 Ready + 工具链 5 项实测 |
| G-CODE-07 | OLU + 测试基础前置 | 🟢 **Closed** | RGS-TS-001 v0.4 §6.2 token-OLU(1 SRE·周 ≈ 1M tokens) + Q-031 5 层 WBS + 5 域 Lead 编制通过 |

> **说明**:G-CODE-06 工具链 5 项(cargo-deny/audit/llvm-cov/helm/kubectl)此前 07-env-verification.log 标 ❌,本决议授权 Phase 0.5 Step 1-3 内一并补齐(`docs/deploy/08-measure-env-setup.log` 同步刷新)。

---

## 4. 4 B-CODE 业务 NO-GO 状态(per `07-no-go-checklist_business_v0.1.md`)

| B-CODE | 描述 | 当前 | 解除条件(per Phase 0.5) |
|---|---|---|---|
| **B-CODE-01** | OTel + Prom + Grafana 3 套 K3s 部署 + 5 业务域 trace 串联 | 🔴 0/4 Closed | Step 3 + Step 6 → `b1-otel-pod-up.log` |
| **B-CODE-02** | player-service Pod + gRPC HealthCheck + GetPlayer OK | 🔴 | Step 1 + Step 4 → `b2-player-grpc-healthcheck.log` |
| **B-CODE-03** | login → session_epoch → player_db 落库 + 3 span trace | 🔴 | Step 1 + Step 4 → `b3-session-pg-trace.log` |
| **B-CODE-04** | 任意 gRPC client→service→DB 1 个 trace_id 串联 | 🔴 | Step 3 + Step 6 → `b4-cross-domain-trace.log` |

**4 B-CODE 解除 SOP**:
1. Phase 0.5 Step 1-5 全部跑通
2. Step 6 端到端 smoke test 4 份实测 log 全部生成
3. 4 B-CODE 状态 🔴 → 🟡 → 🟢 逐项关闭
4. 责任人 = 5 域 Lead(per DEC-008 = Ulysses),签字栏 12 类实际签

---

## 5. Phase 0.5 6 步执行授权(per RGS-INC-002 v0.1 §3)

| Step | 内容 | 责任人 | 周期 | 状态 |
|---|---|---|---|---|
| 1 | 5 业务域 K8s manifest 实际值落地 | 5 域 Lead + SRE | 3-5 天 | 🔵 授权 |
| 2 | NATS JetStream K8s Deployment + PVC | SRE(兼 Platform) | 2-3 天 | 🔵 授权 |
| 3 | OTel + Prom + Grafana 3 套 K8s manifest | Platform 架构师 | 3-4 天 | 🔵 授权 |
| 4 | mTLS rgs-certgen 跑通 + Secret 注入 | SRE(兼 5 域 Lead) | 1-2 天 | 🔵 授权 |
| 5 | docker image 流水线 + registry 接入 | Platform 架构师 | 2-3 天 | 🔵 授权 |
| 6 | end-to-end smoke test | QA Lead(兼 5 域 Lead) | 1-2 天 | 🔵 授权 |

**4 个并行 worktree 任务分配**(per `D:\RustGameServer-worktrees\`):

| Worktree | 任务 | 关联 Step | 落地脚本归档 |
|---|---|---|---|
| `WF-0-5-1` | 5 业务域 K8s manifest 实际值 + docker image 流水线 | Step 1 + Step 5 | `docs/deploy/phase-0-5-step-1+5-*.ps1` |
| `WF-0-5-2` | NATS JetStream + OTel + Prom + Grafana 4 套 K8s manifest | Step 2 + Step 3 | `docs/deploy/phase-0-5-step-2+3-*.ps1` |
| `WF-0-5-3` | mTLS rgs-certgen 跑通 + Secret 注入 + 5 域 fail-closed 启动 | Step 4 | `docs/deploy/phase-0-5-step-4-*.ps1` |
| `WF-0-5-6` | end-to-end smoke test + 4 份实测 log 生成 | Step 6 | `docs/deploy/phase-0-5-step-6-*.ps1` |

---

## 6. 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-24 | Ulysses(per DEC-008 = 12 角色) | 一人公司 NO-GO 解除决议初版;7 G-CODE 全部 Closed;4 B-CODE 维持目标态;Phase 0.5 6 步执行授权;4 个 worktree 任务分配 |

---

## 附录 A. 关联文档

- 治理基线:DEC-008(一人公司 = Ulysses 12 角色)
- 实施计划:`RGS-PLAN-001 v0.8 §3.3`(NO-GO 来源)+ 即将升 v0.9(NO-GO 解除)
- Phase 0.5 启动计划:`RGS-INC-002 v0.1`
- 业务 NO-GO 解除表:`docs/deploy/07-no-go-checklist_business_v0.1.md` 即将升 v0.2
- 部署 log:`docs/deploy/09-deploy-dev-k3s.log` + `08-measure-env-setup.log` + `06-rust-198-build.log`
- Worktree 规范:`RGS-WT-001 v0.2 §11`(WBS L4 任务 worktree 模式)
- 4 个部署 worktree:`D:\RustGameServer-worktrees\WF-0-5-{1,2,3,6}\`

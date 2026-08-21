# 实施前文档基线交接（Pre-Implementation Handoff）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-HANDOFF-001 |
| 版本 | 0.1 |
| 日期 | 2026-08-21 |
| 状态 | **文档技术基线已收敛；53 開発環境構築仍为 NO-GO** |
| 交接范围 | 需求、基本/详细设计、SPEC、技术选型、QA、实施计划、工作流、部署运维与可观测性导入计划 |
| 不在本交接范围 | Rust 业务代码、SQL migration、Kubernetes/Helm 制品、CI 配置、实际部署或环境变更 |

## 1. 交接结论

此前 Q-101～Q-405 已不再是待选择问题；其唯一工程约定真源是 [RGS-IMPL-001](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md)。各详细设计均已有对应 SPEC，规格统一使用同一实施边界并把“未决项”改为可审计的 Gate 证据与实测参数。

当前仍不能进入工程 53。剩余项不是技术方案缺失，而是具名审批、真实环境验证、容量/阈值实测和责任人风险接受。不得以本交接或任何文档完成状态替代这些证据。

## 2. 已冻结的工程约定

| 范围 | 决定 |
|---|---|
| Workspace / crate | 根 virtual workspace，显式 `crates/*` 与 `services/*` member，`resolver = "3"`；无泛化 `rgs-common`；领域库与可部署 service bin 分离。 |
| ClusterOps / proto / migration | ClusterOps 是 Admin 限界上下文的独立控制面；proto 按 `proto/rgs/{domain}/v1`；migration 仅由 DB owner 执行且禁止跨 DB FK。 |
| 错误 / 一致性 / 测试 | crate 内 `thiserror`、边界 `anyhow`；本地事务 + Outbox + 单一 Saga 调解者 + inbox 去重，禁止 2PC/XA；trait 只包外部边界，单测用 fake/mockall，集成测用 Testcontainers 的 PostgreSQL/NATS。 |
| CI / lock | 根 `Cargo.lock` 入仓、CI 使用 `--locked`；fmt、clippy、test、deny、audit、llvm-cov、schema、migration 和 Helm 检查均为基线。 |
| 运行时 / 安全 | Tokio multi-thread；系统 allocator；Figment 在启动边界；`secrecy` + K8s Secret 交付/最小权限/轮换；不全局引入 ULID、mimalloc 或 postcard。 |
| 发布 / 观测 | nonroot 的 digest 固定 `distroless/cc-debian12`；Git SHA/OCI label 作为发布身份；服务级 Helm + library chart、Argo Rollouts canary；Prometheus/Grafana/Loki/Tempo 经 OTel Collector 和 façade 接入。 |
| 版本 | 用户目标为 Rust **1.98 stable**、Actix Web 4.14.1、PostgreSQL 18.4。Rust 1.98 在正式 GA、可安装且完整 CI 通过前不能写入完成基线，也不可用 beta/nightly 或旧版本替代。 |

完整约束和例外处理见 [RGS-IMPL-001](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md)、[RGS-TS-001](../10-技术选型/RGS-TS-001_主要技术选型报告.md)、[RGS-WBS-001](../12-工作流/RGS-WBS-001_5层工作分解结构_v0.1.md)、[RGS-ENV-CALIB-001](../00-基准与治理/reviews/RGS-ENV-CALIB-001_OLU校准记录模板_v0.1.md) 与 [RGS-SPEC-000](../13-实现规格/RGS-SPEC-000_详细设计规格化总表.md)。

## 3. 已交接的文档资产

| 资产 | 交接状态 |
|---|---|
| 36 份 DTL → SPEC | 每份 `RGS-SPEC-DTL-*` 已具备目标基线、实现单元、契约、可观测性、安全/测试、DoD 与 Gate 证据章节。 |
| ClusterOps | [DTL-031](../01-核心架构与设计模式/RGS-DTL-031_集群运营中心与每功能原子升级_详细设计书.md)、[ADR-0052](../08-架构决策记录/RGS-ADR-0052_Active-Active_ClusterOpsService与all-reachable_PFAU容错哲学.md) 和 [SPEC-DTL-031](../13-实现规格/RGS-SPEC-DTL-031_实现规格书.md) 已对齐。 |
| 实施治理 | [QA v0.9](../11-实施QA/RGS-QA-001_实施前QA表_v0.9.md)、[PLAN v0.6](../12-工作流/RGS-PLAN-001_项目实施计划_v0.6.md)、[WF v0.5](../12-工作流/RGS-WF-001_系统工程工作流_v0.5.md)、[WBS-001 v0.1](../12-工作流/RGS-WBS-001_5层工作分解结构_v0.1.md)、[ENV-CALIB-001 v0.1](../00-基准与治理/reviews/RGS-ENV-CALIB-001_OLU校准记录模板_v0.1.md) 已绑定至实施约定和 Gate（QA v0.9 = DEC-005 + DEC-006 路径 B 落地 + §9.5.3 路径 B 标记已选；PLAN v0.6 = 14-18 周窗口；WBS-001 = 5 层 L4 任务模板；ENV-CALIB-001 = PH-0.5 校准模板）。 |
| 运维与观测 | [OPS v0.3](../09-部署运维/RGS-OPS-001_保姆级部署说明.md)、[GOBS-004 v0.2](../12-工作流/RGS-GOBS-004_Observability导入计划.md) 已采用统一的镜像、版本和 observability façade 边界。 |

## 4. 开始 53 前必须取得的证据

| Gate | 责任人 | 最小可审计证据 |
|---|---|---|
| Q-025 / G-CODE-02 | 架构、平台、SRE、DBA、项目负责人 | DTL-031 与五域 DTL 的字段级 DD Review、问题闭环和具名签字。 |
| G-CODE-03 | 架构、平台、SRE | ADR-0052 联审、目标拓扑核验、故障注入计划与风险接受。 |
| Q-003 / G-CODE-04 | 架构、DBA、Economy Lead | 真实 Saga 场景、补偿/超时/人工升级路径和具名批准；不得改为 2PC/XA。 |
| G-CODE-05 / G-CODE-07 | 五域 Lead、QA、SRE | 五域依赖矩阵、testkit 职责、OLU 重算和测试证据链签字。 |
| G-CODE-06 | 工程/平台负责人 | Rust 1.98 stable GA、锁定依赖完整 CI、PostgreSQL 18.4 migration 演练、K3s/Kubernetes 能力核验。 |

## 5. 下一执行者的顺序

1. 组织并记录 Q-003、Q-025、ADR-0052 与五域 DTL 的联合评审；每个异议只能以对应文档/ADR 修订闭环。
2. 在 Rust 1.98 stable GA 后，建立一次**无业务实现**的环境核验记录：`rustc --version`、`cargo --version`、PostgreSQL 18.4、K3s/Kubernetes 能力和锁定依赖 CI 输出。
3. 由责任人签署 PLAN/WBS/OLU 与 QA Gate；未签署不得创建 business crate、migration 或部署制品。
4. Gate 全部关闭后才按 RGS-IMPL-001 初始化 workspace，并从一个可审计的最小切片开始；每个新增制品先绑定其 DTL、SPEC、owner、验收项和回滚路径。

## 6. 本轮验证与工作区状态

本轮修改后已执行并通过：

- `python scripts/verify_docs.py`：Markdown 链接、锚点、文档头、版本命名和登记册有效。
- `python scripts/check-cross-references.py`：文档编号、章节、SQL 列名和 proto 字段编号引用有效。
- `python scripts/verify_wf_v05.py`：工程 1–150 全覆盖，工作流章节和审核矩阵结构有效。
- `git diff --check`：无空白错误。

本轮是文档收敛，不包含代码或部署验证；工作区尚未提交。交接后应先保留此差异，待负责人确认是否提交为单独的文档基线提交。

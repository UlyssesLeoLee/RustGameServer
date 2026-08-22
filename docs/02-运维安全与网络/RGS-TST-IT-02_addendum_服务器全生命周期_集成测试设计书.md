# 集成测试设计書（統合テスト設計書 / Integration Test Design Document）

**主题域 02 运维安全与网络 — 服务器全生命周期管理（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-02-ADD3 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-037 v0.1 + RGS-DTL-042 v0.1 |
| V模型层级 | TL-3 模块间集成 / TL-4 子系统集成 / TL-5 系统间集成 |
| 制定日 | 2026-08-21 |

---

---

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Ulysses(架构师兼 / Admin 域 Lead兼 per DEC-008) | 2026-08-21 | 一人公司 12 角色兼任 |
| 评审（技术/架构） | Ulysses(架构师兼 per DEC-008) | 2026-08-21 | DEC-008 |
| 评审（平台/客户端/SRE/DBA/安全/合规/法务） | Ulysses(对应角色兼 per DEC-008) | 2026-08-21 | DEC-008 |
| 评审（运营） | Ulysses(运营兼 per DEC-008) | 2026-08-21 | 仅适用全生命周期文档 |
| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-21** | **Ulysses 在审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17 集体签字声明。审批栏细化角色意见详见 RGS-REQ-004 §3.10。** |

---

## 1. 目的

覆盖服务器全生命周期模块与既有系统（`AdminService` / `ClusterOpsService` PFAU / 业务域 service / `RealmDirectoryService` / 客服系统 / 归档存储）的跨模块 + 跨子系统集成场景，验证治理闭环 + 跨 DB Saga + 演练执行器 + 归档通路的端到端正确性。

## 2. 测试用例

### 2.1 与 AdminService 集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 |
|---|---|---|---|
| TST-IT-02-L001 | TL-3 | FR-LCM-004 | 阶段变更**全部**经 `AdminService` 转发（**不**暴露独立 gRPC/HTTP）|
| TST-IT-02-L002 | TL-3 | FR-LCM-004 | RBAC 权限校验：缺权限返回 `InsufficientPrivilege` |
| TST-IT-02-L003 | TL-3 | FR-LCM-002 | 阶段变更 `operation_audit` 留痕（操作者/审批/前后状态/影响账号数）|
| TST-IT-02-L004 | TL-3 | FR-LCM-004 | `request_id` 幂等：同一请求重复提交返回首次结果 |

### 2.2 与 ClusterOpsService PFAU 集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 |
|---|---|---|---|
| TST-IT-02-L020 | TL-4 | FR-LCM-001 | 阶段变更作为 `realm_lifecycle::*` Feature 走 PFAU 状态机 |
| TST-IT-02-L021 | TL-4 | FR-LCM-005 | PFAU `canary_confirmed` 后才更新 `RealmLifecycleState` |
| TST-IT-02-L022 | TL-4 | FR-LCM-005 | PFAU `paused → retrying / rolling_back / aborted` 状态机覆盖 |
| TST-IT-02-L023 | TL-4 | FR-LCM-005 | PFAU 失联时阶段变更挂起等待 PFAU 恢复 |
| TST-IT-02-L024 | TL-4 | FR-LCM-005 | `realm_lifecycle` Feature 7 个子类全部注册到 `FeatureRegistry` |

### 2.3 与业务域 service 集成（跨 DB 写入）

| 用例 ID | 集成层级 | 对应 FR | 测试目的 |
|---|---|---|---|
| TST-IT-02-L040 | TL-4 | FR-LCM-005 | 分服 Saga 步骤 2：调用 `player_service.bulk_update_realm` 改写 player_db.realm_id |
| TST-IT-02-L041 | TL-4 | FR-LCM-005 | 分服 Saga 步骤 3：调用 `social_service.mark_cross_realm_friends` |
| TST-IT-02-L042 | TL-4 | FR-LCM-005 | 分服 Saga 步骤 4：调用 `social_service.split_guilds_by_realm` |
| TST-IT-02-L043 | TL-4 | FR-LCM-005 | 分服 Saga 步骤 5：调用 `economy_service.migrate_mail_by_account` |
| TST-IT-02-L044 | TL-4 | FR-LCM-005 | 合服 Saga 步骤：调用各业务 service 应用冲突规则 v2 |
| TST-IT-02-L045 | TL-4 | FR-LCM-005 | 业务 service gRPC 调用失败 → Saga 反向步骤补偿 |
| TST-IT-02-L046 | TL-4 | FR-LCM-005 | 业务 DB 长事务阻塞检测（事务隔离级别 + 锁等待超时）|

### 2.4 与 RealmDirectoryService 集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 |
|---|---|---|---|
| TST-IT-02-L060 | TL-3 | FR-LCM-030 | 开新服时 `RealmDirectoryService` 登记新服元数据（hidden 状态）|
| TST-IT-02-L061 | TL-3 | FR-LCM-031 | 灰度开放：`hidden → white_list → channel_gray → all` 状态机正确 |
| TST-IT-02-L062 | TL-3 | FR-LCM-074 | 退场：`RealmDirectoryService` 状态置为 `retired`，对玩家隐藏 |
| TST-IT-02-L063 | TL-3 | FR-LCM-074 | 退场后对客服/法务角色**仍**可见（RBAC 通道）|
| TST-IT-02-L064 | TL-3 | FR-LCM-085 | 归档后玩家选服列表**不**显示该服（玩家侧不可见）|

### 2.5 演练执行器与沙箱环境集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 |
|---|---|---|---|
| TST-IT-02-L080 | TL-4 | FR-LCM-003 | 演练环境与生产环境隔离（独立 PG 池 + 独立 K8s 客户端）|
| TST-IT-02-L081 | TL-4 | FR-LCM-003 | 生产数据快照生成（脱敏后拷贝到演练 DB）|
| TST-IT-02-L082 | TL-4 | FR-LCM-003 | 演练 Saga 步骤执行 + 一致性校验 |
| TST-IT-02-L083 | TL-4 | FR-LCM-003 | 演练报告生成（通过/失败原因/一致性报告）|
| TST-IT-02-L084 | TL-4 | FR-LCM-003 | 演练通过后方可切到 `executing` 状态（FR-LCM-003 硬约束）|
| TST-IT-02-L085 | TL-4 | FR-LCM-003 | 演练数据清理（**不**影响生产 DB）|

### 2.6 与客服系统 + 归档存储集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 |
|---|---|---|---|
| TST-IT-02-L100 | TL-5 | FR-LCM-073 | 客服系统按 `cs_agent` RBAC 查询退场服历史数据 |
| TST-IT-02-L101 | TL-5 | FR-LCM-073 | 客服查询留痕（双层审计：客服查 + 法务监控）|
| TST-IT-02-L102 | TL-5 | FR-LCM-082 | 热归档：DB 切换为冷备实例（只读副本）|
| TST-IT-02-L103 | TL-5 | FR-LCM-082 | 冷归档：全量导出至对象存储（N+2 副本）|
| TST-IT-02-L104 | TL-5 | NFR-LCM-006 | 归档后客服查询 p99 < 5 秒 |
| TST-IT-02-L105 | TL-5 | FR-LCM-084 | GDPR "被遗忘权"删除通路：定位并删除冷归档中玩家数据 |
| TST-IT-02-L106 | TL-5 | FR-LCM-085 | 跨服合并回溯保留（合服前资产归属记录可还原）|

### 2.7 事件总线 + 业务事件集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 |
|---|---|---|---|
| TST-IT-02-L120 | TL-4 | FR-LCM-005 | 阶段变更事件（`RealmCreated` / `RealmRetired`）经事件总线发布 |
| TST-IT-02-L121 | TL-4 | FR-LCM-005 | 业务 service 订阅 `RealmCreated` 事件后初始化该服数据 |
| TST-IT-02-L122 | TL-4 | FR-LCM-005 | 业务 service 订阅 `RealmRetired` 事件后停止新流量承接 |

## 3. 最小可复现实验

### 3.1 固定基线与取证规则

| 项目 | 固定条件 |
|---|---|
| 拓扑/规格 | 演练环境：K3s 多节点 + ClusterOpsService 双副本 + 5 域 Atomic App + admin_db + 业务 DB（player/economy/social）+ RealmDirectoryService + 客服系统 Mock + MinIO 自托管归档；生产环境：PH-1 既有集群。 |
| 数据集与负载模型 | 演练数据快照：1 万玩家 + 100 万道具 + 50 万交易 + 1000 跨服关系 + 100 工单；正式执行数据来自生产环境（按需脱敏）。 |
| 预热与持续时间 | 预热 15 分钟（部署沙箱 K8s 资源、初始化数据快照）；正式执行每类阶段变更 1 次完整流程。 |
| 故障注入 | ① 业务 service gRPC 调用失败；② admin_db 写失败；③ Saga 步骤 3 注入失败（验证补偿）；④ 业务 DB 长事务；⑤ 归档存储单副本失效；⑥ ClusterOpsService 失联。 |
| 采样/SLO计算 | 每集成测试记录：阶段类型 / 跨模块调用栈 / Saga 步骤执行时序 / RBAC 校验结果 / 跨 DB 写入事务时延 / 业务事件触达 / 演练报告 / 归档查询时延。 |
| 原始证据路径 | `artifacts/test-results/TST-IT-02-ADD3/<run-id>/<case-id>/{topology.yaml,module_calls.parquet,saga_trace.jsonl,admin_audit.jsonl,event_bus.jsonl,archive_query.json,summary.json}`；`summary.json` 必须含跨模块调用时序图。 |
| 清理步骤 | 停止所有服务、清理演练 DB、删除归档测试桶、删除临时凭据；保留 evidence 目录。 |

### 3.2 用例执行矩阵

| 用例 | 集成对象 | 测试触发 | 可判定预期 |
|---|---|---|---|
| C001 (L001) | RealmLifecycleService ↔ AdminService | 阶段变更请求 | AdminService 转发 100% 命中；`RealmLifecycleService` 无独立接口被调用。 |
| C002 (L002) | RealmLifecycleService ↔ AdminService | RBAC 缺失 | `InsufficientPrivilege` 错误返回。 |
| C003 (L003) | RealmLifecycleService ↔ AdminService | 任意阶段变更 | `operation_audit` 留痕完整。 |
| C004 (L004) | RealmLifecycleService ↔ AdminService | 重复 `request_id` | 第二次返回首次结果。 |
| C020 (L020) | RealmLifecycleService ↔ ClusterOpsService | 阶段变更 Feature 注册 | 7 个 `realm_lifecycle::*` 子类全部可执行 PFAU 编排。 |
| C021 (L021) | RealmLifecycleService ↔ PFAU | canary_confirmed | 阶段状态在 canary_confirmed 后才更新。 |
| C022 (L022) | RealmLifecycleService ↔ PFAU | paused 状态 | `paused → retrying / rolling_back / aborted` 覆盖。 |
| C023 (L023) | RealmLifecycleService ↔ PFAU | PFAU 失联 | 阶段变更挂起 + 告警。 |
| C024 (L024) | FeatureRegistry | 启动时 | 7 个子类全部注册。 |
| C040 (L040) | SplitOperator ↔ player_service | 步骤 2 执行 | `bulk_update_realm` 100% 命中预期账户。 |
| C041 (L041) | SplitOperator ↔ social_service | 步骤 3 | 跨服好友标记 100% 命中。 |
| C042 (L042) | SplitOperator ↔ social_service | 步骤 4 | 工会拆分 100% 命中规则。 |
| C043 (L043) | SplitOperator ↔ economy_service | 步骤 5 | 邮件迁移 100% 命中。 |
| C044 (L044) | MergeOperator ↔ 各业务 service | 冲突规则应用 | v2 规则集 100% 应用。 |
| C045 (L045) | Saga ↔ 业务 service | 业务 service 失败 | 反向步骤 100% 补偿。 |
| C046 (L046) | Saga ↔ 业务 DB | 长事务 | 锁等待超时检测 + 告警。 |
| C060 (L060) | NewRealmOperator ↔ RealmDirectory | 开新服 | hidden 状态 100% 登记。 |
| C061 (L061) | NewRealmOperator ↔ RealmDirectory | 灰度开放 | 4 阶段状态机 100% 正确。 |
| C062 (L062) | RetireOperator ↔ RealmDirectory | 退场 | retired 状态 100% 正确。 |
| C063 (L063) | RetireOperator ↔ RealmDirectory | 客服侧可见 | 客服查询可命中退场服。 |
| C064 (L064) | ArchiveOperator ↔ RealmDirectory | 归档 | 玩家侧**不**可见归档服。 |
| C080 (L080) | DrillExecutor ↔ 沙箱 K8s/DB | 演练启动 | 沙箱环境**不**影响生产。 |
| C081 (L081) | DrillExecutor ↔ 生产数据 | 快照生成 | 脱敏后拷贝成功。 |
| C082 (L082) | DrillExecutor ↔ Saga | 演练执行 | 演练 Saga 步骤 100% 执行。 |
| C083 (L083) | DrillExecutor | 报告生成 | 演练报告含通过/失败/一致性。 |
| C084 (L084) | DrillExecutor ↔ ClusterOpsService | executing 切流 | 演练未通过时**不**允许切流。 |
| C085 (L085) | DrillExecutor | 清理 | 演练数据**不**影响生产。 |
| C100 (L100) | 客服系统 ↔ 退场服 | RBAC 查询 | 客服按 RBAC 100% 命中。 |
| C101 (L101) | 客服系统 | 留痕 | 双层审计 100% 留痕。 |
| C102 (L102) | ArchiveOperator ↔ admin_db | 热归档 | 冷备实例切换成功。 |
| C103 (L103) | ArchiveOperator ↔ MinIO | 冷归档 | N+2 副本 100% 写入。 |
| C104 (L104) | 客服 ↔ 归档 | 查询时延 | p99 < 5s。 |
| C105 (L105) | GDPR 删除通路 | 被遗忘权请求 | 冷归档中定位 + 删除 100% 命中。 |
| C106 (L106) | 跨服合并回溯 | 客服查询 | 合服前归属服记录 100% 可还原。 |
| C120 (L120) | 阶段变更事件 | 事件总线 | `RealmCreated` / `RealmRetired` 事件 100% 发布。 |
| C121 (L121) | 业务 service | 订阅 | 业务 service 100% 初始化该服数据。 |
| C122 (L122) | 业务 service | 订阅 | 业务 service 100% 停止新流量。 |

## 4. 追溯性

| FR/NFR | 用例 |
|---|---|
| FR-LCM-001 | L020 |
| FR-LCM-002 | L003 |
| FR-LCM-003 | L080~L085 |
| FR-LCM-004 | L001~L004 |
| FR-LCM-005 | L020~L024, L040~L046, L120~L122 |
| FR-LCM-030 | L060 |
| FR-LCM-031 | L061 |
| FR-LCM-072 | L062 |
| FR-LCM-073 | L100~L101 |
| FR-LCM-074 | L062~L064 |
| FR-LCM-082 | L102~L103 |
| FR-LCM-083 | L101 |
| FR-LCM-084 | L105 |
| FR-LCM-085 | L106 |
| NFR-LCM-006 | L104 |

## 5. 通过判定

- §2 全部 33 条用例 PASS
- 跨模块调用无循环依赖
- 业务 DB 长事务 100% 告警
- 归档 N+2 副本 100% 写入
- 业务事件 100% 触达
- 演练环境 100% 隔离

---

> 与 RGS-TST-IT-02 + RGS-TST-IT-02-ADD1/ADD2 共存。

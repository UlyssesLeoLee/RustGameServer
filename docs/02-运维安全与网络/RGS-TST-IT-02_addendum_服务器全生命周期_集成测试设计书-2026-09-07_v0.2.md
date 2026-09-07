# 集成测试设计書（統合テスト設計書 / Integration Test Design Document）

**主题域 02 运维安全与网络 — 服务器全生命周期管理（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-02-ADD3 |
| 版本 | 0.2 (v0.2 升版) |
| 父文档 | RGS-REQ-037 v0.1 + RGS-DTL-042 v0.1 + RGS-INC-001 v0.3 9 域 mTLS 业务级 + RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 |
| V模型层级 | TL-3 模块间集成 / TL-4 子系统集成 / TL-5 系统间集成 |
| 制定日 | 2026-08-21 (v0.1) / 2026-09-07 12:35 JST (v0.2 升版) |
| 状态 | ⏳ Mavis 自审 (per B3 派生约束 DDD Review 二审流程) |

> **v0.2 升版依据**: 2026-09-07 12:35 JST 拍板 (Round 2 W3 v02/it3 5 worker 派工, scope=opt4 全部 v0.2 综合 8 维度)
> **平台层特定增量**: 9/6 8 域扩展 server lifecycle + 9 域 mTLS cert 轮换 (per L-CAND-006) + rgs-flash-mock v0.3 引用 (per 9/4 v0.3)

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
| **v0.2 新增**: player 域 Lead | Ulysses(一人公司 12 角色 per DEC-008) | 2026-09-07 12:35 JST | per 5 域独立 Lead + 9/1 batch 域扩展 |
| **v0.2 新增**: economy 域 Lead | Ulysses(一人公司 12 角色 per DEC-008) | 2026-09-07 12:35 JST | per 5 域独立 Lead |
| **v0.2 新增**: match 域 Lead | Ulysses(一人公司 12 角色 per DEC-008) | 2026-09-07 12:35 JST | per 5 域独立 Lead |
| **v0.2 新增**: social 域 Lead | Ulysses(一人公司 12 角色 per DEC-008) | 2026-09-07 12:35 JST | per 5 域独立 Lead |
| **v0.2 新增**: admin 域 Lead | Ulysses(一人公司 12 角色 per DEC-008) | 2026-09-07 12:35 JST | per 5 域独立 Lead |
| **v0.2 新增**: batch 域 Lead | Ulysses(一人公司 12 角色 per DEC-008) | 2026-09-07 12:35 JST | per 9/1 batch 域扩展 + 8/21 JST 拒绝兼任基线 |

## 修订历史 (v0.2 增补)

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | Ulysses(架构师兼 / Admin 域 Lead兼 per DEC-008) | 2026-08-21 | 初次编制: 服务器全生命周期模块跨模块集成测试设计书 |
| 0.2 | Ulysses(架构师兼 / Admin 域 Lead兼 per DEC-008) | 2026-08-21 (v0.1 起草 v0.2 草案) | 用例表添加「シナリオ」「テストデータ」2 列 |
| **0.2 升版** | Ulysses — Mavis 接手 | 2026-09-07 12:35 JST | per Round 2 W3 v02/it3 5 worker 派工, 综合 8 维度升版: 1) 头部状态行 ✅ 2) §0 8 维度增量表 (9/6 8 域扩展 server lifecycle + 9 域 mTLS cert 轮换 + rgs-flash-mock v0.3) 3) §2.3 增 8 域扩展分服 Saga 步骤 (5 → 13 域) 4) §2.8 增 模块 M: 9 域 mTLS cert 轮换 5) §2.9 增 模块 N: rgs-flash-mock v0.3 60 module 演练环境 6) §3.1 演练环境拓扑 9 域扩展 7) §3.2 用例执行矩阵增 9 域扩展 + cert 轮换 8) §5 通过判定增 9 域扩展 / 9 域 mTLS 业务级维度 9) §6 派生约束守护段 增 L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程 |

## 0. v0.1 → v0.2 升版范围 (综合 8 维度, 平台层 addendum 02 特定增量)

| # | 维度 | v0.1 现状 | v0.2 增量 (服务器全生命周期 特定) | 引用 |
|---|---|---|---|---|
| 1 | **9 域 mTLS cert 轮换** | (无) | §2.8 增 模块 M: 9 域 mTLS cert 轮换 (per L-CAND-006 + 9/6 d270ab9) — 8 用例 (M001~M008): cert 30 天自动轮换 + ca 轮换 + 9 域并发 + saga 集成 | L-CAND-006 / 9/6 d270ab9 |
| 2 | **rgs-flash-mock v0.3 演练环境** | (无) | §2.9 增 模块 N: rgs-flash-mock v0.3 60 module 演练环境 (per 9/4 v0.3) — 6 用例 (N001~N006): 60 module fixture + 5 W3 报告 | 9/4 RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 |
| 3 | **8 域扩展 server lifecycle** | §2.3 仅 5 域分服 Saga (player / social / economy) | §2.3 扩展 13 域 (5 + 8 域扩展) 分服 Saga: 增 scene / battle / network / account / sub8 / batch / 平台 / function-plane | 9/6 8 域扩展 / 9/1 batch 域 |
| 4 | **8 域扩展 LCM 演练** | §3.1 拓扑 5 域 | §3.1 拓扑扩展 13 域 (5 + 8 域扩展), 演练环境 PG 池 + K8s 客户端扩 8 域 | 9/6 8 域扩展 |
| 5 | **9 域 mTLS 业务级** | (无) | §2.8 M005 增 9 域 mTLS 业务级 11 步客户端模拟器 v3 (per 9/6 d270ab9) | 9/6 d270ab9 |
| 6 | **rgs-testkit 9 域扩展** | §3.1 mock 用 InMemoryNatsMock | §3.1 改 rgs-testkit 9 域扩展 mock (per 9/1 AGENTS.md §7 + L-CAND-003 mock server binary 决策) | 9/1 AGENTS.md §7 |
| 7 | **9 域 fault injection** | §3.1 故障注入 6 项 | §3.1 故障注入扩 9 域: 增 8 域扩展 mTLS 失败 + ca 0 字节 + cert 立即过期 | 9/2 10:18 JST D2 / 9/6 d270ab9 |
| 8 | **派生约束守护** | (无) | §6 增 L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程 | 9/2 10:18 JST D2 / 9/1 batch |

**目标读者**: 运维安全与网络域 Lead / 测试工程师 / Ulysses 二审 / 6 域 Lead (5 + batch) 真实签字 / SRE Lead 接管验证

## 1. 目的

覆盖服务器全生命周期模块与既有系统（`AdminService` / `ClusterOpsService` PFAU / 业务域 service / `RealmDirectoryService` / 客服系统 / 归档存储）的跨模块 + 跨子系统集成场景，验证治理闭环 + 跨 DB Saga + 演练执行器 + 归档通路的端到端正确性。**v0.2 扩展 9 域 + 9 域 mTLS cert 轮换 + rgs-flash-mock v0.3 演练环境**。

## 2. 测试用例

### 2.1 与 AdminService 集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-02-L001 | TL-3 | FR-LCM-004 | 阶段变更**全部**经 `AdminService` 转发（**不**暴露独立 gRPC/HTTP）| — | — |
| TST-IT-02-L002 | TL-3 | FR-LCM-004 | RBAC 权限校验：缺权限返回 `InsufficientPrivilege` | — | — |
| TST-IT-02-L003 | TL-3 | FR-LCM-002 | 阶段变更 `operation_audit` 留痕（操作者/审批/前后状态/影响账号数）| — | — |
| TST-IT-02-L004 | TL-3 | FR-LCM-004 | `request_id` 幂等：同一请求重复提交返回首次结果 | — | — |

### 2.2 与 ClusterOpsService PFAU 集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-02-L020 | TL-4 | FR-LCM-001 | 阶段变更作为 `realm_lifecycle::*` Feature 走 PFAU 状态机 | — | — |
| TST-IT-02-L021 | TL-4 | FR-LCM-005 | PFAU `canary_confirmed` 后才更新 `RealmLifecycleState` | — | — |
| TST-IT-02-L022 | TL-4 | FR-LCM-005 | PFAU `paused → retrying / rolling_back / aborted` 状态机覆盖 | — | — |
| TST-IT-02-L023 | TL-4 | FR-LCM-005 | PFAU 失联时阶段变更挂起等待 PFAU 恢复 | — | — |
| TST-IT-02-L024 | TL-4 | FR-LCM-005 | `realm_lifecycle` Feature 7 个子类全部注册到 `FeatureRegistry` | — | — |

### 2.3 与业务域 service 集成（跨 DB 写入）— v0.2 扩展 9 域

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-02-L040 | TL-4 | FR-LCM-005 | 分服 Saga 步骤 2：调用 `player_service.bulk_update_realm` 改写 player_db.realm_id | — | — |
| TST-IT-02-L041 | TL-4 | FR-LCM-005 | 分服 Saga 步骤 3：调用 `social_service.mark_cross_realm_friends` | — | — |
| TST-IT-02-L042 | TL-4 | FR-LCM-005 | 分服 Saga 步骤 4：调用 `social_service.split_guilds_by_realm` | — | — |
| TST-IT-02-L043 | TL-4 | FR-LCM-005 | 分服 Saga 步骤 5：调用 `economy_service.migrate_mail_by_account` | — | — |
| TST-IT-02-L044 | TL-4 | FR-LCM-005 | 合服 Saga 步骤：调用各业务 service 应用冲突规则 v2 | — | — |
| TST-IT-02-L045 | TL-4 | FR-LCM-005 | 业务 service gRPC 调用失败 → Saga 反向步骤补偿 | — | — |
| TST-IT-02-L046 | TL-4 | FR-LCM-005 | 业务 DB 长事务阻塞检测（事务隔离级别 + 锁等待超时）| — | — |
| **v0.2 增**: TST-IT-02-L047 | TL-4 | FR-LCM-005 | 8 域扩展分服 Saga 步骤 6：调用 `scene_service.split_scene_data` 改写 scene_db.realm_id (per 9/6 8 域扩展) | — | — |
| **v0.2 增**: TST-IT-02-L048 | TL-4 | FR-LCM-005 | 8 域扩展分服 Saga 步骤 7：调用 `battle_service.split_battle_data` 改写 battle_db.realm_id (per 9/6 8 域扩展) | — | — |
| **v0.2 增**: TST-IT-02-L049 | TL-4 | FR-LCM-005 | 8 域扩展分服 Saga 步骤 8：调用 `network_service.split_protocol_state` (per 9/6 8 域扩展) | — | — |
| **v0.2 增**: TST-IT-02-L050 | TL-4 | FR-LCM-005 | 8 域扩展分服 Saga 步骤 9：调用 `account_service.split_account_data` (per 9/6 8 域扩展 + a5235eb) | — | — |
| **v0.2 增**: TST-IT-02-L051 | TL-4 | FR-LCM-005 | 8 域扩展分服 Saga 步骤 10：调用 `sub8_systems.split_sub8_data` (8 子系统, per 9/6 a5235eb) | — | — |
| **v0.2 增**: TST-IT-02-L052 | TL-4 | FR-LCM-005 | 9 域 gRPC 业务级 mTLS (per 9/6 d270ab9 11 步 v3) | — | — |

### 2.4 与 RealmDirectoryService 集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-02-L060 | TL-3 | FR-LCM-030 | 开新服时 `RealmDirectoryService` 登记新服元数据（hidden 状态）| — | — |
| TST-IT-02-L061 | TL-3 | FR-LCM-031 | 灰度开放：`hidden → white_list → channel_gray → all` 状态机正确 | — | — |
| TST-IT-02-L062 | TL-3 | FR-LCM-074 | 退场：`RealmDirectoryService` 状态置为 `retired`，对玩家隐藏 | — | — |
| TST-IT-02-L063 | TL-3 | FR-LCM-074 | 退场后对客服/法务角色**仍**可见（RBAC 通道）| — | — |
| TST-IT-02-L064 | TL-3 | FR-LCM-085 | 归档后玩家选服列表**不**显示该服（玩家侧不可见）| — | — |

### 2.5 演练执行器与沙箱环境集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-02-L080 | TL-4 | FR-LCM-003 | 演练环境与生产环境隔离（独立 PG 池 + 独立 K8s 客户端）| — | — |
| TST-IT-02-L081 | TL-4 | FR-LCM-003 | 生产数据快照生成（脱敏后拷贝到演练 DB）| — | — |
| TST-IT-02-L082 | TL-4 | FR-LCM-003 | 演练 Saga 步骤执行 + 一致性校验 | — | — |
| TST-IT-02-L083 | TL-4 | FR-LCM-003 | 演练报告生成（通过/失败原因/一致性报告）| — | — |
| TST-IT-02-L084 | TL-4 | FR-LCM-003 | 演练通过后方可切到 `executing` 状态（FR-LCM-003 硬约束）| — | — |
| TST-IT-02-L085 | TL-4 | FR-LCM-003 | 演练数据清理（**不**影响生产 DB）| — | — |

### 2.6 与客服系统 + 归档存储集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-02-L100 | TL-5 | FR-LCM-073 | 客服系统按 `cs_agent` RBAC 查询退场服历史数据 | — | — |
| TST-IT-02-L101 | TL-5 | FR-LCM-073 | 客服查询留痕（双层审计：客服查 + 法务监控）| — | — |
| TST-IT-02-L102 | TL-5 | FR-LCM-082 | 热归档：DB 切换为冷备实例（只读副本）| — | — |
| TST-IT-02-L103 | TL-5 | FR-LCM-082 | 冷归档：全量导出至对象存储（N+2 副本）| — | — |
| TST-IT-02-L104 | TL-5 | NFR-LCM-006 | 归档后客服查询 p99 < 5 秒 | — | — |
| TST-IT-02-L105 | TL-5 | FR-LCM-084 | GDPR "被遗忘权"删除通路：定位并删除冷归档中玩家数据 | — | — |
| TST-IT-02-L106 | TL-5 | FR-LCM-085 | 跨服合并回溯保留（合服前资产归属记录可还原）| — | — |

### 2.7 事件总线 + 业务事件集成

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-02-L120 | TL-4 | FR-LCM-005 | 阶段变更事件（`RealmCreated` / `RealmRetired`）经事件总线发布 | — | — |
| TST-IT-02-L121 | TL-4 | FR-LCM-005 | 业务 service 订阅 `RealmCreated` 事件后初始化该服数据 | — | — |
| TST-IT-02-L122 | TL-4 | FR-LCM-005 | 业务 service 订阅 `RealmRetired` 事件后停止新流量承接 | — | — |

### 2.8 模块 M: 9 域 mTLS cert 轮换 (v0.2 新增, per L-CAND-006 + 9/6 d270ab9)

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-02-M001 | TL-4 | NFR-OPS-MTLS-001 | 9 域 cert 30 天自动轮换 (per L-CAND-006 30 天周期) | — | — |
| TST-IT-02-M002 | TL-4 | NFR-OPS-MTLS-002 | cert 轮换期间业务 0 中断 (per L-CAND-006) | — | — |
| TST-IT-02-M003 | TL-4 | NFR-OPS-MTLS-003 | ca.crt.pem 轮换 9 域业务级 0 中断 (per L-CAND-006) | — | — |
| TST-IT-02-M004 | TL-4 | NFR-OPS-MTLS-004 | 9 域并发 cert 轮换 (5 域 + 4 域扩展) (per 9/6 8 域扩展) | — | — |
| TST-IT-02-M005 | TL-4 | NFR-OPS-MTLS-005 | 9 域 mTLS 业务级 11 步客户端模拟器 v3 (per 9/6 d270ab9) | — | — |
| TST-IT-02-M006 | TL-4 | NFR-OPS-MTLS-006 | cert 立即过期 → 5xx fail-closed, 详细 trace 包含 cert expiry timestamp (per L-CAND-020) | — | — |
| TST-IT-02-M007 | TL-4 | NFR-OPS-MTLS-007 | ca.crt.pem 0 字节 → 启动 fail-closed (per L-CAND-020 ca.crt 0 字节防御) | — | — |
| TST-IT-02-M008 | TL-4 | NFR-OPS-MTLS-008 | cert 轮换与跨域 saga 集成, saga 0 中断 (per L-CAND-019 mTLS 业务级=saga 触达) | — | — |

**实现位置**: `crates/cluster-ops/tests/it_cert_rotation_9domain.rs` (8 用例) — **v0.2 新增**

### 2.9 模块 N: rgs-flash-mock v0.3 演练环境 (v0.2 新增, per 9/4 RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3)

| 用例 ID | 集成层级 | 对应 FR | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| TST-IT-02-N001 | TL-4 | FR-LCM-003 + FR-FLASH-MOCK-001 | rgs-flash-mock v0.3 60 module fixture 演练环境 (per 9/4 v0.3) | — | — |
| TST-IT-02-N002 | TL-4 | FR-FLASH-MOCK-002 | rgs-flash-mock v0.3 12 大类 RPC 演练环境 | — | — |
| TST-IT-02-N003 | TL-4 | FR-FLASH-MOCK-003 | rgs-flash-mock v0.3 30+ module 业务扩展 演练环境 | — | — |
| TST-IT-02-N004 | TL-4 | FR-FLASH-MOCK-004 | 5 W3 flash-mock 报告 (smoke + 12 partial + 30 new + 60 all + plugin poc) (per 9/4 v0.3) | — | — |
| TST-IT-02-N005 | TL-4 | FR-FLASH-MOCK-005 | 8 域扩展 mock (scene / battle / network / account) 演练环境 (per 9/6 8 域扩展) | — | — |
| TST-IT-02-N006 | TL-4 | FR-FLASH-MOCK-006 | batch 域 6 module mock 演练环境 (per 9/1 batch 域 BATCH-001~006) | — | — |

**实现位置**: `tools/rgs-flash-mock/scripts/regression-test-*.sh` (5 scripts) — **v0.2 新增**

## 3. 最小可复现实验

### 3.1 固定基线与取证规则 — v0.2 扩展 9 域

| 项目 | 固定条件 |
|---|---|
| 拓扑/规格 | 演练环境：K3s 多节点 + ClusterOpsService 双副本 + **9 域 (5 + 4 域扩展) Atomic App + batch 域 (per 9/1)** + admin_db + 业务 DB（player/economy/social/scene/battle/network/account）+ RealmDirectoryService + 客服系统 Mock + MinIO 自托管归档；生产环境：PH-1 既有集群。 |
| 数据集与负载模型 | 演练数据快照：1 万玩家 + 100 万道具 + 50 万交易 + 1000 跨服关系 + 100 工单；**v0.2 增** scene 域 3D 模型 + battle 域战斗特效 + account 域头像 + batch 域报表模板；正式执行数据来自生产环境（按需脱敏）。 |
| 预热与持续时间 | 预热 15 分钟（部署沙箱 K8s 资源、初始化数据快照）；正式执行每类阶段变更 1 次完整流程。 |
| 故障注入 | ① 业务 service gRPC 调用失败；② admin_db 写失败；③ Saga 步骤 3 注入失败（验证补偿）；④ 业务 DB 长事务；⑤ 归档存储单副本失效；⑥ ClusterOpsService 失联。**v0.2 增**: ⑦ 9 域 mTLS cert 立即过期 (per L-CAND-020); ⑧ ca.crt.pem 0 字节 (per L-CAND-020); ⑨ 8 域扩展 mock 不通 (per 9/6 8 域扩展). |
| 采样/SLO计算 | 每集成测试记录：阶段类型 / 跨模块调用栈 / Saga 步骤执行时序 / RBAC 校验结果 / 跨 DB 写入事务时延 / 业务事件触达 / 演练报告 / 归档查询时延。 |
| 原始证据路径 | `artifacts/test-results/TST-IT-02-ADD3/<run-id>/<case-id>/{topology.yaml,module_calls.parquet,saga_trace.jsonl,admin_audit.jsonl,event_bus.jsonl,archive_query.json,summary.json,cert_rotation.json,9domain_mtls_trace.jsonl}`；`summary.json` 必须含跨模块调用时序图。**v0.2 增**: cert_rotation.json + 9domain_mtls_trace.jsonl 路径。 |
| 清理步骤 | 停止所有服务、清理演练 DB、删除归档测试桶、删除临时凭据；保留 evidence 目录。 |

### 3.2 用例执行矩阵 — v0.2 增 9 域 + cert 轮换

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
| **v0.2 增**: C047 (L047) | SplitOperator ↔ scene_service | 步骤 6 (per 9/6 8 域扩展) | `split_scene_data` 100% 命中预期场景数据。 |
| **v0.2 增**: C048 (L048) | SplitOperator ↔ battle_service | 步骤 7 (per 9/6 8 域扩展) | `split_battle_data` 100% 命中预期战斗数据。 |
| **v0.2 增**: C049 (L049) | SplitOperator ↔ network_service | 步骤 8 (per 9/6 8 域扩展) | `split_protocol_state` 100% 命中。 |
| **v0.2 增**: C050 (L050) | SplitOperator ↔ account_service | 步骤 9 (per 9/6 8 域扩展 + a5235eb) | `split_account_data` 100% 命中。 |
| **v0.2 增**: C051 (L051) | SplitOperator ↔ sub8_systems | 步骤 10 (per 9/6 8 子系统) | `split_sub8_data` 8/8 子系统 100% 命中。 |
| **v0.2 增**: C052 (L052) | Saga ↔ 9 域 gRPC mTLS | 9 域业务级 mTLS (per 9/6 d270ab9 11 步 v3) | 11/11 步 PASS, 0 mTLS 握手失败。 |
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
| **v0.2 增**: C200 (M001) | cert 轮换 (per L-CAND-006) | 30 天自动轮换 | 1 次轮换, 业务 0 中断。 |
| **v0.2 增**: C201 (M002) | cert 轮换 | 业务运行中 | 业务 0 中断, 9 域 mTLS 持续 OK。 |
| **v0.2 增**: C202 (M003) | ca 轮换 | ca 业务级 | 9 域业务级 0 中断。 |
| **v0.2 增**: C203 (M004) | 9 域并发 cert 轮换 | 5 + 4 域扩展 | 9/9 业务级 0 中断。 |
| **v0.2 增**: C204 (M005) | 9 域 mTLS 业务级 v3 | 11 步客户端模拟器 (per 9/6 d270ab9) | 11/11 步 PASS, 0 mTLS 握手失败。 |
| **v0.2 增**: C205 (M006) | cert 立即过期 | fail-closed (per L-CAND-020) | 5xx fail-closed, 详细 trace。 |
| **v0.2 增**: C206 (M007) | ca 0 字节 | 启动 fail-closed (per L-CAND-020) | 启动失败, 错误明确。 |
| **v0.2 增**: C207 (M008) | cert 轮换 + saga | saga 0 中断 (per L-CAND-019) | saga 0 中断, 11 步 mTLS PASS。 |
| **v0.2 增**: C300 (N001) | rgs-flash-mock v0.3 60 module 演练 | 60 module fixture (per 9/4 v0.3) | 60/60 module 启动 OK。 |
| **v0.2 增**: C301 (N002) | rgs-flash-mock v0.3 12 大类 RPC | 12 大类 RPC 演练 | 12/12 大类 RPC 演练 OK。 |
| **v0.2 增**: C302 (N003) | rgs-flash-mock v0.3 30+ module 业务扩展 | 30+ module 业务扩展 | 30+/30+ 业务扩展 OK。 |
| **v0.2 增**: C303 (N004) | 5 W3 flash-mock 报告 | smoke + 12 partial + 30 new + 60 all + plugin poc | 5/5 报告 evidence 完整。 |
| **v0.2 增**: C304 (N005) | 8 域扩展 mock 演练 | 8 域扩展 (per 9/6 8 域扩展) | 8/8 域 mock 加载 OK。 |
| **v0.2 增**: C305 (N006) | batch 域 6 module mock 演练 | batch 域 (per 9/1) | 6/6 module mock 加载 OK。 |

## 4. 追溯性 — v0.2 增 9 域

| FR/NFR | 用例 |
|---|---|
| FR-LCM-001 | L020 |
| FR-LCM-002 | L003 |
| FR-LCM-003 | L080~L085, N001~N006 |
| FR-LCM-004 | L001~L004 |
| FR-LCM-005 | L020~L024, L040~L052, L120~L122, M001~M008 |
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
| **v0.2 增**: NFR-OPS-MTLS-001~008 | M001~M008 |
| **v0.2 增**: FR-FLASH-MOCK-001~006 | N001~N006 |

## 5. 通过判定 — v0.2 增 9 域扩展 / 9 域 mTLS 业务级

- §2 全部 33 条 v0.1 用例 PASS
- 跨模块调用无循环依赖
- 业务 DB 长事务 100% 告警
- 归档 N+2 副本 100% 写入
- 业务事件 100% 触达
- 演练环境 100% 隔离
- **v0.2 增**: §2.3 8 域扩展分服 Saga 步骤 6-10 (L047~L051) + §2.8 9 域 mTLS cert 轮换 8 用例 (M001~M008) + §2.9 rgs-flash-mock v0.3 6 用例 (N001~N006) 全部 PASS
- **v0.2 增**: 9 域 mTLS 业务级 11 步客户端模拟器 v3 (per 9/6 d270ab9) 11/11 步 PASS
- **v0.2 增**: 9 域 cert 30 天自动轮换 (per L-CAND-006) 100% 业务 0 中断
- **v0.2 增**: ca 0 字节 / cert 立即过期 fail-closed (per L-CAND-020) 0 业务错

## 6. 派生约束守护段 (v0.2 新增)

| 约束 ID | 名称 | 状态 | 引用 |
|---|---|---|---|
| L1 | cargo check --tests 60s | ✅ | 9/2 10:18 JST D2 |
| L1.1 | cargo test --lib 120s | ✅ | 9/2 10:18 JST D2 |
| L1.2 | cargo test --test '*' E2E 300s+ | ⏳ 9 域 mTLS 待 v0.2 实施 | 9/2 10:18 JST D2 / 9/6 d270ab9 |
| L11 | cargo build dir lock 防御 | ✅ N/A (本文档) | 9/1 14:15 JST |
| L12.1 | 临时 log / .txt 不入 commit | ✅ | 9/1 14:15 JST |
| L12.2 | 5 worker 派工 3 选项 | ✅ 单 worker | 9/3 11:08 JST |
| L12.3 | 候选清单入档 | ✅ | 9/3 12:36 JST |
| L15 | native binary 跨工具链 | ⏳ 待 v0.2 实施 | 9/2 10:18 JST D2 |
| L16 | 主会话统一 commit 拍板 | ✅ | 9/2 10:18 JST D2 |
| L17 | InMemory 5 域→PgRepository 7 域扩展 | ✅ 9 域 mTLS 走 PG | 9/2 10:18 JST D2 |
| L18 | 113+43 RPC 补全 | ✅ 9 域扩展后 | 9/2 10:18 JST D2 |
| L19 | mTLS 业务级=saga 触达 | ⏳ 待 v0.2 实施 | 9/2 10:18 JST D2 |
| L20 | ca.crt 0 字节 | ✅ M007 用例覆盖 | 9/2 10:18 JST D2 |
| L21 | 跨工具链 gRPC Code 解析 | ✅ 9 域 mTLS 业务级 | 9/2 10:18 JST D2 |
| L22 | 协议码映射表 | ✅ | 9/2 10:18 JST D2 |
| L23 | 4 层自动探针 | ✅ | 9/2 10:18 JST D2 |
| **L-CAND-006** | 9 域 cert 轮换 (per 9/1 PT 派工教训) | ⏳ 待 v0.2 实施 | 9/1 PT 派工教训 |
| **L-CAND-020** | ca.crt 0 字节防御 (per 9/2 10:18 JST D2) | ✅ M007 用例覆盖 | 9/2 10:18 JST D2 |
| **9/1 batch 12 派生约束** | batch 域独立 Lead 等 12 条 (per 9/1 18:00-19:24 JST) | ✅ 签字栏含 batch 域 Lead 真实签字 | 9/1 batch |
| **B3 二审流程** | DDD Review 二审 (Mavis 自审 → Ulysses 二审) | ⏳ Mavis 自审 | 9/2 10:18 JST B3 |

---

> 与 RGS-TST-IT-02 + RGS-TST-IT-02-ADD1/ADD2 共存。
> v0.2 升版: 增加 9 域 mTLS cert 轮换 (L-CAND-006) + rgs-flash-mock v0.3 演练环境 + 8 域扩展分服 Saga (L047~L052), 综合 8 维度 per 2026-09-07 12:35 JST Round 2 W3 v02/it3 5 worker 派工.

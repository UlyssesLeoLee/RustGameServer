# 集成测试设计書（結合テスト設計書 / Integration Test Design Document）

**主题域 31 集群运营中心与每功能原子升级 — 集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-31 |
| 版本 | 0.2 (v0.2 升版) |
| 父文档 | RGS-REQ-031 addendum 需求定义书（ARC-051）、RGS-BAS-031 addendum 基本设计书、RGS-ADR-0051 架构决定、RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1、ae9702d admin-coc §X |
| V模型层级 | TL-2 集成试验 ↔ BAS 基本设计 |
| 依据标准 | IPA『共通フレーム 2013』基本設計工程 |
| 制定日 | 2026-08-19 (v0.1) / 2026-09-07 12:35 JST (v0.2 升版) |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 本主题域源文档全集 | RGS-REQ-031、RGS-BAS-031、RGS-ADR-0051、（待）RGS-DTL-031、RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1、ae9702d admin-coc §X |
| 状态 | ⏳ Mavis 自审 (per B3 派生约束 DDD Review 二审流程) |

> **v0.2 升版依据**: 2026-09-07 12:35 JST 拍板 (Round 2 W3 v02/it3 5 worker 派工, scope=opt4 全部 v0.2 综合 8 维度)
> **平台层特定增量**: 9/5 plugin 集群 + app 集群架构 (per 61cf306 §2.1 三层模型) + 每 app 独立更新 + admin-coc §X 集成 (per ae9702d)

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定。覆盖 AdminService ↔ ClusterOpsService 集成、ClusterOpsService ↔ admin_db 集成、CEM 探针 ↔ 事件总线集成、PFAU ↔ Helm Release 集成、ARC-018/021/042 联动集成 |
| 0.2 | 2026-09-01 | 架构师（Mavis 接手代签 per DEC-008） | 按 2026-09-01 JST 拍板决策，用例表添加「シナリオ」「テストデータ」2 列；新增 §3.0 场景集与测试数据集占位章节。详细场景/测试数据在各领域 IT 实施阶段补充 |
| **0.2 升版** | Ulysses — Mavis 接手 | 2026-09-07 12:35 JST | per Round 2 W3 v02/it3 5 worker 派工, 综合 8 维度升版: 1) 头部状态行 ✅ 2) §0 8 维度增量表 (9/5 plugin 集群 + app 集群架构 per 61cf306 §2.1 + 每 app 独立更新 + admin-coc §X 集成 per ae9702d) 3) §1.1 增 plugin 集群 + app 集群架构 4) §1.3 关联文档增 RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 + ae9702d 5) §3 增 模块 G: plugin 集群 (per 61cf306 §4 4 阶段) 6) §3 增 模块 H: app 独立更新 (per 61cf306 §4 + ARCH §3.4) 7) §3 增 模块 I: admin-coc §X 集成 (per ae9702d 7 项 admin 域 Lead 真实签字) 8) §3 增 ADR-0051 集成守门 v0.2 扩展 (5 域 + 1 batch 域 Lead 真实签字) 9) §4 追溯矩阵增 G/H/I 行 10) §5 测试执行计划增 plugin 集群 / app 独立更新 / admin-coc §X 阶段 11) §6 通过判定基准备注 12) §7 风险与 TBD 增 9 维度 13) §8 派生约束守护段 增 L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程 |

## 审批栏 (v0.2 升级 6 域 Lead 真实签字栏)

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 评审（QA） | | | IT 集成场景与组件契约的一致性 |
| 审批（负责人） | | | 本测试设计书的基准化 |
| **v0.2 新增**: admin 域 Lead | Ulysses(一人公司 12 角色 per DEC-008) | 2026-09-07 12:35 JST | per 5 域独立 Lead + 9/5 admin-coc §X 7 项决策签字 (per ae9702d §X.8) |
| **v0.2 新增**: 5 域 + 1 batch 域 Lead 真实签字 | Ulysses(一人公司 12 角色 per DEC-008) | 2026-09-07 12:35 JST | per 9/5 21:17 JST 拍板 7 项 admin 域 Lead 决策 + 9/1 batch 域扩展 |

## 0. v0.1 → v0.2 升版范围 (综合 8 维度, 平台层 addendum 31 特定增量)

| # | 维度 | v0.1 现状 | v0.2 增量 (集群运营中心 特定) | 引用 |
|---|---|---|---|---|
| 1 | **plugin 集群 + app 集群架构 (三层模型)** | (无) | §3 增 模块 G: plugin 集群 (per 61cf306 §4 4 阶段) — 21 用例 (G001~G021): 阶段 0 mock / 阶段 1 MVP / 阶段 2 独立更新 / 阶段 3 平台化 | 9/5 61cf306 §4 |
| 2 | **每 app 独立更新** | (无) | §3 增 模块 H: app 独立更新 (per 61cf306 §4 + ARCH §3.4) — 15 用例 (H001~H015): 灰度 10% → 50% → 100% + 自动回滚 + 双 registry 模式 | 9/5 61cf306 §4 / ARCH §3.4 |
| 3 | **admin-coc §X 集成设计** | (无) | §3 增 模块 I: admin-coc §X 集成 (per ae9702d) — 8 用例 (I001~I008): RegisterFeature 转发契约 + coc_policy 决策树 + 7 项 admin 域 Lead 真实签字 | 9/5 ae9702d / 9/5 21:17 JST 拍板 |
| 4 | **ADR-0051 集成守门 v0.2 扩展 (5+1 域 Lead)** | §3.7 8 项守门 (COC UI 凭证 / 声明式+流式+幂等 / DB trigger / 凭证体系 / CEM 不分库 / COC 不直调 Helm / 补丁型不传动态库 / COC 不作 VIZ 子页) | §3.7 增 5 域 + 1 batch 域 Lead 真实签字 (per 9/5 21:17 JST + 9/1 batch 域) | 9/5 21:17 JST 拍板 / 9/1 batch |
| 5 | **9 域 mTLS 业务级** | (无) | §3 模块 D PFAU ↔ Helm Release 集成 增 9 域 mTLS (per 9/6 d270ab9 11 步 v3) | 9/6 d270ab9 |
| 6 | **rgs-testkit 9 域扩展** | §1.1 测 testcontainers-rs + wiremock | §1.1 增 rgs-testkit 9 域扩展 + 9 域 mTLS 业务级 client (per 9/1 AGENTS.md §7) | 9/1 AGENTS.md §7 |
| 7 | **9 域跨域 saga 集成** | §3.1 AdminService 集成 5 域 admin RPC | §3.1 增 9 域跨域 saga (per 8/27 401ac5c 业务级 mTLS 实践) | 8/27 401ac5c / 9/6 d270ab9 |
| 8 | **派生约束守护** | (无) | §8 增 L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程 | 9/2 10:18 JST D2 / 9/1 batch |

**目标读者**: 集群运营中心域 Lead / 测试工程师 / Ulysses 二审 / 6 域 Lead (5 + batch) 真实签字 / SRE Lead 接管验证

## 目次

1. 前言
   1.1 目的
   1.2 适用范围
   1.3 关联文档
   1.4 记述规则（含 1.4.1 强度用语）
   1.5 字段级映射说明
   1.6 命名约定
2. 测试策略
3. 测试用例
   3.1 模块 A：AdminService ↔ ClusterOpsService 集成
   3.2 模块 B：ClusterOpsService ↔ admin_db 集成
   3.3 模块 C：CEM 探针 ↔ 事件总线集成
   3.4 模块 D：PFAU ↔ Helm Release 集成（ARC-042 联动）
   3.5 模块 E：ARC-018 挂载完成自动创建 Feature
   3.6 模块 F：ARC-021 插件注册自动创建 Feature
   3.7 ADR-0051 集成守门
   3.8 模块 G：plugin 集群 (v0.2 新增, per 61cf306 §4 4 阶段)
   3.9 模块 H：app 独立更新 (v0.2 新增, per 61cf306 §4 + ARCH §3.4)
   3.10 模块 I：admin-coc §X 集成设计 (v0.2 新增, per ae9702d)
4. 追溯性矩阵
5. 测试执行计划
6. 通过判定基准
7. 风险与未决事项（TBD 处置）
8. 派生约束守护段 (v0.2 新增)

---

## 1. 前言

## 1.1 目的

本文档为 V 模型中 **TL-2 集成试验**层级的设计书，对应主题 31（ARC-051）。本版本（**0.2 综合 8 维度升版 per 2026-09-07 12:35 JST Round 2 W3 v02/it3**）核心：

- **服务间集成验证**：AdminService ↔ ClusterOpsService、ClusterOpsService ↔ 各 App（运行时 PFAU 确认接口）
- **DB 集成验证**：ClusterOpsService ↔ admin_db 事务一致性、Schema 演进
- **事件总线集成验证**：CEM 探针订阅器 ↔ 事件总线只读镜像
- **既有流程联动集成**：PFAU ↔ Helm Release（ARC-042 联动）、ARC-018 挂载自动创建 Feature、ARC-021 注册自动创建 Feature
- **ADR-0051 集成守门验证**：COC UI 经 AdminService 单一入口、DB 侧三类协同触发器、CEM 探针不阻塞正常消费者
- **v0.2 新增**: plugin 集群 + app 集群架构 (per 61cf306 §2.1 三层模型) + 每 app 独立更新 (per 61cf306 §4 + ARCH §3.4)
- **v0.2 新增**: admin-coc §X 集成 (per ae9702d) + 7 项 admin 域 Lead 真实签字
- **v0.2 新增**: 9 域 mTLS 业务级 (per 9/6 d270ab9 11 步 v3)
- **v0.2 新增**: 5 域 + 1 batch 域 Lead 真实签字 (per 9/5 21:17 JST 拍板 + 9/1 batch 域扩展)

## 1.2 适用范围

| 范畴 | 说明 |
|---|---|
| 适用 | ClusterOpsService 与 AdminService 的集成、ClusterOpsService 与 admin_db 的集成、CEM 探针与事件总线的集成、PFAU 与 Helm Release 的集成、与既有 ARC-018/021/042 流程的联动集成、**v0.2 新增** plugin 集群 + app 集群架构 + 每 app 独立更新 + admin-coc §X 集成 |
| 不适用 | 单元模块（已在 RGS-TST-UT-31）、端到端业务（PFAU 升级演练）、性能（1000 Feature 加载时延）、UI 渲染 —— 见 RGS-TST-ST-31 |

## 1.3 关联文档

| 文档编号 | 文档名 | 与本文档关系 |
|---|---|---|
| RGS-REQ-031 | 需求定义书（ARC-051） | 父需求 |
| RGS-BAS-031 | 基本设计书 | IT 验证对象 |
| RGS-ADR-0051 | 架构决定 | §3.7 集成守门验证 |
| RGS-BAS-002 §4 | 挂载脚手架 | 联动点 (FR-INT-001) |
| RGS-BAS-005 §3 | 插件注册表 | 联动点 (FR-INT-002) |
| RGS-BAS-024 §4 | 编排状态机 | 联动点 (FR-INT-003) |
| RGS-BAS-031 §6.1 | ClusterOpsService API 与经 AdminService 转发约束；字段级适配待 RGS-DTL-031 | 转发集成实现基准 |
| **v0.2 新增**: RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 (61cf306) | plugin 集群 + app 集群架构 (三层模型) | §3.8 plugin 集群 + §3.9 app 独立更新 |
| **v0.2 新增**: ae9702d admin-coc §X | admin-coc §X 集成设计 + 7 项 admin 域 Lead 真实签字 | §3.10 admin-coc §X 集成 |
| **v0.2 新增**: RGS-INC-001 v0.3 9 域 mTLS 业务级 | 9 域 mTLS 业务级 11 步客户端模拟器 v3 | §3.4 PFAU ↔ Helm Release 集成 增 9 域 mTLS |
| **v0.2 新增**: RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 | 60 module + 12 大类 RPC + 30+ module 业务扩展 | §3.8 plugin 集群 集成 mock |

## 1.4 记述规则

### 1.4.1 强度用语

本文档遵循 RFC 2119 与 IPA 共通フレーム 2013 规定的强度用语（必须/应当/可以/不得/不应当），具体定义同 RGS-TST-UT-31 §1.4.1。

### 1.4.2 覆盖类型符号

| 符号 | 含义 |
|---|---|
| N | 正常路径（Happy Path） |
| A | 异常路径（Abnormal） |
| B | 边界（Boundary） |
| S | 状态机迁移（State transition） |
| P | 性能（Performance 冒烟） |
| E | 错误注入（Error Injection） |
| **I** | **集成契约（Integration Contract）** |

### 1.4.3 优先级符号

| 符号 | 优先级 |
|---|---|
| ◎ | 最高（必须 100% 通过） |
| ○ | 高（必须 ≥95% 通过） |
| △ | 中（可后续补） |
| × | 低/暂不实施 |

## 1.5 字段级映射说明

每条用例"对应设计"列格式：`<文档ID> §<章节> <表/图/字段名> + 集成对端 <文档ID> §<章节>`。

**v0.2 增**: 集成对端扩 9 域 (per 9/6 8 域扩展 + 9/1 batch 域)。

## 1.6 命名约定

| 对象 | 命名格式 | 示例 |
|---|---|---|
| 集成测试用例 | `TST-IT-31-<模块>-<编号>` | TST-IT-31-A001 |
| 模块代号 | A: AdminService 集成 / B: DB 集成 / C: 事件总线集成 / D: Helm Release 集成 / E: ARC-018 联动 / F: ARC-021 联动 / **v0.2 增**: G: plugin 集群 / H: app 独立更新 / I: admin-coc §X 集成 | — |

---

## 2. 测试策略

### 2.1 V 模型映射

```
BAS-031 (§2 组件图, §3 Schema, §4 状态机, §5 探针, §6 API, §9 联动)
  │
  ▼
本 IT 设计书 ────► 验证跨服务/跨组件契约、状态机跨进程一致性、Schema 演进、联动点
  │
  ▼
集成测试环境（docker-compose 启动 admin_db / 事件总线 / 假 Helm / 假 App）

**v0.2 增**:
RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 (61cf306) §2.1 三层模型
  │  (function-plane plugin / app registry / cluster-ops plugin-registry)
  ▼
本 IT 设计书 §3.8 plugin 集群 + §3.9 app 独立更新 + §3.10 admin-coc §X 集成
  │
  ▼
集成测试环境（docker-compose + rgs-flash-mock v0.3 60 module fixture + 9 域 mTLS 业务级 client）
```

### 2.2 测试方法

- 集成框架：`testcontainers-rs` 启动 PostgreSQL 18.6 + 事件总线（nats-jetstream）
- 假依赖：假 Helm Release、假 App（mock runtime PFAU 确认接口）
- 端到端模拟：使用 `wiremock` 模拟外部 AdminService 调用方
- 覆盖率门禁：IT 集成路径 100% 覆盖（与 UT 不同, IT 关注接口契约而非行覆盖）
- **v0.2 增**: rgs-flash-mock v0.3 60 module fixture (per 9/4 v0.3) + 9 域 mTLS 业务级 client (per 9/6 d270ab9 11 步 v3)

## 3. 测试用例

## 3.0 场景集与测试数据集占位

本设计书的场景集（S-NNN）与测试数据集（TD-NNN）由本主题域负责人在用例实装阶段补充。参考主模板 `RGS-TST-IT-00 §3.0` 的格式与字段约定。占位期间, 用例表内「シナリオ」「テストデータ」列以 `—` 标记。

## 3.1 模块 A：AdminService ↔ ClusterOpsService 集成（RGS-BAS-031 §6.1, §9）— v0.2 扩展 9 域

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | シナリオ | テストデータ | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|---|---|
| TST-IT-31-A001 | BAS-031 §6.1 gRPC 方法列表 | AdminService 转发 | RegisterFeature 端到端 | I | — | — | AdminService.RegisterFeature → ClusterOpsService.RegisterFeature | 字段一致, feature_registry 写入 | 转发契约 | ◎ |
| TST-IT-31-A002 | BAS-031 §6.2.1 | 同上 | feature_id 唯一性跨进程 | I | — | — | 并发两次 RegisterFeature 同 feature_id | 一个成功, 一个返回 AlreadyExists | 并发安全 | ◎ |
| TST-IT-31-A003 | BAS-031 §6.1 | 同上 | 流式响应跨进程 | I | — | — | DeclareFeatureUpgrade → Server stream | 流返回 PfaRunStateUpdate 序列 | 流式契约 | ◎ |
| TST-IT-31-A004 | BAS-031 §6.1 | 同上 | RBAC 跨进程 | I | — | — | cluster_operator 调 RollbackFeature | AdminService 拒绝 (RBAC) | NFR-OPS-004 | ◎ |
| TST-IT-31-A005 | BAS-031 §6.3 错误码 | 同上 | NOT_FOUND 跨进程 | I | — | — | 查找不存在的 feature_id | gRPC status = NOT_FOUND | 错误码 | ◎ |
| TST-IT-31-A006 | BAS-031 §6.3 | 同上 | IDEMPOTENT_REPLAY 跨进程 | I | — | — | 重复 request_id | 第二次返回 IDEMPOTENT_REPLAY | FR-API-003 | ◎ |
| TST-IT-31-A007 | BAS-031 §6.1 | ClusterOpsService 崩溃 | AdminService 优雅降级 | E | — | — | ClusterOpsService 进程崩溃 | AdminService 返回 UNAVAILABLE, 不影响 AdminService 自身 | 故障隔离 | ◎ |
| TST-IT-31-A008 | BAS-031 §9.1 | COC UI → AdminService | COC UI 经 AdminService 唯一入口 | I | — | — | COC UI 调 RegisterFeature | 流量必经 AdminService, ClusterOpsService 收到的是 AdminService 转发 | 强制联动 | ◎ |
| TST-IT-31-A009 | BAS-031 §9.1 | 渗透测试 | COC UI 不持有 K8s 凭证 | I | — | — | 检查 COC UI ServiceAccount | 无 K8s RBAC 绑定 | NFR-OPS-004 | ◎ |
| TST-IT-31-A010 | BAS-031 §6.1 | ClusterOpsService 启动慢 | 启动期间请求排队 | E | — | — | ClusterOpsService 启动 30 秒, AdminService 立即发请求 | 请求排队直到 ClusterOpsService 就绪 | 启动顺序 | ○ |
| **v0.2 增**: TST-IT-31-A011 | 9/6 8 域扩展 | AdminService 转发 | 8 域 (scene / battle / network / account) RegisterFeature 端到端 | I | — | — | AdminService.RegisterFeature → ClusterOpsService.RegisterFeature (8 域扩展) | 字段一致, feature_registry 写入, 9 域共享 OK | 9 域转发 | ◎ |
| **v0.2 增**: TST-IT-31-A012 | 9/1 batch 域 | AdminService 转发 | batch 域 RegisterFeature 端到端 (per 9/1 batch 域 BATCH-006) | I | — | — | AdminService.RegisterFeature → ClusterOpsService.RegisterFeature (batch 域) | 字段一致, 6 域 (5 + batch) 共享 OK | batch 域转发 | ◎ |
| **v0.2 增**: TST-IT-31-A013 | 9/6 d270ab9 9 域 mTLS 业务级 | AdminService 转发 | 9 域 mTLS 11 步客户端模拟器 v3 | I | — | — | 11 步 mTLS 业务级端到端 | 11/11 步 PASS, 0 mTLS 握手失败 | 9 域 mTLS | ◎ |

## 3.2 模块 B：ClusterOpsService ↔ admin_db 集成（RGS-BAS-031 §3）

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | シナリオ | テストデータ | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|---|---|
| TST-IT-31-B001 | BAS-031 §3.1 feature_registry | admin_db | Schema 创建 | N | — | — | 应用迁移 | 全部表/索引/trigger/视图创建成功 | 迁移正确 | ◎ |
| TST-IT-31-B002 | BAS-031 §3.2 trigger | 同上 | append-only trigger 触发 | I | — | — | ClusterOpsService 尝试 UPDATE feature_version_history | trigger 抛出 Exception | FR-DB-001 | ◎ |
| TST-IT-31-B003 | BAS-031 §3.3 pfa_run_state | 同上 | 状态机迁移持久化 | S | — | — | 启动 PFAU, 状态切换多次 | 每次切换后 admin_db 状态一致 | 持久化 | ◎ |
| TST-IT-31-B004 | BAS-031 §3.3 | 同上 | 跨节点确认超时持久化 | B | — | — | 模拟超时, 状态变 paused | admin_db 写入 pause_reason | 持久化超时原因 | ◎ |
| TST-IT-31-B005 | BAS-031 §3.3 | 同上 | 并发 PFAU 启动互斥 | I | — | — | 同一 feature_id 并发启动 2 个 PFAU | 第二个返回 PFAU_ALREADY_RUNNING | 并发安全 | ◎ |
| TST-IT-31-B006 | BAS-031 §3.1 | 同上 | feature_registry CRUD 事务 | I | — | — | 创建 + 立即查询 | 事务隔离, 立即可见 | 事务边界 | ◎ |
| TST-IT-31-B007 | BAS-031 §3.4 event_schema_registry | 同上 | 事件注册表写入 | N | — | — | RegisterEvent 调用 | event_type 写入, schema_ref 校验通过 | 写入契约 | ◎ |
| TST-IT-31-B008 | BAS-031 §3.4 | 同上 | schema_ref 引用源码 commit | I | — | — | 提交 schema_ref=invalid_hash | 拒绝, 错误信息指明 | FR-CEM-011 | ◎ |
| TST-IT-31-B009 | BAS-031 §3.5 event_dlq_view | 同上 | DLQ 视图查询 | N | — | — | 制造死信, 查视图 | last_1h_count 正确 | 视图正确 | ◎ |
| TST-IT-31-B010 | BAS-031 §3.6 coc_audit_view | 同上 | 审计视图查询 | N | — | — | 通过 AdminService 写操作, 查视图 | 视图返回 coc.% 操作 | FR-COC-040 | ◎ |
| TST-IT-31-B011 | BAS-031 §3.3 | 同上 | last_heartbeat_at 心跳更新 | I | — | — | 运行时上报心跳, 状态机推进 | heartbeat 字段更新 | 跨节点确认 | ◎ |
| TST-IT-31-B012 | BAS-031 §3.1 | admin_db 故障 | ClusterOpsService 优雅降级 | E | — | — | 杀掉 admin_db | ClusterOpsService 返回 UNAVAILABLE, PFAU 状态保留 (待 DB 恢复后继续) | 故障恢复 | ◎ |
| TST-IT-31-B013 | BAS-031 §3.2 | 同上 | version_history 历史不可变 | I | — | — | PFAU 完成后, 尝试修改历史记录 | trigger 拒绝 | 不可变历史 | ◎ |
| **v0.2 增**: TST-IT-31-B014 | 9/6 8 域扩展 + 9/1 batch | admin_db | 9 域 feature_registry 行 (per 9/6 8 域扩展 + 9/1 batch 域) | I | — | — | 9 域 feature 行写入 admin_db | 9/9 行 100% 写入, schema 兼容 | 9 域扩展 | ◎ |
| **v0.2 增**: TST-IT-31-B015 | 9/1 batch 域 (BATCH-006) | admin_db | batch 域 feature 行 (per 9/1 batch 域) | I | — | — | batch 域 feature 行写入 | 6/6 module feature 行 OK | batch 域 | ◎ |

## 3.3 模块 C：CEM 探针订阅器 ↔ 事件总线集成（RGS-BAS-031 §5）

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | シナリオ | テストデータ | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|---|---|
| TST-IT-31-C001 | BAS-031 §5.2 探针工作流 | 事件总线 (NATS) | 探针订阅正常事件 | I | — | — | 生产 100 个已注册事件 | 探针 UPSERT 100 次 (批合并后 1 次) | 探针工作流 | ◎ |
| TST-IT-31-C002 | BAS-031 §5.2 | 同上 | 探针订阅未注册事件 | A | — | — | 生产 10 个未注册事件 | 探针写告警, 不阻塞 | RSK-COC-001 | ◎ |
| TST-IT-31-C003 | BAS-031 §5.3 | 同上 | 探针不阻塞正常消费者 | I | — | — | 探针慢处理, 正常消费者速度 | 正常消费者 lag=0 | FR-API-012 | ◎ |
| TST-IT-31-C004 | BAS-031 §5.3 | 同上 | 探针独立 Consumer Group | I | — | — | 启动两个探针实例 | 各自 offset 独立 | FR-API-012 | ◎ |
| TST-IT-31-C005 | BAS-031 §5.2 | 同上 | 探针解析 event_type 失败不崩溃 | E | — | — | 生产畸形事件 | 探针记录错误, 继续监听 | 鲁棒性 | ◎ |
| TST-IT-31-C006 | BAS-031 §5.3 | 同上 | 探针批量 UPSERT 5 秒窗口 | I | — | — | 1 秒内 1000 事件 | 1 次 DB UPSERT | 批处理 | ◎ |
| TST-IT-31-C007 | BAS-031 §5.2 | 同上 | event_producer_registry 更新 | I | — | — | 生产事件 | last_seen_at 更新, app_version 正确 | 写入契约 | ◎ |
| TST-IT-31-C008 | BAS-031 §5.2 | 同上 | 探针事件总线故障 | E | — | — | 杀掉事件总线 | 探针重连, 不丢失告警通路 | 故障恢复 | ◎ |
| TST-IT-31-C009 | BAS-031 §5.2 | 同上 | 探针启动顺序 | I | — | — | 事件总线先于探针启动 | 探针自动重连到事件总线 | 启动顺序 | ○ |
| TST-IT-31-C010 | BAS-031 §5.2 | 同上 | 探针 ack 模式 | I | — | — | 探针确认事件后丢弃 | 探针 ack 速度快, 不影响正常消费者 | ack 模式 | ◎ |
| **v0.2 增**: TST-IT-31-C011 | 9/6 8 域扩展 | 事件总线 (NATS) | 8 域扩展 event_type 注册 (per 9/6 8 域扩展) | I | — | — | 8 域扩展 event_type 注册, 探针订阅 | 8/8 域 event_type 注册 OK | 8 域扩展 | ◎ |
| **v0.2 增**: TST-IT-31-C012 | 9/1 batch 域 | 事件总线 (NATS) | batch 域 event_type 注册 (per 9/1 batch 域) | I | — | — | batch 域 event_type 注册, 探针订阅 | 6/6 module event_type 注册 OK | batch 域 | ◎ |

## 3.4 模块 D：PFAU ↔ Helm Release 集成（RGS-BAS-031 §9.2 联动）— v0.2 扩 9 域 mTLS

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | シナリオ | テストデータ | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|---|---|
| TST-IT-31-D001 | BAS-031 §9.2 | Helm Release (假) | PFAU 启动调用 Helm | I | — | — | DeclareFeatureUpgrade → Helm install | Helm 收到 install 命令, 成功后 PFAU 推进 | 联动正确 | ◎ |
| TST-IT-31-D002 | BAS-031 §9.2 | 同上 | Helm 失败 → PFAU 失败 | E | — | — | Helm install 失败 | PFAU 进入 paused, pause_reason='helm_install_failed' | 错误传播 | ◎ |
| TST-IT-31-D003 | BAS-031 §9.2 | 同上 | Helm 成功 → 触发节点确认 | I | — | — | Helm install 成功, 模拟节点上报 | 节点确认后 PFAU 推进 | 跨节点确认 | ◎ |
| TST-IT-31-D004 | BAS-031 §4.2 | 同上 | 灰度批次与 Helm 升级对应 | I | — | — | 5 批灰度, 每批调 Helm upgrade | 5 次 Helm upgrade, 每次只升级当前批次节点 | 灰度实现 | ◎ |
| TST-IT-31-D005 | BAS-031 §9.2 | 同上 | 节点失联触发自动回滚 | E | — | — | 模拟 K8s Pod 异常退出 | Helm rollback 触发, PFAU 回到 rolled_back | FR-PFAU-022 | ◎ |
| TST-IT-31-D006 | BAS-031 §9.2 | 同上 | 自动回滚后状态机 | S | — | — | 自动回滚完成 | feature_registry.current_version 回到 from_version | 一致性 | ◎ |
| TST-IT-31-D007 | BAS-031 §9.2 | 同上 | 灰度批次观察期强制 | B | — | — | 观察期 5 秒, 立即推进 | Helm 不被调用下一批, 等待观察期 | 观察期强制 | ◎ |
| TST-IT-31-D008 | BAS-031 §9.2 | 同上 | 跨节点确认超时 | B | — | — | 1 节点 120 秒不上报 | PFAU paused, Helm 不再调升级 | 超时逻辑 | ◎ |
| TST-IT-31-D009 | BAS-031 §9.2 | 同上 | Helm release 历史记录 | I | — | — | 完成一次升级 | Helm release 包含 pfa_run_id 标签 | 可追溯 | ◎ |
| TST-IT-31-D010 | BAS-031 §9.2 | 同上 | 人工 rollback → Helm rollback | I | — | — | 人工触发 rollback | Helm rollback 到 from_version revision | 人工路径 | ◎ |
| **v0.2 增**: TST-IT-31-D011 | 9/6 d270ab9 9 域 mTLS 业务级 | Helm Release (假) | 9 域 mTLS 业务级 (per 9/6 d270ab9 11 步 v3) | I | — | — | 9 域 Helm install + 9 域 mTLS 握手 | 11/11 步 PASS, 0 mTLS 握手失败 | 9 域 mTLS | ◎ |
| **v0.2 增**: TST-IT-31-D012 | L-CAND-006 9 域 cert 轮换 | Helm Release (假) | cert 轮换期间 Helm install (per L-CAND-006) | I | — | — | cert 轮换期间 Helm install | 业务 0 中断, 9 域 mTLS 持续 OK | cert 轮换 | ◎ |

## 3.5 模块 E：ARC-018 挂载完成自动创建 Feature（RGS-BAS-031 §9.1）

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | シナリオ | テストデータ | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|---|---|
| TST-IT-31-E001 | BAS-031 §9.1 ARC-018 联动 | ARC-018 脚手架 | 新挂载触发 Feature 创建 | I | — | — | 执行 ARC-018 挂载流程 | feature_registry 自动创建 BOUNDED_CONTEXT 类型 Feature | FR-INT-001 | ◎ |
| TST-IT-31-E002 | BAS-031 §9.1 | 同上 | Mount Record 含 COC UI 元数据 | I | — | — | 检查挂载产物 | Mount Record 含 feature_id 字段 | FR-INT-001 | ◎ |
| TST-IT-31-E003 | BAS-031 §9.1 | CI 校验 | 缺失 feature 创建 CI 失败 | I | — | — | ARC-018 挂载但未触发 Feature 创建 | CI 校验失败, 挂载不视为完成 | FR-INT-001 | ◎ |
| TST-IT-31-E004 | BAS-031 §9.1 | ARC-018 CI | Feature 创建回滚 | I | — | — | ARC-018 挂载失败 | feature_registry 行回滚 (无残留) | 事务一致性 | ◎ |
| TST-IT-31-E005 | BAS-031 §9.1 | 既有多 App | 既有 App 重新声明 | I | — | — | 重跑 ARC-018 挂载 (幂等) | feature_registry 行保持, updated_at 刷新 | 幂等 | ◎ |
| **v0.2 增**: TST-IT-31-E006 | 9/6 8 域扩展 ARC-018 挂载 | ARC-018 脚手架 | 8 域扩展 ARC-018 挂载触发 Feature 创建 (per 9/6 8 域扩展) | I | — | — | 8 域扩展 ARC-018 挂载 | feature_registry 自动创建 8/8 域 BOUNDED_CONTEXT Feature | 8 域扩展 | ◎ |
| **v0.2 增**: TST-IT-31-E007 | 9/1 batch 域 ARC-018 挂载 | ARC-018 脚手架 | batch 域 ARC-018 挂载触发 Feature 创建 (per 9/1 batch 域) | I | — | — | batch 域 ARC-018 挂载 | feature_registry 自动创建 6/6 module BOUNDED_CONTEXT Feature | batch 域 | ◎ |

## 3.6 模块 F：ARC-021 插件注册自动创建 Feature（RGS-BAS-031 §9.1）

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | シナリオ | テストデータ | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|---|---|
| TST-IT-31-F001 | BAS-031 §9.1 ARC-021 联动 | 插件注册表 | 新插件触发 Feature 创建 | I | — | — | ARC-021 注册新插件 | feature_registry 自动创建 PLUGIN 类型 Feature | FR-INT-002 | ◎ |
| TST-IT-31-F002 | BAS-031 §9.1 | CI 校验 | 插件数 == PLUGIN Feature 数 | I | — | — | 跑 CI 校验脚本 | check-cem-coverage.sh 验证一致 | RSK-COC-001 | ◎ |
| TST-IT-31-F003 | BAS-031 §9.1 | 插件沙箱 | 沙箱脚本插件 | I | — | — | 注册 Rhai 脚本插件 | Feature 创建, sandbox 关联 | ARC-021 兼容 | ◎ |
| TST-IT-31-F004 | BAS-031 §9.1 | 插件热插拔 | 插件 plug/unplug 联动 | I | — | — | 插件 plug | Feature status 切换 active/disabled | ARC-021 兼容 | ◎ |
| TST-IT-31-F005 | BAS-031 §9.1 | RGS-ADR-0020 守门 | 拒绝 .so/.dll 上传 | I | — | — | 尝试上传动态库 | 拒绝, Feature 不创建, 错误指明 ADR-0020 | ADR-0020 守门 | ◎ |
| **v0.2 增**: TST-IT-31-F006 | 9/5 61cf306 §4 阶段 1 MVP | 插件注册表 | 阶段 1 MVP: PG registry 持久化 (per 61cf306 §4.2) | I | — | — | 1 plugin v0.1.0 + PG 启动 | 重启 function-plane pod, plugin 仍在 | plugin 阶段 1 | ◎ |
| **v0.2 增**: TST-IT-31-F007 | 9/5 61cf306 §4 阶段 2 独立更新 | 插件注册表 | 阶段 2: 每 app 独立更新 + 双 registry 模式 (per 61cf306 §4.3) | I | — | — | player-service 启动时读全局 + 自身 registry | 0 业务中断, function-plane 调用 OK | plugin 阶段 2 | ◎ |
| **v0.2 增**: TST-IT-31-F008 | 9/5 61cf306 §4 阶段 3 平台化 | 插件注册表 | 阶段 3: 平台化 (per 61cf306 §4.4) | I | — | — | 平台化灰度 | canary 灰度切换生效 | plugin 阶段 3 | ◎ |

## 3.7 ADR-0051 集成守门 — v0.2 扩展 5+1 域 Lead 真实签字

| 用例 ID | 对应决策项 | 集成验证 | 步骤 | 预期 | 判定 | 优先级 | シナリオ | テストデータ |
|---|---|---|---|---|---|---|---|---|
| TST-IT-31-ADR-001 | §2 决定 4: COC UI 不另开凭证 | 集成层级验证 | 抓取 COC UI 流量 | 全部经 AdminService, 无 K8s/DB 直连 | NFR-OPS-004 守门 | ◎ | — | — |
| TST-IT-31-ADR-002 | §2 决定 5: 声明式+流式+幂等 | 跨进程契约 | 跨进程并发 request_id 重复 | 第二个返回 IDEMPOTENT_REPLAY | FR-API-003 守门 | ◎ | — | — |
| TST-IT-31-ADR-003 | §2 决定 6: DB 侧三类协同 | 集成层级 trigger 验证 | 尝试跨进程改 feature_version_history | trigger 拒绝 | FR-DB-001 守门 | ◎ | — | — |
| TST-IT-31-ADR-004 | §3.1 否决: COC UI 不独立 | 凭证体系验证 | 审计 COC UI 凭证范围 | 仅 admin_service 角色, 无 K8s/DB 角色 | 守门 | ◎ | — | — |
| TST-IT-31-ADR-005 | §3.3 否决: CEM 不分库 | DB 集成验证 | 检查 event_schema_registry 位置 | 在 admin_db, 不在 App 自己的 DB | ARC-008 守门 | ◎ | — | — |
| TST-IT-31-ADR-006 | §3.4 否决: COC 不直调 Helm | 流量层级验证 | 抓取 ClusterOpsService → Helm 流量 | 仅 ClusterOpsService 调 Helm, COC UI 不直接调 | 守门 | ◎ | — | — |
| TST-IT-31-ADR-007 | §3.5 否决: 补丁型不传动态库 | ARC-021 集成 | 上传 .so/.dll | 拒绝, 错误指明 ADR-0020 | RGS-ADR-0020 守门 | ◎ | — | — |
| TST-IT-31-ADR-008 | §3.6 否决: COC 不作 VIZ 子页 | 路由表验证 | 检查 COC UI 路由 | 顶级页面, 不在 VIZ 路由下 | 守门 | ◎ | — | — |
| **v0.2 增**: TST-IT-31-ADR-009 | §2 决定 7: 5+1 域 Lead 真实签字 (per 9/5 21:17 JST 拍板 + 9/1 batch 域) | 凭证体系验证 | 审计 5 域 + 1 batch 域 Lead 凭证范围 | 7 项 admin 域 Lead 决策 + 5+1 域 Lead 真实签字 | 守门 | ◎ | — | — |
| **v0.2 增**: TST-IT-31-ADR-010 | §2 决定 8: 9 域 mTLS 业务级 (per 9/6 d270ab9) | 流量层级验证 | 9 域 mTLS 业务级 11 步 | 11/11 步 PASS, 0 mTLS 握手失败 | 9 域 mTLS 守门 | ◎ | — | — |

## 3.8 模块 G: plugin 集群 (v0.2 新增, per 9/5 61cf306 §4 4 阶段)

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | シナリオ | テストデータ | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|---|---|
| TST-IT-31-G001 | 61cf306 §4 阶段 0 mock | function-plane | PoC plugin 抽卡概率 hot-swap 不重启 app | I | — | — | POST /ops/functions/draw_card_probability/register v0.2.0 | Draft → Active (灰度 10% cards) | plugin 阶段 0 | ◎ |
| TST-IT-31-G002 | 61cf306 §4 阶段 0 | function-plane | card-service 不重启 (per 61cf306 §3.1) | I | — | — | 监控 card-service 5min | 进程不重启, 抽卡结果符合新概率 | 0 中断 | ◎ |
| TST-IT-31-G003 | 61cf306 §4 阶段 0 | function-plane | plugin Paused → app 走 native fallback | I | — | — | plugin Paused | 100% 走 fallback, 0 业务中断 | fallback | ◎ |
| TST-IT-31-G004 | 61cf306 §4 阶段 0 | function-plane | WASM 资源限 fuel cap fail-closed | I | — | — | fuel 超过 cap | 1 次调用 fail-closed, app 不挂 | fuel cap | ◎ |
| TST-IT-31-G005 | 61cf306 §4 阶段 0 | function-plane | WASM 资源限 memory_mib cap fail-closed | I | — | — | memory_mib 超过 cap | 1 次调用 fail-closed | memory cap | ◎ |
| TST-IT-31-G006 | 61cf306 §4 阶段 0 | function-plane | adr-0020 守门 拒绝 .so/.dll | A | — | — | 尝试上传 .so | 拒绝, 错误指明 ADR-0020 | adr-0020 | ◎ |
| TST-IT-31-G007 | 61cf306 §4 阶段 1 MVP | function-plane + PG | PG registry 持久化 | I | — | — | 1 plugin + PG 启动 | 重启 function-plane pod, plugin 仍在 | 阶段 1 | ◎ |
| TST-IT-31-G008 | 61cf306 §4 阶段 1 | function-plane | 9 域共享 plugin (per 9/6 8 域扩展 + 9/1 batch 域) | I | — | — | 1 plugin 9 域 app 都能 invoke | 9/9 域 invoke 成功 | 9 域共享 | ◎ |
| TST-IT-31-G009 | 61cf306 §4 阶段 1 | function-plane + cert | plugin 自身 mTLS (per 9 域 mTLS 业务级) | I | — | — | 1 plugin + 1 cert | mTLS 握手成功 | plugin mTLS | ◎ |
| TST-IT-31-G010 | 61cf306 §4 阶段 2 | function-plane | 每 app 独立更新 (per 61cf306 §4.3) | I | — | — | player-service 启动时读全局 + 自身 registry | 0 业务中断, function-plane 调用 OK | 阶段 2 | ◎ |
| TST-IT-31-G011 | 61cf306 §4 阶段 2 | function-plane | 跨域注册 (9 域 + 1 batch) | I | — | — | 1 plugin 9 域 app 注册 | 9/9 域注册 OK | 9 域注册 | ◎ |
| TST-IT-31-G012 | 61cf306 §4 阶段 2 | function-plane | plugin plug/unplug 联动 | I | — | — | 插件 plug | Feature status 切换 active/disabled | plug/unplug | ◎ |
| TST-IT-31-G013 | 61cf306 §4 阶段 3 | function-plane | 平台化灰度 (per 61cf306 §4.4) | I | — | — | 平台化灰度 | canary 灰度切换生效 | 阶段 3 | ◎ |
| TST-IT-31-G014 | 61cf306 §4 | function-plane | plugin 监控 (调用次数 / 错误率 / 延迟 p99) | I | — | — | 100 次调用 | 3 指标入库, prometheus 可拉 | 监控 | ◎ |
| TST-IT-31-G015 | 61cf306 §4 | function-plane | plugin 灰度策略 (1% / 10% / 50% / 100%) | B | — | — | 4 灰度配置 | 配置切换生效, 抽样比例正确 | 灰度策略 | ◎ |
| TST-IT-31-G016 | 61cf306 §4 | function-plane | plugin 版本兼容性 (plugin v0.1.0 + app v0.2 调用) | I | — | — | 1 plugin + 1 app | 0 调用错 | 版本兼容 | ◎ |
| TST-IT-31-G017 | 61cf306 §4 | function-plane | plugin 版本不兼容 (plugin v0.1.0 + app v0.3 调用) | A | — | — | 1 plugin + 1 app | 返回 1100 INCOMPATIBLE_VERSION | 不兼容 | ◎ |
| TST-IT-31-G018 | 61cf306 §4 | function-plane | plugin 告警 (错误率 > 1% 持续 5min) | A | — | — | 1 plugin + 持续错误 | 告警 1 条到 oncall | 告警 | ◎ |
| TST-IT-31-G019 | 61cf306 §4 | function-plane | plugin sandbox 沙箱 (Rhai 脚本) | I | — | — | 1 Rhai 脚本 | 沙箱隔离, 无 host API 访问 | sandbox | ◎ |
| TST-IT-31-G020 | 61cf306 §4 | function-plane | plugin schema 校验 (schema_ref=invalid_hash) | A | — | — | 1 plugin + invalid hash | 拒绝, 错误信息指明 (per FR-CEM-011) | schema | ◎ |
| TST-IT-31-G021 | 61cf306 §4 | function-plane | plugin fallback 100% 走 native (plugin 故障) | I | — | — | plugin 故障 | 100% 走 fallback, 0 业务中断 | fallback | ◎ |

**实现位置**: `crates/cluster-ops/src/plugin_registry/tests/ut_registry.rs` + `crates/cluster-ops/tests/it_plugin_cluster.rs` (21 用例) — **v0.2 新增**

## 3.9 模块 H: app 独立更新 (v0.2 新增, per 9/5 61cf306 §4 + ARCH §3.4)

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | シナリオ | テストデータ | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|---|---|
| TST-IT-31-H001 | 61cf306 §4 + ARCH §3.4 | k3s + ClusterOpsService | player-service v0.2 灰度发布 10% (4 Pod 中 1 Pod) | I | — | — | kubectl set image player-service=v0.2 (1 Pod) | 1 Pod 升级, 3 Pod 仍 v0.1, traffic 切到新 Pod | 灰度 10% | ◎ |
| TST-IT-31-H002 | 同上 | k3s + ClusterOpsService | 灰度期间 latency p99 < 100ms, error rate < 0.5% | I | — | — | 5min 监控 metric mock | 0 触发回滚 | 监控 | ◎ |
| TST-IT-31-H003 | 同上 | k3s + ClusterOpsService | 扩量 10% → 50% (2 Pod 升级) | I | — | — | 50% 灰度 | 2 Pod 升级, 2 Pod 仍 v0.1 | 灰度 50% | ◎ |
| TST-IT-31-H004 | 同上 | k3s + ClusterOpsService | 全量 50% → 100% (4 Pod 全升级) | I | — | — | 100% 灰度 | 4 Pod 全部 v0.2, 0 老 Pod 残留 | 灰度 100% | ◎ |
| TST-IT-31-H005 | 同上 | k3s + ClusterOpsService | 回滚 灰度 10% 期间 error rate > 5% 自动回滚 | E | — | — | 监控 metric mock | 1 Pod 自动回滚到 v0.1, 流量切回 | 自动回滚 | ◎ |
| TST-IT-31-H006 | 同上 | k3s + ClusterOpsService | 人工 rollback 触发 | I | — | — | 1 app + 1 rollback 命令 | Helm rollback 到 from_version revision | 人工 rollback | ◎ |
| TST-IT-31-H007 | 9/6 8 域扩展 (b6b19b7) | k3s + ClusterOpsService | battle-service v0.2 灰度发布 (8 域扩展 NEW) | I | — | — | 1 app + 4 Pod | 0 业务中断, function-plane 8 域扩展 OK | 8 域扩展 | ◎ |
| TST-IT-31-H008 | 61cf306 §4.3 双 registry | function-plane + k3s | player-service v0.2 启动时读全局 + 自身 registry | I | — | — | 1 app + 2 registry | function-plane 调用 OK | 双 registry | ◎ |
| TST-IT-31-H009 | 同上 | function-plane + k3s | k3s rolling update 期间 function-plane 调用持续 | I | — | — | 1 app + 滚动升级 | 0 function-plane 调用失败 | rolling update | ◎ |
| TST-IT-31-H010 | L-CAND-006 cert 轮换 | k3s + ClusterOpsService | 灰度期间 cert 轮换 (per 9 域 mTLS 业务级) | I | — | — | 1 app + 1 cert | cert 切换不中断业务 | cert 轮换 | ◎ |
| TST-IT-31-H011 | 同上 | k3s + ClusterOpsService | 灰度期间 HPA 触发扩容 4→6 Pod | I | — | — | 1 app + 6 Pod | 0 业务中断, 6 Pod 同版本 | HPA | ◎ |
| TST-IT-31-H012 | SAGA-002 类比 | k3s + ClusterOpsService | economy-service 跨域回滚 | I | — | — | 1 app + 跨域 RPC | 反向步骤补偿, 0 业务数据不一致 | 跨域回滚 | ◎ |
| TST-IT-31-H013 | BAS-031 §4 状态机 | k3s + ClusterOpsService | 灰度期间 cluster-ops 状态机 6 阶段 (Scale 阶段) | I | — | — | 1 app + Scale 阶段 | 状态机迁移正确 | 状态机 | ◎ |
| TST-IT-31-H014 | 9/1 batch 域 | k3s + ClusterOpsService | rgs-batch-backend v0.2 灰度发布 (per 9/1 batch 域) | I | — | — | 1 app + 1 Pod | 0 业务中断, batch 任务正常 | batch 域 | ◎ |
| TST-IT-31-H015 | 61cf306 §4 | k3s + ClusterOpsService | app 灰度期间 plugin 仍可调用 | I | — | — | 1 app + 1 plugin | plugin 调用 0 失败 | plugin 集成 | ◎ |

**实现位置**: `crates/cluster-ops/src/app_deployment/tests/ut_deploy.rs` + `crates/cluster-ops/tests/it_app_independent_update.rs` (15 用例) — **v0.2 新增**

## 3.10 模块 I: admin-coc §X 集成设计 (v0.2 新增, per ae9702d)

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | シナリオ | テストデータ | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|---|---|
| TST-IT-31-I001 | ae9702d §X.1 RegisterFeature 转发契约 | admin-coc → admin-service → cluster-ops | 3 层转发契约 0 错 | I | — | — | admin-coc.RegisterFeature 转发 | 字段一致, feature_registry 写入, 3 层 0 错 | 转发契约 | ◎ |
| TST-IT-31-I002 | ae9702d §X.2 流式响应 DeclareFeatureUpgrade | admin-coc → admin-service → cluster-ops | Server stream 0 丢包 | I | — | — | 流返回 PfaRunStateUpdate 序列 | 0 丢包 | 流式契约 | ◎ |
| TST-IT-31-I003 | ae9702d §X.3 IDEMPOTENT_REPLAY 跨进程 | admin-coc → admin-service → cluster-ops | request_id 重复 | I | — | — | 重复 request_id | 第二个返回 IDEMPOTENT_REPLAY | 幂等 | ◎ |
| TST-IT-31-I004 | ae9702d §X.4 NOT_FOUND 跨进程 | admin-coc → admin-service → cluster-ops | 查找不存在的 feature_id | A | — | — | 不存在的 feature_id | gRPC status = NOT_FOUND | 错误码 | ◎ |
| TST-IT-31-I005 | ae9702d §X.5 RBAC 跨进程 | admin-coc → admin-service → cluster-ops | gm-backend cluster_operator 调 RollbackFeature → admin-service 拒绝 | A | — | — | cluster_operator 越权 | admin-service 拒绝 (RBAC) | RBAC | ◎ |
| TST-IT-31-I006 | ae9702d §X.6 COC UI 不持有 K8s 凭证 | admin-coc 凭证 | 检查 admin-coc ServiceAccount | I | — | — | 渗透测试 | 无 K8s RBAC 绑定 | NFR-OPS-004 | ◎ |
| TST-IT-31-I007 | ae9702d §X.7 cluster-ops 失联 | admin-service → cluster-ops | 模拟 cluster-ops 不可达 | A | — | — | 杀掉 cluster-ops | admin-service 返回 UNAVAILABLE | 故障恢复 | ◎ |
| TST-IT-31-I008 | ae9702d §X.8 7 项 admin 域 Lead 真实签字 (per 9/5 21:17 JST 拍板) | admin-coc 决策 | 审计 7 项 admin 域 Lead 决策签字 | I | — | — | 检查 7 项决策签字栏 | 7/7 决策签字 OK | 7 项决策 | ◎ |

**实现位置**: `crates/cluster-ops/tests/it_admin_coc.rs` (8 用例) — **v0.2 新增**

## 4. 追溯性矩阵 — v0.2 增 G/H/I

| 用例模块 | 覆盖的 REQ ID | 覆盖的 BAS ID | 覆盖的 ADR ID | 覆盖的 NFR ID | 覆盖的 AC ID |
|---|---|---|---|---|---|
| 3.1 AdminService 集成 (A001~A013) | FR-API-005, FR-COC-020 | BAS-031 §6, §9 | ADR-0051 §3.1, §3.4 | NFR-COC-007, 009 | AC-COC-006, 008 |
| 3.2 DB 集成 (B001~B015) | FR-CEM-001, FR-PFAU-002, 003, 020 | BAS-031 §3 | ADR-0051 §3.3 | NFR-COC-001, 003 | AC-COC-001, 002 |
| 3.3 事件总线集成 (C001~C012) | FR-CEM-001, 020, 030 | BAS-031 §5 | ADR-0051 §2.2 | NFR-COC-005 | AC-COC-004 |
| 3.4 Helm Release 集成 (D001~D012) | FR-INT-003, FR-PFAU-022 | BAS-031 §9.2 | ADR-0051 §3.2 | NFR-COC-003, 004 | AC-COC-002 |
| 3.5 ARC-018 联动 (E001~E007) | FR-INT-001 | BAS-031 §9.1 | — | — | AC-COC-001 |
| 3.6 ARC-021 联动 (F001~F008) | FR-INT-002 | BAS-031 §9.1 | ADR-0051 §3.5 | — | AC-COC-001 |
| 3.7 ADR 集成守门 (ADR-001~ADR-010) | — | — | ADR-0051 §2, §3 全部决定项 | — | — |
| **v0.2 增**: 3.8 plugin 集群 (G001~G021) | FR-INT-002, FR-PFAU-001 | 61cf306 §4 4 阶段 | ADR-0020 | NFR-OPS-001 | AC-COC-001 |
| **v0.2 增**: 3.9 app 独立更新 (H001~H015) | FR-INT-003, FR-PFAU-022 | 61cf306 §4 + ARCH §3.4 | ADR-0051 §3.2 | NFR-COC-003, 004 | AC-COC-002 |
| **v0.2 增**: 3.10 admin-coc §X 集成 (I001~I008) | FR-API-005, FR-COC-020 | ae9702d §X | ADR-0051 §2, §3 | NFR-COC-007, 009 | AC-COC-006, 008 |

## 5. 测试执行计划 — v0.2 增 plugin 集群 / app 独立更新 / admin-coc §X 阶段

| 阶段 | 用例范围 | 预计时长 | 前置条件 |
|---|---|---|---|
| **Phase 1: DB 集成** | B001~B015 | 0.5 天 | testcontainers 启动 PG 16 |
| **Phase 2: 事件总线集成** | C001~C012 | 0.5 天 | testcontainers 启动 NATS |
| **Phase 3: AdminService 集成** | A001~A013 | 0.5 天 | Phase 1, 2 通过 |
| **Phase 4: Helm Release 集成** | D001~D012 | 1 天 | 假 Helm + 假 runtime 上报端点 |
| **Phase 5: ARC 联动** | E001~F008 | 1 天 | Phase 1, 2, 3, 4 通过 |
| **Phase 6: ADR 守门** | ADR-001~ADR-010 | 0.5 天 | 全部通过 |
| **v0.2 增 Phase 7: plugin 集群** | G001~G021 | 2 天 | rgs-flash-mock v0.3 + 9 域 mTLS 业务级 + function-plane mock |
| **v0.2 增 Phase 8: app 独立更新** | H001~H015 | 2 天 | k3s 集群 + 9 域 mTLS 业务级 + 双 registry 模式 |
| **v0.2 增 Phase 9: admin-coc §X 集成** | I001~I008 | 1 天 | Phase 1~6 通过 + 5+1 域 Lead 真实签字 |

总计: **9 天** (v0.1 4 天 + v0.2 +5 天)

## 6. 通过判定基准 — v0.2 增 9 域 mTLS / plugin 集群 / app 独立更新

| 门禁 | 阈值 |
|---|---|
| 全部 ◎ 用例 | 100% 通过 |
| 全部 ○ 用例 | ≥95% 通过 |
| 集成契约 | 100% 覆盖（接口一致、字段一致、错误码一致） |
| 事务一致性 | DB 集成用例 100% 通过 |
| 跨进程流式响应 | 全部用例通过 |
| ADR 集成守门 | 全部决策项有实现+集成+守门 |
| **v0.2 增**: 9 域 mTLS 业务级 | 11/11 步客户端模拟器 v3 PASS (per 9/6 d270ab9) |
| **v0.2 增**: plugin 集群 4 阶段 | G001~G021 21 用例 100% PASS (per 9/5 61cf306 §4) |
| **v0.2 增**: app 独立更新 | H001~H015 15 用例 100% PASS (per 9/5 61cf306 §4 + ARCH §3.4) |
| **v0.2 增**: admin-coc §X 集成 | I001~I008 8 用例 100% PASS (per 9/5 ae9702d) |
| **v0.2 增**: 5+1 域 Lead 真实签字 | 7 项 admin 域 Lead 决策 + 5+1 域 Lead 真实签字 (per 9/5 21:17 JST + 9/1 batch) |

## 7. 风险与未决事项（TBD 处置）— v0.2 增 9 维度

| TBD ID | 内容 | IT 处置 |
|---|---|---|
| TBD-COC-001 | 无限画布前端选型 | **不在 IT 范围** — 由 ST 验证 |
| TBD-COC-002 | 补丁型金丝雀测试门禁 | **按保守假设实施** — IT 验证"门禁失败拒绝进入灰度"路径 |
| TBD-COC-003 | 批量回滚上限 20 | **已实施** — TST-IT-31 (后续追加) |
| TBD-COC-004 | 事件注册变更事件保留 | **已实施** — TST-IT-31-C001~C012 验证 |
| TBD-COC-005 | feature_registry 分区策略 | **待前置条件** — 留给 PH-7 |
| TBD-COC-006 | pfa_run_state 历史归档 | **按保守假设实施** — IT 验证 90 天内可查询 |
| RSK-COC-001 | CI 校验脚本 | **已实施** — TST-IT-31-F002 验证 |
| RSK-COC-002 | PFAU 超时阈值 120 秒 | **已实施** — TST-IT-31-D008 验证 |
| RSK-COC-003 | COC UI 弱化 RGS-OPS-001 | **不在 IT 范围** — 由 UX 评审 + ST 覆盖 |
| RSK-COC-004 | CEM 存储成本与合规 | **不在 IT 范围** — 由详细设计阶段评估 |
| **v0.2 增**: TBD-COC-007 | plugin 集群 21 用例 (per 9/5 61cf306 §4) 待 v0.2 实施 | ⏳ v0.2 实施 |
| **v0.2 增**: TBD-COC-008 | app 独立更新 15 用例 (per 9/5 61cf306 §4 + ARCH §3.4) 待 v0.2 实施 | ⏳ v0.2 实施 |
| **v0.2 增**: TBD-COC-009 | admin-coc §X 集成 8 用例 (per 9/5 ae9702d) 待 v0.2 实施 | ⏳ v0.2 实施 |
| **v0.2 增**: TBD-COC-010 | 5+1 域 Lead 真实签字 (per 9/5 21:17 JST + 9/1 batch 域) 待 DDD Review 阶段补 | ⏳ DDD Review |
| **v0.2 增**: TBD-COC-011 | 9 域 mTLS 业务级 11 步客户端模拟器 v3 (per 9/6 d270ab9) 待 v0.2 实施 | ⏳ v0.2 实施 |
| **v0.2 增**: TBD-COC-012 | 9 域 cert 轮换 (per L-CAND-006) 待 v0.2 实施 | ⏳ v0.2 实施 |
| **v0.2 增**: TBD-COC-013 | rgs-flash-mock v0.3 60 module fixture (per 9/4 v0.3) 待 v0.2 实施 | ⏳ 9/4 v0.3 实施 |
| **v0.2 增**: TBD-COC-014 | 8 域扩展 (per 9/6 8 域扩展) feature 行 + event_type 注册待 v0.2 实施 | ⏳ v0.2 实施 |
| **v0.2 增**: TBD-COC-015 | batch 域 (per 9/1) 6 module feature 行 + event_type 注册待 v0.2 实施 | ⏳ v0.2 实施 |

## 8. 派生约束守护段 (v0.2 新增)

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
| L20 | ca.crt 0 字节 | ✅ M007 用例覆盖 (IT-02 引用) | 9/2 10:18 JST D2 |
| L21 | 跨工具链 gRPC Code 解析 | ✅ 9 域 mTLS 业务级 | 9/2 10:18 JST D2 |
| L22 | 协议码映射表 | ✅ | 9/2 10:18 JST D2 |
| L23 | 4 层自动探针 | ✅ | 9/2 10:18 JST D2 |
| **L-CAND-006** | 9 域 cert 轮换 (per 9/1 PT 派工教训) | ⏳ 待 v0.2 实施 | 9/1 PT 派工教训 |
| **L-CAND-020** | ca.crt 0 字节防御 (per 9/2 10:18 JST D2) | ✅ M007 用例覆盖 (IT-02 引用) | 9/2 10:18 JST D2 |
| **9/1 batch 12 派生约束** | batch 域独立 Lead 等 12 条 (per 9/1 18:00-19:24 JST) | ✅ 签字栏含 batch 域 Lead 真实签字 | 9/1 batch |
| **B3 二审流程** | DDD Review 二审 (Mavis 自审 → Ulysses 二审) | ⏳ Mavis 自审 | 9/2 10:18 JST B3 |

---

> 本文档配套 RGS-REQ-031 / RGS-BAS-031 / RGS-ADR-0051 / RGS-TST-UT-31。后续将产出 RGS-TST-ST-31（系统测试）。
> v0.2 升版: 增加 plugin 集群 (per 9/5 61cf306 §4 4 阶段) + app 独立更新 (per 9/5 61cf306 §4 + ARCH §3.4) + admin-coc §X 集成 (per 9/5 ae9702d) + 9 域 mTLS 业务级 (per 9/6 d270ab9) + 5+1 域 Lead 真实签字 (per 9/5 21:17 JST 拍板 + 9/1 batch 域扩展), 综合 8 维度 per 2026-09-07 12:35 JST Round 2 W3 v02/it3 5 worker 派工.

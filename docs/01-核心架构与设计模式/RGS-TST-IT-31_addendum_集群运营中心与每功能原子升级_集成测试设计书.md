# 集成测试设计書（結合テスト設計書 / Integration Test Design Document）

**主题域 31 集群运营中心与每功能原子升级 — 集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-31 |
| 版本 | 0.1 |
| 父文档 | RGS-REQ-031 addendum 需求定义书（ARC-051）、RGS-BAS-031 addendum 基本设计书、RGS-ADR-0051 架构决定 |
| V模型层级 | TL-2 集成试验 ↔ BAS 基本设计 |
| 依据标准 | IPA『共通フレーム 2013』基本設計工程 |
| 制定日 | 2026-08-19 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 本主题域源文档全集 | RGS-REQ-031、RGS-BAS-031、RGS-ADR-0051、（待）RGS-DTL-031 |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定。覆盖 AdminService ↔ ClusterOpsService 集成、ClusterOpsService ↔ admin_db 集成、CEM 探针 ↔ 事件总线集成、PFAU ↔ Helm Release 集成、ARC-018/021/042 联动集成 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 评审（QA） | | | IT 集成场景与组件契约的一致性 |
| 审批（负责人） | | | 本测试设计书的基准化 |

---

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
4. 追溯性矩阵
5. 测试执行计划
6. 通过判定基准
7. 风险与未决事项（TBD 处置）

---

## 1. 前言

## 1.1 目的

本文档为 V 模型中 **TL-2 集成试验**层级的设计书，对应主题 31（ARC-051）。本版本（0.1）核心：

- **服务间集成验证**：AdminService ↔ ClusterOpsService、ClusterOpsService ↔ 各 App（运行时 PFAU 确认接口）
- **DB 集成验证**：ClusterOpsService ↔ admin_db 事务一致性、Schema 演进
- **事件总线集成验证**：CEM 探针订阅器 ↔ 事件总线只读镜像
- **既有流程联动集成**：PFAU ↔ Helm Release（ARC-042 联动）、ARC-018 挂载自动创建 Feature、ARC-021 注册自动创建 Feature
- **ADR-0051 集成守门验证**：COC UI 经 AdminService 单一入口、DB 侧三类协同触发器、CEM 探针不阻塞正常消费者

## 1.2 适用范围

| 范畴 | 说明 |
|---|---|
| 适用 | ClusterOpsService 与 AdminService 的集成、ClusterOpsService 与 admin_db 的集成、CEM 探针与事件总线的集成、PFAU 与 Helm Release 的集成、与既有 ARC-018/021/042 流程的联动集成 |
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

## 1.6 命名约定

| 对象 | 命名格式 | 示例 |
|---|---|---|
| 集成测试用例 | `TST-IT-31-<模块>-<编号>` | TST-IT-31-A001 |
| 模块代号 | A: AdminService 集成 / B: DB 集成 / C: 事件总线集成 / D: Helm Release 集成 / E: ARC-018 联动 / F: ARC-021 联动 | — |

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
```

### 2.2 测试方法

- 集成框架：`testcontainers-rs` 启动 PostgreSQL 18.4 + 事件总线（nats-jetstream）
- 假依赖：假 Helm Release、假 App（mock runtime PFAU 确认接口）
- 端到端模拟：使用 `wiremock` 模拟外部 AdminService 调用方
- 覆盖率门禁：IT 集成路径 100% 覆盖（与 UT 不同, IT 关注接口契约而非行覆盖）

---

## 3. 测试用例

## 3.1 模块 A：AdminService ↔ ClusterOpsService 集成（RGS-BAS-031 §6.1, §9）

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| TST-IT-31-A001 | BAS-031 §6.1 gRPC 方法列表 | AdminService 转发 | RegisterFeature 端到端 | I | AdminService.RegisterFeature → ClusterOpsService.RegisterFeature | 字段一致, feature_registry 写入 | 转发契约 | ◎ |
| TST-IT-31-A002 | BAS-031 §6.2.1 | 同上 | feature_id 唯一性跨进程 | I | 并发两次 RegisterFeature 同 feature_id | 一个成功, 一个返回 AlreadyExists | 并发安全 | ◎ |
| TST-IT-31-A003 | BAS-031 §6.1 | 同上 | 流式响应跨进程 | I | DeclareFeatureUpgrade → Server stream | 流返回 PfaRunStateUpdate 序列 | 流式契约 | ◎ |
| TST-IT-31-A004 | BAS-031 §6.1 | 同上 | RBAC 跨进程 | I | cluster_operator 调 RollbackFeature | AdminService 拒绝 (RBAC) | NFR-OPS-004 | ◎ |
| TST-IT-31-A005 | BAS-031 §6.3 错误码 | 同上 | NOT_FOUND 跨进程 | I | 查找不存在的 feature_id | gRPC status = NOT_FOUND | 错误码 | ◎ |
| TST-IT-31-A006 | BAS-031 §6.3 | 同上 | IDEMPOTENT_REPLAY 跨进程 | I | 重复 request_id | 第二次返回 IDEMPOTENT_REPLAY | FR-API-003 | ◎ |
| TST-IT-31-A007 | BAS-031 §6.1 | ClusterOpsService 崩溃 | AdminService 优雅降级 | E | ClusterOpsService 进程崩溃 | AdminService 返回 UNAVAILABLE, 不影响 AdminService 自身 | 故障隔离 | ◎ |
| TST-IT-31-A008 | BAS-031 §9.1 | COC UI → AdminService | COC UI 经 AdminService 唯一入口 | I | COC UI 调 RegisterFeature | 流量必经 AdminService, ClusterOpsService 收到的是 AdminService 转发 | 强制联动 | ◎ |
| TST-IT-31-A009 | BAS-031 §9.1 | 渗透测试 | COC UI 不持有 K8s 凭证 | I | 检查 COC UI ServiceAccount | 无 K8s RBAC 绑定 | NFR-OPS-004 | ◎ |
| TST-IT-31-A010 | BAS-031 §6.1 | ClusterOpsService 启动慢 | 启动期间请求排队 | E | ClusterOpsService 启动 30 秒, AdminService 立即发请求 | 请求排队直到 ClusterOpsService 就绪 | 启动顺序 | ○ |

## 3.2 模块 B：ClusterOpsService ↔ admin_db 集成（RGS-BAS-031 §3）

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| TST-IT-31-B001 | BAS-031 §3.1 feature_registry | admin_db | Schema 创建 | N | 应用迁移 | 全部表/索引/trigger/视图创建成功 | 迁移正确 | ◎ |
| TST-IT-31-B002 | BAS-031 §3.2 trigger | 同上 | append-only trigger 触发 | I | ClusterOpsService 尝试 UPDATE feature_version_history | trigger 抛出 Exception | FR-DB-001 | ◎ |
| TST-IT-31-B003 | BAS-031 §3.3 pfa_run_state | 同上 | 状态机迁移持久化 | S | 启动 PFAU, 状态切换多次 | 每次切换后 admin_db 状态一致 | 持久化 | ◎ |
| TST-IT-31-B004 | BAS-031 §3.3 | 同上 | 跨节点确认超时持久化 | B | 模拟超时, 状态变 paused | admin_db 写入 pause_reason | 持久化超时原因 | ◎ |
| TST-IT-31-B005 | BAS-031 §3.3 | 同上 | 并发 PFAU 启动互斥 | I | 同一 feature_id 并发启动 2 个 PFAU | 第二个返回 PFAU_ALREADY_RUNNING | 并发安全 | ◎ |
| TST-IT-31-B006 | BAS-031 §3.1 | 同上 | feature_registry CRUD 事务 | I | 创建 + 立即查询 | 事务隔离, 立即可见 | 事务边界 | ◎ |
| TST-IT-31-B007 | BAS-031 §3.4 event_schema_registry | 同上 | 事件注册表写入 | N | RegisterEvent 调用 | event_type 写入, schema_ref 校验通过 | 写入契约 | ◎ |
| TST-IT-31-B008 | BAS-031 §3.4 | 同上 | schema_ref 引用源码 commit | I | 提交 schema_ref=invalid_hash | 拒绝, 错误信息指明 | FR-CEM-011 | ◎ |
| TST-IT-31-B009 | BAS-031 §3.5 event_dlq_view | 同上 | DLQ 视图查询 | N | 制造死信, 查视图 | last_1h_count 正确 | 视图正确 | ◎ |
| TST-IT-31-B010 | BAS-031 §3.6 coc_audit_view | 同上 | 审计视图查询 | N | 通过 AdminService 写操作, 查视图 | 视图返回 coc.% 操作 | FR-COC-040 | ◎ |
| TST-IT-31-B011 | BAS-031 §3.3 | 同上 | last_heartbeat_at 心跳更新 | I | 运行时上报心跳, 状态机推进 | heartbeat 字段更新 | 跨节点确认 | ◎ |
| TST-IT-31-B012 | BAS-031 §3.1 | admin_db 故障 | ClusterOpsService 优雅降级 | E | 杀掉 admin_db | ClusterOpsService 返回 UNAVAILABLE, PFAU 状态保留 (待 DB 恢复后继续) | 故障恢复 | ◎ |
| TST-IT-31-B013 | BAS-031 §3.2 | 同上 | version_history 历史不可变 | I | PFAU 完成后, 尝试修改历史记录 | trigger 拒绝 | 不可变历史 | ◎ |

## 3.3 模块 C：CEM 探针订阅器 ↔ 事件总线集成（RGS-BAS-031 §5）

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| TST-IT-31-C001 | BAS-031 §5.2 探针工作流 | 事件总线 (NATS) | 探针订阅正常事件 | I | 生产 100 个已注册事件 | 探针 UPSERT 100 次 (批合并后 1 次) | 探针工作流 | ◎ |
| TST-IT-31-C002 | BAS-031 §5.2 | 同上 | 探针订阅未注册事件 | A | 生产 10 个未注册事件 | 探针写告警, 不阻塞 | RSK-COC-001 | ◎ |
| TST-IT-31-C003 | BAS-031 §5.3 | 同上 | 探针不阻塞正常消费者 | I | 探针慢处理, 正常消费者速度 | 正常消费者 lag=0 | FR-API-012 | ◎ |
| TST-IT-31-C004 | BAS-031 §5.3 | 同上 | 探针独立 Consumer Group | I | 启动两个探针实例 | 各自 offset 独立 | FR-API-012 | ◎ |
| TST-IT-31-C005 | BAS-031 §5.2 | 同上 | 探针解析 event_type 失败不崩溃 | E | 生产畸形事件 | 探针记录错误, 继续监听 | 鲁棒性 | ◎ |
| TST-IT-31-C006 | BAS-031 §5.3 | 同上 | 探针批量 UPSERT 5 秒窗口 | I | 1 秒内 1000 事件 | 1 次 DB UPSERT | 批处理 | ◎ |
| TST-IT-31-C007 | BAS-031 §5.2 | 同上 | event_producer_registry 更新 | I | 生产事件 | last_seen_at 更新, app_version 正确 | 写入契约 | ◎ |
| TST-IT-31-C008 | BAS-031 §5.2 | 同上 | 探针事件总线故障 | E | 杀掉事件总线 | 探针重连, 不丢失告警通路 | 故障恢复 | ◎ |
| TST-IT-31-C009 | BAS-031 §5.2 | 同上 | 探针启动顺序 | I | 事件总线先于探针启动 | 探针自动重连到事件总线 | 启动顺序 | ○ |
| TST-IT-31-C010 | BAS-031 §5.2 | 同上 | 探针 ack 模式 | I | 探针确认事件后丢弃 | 探针 ack 速度快, 不影响正常消费者 | ack 模式 | ◎ |

## 3.4 模块 D：PFAU ↔ Helm Release 集成（RGS-BAS-031 §9.2 联动）

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| TST-IT-31-D001 | BAS-031 §9.2 | Helm Release (假) | PFAU 启动调用 Helm | I | DeclareFeatureUpgrade → Helm install | Helm 收到 install 命令, 成功后 PFAU 推进 | 联动正确 | ◎ |
| TST-IT-31-D002 | BAS-031 §9.2 | 同上 | Helm 失败 → PFAU 失败 | E | Helm install 失败 | PFAU 进入 paused, pause_reason='helm_install_failed' | 错误传播 | ◎ |
| TST-IT-31-D003 | BAS-031 §9.2 | 同上 | Helm 成功 → 触发节点确认 | I | Helm install 成功, 模拟节点上报 | 节点确认后 PFAU 推进 | 跨节点确认 | ◎ |
| TST-IT-31-D004 | BAS-031 §4.2 | 同上 | 灰度批次与 Helm 升级对应 | I | 5 批灰度, 每批调 Helm upgrade | 5 次 Helm upgrade, 每次只升级当前批次节点 | 灰度实现 | ◎ |
| TST-IT-31-D005 | BAS-031 §9.2 | 同上 | 节点失联触发自动回滚 | E | 模拟 K8s Pod 异常退出 | Helm rollback 触发, PFAU 回到 rolled_back | FR-PFAU-022 | ◎ |
| TST-IT-31-D006 | BAS-031 §9.2 | 同上 | 自动回滚后状态机 | S | 自动回滚完成 | feature_registry.current_version 回到 from_version | 一致性 | ◎ |
| TST-IT-31-D007 | BAS-031 §9.2 | 同上 | 灰度批次观察期强制 | B | 观察期 5 秒, 立即推进 | Helm 不被调用下一批, 等待观察期 | 观察期强制 | ◎ |
| TST-IT-31-D008 | BAS-031 §9.2 | 同上 | 跨节点确认超时 | B | 1 节点 120 秒不上报 | PFAU paused, Helm 不再调升级 | 超时逻辑 | ◎ |
| TST-IT-31-D009 | BAS-031 §9.2 | 同上 | Helm release 历史记录 | I | 完成一次升级 | Helm release 包含 pfa_run_id 标签 | 可追溯 | ◎ |
| TST-IT-31-D010 | BAS-031 §9.2 | 同上 | 人工 rollback → Helm rollback | I | 人工触发 rollback | Helm rollback 到 from_version revision | 人工路径 | ◎ |

## 3.5 模块 E：ARC-018 挂载完成自动创建 Feature（RGS-BAS-031 §9.1）

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| TST-IT-31-E001 | BAS-031 §9.1 ARC-018 联动 | ARC-018 脚手架 | 新挂载触发 Feature 创建 | I | 执行 ARC-018 挂载流程 | feature_registry 自动创建 BOUNDED_CONTEXT 类型 Feature | FR-INT-001 | ◎ |
| TST-IT-31-E002 | BAS-031 §9.1 | 同上 | Mount Record 含 COC UI 元数据 | I | 检查挂载产物 | Mount Record 含 feature_id 字段 | FR-INT-001 | ◎ |
| TST-IT-31-E003 | BAS-031 §9.1 | CI 校验 | 缺失 feature 创建 CI 失败 | I | ARC-018 挂载但未触发 Feature 创建 | CI 校验失败, 挂载不视为完成 | FR-INT-001 | ◎ |
| TST-IT-31-E004 | BAS-031 §9.1 | ARC-018 CI | Feature 创建回滚 | I | ARC-018 挂载失败 | feature_registry 行回滚 (无残留) | 事务一致性 | ◎ |
| TST-IT-31-E005 | BAS-031 §9.1 | 既有多 App | 既有 App 重新声明 | I | 重跑 ARC-018 挂载 (幂等) | feature_registry 行保持, updated_at 刷新 | 幂等 | ◎ |

## 3.6 模块 F：ARC-021 插件注册自动创建 Feature（RGS-BAS-031 §9.1）

| 用例 ID | 对应设计 | 集成对端 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|---|
| TST-IT-31-F001 | BAS-031 §9.1 ARC-021 联动 | 插件注册表 | 新插件触发 Feature 创建 | I | ARC-021 注册新插件 | feature_registry 自动创建 PLUGIN 类型 Feature | FR-INT-002 | ◎ |
| TST-IT-31-F002 | BAS-031 §9.1 | CI 校验 | 插件数 == PLUGIN Feature 数 | I | 跑 CI 校验脚本 | check-cem-coverage.sh 验证一致 | RSK-COC-001 | ◎ |
| TST-IT-31-F003 | BAS-031 §9.1 | 插件沙箱 | 沙箱脚本插件 | I | 注册 Rhai 脚本插件 | Feature 创建, sandbox 关联 | ARC-021 兼容 | ◎ |
| TST-IT-31-F004 | BAS-031 §9.1 | 插件热插拔 | 插件 plug/unplug 联动 | I | 插件 plug | Feature status 切换 active/disabled | ARC-021 兼容 | ◎ |
| TST-IT-31-F005 | BAS-031 §9.1 | RGS-ADR-0020 守门 | 拒绝 .so/.dll 上传 | I | 尝试上传动态库 | 拒绝, Feature 不创建, 错误指明 ADR-0020 | ADR-0020 守门 | ◎ |

## 3.7 ADR-0051 集成守门

| 用例 ID | 对应决策项 | 集成验证 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|
| TST-IT-31-ADR-001 | §2 决定 4: COC UI 不另开凭证 | 集成层级验证 | 抓取 COC UI 流量 | 全部经 AdminService, 无 K8s/DB 直连 | NFR-OPS-004 守门 | ◎ |
| TST-IT-31-ADR-002 | §2 决定 5: 声明式+流式+幂等 | 跨进程契约 | 跨进程并发 request_id 重复 | 第二个返回 IDEMPOTENT_REPLAY | FR-API-003 守门 | ◎ |
| TST-IT-31-ADR-003 | §2 决定 6: DB 侧三类协同 | 集成层级 trigger 验证 | 尝试跨进程改 feature_version_history | trigger 拒绝 | FR-DB-001 守门 | ◎ |
| TST-IT-31-ADR-004 | §3.1 否决: COC UI 不独立 | 凭证体系验证 | 审计 COC UI 凭证范围 | 仅 admin_service 角色, 无 K8s/DB 角色 | 守门 | ◎ |
| TST-IT-31-ADR-005 | §3.3 否决: CEM 不分库 | DB 集成验证 | 检查 event_schema_registry 位置 | 在 admin_db, 不在 App 自己的 DB | ARC-008 守门 | ◎ |
| TST-IT-31-ADR-006 | §3.4 否决: COC 不直调 Helm | 流量层级验证 | 抓取 ClusterOpsService → Helm 流量 | 仅 ClusterOpsService 调 Helm, COC UI 不直接调 | 守门 | ◎ |
| TST-IT-31-ADR-007 | §3.5 否决: 补丁型不传动态库 | ARC-021 集成 | 上传 .so/.dll | 拒绝, 错误指明 ADR-0020 | RGS-ADR-0020 守门 | ◎ |
| TST-IT-31-ADR-008 | §3.6 否决: COC 不作 VIZ 子页 | 路由表验证 | 检查 COC UI 路由 | 顶级页面, 不在 VIZ 路由下 | 守门 | ◎ |

---

## 4. 追溯性矩阵

| 用例模块 | 覆盖的 REQ ID | 覆盖的 BAS ID | 覆盖的 ADR ID | 覆盖的 NFR ID | 覆盖的 AC ID |
|---|---|---|---|---|---|
| 3.1 AdminService 集成 (A001~A010) | FR-API-005, FR-COC-020 | BAS-031 §6, §9 | ADR-0051 §3.1, §3.4 | NFR-COC-007, 009 | AC-COC-006, 008 |
| 3.2 DB 集成 (B001~B013) | FR-CEM-001, FR-PFAU-002, 003, 020 | BAS-031 §3 | ADR-0051 §3.3 | NFR-COC-001, 003 | AC-COC-001, 002 |
| 3.3 事件总线集成 (C001~C010) | FR-CEM-001, 020, 030 | BAS-031 §5 | ADR-0051 §2.2 | NFR-COC-005 | AC-COC-004 |
| 3.4 Helm Release 集成 (D001~D010) | FR-INT-003, FR-PFAU-022 | BAS-031 §9.2 | ADR-0051 §3.2 | NFR-COC-003, 004 | AC-COC-002 |
| 3.5 ARC-018 联动 (E001~E005) | FR-INT-001 | BAS-031 §9.1 | — | — | AC-COC-001 |
| 3.6 ARC-021 联动 (F001~F005) | FR-INT-002 | BAS-031 §9.1 | ADR-0051 §3.5 | — | AC-COC-001 |
| 3.7 ADR 集成守门 (ADR-001~ADR-008) | — | — | ADR-0051 §2, §3 全部决定项 | — | — |

---

## 5. 测试执行计划

| 阶段 | 用例范围 | 预计时长 | 前置条件 |
|---|---|---|---|
| **Phase 1: DB 集成** | B001~B013 | 0.5 天 | testcontainers 启动 PG 16 |
| **Phase 2: 事件总线集成** | C001~C010 | 0.5 天 | testcontainers 启动 NATS |
| **Phase 3: AdminService 集成** | A001~A010 | 0.5 天 | Phase 1, 2 通过 |
| **Phase 4: Helm Release 集成** | D001~D010 | 1 天 | 假 Helm + 假 runtime 上报端点 |
| **Phase 5: ARC 联动** | E001~F005 | 1 天 | Phase 1, 2, 3, 4 通过 |
| **Phase 6: ADR 守门** | ADR-001~ADR-008 | 0.5 天 | 全部通过 |

总计: **4 天**

---

## 6. 通过判定基准

| 门禁 | 阈值 |
|---|---|
| 全部 ◎ 用例 | 100% 通过 |
| 全部 ○ 用例 | ≥95% 通过 |
| 集成契约 | 100% 覆盖（接口一致、字段一致、错误码一致） |
| 事务一致性 | DB 集成用例 100% 通过 |
| 跨进程流式响应 | 全部用例通过 |
| ADR 集成守门 | 全部决策项有实现+集成+守门 |

---

## 7. 风险与未决事项（TBD 处置）

| TBD ID | 内容 | IT 处置 |
|---|---|---|
| TBD-COC-001 | 无限画布前端选型 | **不在 IT 范围** — 由 ST 验证 |
| TBD-COC-002 | 补丁型金丝雀测试门禁 | **按保守假设实施** — IT 验证"门禁失败拒绝进入灰度"路径 |
| TBD-COC-003 | 批量回滚上限 20 | **已实施** — TST-IT-31 (后续追加) |
| TBD-COC-004 | 事件注册变更事件保留 | **已实施** — TST-IT-31-C001~C010 验证 |
| TBD-COC-005 | feature_registry 分区策略 | **待前置条件** — 留给 PH-7 |
| TBD-COC-006 | pfa_run_state 历史归档 | **按保守假设实施** — IT 验证 90 天内可查询 |
| RSK-COC-001 | CI 校验脚本 | **已实施** — TST-IT-31-F002 验证 |
| RSK-COC-002 | PFAU 超时阈值 120 秒 | **已实施** — TST-IT-31-D008 验证 |
| RSK-COC-003 | COC UI 弱化 RGS-OPS-001 | **不在 IT 范围** — 由 UX 评审 + ST 覆盖 |
| RSK-COC-004 | CEM 存储成本与合规 | **不在 IT 范围** — 由详细设计阶段评估 |

---

> 本文档配套 RGS-REQ-031 / RGS-BAS-031 / RGS-ADR-0051 / RGS-TST-UT-31。后续将产出 RGS-TST-ST-31（系统测试）。

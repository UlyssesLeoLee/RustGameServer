# 系统测试设计書（システムテスト設計書 / System Test Design Document）

**主题域 31 集群运营中心与每功能原子升级 — 系统测试（ST）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-ST-31 |
| 版本 | 0.1 |
| 父文档 | RGS-REQ-031 addendum 需求定义书（ARC-051）、RGS-BAS-031 addendum 基本设计书、RGS-ADR-0051 架构决定 |
| V模型层级 | TL-3 系统试验 ↔ REQ 需求定义 |
| 依据标准 | IPA『共通フレーム 2013』要件定義工程 |
| 制定日 | 2026-08-19 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 本主题域源文档全集 | RGS-REQ-031、RGS-BAS-031、RGS-ADR-0051 |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定。覆盖 AC-COC-001~010 全部验收标准：功能矩阵加载、PFAU 升级演练、补丁型 Feature 演练、事件流视图、DLQ 重放、渗透测试、AdminService 故障注入、既有 GM 后台回归、COC UI 回滚、声明式扩展 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 评审（架构） | | | AC 覆盖度与 REQ-031 §12 验收标准 100% 对应 |
| 评审（QA） | | | 端到端演练与 NFR 目标值 |
| 评审（安全） | | | 渗透测试用例与 NFR-OPS-004 守门 |
| 审批（负责人） | | | 本测试设计书的基准化 |

---

## 目次

1. 前言
   1.1 目的
   1.2 适用范围
   1.3 关联文档
   1.4 记述规则（含 1.4.1 强度用语）
   1.5 验收标准对应表（AC ↔ 用例）
   1.6 命名约定
2. 测试策略
3. 测试用例
   3.1 AC-COC-001: 功能矩阵首页加载
   3.2 AC-COC-002: PFAU 升级演练（插件型）
   3.3 AC-COC-003: 补丁型 Feature 演练
   3.4 AC-COC-004: 事件流视图呈现
   3.5 AC-COC-005: DLQ 重放演练
   3.6 AC-COC-006: 渗透测试
   3.7 AC-COC-007: AdminService 故障注入
   3.8 AC-COC-008: 既有 GM 后台回归
   3.9 AC-COC-009: COC UI 自身回滚
   3.10 AC-COC-010: 声明式扩展（新 Feature 类型）
4. NFR 端到端验证
5. 追溯性矩阵
6. 测试执行计划
7. 通过判定基准
8. 风险与未决事项（TBD 处置）

---

## 1. 前言

## 1.1 目的

本文档为 V 模型中 **TL-3 系统试验**层级的设计书，对应主题 31（ARC-051 集群运营中心 + 中心事件管理 + 每功能原子升级）。本版本（0.1）核心：

- **100% 覆盖 REQ-031 §12 验收标准**（AC-COC-001~010），每条 AC 一组端到端演练
- **NFR 目标值验证**：NFR-COC-001~010 端到端实测
- **既有功能回归**：GM 后台既有功能（AC-OPS-001~005）不因 COC UI 新增而失败
- **渗透测试**：COC UI 不持有 K8s/DB 直连凭证（NFR-OPS-004 守门）
- **故障注入**：AdminService 不可用时 COC UI 降级到只读
- **可回滚性**：COC UI 自身 30 分钟内可回滚，PFAU 进行中实例可见性不丢失

## 1.2 适用范围

| 范畴 | 说明 |
|---|---|
| 适用 | COC UI 端到端演练、PFAU 状态机端到端、事件流端到端、DLQ 重放演练、AdminService 故障降级、渗透测试、既有 GM 后台回归 |
| 不适用 | 单元/集成模块（已在 UT-31 / IT-31）、部署运维（属 RGS-OPS-001）、业务功能（属各域 ST） |

## 1.3 关联文档

| 文档编号 | 文档名 | 与本文档关系 |
|---|---|---|
| RGS-REQ-031 §12 | 验收标准 | ST 验证对象 |
| RGS-REQ-031 §10 | NFR | ST 验证对象 |
| RGS-BAS-031 | 基本设计书 | ST 集成对象 |
| RGS-ADR-0051 | 架构决定 | §3.6 守门 |
| RGS-TST-UT-31 | 单元测试设计书 | 前置 |
| RGS-TST-IT-31 | 集成测试设计书 | 前置 |
| RGS-REQ-007 §8 | AC-OPS-001~005 | 回归测试基准 |
| RGS-OPS-001 §10.4 | 关联文档 | 部署参考 |

## 1.4 记述规则

### 1.4.1 强度用语

本文档遵循 RFC 2119 与 IPA 共通フレーム 2013 规定的强度用语（必须/应当/可以/不得/不应当），具体定义同 RGS-TST-UT-31 §1.4.1。

### 1.4.2 覆盖类型符号

| 符号 | 含义 |
|---|---|---|
| **E2E** | 端到端演练（End-to-End） |
| P | 性能（Performance 实测） |
| SEC | 安全性（Security） |
| REG | 回归（Regression） |
| ROLL | 可回滚性（Rollback） |
| FAULT | 故障注入（Fault Injection） |

### 1.4.3 优先级符号

| 符号 | 优先级 |
|---|---|
| ◎ | 最高（必须 100% 通过） |
| ○ | 高（必须 ≥95% 通过） |
| △ | 中（可后续补） |
| × | 低/暂不实施 |

## 1.5 验收标准对应表

| AC ID | 验收内容 | ST 用例模块 |
|---|---|---|
| AC-COC-001 | 功能矩阵首页加载 | §3.1 |
| AC-COC-002 | PFAU 升级演练（插件型） | §3.2 |
| AC-COC-003 | 补丁型 Feature 演练 | §3.3 |
| AC-COC-004 | 事件流视图呈现 | §3.4 |
| AC-COC-005 | DLQ 重放演练 | §3.5 |
| AC-COC-006 | 渗透测试 | §3.6 |
| AC-COC-007 | AdminService 故障注入 | §3.7 |
| AC-COC-008 | 既有 GM 后台回归 | §3.8 |
| AC-COC-009 | COC UI 自身回滚 | §3.9 |
| AC-COC-010 | 声明式扩展（新 Feature 类型） | §3.10 |

## 1.6 命名约定

| 对象 | 命名格式 | 示例 |
|---|---|---|
| 系统测试用例 | `TST-ST-31-AC<AC编号>-<编号>` | TST-ST-31-AC001-01 |
| 关联 AC | `AC-COC-001` | — |

---

## 2. 测试策略

### 2.1 V 模型映射

```
REQ-031 (§12 验收标准 AC-COC-001~010, §10 NFR)
  │
  ▼
本 ST 设计书 ────► 验证 AC 与 NFR 端到端
  │
  ▼
预发布/生产环境（完整集群：K8s + PostgreSQL 18.4 + Redis 7 + 事件总线 + 全部 Atomic App）
```

### 2.2 测试方法

- 端到端框架：Playwright（COC UI 浏览器自动化）+ k6（性能）+ 自定义 Go/Rust 集成测试
- 测试环境：预发布环境（生产规模 T2 档，模拟 22+ Atomic App 集群）
- 演练工具：自研 `coc-e2e-runner`（基于 RGS-BAS-031 §6 API 封装）
- 渗透测试：第三方安全团队 + 内部安全负责人

### 2.3 测试数据

- 预置 22+ Feature 元数据（覆盖 4 种形态）
- 预置 50+ 事件族注册表
- 预置 3 个进行中 PFAU 实例（declared / canary_in_progress / canary_confirmed）

---

## 3. 测试用例

## 3.1 AC-COC-001: 功能矩阵首页加载

| 用例 ID | 对应验收 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|
| TST-ST-31-AC001-01 | AC-COC-001 | 功能矩阵首屏 | E2E | cluster_operator 登录 GM 后台, 进入 COC UI 首页 | 显示 Feature×版本×节点 三维表, ≤2 秒加载完成 | NFR-COC-001 p95<2s | ◎ |
| TST-ST-31-AC001-02 | AC-COC-001 | 22+ Feature 全展示 | E2E | 检查页面 | 22+ Feature 全可见 | 完整性 | ◎ |
| TST-ST-31-AC001-03 | AC-COC-001 | 缓存漂移检测 | E2E | 手动改 feature_registry.current_version, 刷新页面 | 5 秒内反映新值 | 物化视图刷新 | ◎ |
| TST-ST-31-AC001-04 | AC-COC-001 | 类型筛选 | E2E | 选 feature_type=PLUGIN | 仅显示插件 | FR-COC-010 | ◎ |
| TST-ST-31-AC001-05 | AC-COC-001 | 状态筛选 | E2E | 选 status=in_progress | 仅显示进行中 | FR-COC-010 | ◎ |
| TST-ST-31-AC001-06 | AC-COC-001 | 详情页跳转 | E2E | 点击某 Feature | 详情页含版本历史/依赖/事件/健康 | FR-COC-011 | ◎ |

## 3.2 AC-COC-002: PFAU 升级演练（插件型）

| 用例 ID | 对应验收 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|
| TST-ST-31-AC002-01 | AC-COC-002 | 完整升级流程 | E2E | cluster_admin 升级某插件 Feature 从 v1.0 → v2.0, 5 批灰度, 每批 20% 节点 | 全流程: declared → canary_in_progress(×5) → canary_confirmed → completed | FR-PFAU-010 | ◎ |
| TST-ST-31-AC002-02 | AC-COC-002 | 灰度批次进度可见 | E2E | 检查灰度面板 | 实时显示"批次 1/5, 确认 4/4 节点" | NFR-COC-003 | ◎ |
| TST-ST-31-AC002-03 | AC-COC-002 | 观察期强制 | E2E | 第 1 批完成后立即查 | 当前批次等待观察期, 不自动推进 | FR-PFAU-012 | ◎ |
| TST-ST-31-AC002-04 | AC-COC-002 | 节点确认失败 → 暂停 | E2E | 模拟 1 节点 120 秒不上报 | PFAU 暂停, pause_reason='confirmation_timeout' | FR-PFAU-021 | ◎ |
| TST-ST-31-AC002-05 | AC-COC-002 | 暂停后人工 retry | E2E | cluster_admin 选 retry | 失败节点重试, 状态推进 | FR-PFAU-010 | ◎ |
| TST-ST-31-AC002-06 | AC-COC-002 | 暂停后人工 rollback | E2E | cluster_admin 选 rollback | 启动回滚, 状态到 rolled_back, current_version 回到 v1.0 | FR-PFAU-011 | ◎ |
| TST-ST-31-AC002-07 | AC-COC-002 | 无中间态 | E2E | 升级完成后查全集群节点 | 全部 v2.0, 无 v1.0 残留 | 原子性 | ◎ |
| TST-ST-31-AC002-08 | AC-COC-002 | feature_version_history 追加 | E2E | 升级完成后查历史 | 新增 1 行 state=active | FR-PFAU-003 | ◎ |
| TST-ST-31-AC002-09 | AC-COC-002 | 审计记录 | E2E | 查 operation_audit | 含 feature.upgrade 操作 | FR-COC-040 | ◎ |

## 3.3 AC-COC-003: 补丁型 Feature 演练

| 用例 ID | 对应验收 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|
| TST-ST-31-AC003-01 | AC-COC-003 | 未经门禁拒绝 | E2E | 上线未经自动化测试的补丁 | 拒绝进入灰度, 错误指明门禁失败 | FR-PFAU-041 | ◎ |
| TST-ST-31-AC003-02 | AC-COC-003 | 门禁通过进入灰度 | E2E | 上线已通过门禁的补丁 | 正常进入灰度, FF 切换 | FR-PFAU-041 | ◎ |
| TST-ST-31-AC003-03 | AC-COC-003 | 回滚仅切 FF | E2E | 回滚补丁 Feature | FF 关闭, K8s 镜像**不**回退 | FR-PFAU-042 | ◎ |
| TST-ST-31-AC003-04 | AC-COC-003 | 验证镜像未变 | E2E | 检查 K8s Deployment image tag | 仍为新版本, 未回退 | FR-PFAU-042 | ◎ |
| TST-ST-31-AC003-05 | AC-COC-003 | 补丁不改既有 API | E2E | 尝试补丁修改既有 gRPC 方法 | CI 校验失败, 拒绝 | ARC-015 | ◎ |
| TST-ST-31-AC003-06 | AC-COC-003 | 拒绝 .so/.dll | E2E | 尝试上传动态库 | 拒绝, 错误指明 ADR-0020 | RGS-ADR-0020 | ◎ |

## 3.4 AC-COC-004: 事件流视图呈现

| 用例 ID | 对应验收 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|
| TST-ST-31-AC004-01 | AC-COC-004 | 活跃事件族列表 | E2E | 进入事件流页面 | 50+ 事件族, 每条附"Producer数/Consumer Group数/p99延迟/死信率" | FR-COC-030 | ◎ |
| TST-ST-31-AC004-02 | AC-COC-004 | 数据来自既有可观测性 | SEC | 抓取指标查询流量 | 走 OTel/Prometheus 读端点, **不**新增采集 SDK | ARC-017 | ◎ |
| TST-ST-31-AC004-03 | AC-COC-004 | 详情页 | E2E | 点击某事件族 | 详情含 Schema 当前版本/历史, Producer 列表, Consumer 列表, DLQ 列表, 可重放历史 | FR-COC-031 | ◎ |
| TST-ST-31-AC004-04 | AC-COC-004 | 性能 | P | 加载事件流页面 | p95 < 3 秒 (NFR-COC-002) | NFR-COC-002 | ◎ |
| TST-ST-31-AC004-05 | AC-COC-004 | 读写隔离 | P | 同时跑高频生产路径 + 画布高频操作 | 生产路径延迟无劣化 (NFR-VIZ-003 同精神) | NFR-COC-005 | ◎ |
| TST-ST-31-AC004-06 | AC-COC-004 | 未注册事件告警 | E2E | 生产未注册事件 | 探针告警, 事件流页面显示 | FR-CEM-002 | ◎ |

## 3.5 AC-COC-005: DLQ 重放演练

| 用例 ID | 对应验收 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|
| TST-ST-31-AC005-01 | AC-COC-005 | DLQ 查询 | E2E | 进入 DLQ 页面 | 显示死信列表, 含 original_event_id | FR-CEM-041 | ◎ |
| TST-ST-31-AC005-02 | AC-COC-005 | 重放请求经 AdminService | SEC | 抓取重放流量 | 经 AdminService, **不**直调事件总线 | FR-COC-020 | ◎ |
| TST-ST-31-AC005-03 | AC-COC-005 | 白名单强制 | E2E | 重放不带 Consumer Group 白名单 | REPLAY_DENIED | FR-CEM-052 | ◎ |
| TST-ST-31-AC005-04 | AC-COC-005 | 幂等 | E2E | 重复提交相同 replay_request_id | 第二次 IDEMPOTENT_REPLAY, 不重放 | FR-CEM-042 | ◎ |
| TST-ST-31-AC005-05 | AC-COC-005 | 仅投递白名单 | E2E | 重放到白名单 [group_a, group_b] | group_a/group_b 收到, group_c 没收 | FR-CEM-052 | ◎ |
| TST-ST-31-AC005-06 | AC-COC-005 | 审计 | E2E | 查 operation_audit | 含 dlq.replay 操作 | FR-COC-040 | ◎ |
| TST-ST-31-AC005-07 | AC-COC-005 | 重放 RBAC | SEC | cluster_operator 尝试重放 | RBAC_DENIED | FR-COC-041 | ◎ |
| **TST-ST-31-AC005-08** | AC-COC-005 | DLQ 单条丢弃 | E2E | cluster_admin 选某条 DLQ 事件, 填丢弃原因, 点丢弃 | 事件从 DLQ 物理删除, 不再可重放 | FR-CEM-041 (自审补强) | ◎ |
| **TST-ST-31-AC005-09** | AC-COC-005 | 丢弃必经 AdminService | SEC | 抓取丢弃流量 | 经 AdminService, 不直调事件总线 | FR-COC-020 | ◎ |
| **TST-ST-31-AC005-10** | AC-COC-005 | 丢弃无理由拒绝 | E2E | cluster_admin 丢弃时不填 discard_reason | DISCARD_DENIED | FR-CEM-041 | ◎ |
| **TST-ST-31-AC005-11** | AC-COC-005 | 丢弃幂等 | E2E | 重复提交相同 dlq_event_id + request_id | 第二次 IDEMPOTENT_REPLAY, 不重复删除 | FR-API-003 | ◎ |
| **TST-ST-31-AC005-12** | AC-COC-005 | 丢弃审计 | E2E | 丢弃成功后查 operation_audit | 含 dlq.discard 操作, 含 discard_reason | FR-COC-040 | ◎ |
| **TST-ST-31-AC005-13** | AC-COC-005 | 丢弃 RBAC | SEC | cluster_operator 尝试丢弃 | RBAC_DENIED | FR-COC-041 | ◎ |

## 3.6 AC-COC-006: 渗透测试

| 用例 ID | 对应验收 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|
| TST-ST-31-AC006-01 | AC-COC-006 | COC UI ServiceAccount 凭证范围 | SEC | 检查 COC UI 部署 ServiceAccount | 仅 admin_service ClusterRole, 无 K8s admin/db 直连 | NFR-OPS-004 | ◎ |
| TST-ST-31-AC006-02 | AC-COC-006 | COC UI 不直调 K8s API | SEC | 抓取 COC UI 流量 | 全部经 AdminService, 无 K8s API Server 流量 | NFR-OPS-004 | ◎ |
| TST-ST-31-AC006-03 | AC-COC-006 | COC UI 不直连业务 DB | SEC | 抓取 COC UI → DB 流量 | 走 ClusterOpsService, COC UI 不持有 DB 凭证 | NFR-OPS-004 | ◎ |
| TST-ST-31-AC006-04 | AC-COC-006 | COC UI 不直调运行时控制通道 | SEC | 抓取 COC UI → runtime 流量 | 全部经 AdminService 转发 | NFR-OPS-004 | ◎ |
| TST-ST-31-AC006-05 | AC-COC-006 | ClusterOpsService 不暴露外部 | SEC | 从外部网络访问 ClusterOpsService gRPC 端口 | 连接被 NetworkPolicy 拒绝 | ARC-039 精神 | ◎ |
| TST-ST-31-AC006-06 | AC-COC-006 | 二级提权防护 | SEC | cluster_operator 尝试调高危操作 | RBAC_DENIED | FR-COC-041 | ◎ |
| TST-ST-31-AC006-07 | AC-COC-006 | SQL 注入 | SEC | 在 feature_id 输入 SQL 注入 | 输入校验拒绝 | NFR-SE-* | ◎ |
| TST-ST-31-AC006-08 | AC-COC-006 | 跨站脚本 | SEC | 在 feature display_name 输入 XSS | 输出转义, 不执行 | NFR-SE-* | ◎ |

## 3.7 AC-COC-007: AdminService 故障注入

| 用例 ID | 对应验收 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|
| TST-ST-31-AC007-01 | AC-COC-007 | AdminService 不可用 | FAULT | 杀掉 AdminService Pod | COC UI 进入"只读模式"提示, 写操作被拒绝 | NFR-COC-004 | ◎ |
| TST-ST-31-AC007-02 | AC-COC-007 | 写操作拒绝 | FAULT | AdminService 不可用时尝试写操作 | 拒绝, 不静默重试 | NFR-COC-004 | ◎ |
| TST-ST-31-AC007-03 | AC-COC-007 | 只读模式可用 | FAULT | AdminService 不可用时进入功能矩阵 | 可读, 但不写 | NFR-COC-004 | ◎ |
| TST-ST-31-AC007-04 | AC-COC-007 | 审计通路仍可用 | FAULT | AdminService 不可用时查审计 | 审计查询可用 (走直连 ClusterOpsService gRPC) | NFR-COC-004 | ◎ |
| TST-ST-31-AC007-05 | AC-COC-007 | AdminService 恢复 | FAULT | 重启 AdminService | COC UI 自动恢复写操作 | 恢复 | ◎ |
| TST-ST-31-AC007-06 | AC-COC-007 | PFAU 实例不丢 | FAULT | AdminService 不可用 5 分钟, PFAU 进行中 | PFAU 实例可见 (admin_db 持久化), UI 可见性降级到只读 | NFR-COC-010 | ◎ |
| TST-ST-31-AC007-07 | AC-COC-007 | 玩家侧不受影响 | FAULT | AdminService 不可用时检查玩家业务 | 玩家侧无感知 (隔离设计) | NFR-OPS-006 | ◎ |

## 3.8 AC-COC-008: 既有 GM 后台回归

| 用例 ID | 对应验收 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|
| TST-ST-31-AC008-01 | AC-COC-008 | AC-OPS-001 | REG | 跑既有 GM 后台 6 类操作测试 | 全部通过 (封禁/踢人/禁言/补偿/维护模式/数值表热更新) | AC-OPS-001 | ◎ |
| TST-ST-31-AC008-02 | AC-COC-008 | AC-OPS-002 | REG | 模拟 AdminService 不可用, 玩家侧测试 | 玩家侧无感知 | AC-OPS-002 | ◎ |
| TST-ST-31-AC008-03 | AC-COC-008 | AC-OPS-003 | REG | 渗透测试既有 GM 后台 | 凭证范围未变 | AC-OPS-003 | ◎ |
| TST-ST-31-AC008-04 | AC-COC-008 | AC-OPS-004 | REG | 高危操作二次确认 | 触发二次确认, 二次确认动作有审计 | AC-OPS-004 | ◎ |
| TST-ST-31-AC008-05 | AC-COC-008 | AC-OPS-005 | REG | 告警从触发到送达 | p99 时延满足 NFR-OPS-002 | AC-OPS-005 | ◎ |
| TST-ST-31-AC008-06 | AC-COC-008 | 既有 RBAC | REG | 既有角色 (admin/operator/viewer) 测试 | 全部行为未变 | RBAC 不变 | ◎ |
| TST-ST-31-AC008-07 | AC-COC-008 | 既有页面布局 | REG | 截图对比既有 GM 后台布局 | 布局未变 | UI 兼容性 | ◎ |

## 3.9 AC-COC-009: COC UI 自身回滚

| 用例 ID | 对应验收 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|
| TST-ST-31-AC009-01 | AC-COC-009 | Helm 回滚 ≤30min | ROLL | COC UI 上线, 触发回滚 (helm rollback) | 30 分钟内回到上一版本 | NFR-COC-010 | ◎ |
| TST-ST-31-AC009-02 | AC-COC-009 | PFAU 实例可见性不丢 | ROLL | COC UI 回滚后, 检查 PFAU 进行中实例 | 实例仍可见 (admin_db 持久化) | NFR-COC-010 | ◎ |
| TST-ST-31-AC009-03 | AC-COC-009 | 功能矩阵仍可查 | ROLL | COC UI 回滚后查功能矩阵 | 可查 (走 ClusterOpsService gRPC) | NFR-COC-004 | ◎ |
| TST-ST-31-AC009-04 | AC-COC-009 | 新版本 PFAU 写操作暂停 | ROLL | COC UI 回滚时若有 PFAU 在写 | 写操作可继续 (经 AdminService), 不被回滚影响 | 隔离 | ◎ |
| TST-ST-31-AC009-05 | AC-COC-009 | 既有 GM 后台不受影响 | ROLL | COC UI 回滚后查既有 GM 后台 | 既有页面正常 | NFR-COC-009 | ◎ |

## 3.10 AC-COC-010: 声明式扩展（新 Feature 类型）

| 用例 ID | 对应验收 | 测试目的 | 覆盖类型 | 步骤 | 预期 | 判定 | 优先级 |
|---|---|---|---|---|---|---|---|
| TST-ST-31-AC010-01 | AC-COC-010 | 新增 Feature 类型 | E2E | 添加 'experiment' 类型 (PFAU 状态机变体) | 仅扩展 feature_type 枚举 + PFAU 变体, COC UI 核心代码未改 | NFR-COC-006 | ◎ |
| TST-ST-31-AC010-02 | AC-COC-010 | COC UI 渲染新类型 | E2E | COC UI 功能矩阵 | 自动识别新类型, 渲染对应图标/列 | 声明式配置 | ◎ |
| TST-ST-31-AC010-03 | AC-COC-010 | PFAU 状态机变体生效 | E2E | 新类型的 PFAU 实例 | 按变体状态机运行 | 扩展性 | ◎ |
| TST-ST-31-AC010-04 | AC-COC-010 | 视图声明式配置 | E2E | 加新业务视图 (如"实验视图") | 仅新增声明式配置, 前端不开发 | NFR-COC-006 | ◎ |

---

## 4. NFR 端到端验证

| NFR ID | 目标值 | ST 验证 | 测试方法 | 判定 | 优先级 |
|---|---|---|---|---|---|
| NFR-COC-001 | 功能矩阵首页 p95<2s | TST-ST-31-AC001-01 | k6 模拟 50 并发用户加载首页 | p95 < 2s | ◎ |
| NFR-COC-002 | 事件流视图 p95<3s | TST-ST-31-AC004-04 | k6 模拟 20 并发 | p95 < 3s | ◎ |
| NFR-COC-003 | PFAU 状态变更 p99<1s | TST-ST-31-AC002-02 | 端到端计时 | p99 < 1s | ◎ |
| NFR-COC-004 | COC UI 不可用时 PFAU 可见 | TST-ST-31-AC007-04 | 故障注入 | 降级到只读 | ◎ |
| NFR-COC-005 | 画布查询不干扰生产路径 | TST-ST-31-AC004-05 | 性能隔离测试 | 生产延迟无劣化 | ◎ |
| NFR-COC-006 | 新增 Feature 类型不修改 UI 核心 | TST-ST-31-AC010-01 | 扩展性测试 | 满足 | ◎ |
| NFR-COC-007 | RBAC + 幂等 + 安全 | TST-ST-31-AC006-01~08 | 渗透测试 | 全部满足 | ◎ |
| NFR-COC-008 | 3 年审计保留 | REG | 查 operation_audit | 满足 | ◎ |
| NFR-COC-009 | 既有 GM 后台不变 | TST-ST-31-AC008-01~07 | 回归测试 | 全部通过 | ◎ |
| NFR-COC-010 | 30 分钟回滚 | TST-ST-31-AC009-01~05 | 回滚演练 | ≤30 分钟 | ◎ |

---

## 5. 追溯性矩阵

| AC ID | ST 用例 | 覆盖的 REQ FR | 覆盖的 NFR |
|---|---|---|---|
| AC-COC-001 | TST-ST-31-AC001-01~06 | FR-COC-001~012 | NFR-COC-001 |
| AC-COC-002 | TST-ST-31-AC002-01~09 | FR-PFAU-010, 011, 012, 020, 021, 050, 051 | NFR-COC-003, 004 |
| AC-COC-003 | TST-ST-31-AC003-01~06 | FR-PFAU-040, 041, 042 | — |
| AC-COC-004 | TST-ST-31-AC004-01~06 | FR-COC-030, 031, 032, FR-CEM-002 | NFR-COC-002, 005 |
| AC-COC-005 | TST-ST-31-AC005-01~07 | FR-CEM-041, 042, 051, 052, FR-COC-041 | NFR-COC-007 |
| AC-COC-006 | TST-ST-31-AC006-01~08 | FR-COC-020, 040, 041, 042 | NFR-COC-007, 008 |
| AC-COC-007 | TST-ST-31-AC007-01~07 | FR-COC-020, NFR-COC-004, 010 | NFR-COC-004, 010 |
| AC-COC-008 | TST-ST-31-AC008-01~07 | (RGS-REQ-007 AC-OPS-001~005) | NFR-COC-009 |
| AC-COC-009 | TST-ST-31-AC009-01~05 | (NFR-COC-010) | NFR-COC-009, 010 |
| AC-COC-010 | TST-ST-31-AC010-01~04 | (NFR-COC-006) | NFR-COC-006 |

---

## 6. 测试执行计划

| 阶段 | 用例范围 | 预计时长 | 前置条件 |
|---|---|---|---|
| **Phase 1: 预发布环境部署** | — | 1 天 | 22+ Feature 元数据预置 |
| **Phase 2: 端到端功能演练** | AC-001 ~ AC-005 | 2 天 | Phase 1 完成 |
| **Phase 3: 渗透测试** | AC-006 | 1 天 (含第三方) | Phase 2 通过 |
| **Phase 4: 故障注入** | AC-007 | 0.5 天 | Phase 2 通过 |
| **Phase 5: 回归测试** | AC-008 | 0.5 天 | Phase 2 通过 |
| **Phase 6: 回滚演练** | AC-009 | 0.5 天 | Phase 1 完成 |
| **Phase 7: 扩展性验证** | AC-010 | 0.5 天 | Phase 2 通过 |
| **Phase 8: NFR 实测** | §4 全部 | 1 天 | 全部 Phase 通过 |

总计: **7 天**

---

## 7. 通过判定基准

| 门禁 | 阈值 |
|---|---|
| 全部 10 个 AC | 100% 通过 |
| 全部 NFR | 100% 满足目标值 |
| 全部 ◎ 用例 | 100% 通过 |
| 全部 ○ 用例 | ≥95% 通过 |
| 渗透测试 | 0 高危 / 0 中危漏洞 |
| 既有 GM 后台回归 | 100% 通过 |
| COC UI 回滚 | ≤30 分钟 |

---

## 8. 风险与未决事项（TBD 处置）

| TBD ID | 内容 | ST 处置 |
|---|---|---|
| TBD-COC-001 | 无限画布前端选型 | **由 ST 实测验证** — TST-ST-31-AC004-01~06 验证 |
| TBD-COC-002 | 补丁型金丝雀测试门禁 | **按保守假设实施** — ST 验证"门禁失败拒绝进入灰度"路径 |
| TBD-COC-003 | 批量回滚上限 20 | **已实施** — ST 验证 |
| TBD-COC-004 | 事件注册变更事件保留 | **已实施** — ST 验证 |
| TBD-COC-005 | feature_registry 分区策略 | **由性能测试覆盖** — ST §4 NFR-COC-001 |
| TBD-COC-006 | pfa_run_state 历史归档 | **按保守假设实施** — ST 验证 90 天内可查询 |
| RSK-COC-001 | CI 校验脚本 | **已实施** — ST 验证 |
| RSK-COC-002 | PFAU 超时阈值 120 秒 | **已实施** — ST 验证 |
| RSK-COC-003 | COC UI 弱化 RGS-OPS-001 | **由 UX 评审 + ST AC-008 覆盖** |
| RSK-COC-004 | CEM 存储成本与合规 | **由 ST 性能 + 详细设计评估** |
| ISS-092 | COC UI OLU 核算 | **待 PH-7 前完成** |

---

> 本文档配套 RGS-REQ-031 / RGS-BAS-031 / RGS-ADR-0051 / RGS-TST-UT-31 / RGS-TST-IT-31。完成 ARC-051 全部 V 模型层级（REQ → BAS → DTL → UT → IT → ST）的设计与验证闭环。

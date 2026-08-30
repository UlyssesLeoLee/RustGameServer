# 单元测试设计书（GM 后台 / Unit Test Design Document）

**目录 08 GM 后台  单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-08 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-003 运维与 GM 后台管控 / RGS-DTL-003 详细设计 / RGS-DTL-040 Admin 域详细设计 |
| 适用范围 | 验证 GM 后台 APIGW(gm-backend crate)的纯函数式业务逻辑、数据结构、状态机、序列化 |
| V 模型层级 | TL-1 单元测试 → DTL 详细设计 |
| 编制标准 | IPA 共通フレーム 2013(SLCP-JCF2013)详细设计工程 / RGS-REQ-001 §12.1 |
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签） |
| 编制日期 | 2026-08-27 23:35 JST |
| 密级 | 内部限定(Internal Use Only) |
| 许可证 | Apache-2.0(本仓库) |
| 关联源代码文档 | RGS-REQ-007, RGS-BAS-003, RGS-REQ-019, RGS-REQ-020, RGS-REQ-024, RGS-BAS-021, RGS-DTL-003, RGS-DTL-040 |
| 关联基本设计 | RGS-BAS-003, RGS-BAS-009, RGS-BAS-021 |
| 关联测试代码 | `crates/gm-backend/tests/ut_config.rs`（已实现 6 测试） |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签） | 2026-08-27 23:35 JST | 初次编制:8 域第 8 域 GM 后台测试设计书（补全 7 域→8 域覆盖缺口） |

## 签字栏

| 角色 | 署名 | 签字日期 | 备注 |
|---|---|---|---|
| 编制（兼签）| 架构师 | 2026-08-27 | per DEC-008 一人公司 12 角色兼任 |
| 需求（架构师）| | | per 2026-08-26 04:30 JST DDD Review 阶段补 |
| 设计 QA 员 | | | 待具名（per Q2 OPEN-QA） |
| 变更控制委员会 | | | DDD Review 阶段补 |

## 目录

1. 前言（Preface）
   1.1 目的（Purpose）
   1.2 适用范围（Scope）
   1.3 关联文档（Related Documents）
   1.4 术语与标记规则（Notation Rules）
   1.5 字段级映射说明
   1.6 命名约定（Naming Convention）
2. 测试策略（Test Strategy）
3. 测试用例（Test Cases）
4. 追溯矩阵（Traceability Matrix）
5. 测试执行计划（Test Execution Plan）
6. 通过判定标准（Pass Criteria）
7. 风险与未决事项（Risks and TBDs）

注：本文件实际以下章节内容为准。

---

## 1. 前言

## 1.1 目的（Purpose）

本文件为 V 模型 **TL-1 单元测试**层级设计书，对应详细设计 **RGS-BAS-003（运维与 GM 后台管控基本设计书）/ RGS-DTL-003（运维与 GM 后台管控详细设计书）/ RGS-DTL-040（Admin 域详细设计书）**。本版本为 0.1 初次编制（per 2026-08-27 23:35 JST Ulysses "UT/IT/ST 测试设计书齐全吗"指令，补全 8 域第 8 域 GM 后台测试设计书缺口）。

- 将 RGS-BAS-003 §3.1-§3.4 各项 GM 后台 API 字段级 API 设计与 RGS-DTL-040 Admin 域 5 域契约，分解为**可执行单元测试用例**
- 验证 gm-backend crate 内部纯函数逻辑：GmConfig 配载、handler 输入输出、Router 路由、health endpoint 边界
- 配合 RGS-REQ-001 §12.2 QA-001 单元测试覆盖率要求，单测覆盖率 ≥ 80%
- 满足 QA-002 主要路径覆盖率要求，业务逻辑覆盖率 ≥ 70%

## 1.2 适用范围（Scope）

| 边界 | 说明 |
|---|---|
| 包含 | gm-backend crate（src/lib.rs + src/main.rs）中所有 pub fn + pub struct + handler 函数；gm-backend 5 域 + GM 后台 endpoint 全部字段 |
| 排除 | 集成测试（见 RGS-TST-IT-08）、系统测试（见 RGS-TST-ST-08）、k3s 部署验证（见 `scripts/e2e-smoke.ps1`）、JWT validation v0.2（见 §7 TBD-08-01） |

## 1.3 关联文档（Related Documents）

| 文档编号 | 文档名 | 与本文件关系 |
|---|---|---|
| RGS-REQ-007 运维与 GM 后台管控 需求定义书 | 需求 | 来源 |
| RGS-REQ-019 智能决策层（无埋点可观测性增强）需求定义书 | 需求 | 观测字段来源 |
| RGS-REQ-024 GM 后台多人可观测化漏斗 需求定义书 | 需求 | 观测字段来源 |
| RGS-BAS-003 运维与 GM 后台管控 基本设计书 | 设计 | 父文档 |
| RGS-BAS-021 GM 后台多人可观测化漏斗 基本设计书 | 设计 | 父文档 |
| RGS-DTL-003 运维与 GM 后台管控 详细设计书 | 详细设计 | 父文档 |
| RGS-DTL-040 Admin 域 详细设计书 | 详细设计 | 父文档 |
| RGS-TST-UT-01 核心架构与设计模式 单元测试设计书 | 参考 | V 模型对应 |
| RGS-TST-UT-00 基准与治理 单元测试设计书 | 参考 | V 模型对应 |
| RGS-IMPL-001 实施约定与工程边界 | 实施约束 | 见 workspace / dep 约束 |

## 1.4 术语与标记规则（Notation Rules）

### 1.4.1 强约束标记（RFC 2119 / IPA 共通框架 2013）

| 中文 | 英文 | 强约束度 |
|---|---|---|
| **必须** / 必 | MUST | 强制 |
| **应当** / 应 | SHOULD | 强推荐 |
| **不得** / 禁 | MUST NOT | 强制 |
| **可** / 许 | MAY | 可选 |

### 1.4.2 优先级

| 标记 | 含义 | 处理 |
|---|---|---|
| P0 | 紧急 | 当前阶段必须实现 |
| P1 | 强推荐 | 当前阶段应实现 |
| P2 | 推荐 | 中后期补 |
| P3 | 范围外 | 留待下期 |

### 1.4.3 标识符体系

- `RGS-TST-{UT|IT|ST}-XX-NNN`：测试设计书
- `RGS-TST-{UT|IT|ST}-XX-NNN-AAA`：测试用例
- `RGS-{REQ|BAS|DTL}-NNN`：核心文档
- `RGS-ADR-NNNN`：架构决策记录
- `NFR-<类>-NNN`：非功能需求
- `AC-NNN` / `VF-NNN` / `FT-NNN`：验收 / 验证 / 容错用例

## 1.5 字段级映射说明

本版本为 0.1 初次编制，**强调字段级映射**：每个测试用例"对应需求"列精确到"REQ/BAS/DTL §X.Y + 字段/接口"。

**V 模型强对应**：本文件对应"GM 后台 APIGW 详细设计 + 字段级 API"，"V 左"上游 REQ/BAS，"V 右"下游 IT/ST。

## 1.6 命名约定（Naming Convention）

- 测试 ID：`TST-{UT|IT|ST}-08-NNN`
- V 模型层级标注：UT 无标注
- 用例类型：N=正常 / A=异常 / B=边界 / P=性能(不适用 UT) / S=状态机
- 测试运行时：`cargo test -p gm-backend`

---

## 2. 测试策略

## 2.1 V 模型对应关系

```
需求   RGS-REQ-007/019/024  → ST  (RGS-TST-ST-08)
设计   RGS-BAS-003/021       → IT  (RGS-TST-IT-08)
详细   RGS-DTL-003/040       → UT  (RGS-TST-UT-08,本文件)
实现   Rust 源码             ←
```

## 2.2 测试层次

| 层次 | 范围 | 工具 |
|---|---|---|
| L1 | crate 内模块 | `cargo test -p gm-backend` |
| L2 | workspace 内 crate | N/A(gm-backend 是单 binary crate) |

## 2.3 接口契约

- gRPC 客户端 stub：未来 v0.2 实装 admin-service gRPC client
- HTTP 入口：axum 0.7 (RGS-BAS-003 §2.1)
- 环境变量：GmConfig::from_env() 解析

## 2.4 测试质量目标

| 维度 | 目标 |
|---|---|
| 字段级覆盖率 | 100%（覆盖当前 stub 实现的既有字段，**不含 v0.2 admin-service 协议字段**——per 2026-08-28 跨反馈 F8 处置） |
| 业务路径覆盖率 | ≥ 70% |
| 缺陷密度 | ≤ 1.0 个/KLOC（QA-004） |
| 边界用例 | 至少 3 类（无效 SocketAddr / 缺失 env / 配置越界） |

---

## 3. 测试用例

## 3.1 模块 A：GmConfig 配载（无上游详细设计依据，实现阶段新增）

> **2026-08-28 跨反馈 F7 处置**:GmConfig 的 http_addr/health_addr/admin_grpc_endpoint/jwt_secret 四个字段在 RGS-DTL-040、RGS-DTL-003、RGS-BAS-003 全文中均无对应设计条目,本模块为"实现阶段自行引入、未走详细设计流程"的配置项。

| 测试 ID | 对应需求 | 字段/映射 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-08-A001 | 无上游设计依据,实现阶段新增 | http_addr=0.0.0.0:8443, health_addr=0.0.0.0:8081, admin_grpc_endpoint="https://admin-service:50055", jwt_secret="dev-only-do-not-use-in-prod" | N | env 缺失时 from_env 返回默认值 |
| TST-UT-08-A002 | 无上游设计依据,实现阶段新增 | http_addr, health_addr, admin_grpc_endpoint, jwt_secret 4 字段 | N | 显式 set_var 后 from_env 读到覆盖值 |
| TST-UT-08-A003 | 无上游设计依据,实现阶段新增 | http_addr 解析失败 | A | 返回 anyhow::Error，msg 含 "invalid GM_HTTP_ADDR" |
| TST-UT-08-A004 | 无上游设计依据,实现阶段新增 | health_addr 解析失败 | A | 返回 anyhow::Error，msg 含 "invalid GM_HEALTH_ADDR" |
| TST-UT-08-A005 | 无上游设计依据,实现阶段新增 | 3 参数（http/str, health/str, admin/str） | N | builder 构造出符合预期 GmConfig |
| TST-UT-08-A006 | 无上游设计依据,实现阶段新增 | GmConfig Clone 实现 | N | clone 后 PartialEq 相等 |

**实现位置**：`crates/gm-backend/tests/ut_config.rs`（6 测试,2026-08-27 22:53 JST 19/19 PASS）

## 3.2 模块 B：AppState + Router 构造（无对应详细设计章节，实现阶段新增）

> **2026-08-28 跨反馈 F7 处置**:DTL-040 §3 全文仅是 AdminService/ClusterOpsService/COC UI 三层职责表,无 §3.1/§3.2/§3.3/§3.4 编号子章节,亦不涉及 axum Router 构造。Router 7 路由与 health 2 路由属实现阶段设计。

| 测试 ID | 对应需求 | 字段/映射 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-08-B001 | 无对应详细设计,实现阶段新增 | 7 路由：/healthz, /readyz, /api/v1/gm/health/view, /api/v1/gm/ban, /api/v1/gm/compensation, /api/v1/gm/maintenance, /api/v1/audit/logs | N | build_router() 包含所有 7 路由 |
| TST-UT-08-B002 | 无对应详细设计,实现阶段新增 | build_health_router() 仅 2 路由 | N | health 路由不含 /api/v1/gm/* 业务端点 |
| TST-UT-08-B003 | 无对应详细设计,实现阶段新增 | AppState Clone 实现 | N | Clone 后两个 handle 共享同一 config |

**实现位置**：`crates/gm-backend/tests/integration_gm_basic.rs`（含部分 L1 单元测试 + in-process 集成）

## 3.3 模块 C：fail-closed 启动（BAS-003 §2.1 启动约束 + 实现阶段扩展）

> **2026-08-28 跨反馈 F7 处置**:DTL-040 §3 全文仅是 AdminService/ClusterOpsService/COC UI 三层职责表,无 §3.4 编号子章节,亦不涉及 fail-closed 启动语义。C002 标注"无对应详细设计,实现阶段预留"。

| 测试 ID | 对应需求 | 字段/映射 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-08-C001 | BAS-003 §2.1 + 5 域 fail-closed 模式 | env 全部 0.0.0.0:0 | N | 5s 内启动，stderr 含 "starting GM APIGW" 或进程仍在跑 |
| TST-UT-08-C002 | 无对应详细设计,实现阶段预留 | RGS_TLS_DIR 缺失 | A | **TBD-08-02** v0.2 实装: mTLS 必须 fail-closed（per 5 域 RGS_ALLOW_INSECURE_GRPC=0 默认） |

**实现位置**：`crates/gm-backend/tests/fail_closed_start.rs`（1 测试）

## 3.4 模块 D：Handler 输入输出（多源追溯，见各行）

> **2026-08-28 跨反馈 F7 处置**:模块 D 7 行 BAS-003 引用逐条已核对: D001→§3.4(QueryHealthView), D002/D003→`RGS-BAS-001 §6.3.4`(AdminService 既有方法定义处,本模块 UT-08 §1.3 关联文档此前未列), D004→§3.3(SetMaintenanceMode), D005→§3.4(QueryAuditLog)保留, D006/D007 标注"无 BAS-003 §3 对应依据"(k8s 探针非 AdminService 方法)。DTL-040 §3.3 整体不存在,处理同模块 A/B/C。

| 测试 ID | 对应需求 | 字段/映射 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-08-D001 | BAS-003 §3.4 QueryHealthView（stub 字段见 F8 处置） | 响应字段:service, admin_endpoint, mode | N | 返回 200 + 含 3 字段 |
| TST-UT-08-D002 | RGS-BAS-001 §6.3.4 AdminService.BanAccount（既有方法，§3 范围外） | 响应字段:status, op | N | 返回 202 + status=queued, op=ban |
| TST-UT-08-D003 | RGS-BAS-001 §6.3.4 AdminService.GrantCompensation（既有方法，§3 范围外） | 响应字段:status, op | N | 返回 202 + status=queued, op=compensation |
| TST-UT-08-D004 | BAS-003 §3.3 SetMaintenanceMode（stub 字段见 F8 处置） | 响应字段:status, op | N | 返回 202 + status=queued, op=maintenance |
| TST-UT-08-D005 | BAS-003 §3.4 QueryAuditLog（stub 字段见 F8 处置） | 响应字段:items, next | N | 返回 200 + items=[] + next=stub |
| TST-UT-08-D006 | 无 BAS-003 §3 对应依据（k8s 探针） | 响应字段:status, service | N | 返回 200 + service=gm-backend |
| TST-UT-08-D007 | 无 BAS-003 §3 对应依据（k8s 探针） | 响应字段:status, service | N | 返回 200 + service=gm-backend |

**实现位置**：`crates/gm-backend/tests/integration_gm_basic.rs`（12 测试）

## 3.5 模块 E：Router 路由边界（BAS-003 §2.1 路由表 + 实现阶段新增边界）

> **2026-08-28 跨反馈 F7 处置**:DTL-040 §3.2 编号子章节不存在,本模块路由边界测试无对应详细设计,实现阶段新增。

| 测试 ID | 对应需求 | 字段/映射 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-08-E001 | BAS-003 §2.1 GET 端点拒绝 POST | /healthz POST | A | 返回 405 Method Not Allowed |
| TST-UT-08-E002 | BAS-003 §2.1 POST 端点拒绝 GET | /api/v1/gm/ban GET | A | 返回 405 |
| TST-UT-08-E003 | 无对应详细设计,实现阶段新增 | /api/v1/gm/nonexistent GET | A | 返回 404 Not Found |
| TST-UT-08-E004 | 无对应详细设计,实现阶段新增 | /api/v1/gm/health/view GET (在 build_health_router) | A | 返回 404 |

**实现位置**：`crates/gm-backend/tests/integration_gm_basic.rs`（4 测试）

---

## 4. 追溯矩阵（Traceability Matrix）

| 测试 ID | RGS-REQ | RGS-BAS | RGS-DTL | 测试代码 |
|---|---|---|---|---|
| TST-UT-08-A001 | REQ-007 §3.4 | 无上游设计依据（实现阶段新增） | 无上游设计依据（实现阶段新增） | `ut_config.rs::gm_config_defaults_when_env_missing` |
| TST-UT-08-A002 | REQ-007 §3.4 | 无上游设计依据（实现阶段新增） | 无上游设计依据（实现阶段新增） | `ut_config.rs::gm_config_respects_env_overrides` |
| TST-UT-08-A003 | REQ-007 §3.4 | 无上游设计依据（实现阶段新增） | 无上游设计依据（实现阶段新增） | `ut_config.rs::gm_config_rejects_invalid_socket_addr` |
| TST-UT-08-A004 | REQ-007 §3.4 | 无上游设计依据（实现阶段新增） | 无上游设计依据（实现阶段新增） | `ut_config.rs::gm_config_rejects_invalid_health_addr` |
| TST-UT-08-A005 | REQ-007 §3.4 | 无上游设计依据（实现阶段新增） | 无上游设计依据（实现阶段新增） | `ut_config.rs::gm_config_for_test_builder` |
| TST-UT-08-A006 | REQ-007 §3.4 | 无上游设计依据（实现阶段新增） | 无上游设计依据（实现阶段新增） | `ut_config.rs::gm_config_clone_equality` |
| TST-UT-08-B001~B003 | REQ-007 §3.1 | BAS-003 §2.1 | 无对应详细设计（实现阶段新增） | `integration_gm_basic.rs::*_returns_*` |
| TST-UT-08-C001 | REQ-007 §3.1 | BAS-003 §2.1 | 无对应详细设计（实现阶段扩展） | `fail_closed_start.rs::gm_backend_starts_with_defaults` |
| TST-UT-08-C002 | REQ-007 §3.1 | BAS-003 §2.1 | 无对应详细设计（实现阶段预留） | **TBD-08-02** v0.2 实装 |
| TST-UT-08-D001 | REQ-007 §3.4 | BAS-003 §3.4 QueryHealthView（stub 字段） | 无对应详细设计（实现阶段扩展） | `integration_gm_basic.rs::health_view_returns_*` |
| TST-UT-08-D002 | REQ-007 §3.4 | BAS-001 §6.3.4 AdminService.BanAccount（既有方法） | 无对应详细设计 | `integration_gm_basic.rs::ban_account_returns_*` |
| TST-UT-08-D003 | REQ-007 §3.4 | BAS-001 §6.3.4 AdminService.GrantCompensation（既有方法） | 无对应详细设计 | `integration_gm_basic.rs::grant_compensation_returns_*` |
| TST-UT-08-D004 | REQ-007 §3.4 | BAS-003 §3.3 SetMaintenanceMode（stub 字段） | 无对应详细设计（实现阶段扩展） | `integration_gm_basic.rs::set_maintenance_returns_*` |
| TST-UT-08-D005 | REQ-007 §3.4 | BAS-003 §3.4 QueryAuditLog（stub 字段） | 无对应详细设计 | `integration_gm_basic.rs::query_audit_returns_*` |
| TST-UT-08-D006 | REQ-007 §3.4 | 无对应详细设计（k8s 探针） | 无对应详细设计（k8s 探针） | `integration_gm_basic.rs::healthz_returns_*` |
| TST-UT-08-D007 | REQ-007 §3.4 | 无对应详细设计（k8s 探针） | 无对应详细设计（k8s 探针） | `integration_gm_basic.rs::readyz_returns_*` |
| TST-UT-08-E001 | REQ-007 §3.1 | BAS-003 §2.1 | 无对应详细设计（实现阶段新增） | `integration_gm_basic.rs::main_router_does_not_accept_*` |
| TST-UT-08-E002 | REQ-007 §3.1 | BAS-003 §2.1 | 无对应详细设计（实现阶段新增） | `integration_gm_basic.rs::main_router_does_not_accept_*` |
| TST-UT-08-E003 | REQ-007 §3.1 | 无对应详细设计（实现阶段新增） | 无对应详细设计（实现阶段新增） | `integration_gm_basic.rs::unknown_route_*` |
| TST-UT-08-E004 | REQ-007 §3.1 | 无对应详细设计（实现阶段新增） | 无对应详细设计（实现阶段新增） | `integration_gm_basic.rs::health_router_does_not_expose_*` |

**总计**：22 测试用例 ID（21 已实现 + 1 TBD-08-02 待 v0.2）。模块 A 6 + 模块 B 3 + 模块 C 2 + 模块 D 7 + 模块 E 4 = 22（per 2026-08-28 跨反馈 F3/F7 处置，原 §4 自报"23"已纠正）。

> **2026-08-28 跨反馈 F7 处置续**:原 §4 追溯矩阵"全部 22 条"用例 ID 都在 DTL-040 列标了一个 §3.x 子章节号,而 DTL-040 §3 全文没有任何数字编号子章节,22 条无一存在。已逐条改为"无对应详细设计(实现阶段新增/扩展/预留)"或挂到真实存在的父章节(BAS-001 §6.3.4 / BAS-003 §2.1 / §3.1-§3.4)。DTL-040 因文档头自标"契约骨架·待评审·不得作为实施授权",本表仅保留"实现阶段新增/扩展"占位,不在此引用。

---

## 5. 测试执行计划（Test Execution Plan）

| 阶段 | 工具 | 命令 | 触发 |
|---|---|---|---|
| L1 本地 | cargo | `cargo test -p gm-backend` | 每次 commit 必跑 |
| L1 CI | cargo + CI | `.github/workflows/rust-ci.yml` | push to main |
| L1 dev-deps 锁版本 | serial_test 0.5 | env 隔离 | 必跑 |
| 覆盖率 | cargo-llvm-cov | 后续 | TBD |

**已知 bug**：
- 本机 CI 暂未集成 gm-backend 5 域 PG fixture（per 6763baa 5 域已集成，gm-backend 暂不需要 DB）
- DDD Review 阶段需补覆盖率门槛

---

## 6. 通过判定标准（Pass Criteria）

| 维度 | 通过阈值 |
|---|---|
| 测试通过率 | 100% |
| 字段级映射覆盖率 | 100%（覆盖当前 stub 实现的既有字段——GmConfig 4 字段 + 7 endpoint × 1-3 字段；**不含 v0.2 admin-service 协议字段**，per 2026-08-28 跨反馈 F8 处置） |
| 编译警告 | 0（除 `#[allow(dead_code)] jwt_secret` 故意外） |
| 业务路径覆盖率 | ≥ 80%（per QA-001 80% 阈值） |
| 测试代码行 | 19 测试函数 ≥ 200 行 |
| env 隔离 | `#[serial]` 标记 4 个 env-mutating 测试 |
| 文档同步 | 本设计书 v0.1 + 测试代码 + 5 域/cluster-ops 模式对齐 |

---

## 7. 风险与未决事项（Risks and TBDs）

| 编号 | 描述 | 风险等级 | 解决路径 |
|---|---|---|---|
| TBD-08-01 | JWT validation 未实装（`jwt_secret` 字段保留 `#[allow(dead_code)]`） | P1 | v0.2 实装：JWT middleware + `axum::middleware::from_fn` |
| TBD-08-02 | mTLS 启动 fail-closed 路径未实装（per 5 域 RGS_ALLOW_INSECURE_GRPC=0 模式） | P1 | v0.2 实装：`shared_platform::tls::load_server_tls_config` 集成 |
| TBD-08-03 | 5 个 GM endpoint 仍 stub（返回硬编码 202 queued） | P2 | v0.2 实装：admin-service gRPC client + rgs-testkit mock。<br/>**2026-08-28 跨反馈 F8 处置补充**——v0.2 实装时需新增/调整测试覆盖以下 BAS-003/DTL-003 字段级协议字段（当前 stub 字段 ≠ 设计字段，UT-08-D001/D004/D005 未测到这些）：<br/>① `SetMaintenanceModeResponse` 新增 `propagation_status`（枚举 PROPAGATING/CONVERGED，per BAS-003 §3.3 + DTL-003 §3.3）——覆盖 D004<br/>② `QueryHealthViewResponse` = `repeated ServiceHealthEntry services`，每条含 `service_name`/`ready`/`queue_depth`/`db_pool_usage_ratio`/`checked_at_ms`（per BAS-003 §3.4 + DTL-003 §3.4）——覆盖 D001<br/>③ `QueryAuditLogResponse` = `repeated AuditLogEntry entries` + `bool has_more`（per BAS-003 §3.4 + DTL-003 §3.4）——覆盖 D005 |
| TBD-08-04 | 审计查询 endpoint（`/api/v1/audit/logs`）返回空 items | P2 | v0.2 实装：audit_log 表 join 查询 |
| TBD-08-05 | 覆盖率阈值门槛（80% / 70%）未在 CI 强制 | P2 | 加 cargo-llvm-cov + 阈值 |
| TBD-08-06 | `axum-test 16` 与 7 域使用的 `wiremock` 不一致 | P3 | 评估统一 mock 工具 |
| TBD-08-07 | `rgs-testkit` dev-dep 已声明但 19 测试未使用 | P3 | v0.2 集成 admin-service gRPC client 时使用 |
| TBD-08-08 | 5 域 Lead 实际具名状态（per OPEN-QA Q2）：gm-backend 域 Lead 仍未具名 | P1 | DDD Review 阶段需 Ulysses 决策 |

**保留派生约束**（per 2026-08-26 04:30 JST）：
- 禁"per X 历史形态"等回溯叙事
- 引用 BAS 必须 git log -p --follow 实证
- 缺标比错标安全
- 子代理授权边界要写明"无证据叙事 = 禁止"

---

**作者**：架构师（Mavis 接手 agent per DEC-008,代签）  
**时间**：2026-08-27 23:35 JST  
**后续**：DDD Review 时由 Ulysses + 5 域 Lead 联合审

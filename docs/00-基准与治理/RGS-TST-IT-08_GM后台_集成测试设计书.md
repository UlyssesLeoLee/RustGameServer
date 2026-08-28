# 集成测试设计书（GM 后台 / Integration Test Design Document）

**目录 08 GM 后台  集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-08 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-003 运维与 GM 后台管控 / RGS-DTL-003 详细设计 / RGS-DTL-040 Admin 域详细设计 |
| 适用范围 | 验证 GM 后台 APIGW(gm-backend)与 axum Router、HTTP 端到端、gm-backend ↔ admin-service 集成（未来 v0.2）|
| V 模型层级 | TL-2 接口契约 / TL-3 协议一致性 / TL-4 集成(端到端) |
| 编制标准 | IPA 共通フレーム 2013(SLCP-JCF2013)详细设计工程 |
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签） |
| 编制日期 | 2026-08-27 23:38 JST |
| 密级 | 内部限定(Internal Use Only) |
| 许可证 | Apache-2.0(本仓库) |
| 关联源代码文档 | RGS-REQ-007, RGS-REQ-019, RGS-REQ-024, RGS-BAS-003, RGS-BAS-021, RGS-DTL-003, RGS-DTL-040 |
| 关联基本设计 | RGS-BAS-003, RGS-BAS-009, RGS-BAS-021 |
| 关联测试代码 | `crates/gm-backend/tests/integration_gm_basic.rs`（已实现 12 测试） |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签） | 2026-08-27 23:38 JST | 初次编制：8 域第 8 域 GM 后台集成测试设计书（补全 7 域→8 域覆盖缺口） |

## 签字栏

| 角色 | 署名 | 签字日期 | 备注 |
|---|---|---|---|
| 编制（兼签）| 架构师 | 2026-08-27 | per DEC-008 一人公司 12 角色兼任 |
| 需求（架构师）| | | DDD Review 阶段补 |
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

本文件为 V 模型 **TL-2 接口契约 / TL-3 协议一致性 / TL-4 集成（端到端）**层级设计书，对应 RGS-BAS-003（运维与 GM 后台管控基本设计书）/ RGS-DTL-003（运维与 GM 后台管控详细设计书）/ RGS-DTL-040（Admin 域详细设计书）。本版本为 0.1 初次编制（per 2026-08-27 23:35 JST Ulysses 指令"UT/IT/ST 测试设计书齐全吗"）。

- 验证 RGS-BAS-003 §2.1 中各 GM 后台 **HTTP 接口契约**的端到端行为
- 验证 GM 后台 7 endpoint（healthz / readyz / 5 GM 操作 + 审计查询）字段级响应 schema
- 验证 axum Router 路由表（7 主路由 + 2 health 路由）的 method/route 边界
- 验证 8081 health-only router 与 8443 主 router 的端口隔离
- 为 v0.2 实装 admin-service gRPC client 后，预留**端到端集成**测试框架

## 1.2 适用范围（Scope）

| 边界 | 说明 |
|---|---|
| 包含 | gm-backend crate 全部 7 endpoint 的 HTTP 端到端调用（in-process axum-test server）|
| 排除 | 单元测试（见 RGS-TST-UT-08）、系统测试（见 RGS-TST-ST-08）、跨进程 k3s 部署（见 e2e-smoke）、admin-service gRPC 实际调用（v0.2） |
| 范围 | RGS-BAS-003 §3.1-§3.4 全部字段、4.1 RBAC 矩阵（v0.2）、4.2 审计查询（v0.2） |

## 1.3 关联文档（Related Documents）

| 文档编号 | 文档名 | 与本文件关系 |
|---|---|---|
| RGS-REQ-007 运维与 GM 后台管控 需求定义书 | 需求 | 来源 |
| RGS-REQ-019 智能决策层（无埋点可观测性增强）需求定义书 | 需求 | 观测字段 |
| RGS-REQ-024 GM 后台多人可观测化漏斗 需求定义书 | 需求 | 观测字段 |
| RGS-BAS-003 运维与 GM 后台管控 基本设计书 | 设计 | 父文档 |
| RGS-BAS-021 GM 后台多人可观测化漏斗 基本设计书 | 设计 | 父文档 |
| RGS-DTL-003 运维与 GM 后台管控 详细设计书 | 详细设计 | 父文档 |
| RGS-DTL-040 Admin 域 详细设计书 | 详细设计 | 父文档 |
| RGS-TST-IT-01 核心架构与设计模式 集成测试设计书 | 参考 | V 模型对应 |
| RGS-TST-IT-00 基准与治理 集成测试设计书 | 参考 | V 模型对应 |
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

本版本为 0.1 初次编制，**强调字段级映射**：每个集成测试用例"对应需求"列精确到"BAS-003 §X.Y + 字段/schema/响应码"。

**V 模型强对应**：本文件对应"GM 后台 APIGW 集成层 + 字段级 API"。

## 1.6 命名约定（Naming Convention）

- 测试 ID：`TST-{UT|IT|ST}-08-NNN`
- V 模型层级标注：IT 标 [TL-2/3/4/5]
- 用例类型：N=正常 / A=异常 / B=边界 / P=性能（不适用 IT） / S=状态机
- 测试运行时：`cargo test -p gm-backend --test integration_gm_basic`

---

## 2. 测试策略

## 2.1 V 模型对应关系

```
需求   RGS-REQ-007/019/024  → ST  (RGS-TST-ST-08)
设计   RGS-BAS-003/021       → IT  (RGS-TST-IT-08,本文件)
详细   RGS-DTL-003/040       → UT  (RGS-TST-UT-08)
实现   Rust 源码             ←
```

## 2.2 集成层次

| 层次 | 范围 | 工具 |
|---|---|---|
| L3 | in-process axum-test 启动完整 Router，HTTP 请求 + 响应 schema 断言 | `axum-test 16` |
| L4 | gm-backend 端到端（含未来 admin-service gRPC client）| TBD v0.2 |
| L5 | 跨进程 k3s pod | `scripts/e2e-smoke.ps1`（不在 IT 范围） |

## 2.3 协议契约

- HTTP 1.1 over TLS（8443 主，dev 模式 plain）/ plain HTTP（8081 探针）
- 请求 schema：BAS-003 §3.1-§3.4 5 域 + §4 审计
- 响应 schema：JSON `{ status, service, items, next, op, mode, admin_endpoint }`
- 错误码：200 成功 / 202 Accepted(stub) / 405 Method Not Allowed / 404 Not Found

## 2.4 测试质量目标

| 维度 | 目标 |
|---|---|
| 接口契约覆盖率 | 100%（7 endpoint × 1-3 字段） |
| 路由边界覆盖率 | 100%（method 错配 + 未知路径 + 隔离） |
| 缺陷密度 | ≤ 1.0 个/KLOC（QA-004） |

---

## 3. 测试用例

## 3.1 模块 A：HTTP 健康端点（BAS-003 §3.4）

| 测试 ID | 对应需求 | 字段/schema | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-IT-08-A001 | BAS-003 §3.4 /healthz | 响应：200, `{ "status":"ok", "service":"gm-backend" }` | [TL-2] | N | healthz 返回 200 + 2 字段 |
| TST-IT-08-A002 | BAS-003 §3.4 /readyz | 响应：200, `{ "status":"ready", "service":"gm-backend" }` | [TL-2] | N | readyz 返回 200 + 2 字段 |
| TST-IT-08-A003 | BAS-003 §2.1 build_health_router 隔离 | /healthz GET（通过 8081 router）| [TL-2] | N | health router 暴露 /healthz |

**实现位置**：`crates/gm-backend/tests/integration_gm_basic.rs`（3 测试）

## 3.2 模块 B：GM 业务端点（BAS-003 §3.1-§3.4）

| 测试 ID | 对应需求 | 字段/schema | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-IT-08-B001 | BAS-003 §3.1 /api/v1/gm/health/view | 响应：200, `{ "service", "admin_endpoint", "mode" }` | [TL-2/3] | N | health_view 返回 3 字段 + admin_endpoint 来自 config |
| TST-IT-08-B002 | BAS-003 §3.4 /api/v1/gm/ban | 响应：202, `{ "status":"queued", "op":"ban" }` | [TL-2] | N | ban_account 返回 202 + 2 字段 |
| TST-IT-08-B003 | BAS-003 §3.4 /api/v1/gm/compensation | 响应：202, `{ "status":"queued", "op":"compensation" }` | [TL-2] | N | grant_compensation 返回 202 + 2 字段 |
| TST-IT-08-B004 | BAS-003 §3.4 /api/v1/gm/maintenance | 响应：202, `{ "status":"queued", "op":"maintenance" }` | [TL-2] | N | set_maintenance 返回 202 + 2 字段 |
| TST-IT-08-B005 | BAS-003 §4.2 /api/v1/audit/logs | 响应：200, `{ "items":[], "next":"stub" }` | [TL-2] | N | query_audit 返回空 items（v0.2 实装 join） |

**实现位置**：`crates/gm-backend/tests/integration_gm_basic.rs`（5 测试）

## 3.3 模块 C：路由边界（BAS-003 §2.1 路由表）

| 测试 ID | 对应需求 | 字段/schema | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-IT-08-C001 | BAS-003 §2.1 GET 端点拒绝 POST | POST /healthz | [TL-2] | A | 返回 405 Method Not Allowed |
| TST-IT-08-C002 | BAS-003 §2.1 POST 端点拒绝 GET | GET /api/v1/gm/ban | [TL-2] | A | 返回 405 |
| TST-IT-08-C003 | BAS-003 §2.1 未知路由 | GET /api/v1/gm/nonexistent | [TL-2] | A | 返回 404 Not Found |
| TST-IT-08-C004 | BAS-003 §2.1 health 路由不含 GM 端点 | GET /api/v1/gm/health/view（通过 build_health_router）| [TL-2] | A | 返回 404（端口隔离） |

**实现位置**：`crates/gm-backend/tests/integration_gm_basic.rs`（4 测试）

## 3.4 模块 D：端到端集成（v0.2 TBD，BAS-003 §4.1 RBAC + admin-service gRPC）

| 测试 ID | 对应需求 | 字段/schema | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-IT-08-D001 | BAS-003 §4.1 RBAC GM_OPERATOR | JWT 角色 = GM_OPERATOR | [TL-4] | N | ban_account 200 + 调 admin-service.BanAccount |
| TST-IT-08-D002 | BAS-003 §4.1 RBAC GM_ADMIN | JWT 角色 = GM_ADMIN | [TL-4] | N | grant_compensation 200 + 调 admin-service.GrantCompensation |
| TST-IT-08-D003 | BAS-003 §4.1 RBAC SRE | JWT 角色 = SRE | [TL-4] | N | set_maintenance 200 + 调 admin-service.SetMaintenance |
| TST-IT-08-D004 | BAS-003 §4.1 RBAC 拒绝 | JWT 角色 = GM_VIEWER | [TL-4] | A | 返回 403 Forbidden |
| TST-IT-08-D005 | BAS-003 §4.2 audit join admin-service.QueryAudit | query_audit 调 admin-service | [TL-4] | N | 返回 200 + items ≥ 1 + next=string |

**实现位置**：`crates/gm-backend/tests/integration_admin_grpc.rs`（**TBD v0.2**,占位,等 admin-service gRPC client 实装后补）

## 3.5 模块 E：TLS / mTLS 集成（v0.2 TBD，BAS-003 §2.1）

| 测试 ID | 对应需求 | 字段/schema | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-IT-08-E001 | BAS-003 §2.1 HTTPS 8443 | rustls + server.pem + ca.pem | [TL-4] | N | https:// 客户端证书验证通过 |
| TST-IT-08-E002 | BAS-003 §2.1 mTLS 拒绝 | 客户端无证书 | [TL-4] | A | 返回 401 Unauthorized |
| TST-IT-08-E003 | BAS-003 §2.1 mTLS 拒绝 | 客户端证书非 CA 签发 | [TL-4] | A | 返回 401 |

**实现位置**：`crates/gm-backend/tests/integration_tls.rs`（**TBD v0.2**）

---

## 4. 追溯矩阵（Traceability Matrix）

| 测试 ID | RGS-REQ | RGS-BAS | RGS-DTL | 测试代码 |
|---|---|---|---|---|
| TST-IT-08-A001 | REQ-007 §3.4 | BAS-003 §3.4 | DTL-040 §3.3 | `integration_gm_basic.rs::healthz_returns_ok_with_service_name` |
| TST-IT-08-A002 | REQ-007 §3.4 | BAS-003 §3.4 | DTL-040 §3.3 | `integration_gm_basic.rs::readyz_returns_ready` |
| TST-IT-08-A003 | REQ-007 §3.1 | BAS-003 §2.1 | DTL-040 §3.2 | `integration_gm_basic.rs::health_router_also_exposes_healthz` |
| TST-IT-08-B001 | REQ-007 §3.1 | BAS-003 §3.1 | DTL-040 §3.3 | `integration_gm_basic.rs::health_view_returns_admin_endpoint_from_config` |
| TST-IT-08-B002 | REQ-007 §3.4 | BAS-003 §3.4 | DTL-040 §3.3 | `integration_gm_basic.rs::ban_account_returns_202_queued` |
| TST-IT-08-B003 | REQ-007 §3.4 | BAS-003 §3.4 | DTL-040 §3.3 | `integration_gm_basic.rs::grant_compensation_returns_202_queued` |
| TST-IT-08-B004 | REQ-007 §3.4 | BAS-003 §3.4 | DTL-040 §3.3 | `integration_gm_basic.rs::set_maintenance_returns_202_queued` |
| TST-IT-08-B005 | REQ-024 §3 | BAS-021 §3 | DTL-040 §3.3 | `integration_gm_basic.rs::query_audit_returns_empty_items_stub` |
| TST-IT-08-C001 | REQ-007 §3.1 | BAS-003 §2.1 | DTL-040 §3.2 | `integration_gm_basic.rs::main_router_does_not_accept_post_on_get_endpoints` |
| TST-IT-08-C002 | REQ-007 §3.1 | BAS-003 §2.1 | DTL-040 §3.2 | `integration_gm_basic.rs::main_router_does_not_accept_get_on_post_endpoints` |
| TST-IT-08-C003 | REQ-007 §3.1 | BAS-003 §2.1 | DTL-040 §3.2 | `integration_gm_basic.rs::unknown_route_returns_404` |
| TST-IT-08-C004 | REQ-007 §3.1 | BAS-003 §2.1 | DTL-040 §3.2 | `integration_gm_basic.rs::health_router_does_not_expose_gm_endpoints` |
| TST-IT-08-D001~D005 | REQ-007 §4 | BAS-003 §4.1 | DTL-040 §3.5 | **TBD v0.2** |
| TST-IT-08-E001~E003 | REQ-007 §2 | BAS-003 §2.1 | DTL-040 §3.6 | **TBD v0.2** |

**总计**：20 测试用例 ID（12 已实现 + 8 v0.2 TBD）

---

## 5. 测试执行计划（Test Execution Plan）

| 阶段 | 工具 | 命令 | 触发 |
|---|---|---|---|
| L3 本地 | cargo + axum-test | `cargo test -p gm-backend --test integration_gm_basic` | 每次 commit |
| L3 CI | cargo + CI | `.github/workflows/rust-ci.yml` | push to main |
| L4 v0.2 | cargo + rgs-testkit | `cargo test -p gm-backend --test integration_admin_grpc` | v0.2 集成完成 |
| L4 v0.2 | cargo + rustls | `cargo test -p gm-backend --test integration_tls` | v0.2 TLS 实装 |

**已知 bug**：
- 5 域集成测试 PG fixture 已由 commit 8e5fe38 接入（gm-backend 暂不需要 DB）
- DDD Review 阶段需补 v0.2 测试用例的 gRPC client mock 选型

---

## 6. 通过判定标准（Pass Criteria）

| 维度 | 通过阈值 |
|---|---|
| 测试通过率 | 100%（12 已实现） |
| 接口契约覆盖率 | 100%（7 endpoint 字段级） |
| 路由边界覆盖率 | 100%（method / 404 / 隔离） |
| 编译警告 | 0 |
| 业务路径覆盖率 | ≥ 70%（per QA-002 70% 阈值） |
| 字段级映射 | 100% 测试用例对应到具体 REQ/BAS/DTL § |

---

## 7. 风险与未决事项（Risks and TBDs）

| 编号 | 描述 | 风险等级 | 解决路径 |
|---|---|---|---|
| TBD-08-01 | JWT validation 未实装（per UT 设计书 TBD-08-01） | P1 | v0.2 |
| TBD-08-02 | mTLS 启动 fail-closed 路径未实装 | P1 | v0.2 |
| TBD-08-03 | admin-service gRPC client 未实装，5 endpoint 全 stub | P2 | v0.2 |
| TBD-08-04 | audit join admin-service.QueryAudit 未实装 | P2 | v0.2 |
| TBD-08-05 | 跨进程 k3s 集成测试（e2e-smoke）不在 IT 范围 | P3 | 见 RGS-TST-ST-08 |
| TBD-08-06 | `axum-test 16` 与 7 域使用的 `wiremock` 不一致 | P3 | 评估统一 |
| TBD-08-07 | 5 域 Lead 实际具名状态（per OPEN-QA Q2） | P1 | DDD Review |

**保留派生约束**（per 2026-08-26 04:30 JST）：
- 禁"per X 历史形态"等回溯叙事
- 引用 BAS 必须 git log -p --follow 实证
- 缺标比错标安全
- 子代理授权边界要写明"无证据叙事 = 禁止"

---

**作者**：架构师（Mavis 接手 agent per DEC-008,代签）  
**时间**：2026-08-27 23:38 JST  
**后续**：DDD Review 时由 Ulysses + 5 域 Lead + GM 后台域 Lead 联合审

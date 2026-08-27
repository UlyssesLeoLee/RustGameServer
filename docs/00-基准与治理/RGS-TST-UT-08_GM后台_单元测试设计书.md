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
| 字段级覆盖率 | 100%（全部 GM endpoint 字段 + GmConfig 字段） |
| 业务路径覆盖率 | ≥ 70% |
| 缺陷密度 | ≤ 1.0 个/KLOC（QA-004） |
| 边界用例 | 至少 3 类（无效 SocketAddr / 缺失 env / 配置越界） |

---

## 3. 测试用例

## 3.1 模块 A：GmConfig 配载（RGS-DTL-040 §3.1）

| 测试 ID | 对应需求 | 字段/映射 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-08-A001 | DTL-040 §3.1 GmConfig 默认值 | http_addr=0.0.0.0:8443, health_addr=0.0.0.0:8081, admin_grpc_endpoint="https://admin-service:50055", jwt_secret="dev-only-do-not-use-in-prod" | N | env 缺失时 from_env 返回默认值 |
| TST-UT-08-A002 | DTL-040 §3.1 GmConfig env 覆盖 | http_addr, health_addr, admin_grpc_endpoint, jwt_secret 4 字段 | N | 显式 set_var 后 from_env 读到覆盖值 |
| TST-UT-08-A003 | DTL-040 §3.1 GmConfig::from_env 错误处理 | http_addr 解析失败 | A | 返回 anyhow::Error，msg 含 "invalid GM_HTTP_ADDR" |
| TST-UT-08-A004 | DTL-040 §3.1 GmConfig::from_env 错误处理 | health_addr 解析失败 | A | 返回 anyhow::Error，msg 含 "invalid GM_HEALTH_ADDR" |
| TST-UT-08-A005 | DTL-040 §3.1 GmConfig::for_test builder | 3 参数（http/str, health/str, admin/str） | N | builder 构造出符合预期 GmConfig |
| TST-UT-08-A006 | DTL-040 §3.1 GmConfig Clone + PartialEq | GmConfig Clone 实现 | N | clone 后 PartialEq 相等 |

**实现位置**：`crates/gm-backend/tests/ut_config.rs`（6 测试,2026-08-27 22:53 JST 19/19 PASS）

## 3.2 模块 B：AppState + Router 构造（RGS-BAS-003 §2.1）

| 测试 ID | 对应需求 | 字段/映射 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-08-B001 | BAS-003 §2.1 Router 7 路由 | 7 路由：/healthz, /readyz, /api/v1/gm/health/view, /api/v1/gm/ban, /api/v1/gm/compensation, /api/v1/gm/maintenance, /api/v1/audit/logs | N | build_router() 包含所有 7 路由 |
| TST-UT-08-B002 | BAS-003 §2.1 health_router 隔离 | build_health_router() 仅 2 路由 | N | health 路由不含 /api/v1/gm/* 业务端点 |
| TST-UT-08-B003 | BAS-003 §2.1 AppState Clone | AppState Clone 实现 | N | Clone 后两个 handle 共享同一 config |

**实现位置**：`crates/gm-backend/tests/integration_gm_basic.rs`（含部分 L1 单元测试 + in-process 集成）

## 3.3 模块 C：fail-closed 启动（RGS-BAS-003 §2.1 启动约束）

| 测试 ID | 对应需求 | 字段/映射 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-08-C001 | BAS-003 §2.1 + 5 域 fail-closed 模式 | env 全部 0.0.0.0:0 | N | 5s 内启动，stderr 含 "starting GM APIGW" 或进程仍在跑 |
| TST-UT-08-C002 | BAS-003 §2.1 mTLS 启动路径(预留) | RGS_TLS_DIR 缺失 | A | **TBD-08-02** v0.2 实装: mTLS 必须 fail-closed（per 5 域 RGS_ALLOW_INSECURE_GRPC=0 默认） |

**实现位置**：`crates/gm-backend/tests/fail_closed_start.rs`（1 测试）

## 3.4 模块 D：Handler 输入输出（BAS-003 §3.1-§3.4）

| 测试 ID | 对应需求 | 字段/映射 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-08-D001 | BAS-003 §3.1 health_view | 响应字段:service, admin_endpoint, mode | N | 返回 200 + 含 3 字段 |
| TST-UT-08-D002 | BAS-003 §3.4 ban_account | 响应字段:status, op | N | 返回 202 + status=queued, op=ban |
| TST-UT-08-D003 | BAS-003 §3.4 grant_compensation | 响应字段:status, op | N | 返回 202 + status=queued, op=compensation |
| TST-UT-08-D004 | BAS-003 §3.4 set_maintenance | 响应字段:status, op | N | 返回 202 + status=queued, op=maintenance |
| TST-UT-08-D005 | BAS-003 §3.4 query_audit | 响应字段:items, next | N | 返回 200 + items=[] + next=stub |
| TST-UT-08-D006 | BAS-003 §3.4 healthz | 响应字段:status, service | N | 返回 200 + service=gm-backend |
| TST-UT-08-D007 | BAS-003 §3.4 readyz | 响应字段:status, service | N | 返回 200 + service=gm-backend |

**实现位置**：`crates/gm-backend/tests/integration_gm_basic.rs`（12 测试）

## 3.5 模块 E：Router 路由边界（RGS-BAS-003 §2.1 路由表）

| 测试 ID | 对应需求 | 字段/映射 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-08-E001 | BAS-003 §2.1 GET 端点拒绝 POST | /healthz POST | A | 返回 405 Method Not Allowed |
| TST-UT-08-E002 | BAS-003 §2.1 POST 端点拒绝 GET | /api/v1/gm/ban GET | A | 返回 405 |
| TST-UT-08-E003 | BAS-003 §2.1 未知路由 | /api/v1/gm/nonexistent GET | A | 返回 404 Not Found |
| TST-UT-08-E004 | BAS-003 §2.1 health 路由不含 GM 端点 | /api/v1/gm/health/view GET (在 build_health_router) | A | 返回 404 |

**实现位置**：`crates/gm-backend/tests/integration_gm_basic.rs`（4 测试）

---

## 4. 追溯矩阵（Traceability Matrix）

| 测试 ID | RGS-REQ | RGS-BAS | RGS-DTL | 测试代码 |
|---|---|---|---|---|
| TST-UT-08-A001 | REQ-007 §3.4 | BAS-003 §3.1 | DTL-040 §3.1 | `ut_config.rs::gm_config_defaults_when_env_missing` |
| TST-UT-08-A002 | REQ-007 §3.4 | BAS-003 §3.1 | DTL-040 §3.1 | `ut_config.rs::gm_config_respects_env_overrides` |
| TST-UT-08-A003 | REQ-007 §3.4 | BAS-003 §3.1 | DTL-040 §3.1 | `ut_config.rs::gm_config_rejects_invalid_socket_addr` |
| TST-UT-08-A004 | REQ-007 §3.4 | BAS-003 §3.1 | DTL-040 §3.1 | `ut_config.rs::gm_config_rejects_invalid_health_addr` |
| TST-UT-08-A005 | REQ-007 §3.4 | BAS-003 §3.1 | DTL-040 §3.1 | `ut_config.rs::gm_config_for_test_builder` |
| TST-UT-08-A006 | REQ-007 §3.4 | BAS-003 §3.1 | DTL-040 §3.1 | `ut_config.rs::gm_config_clone_equality` |
| TST-UT-08-B001~B003 | REQ-007 §3.1 | BAS-003 §2.1 | DTL-040 §3.2 | `integration_gm_basic.rs::*_returns_*` |
| TST-UT-08-C001 | REQ-007 §3.1 | BAS-003 §2.1 | DTL-040 §3.4 | `fail_closed_start.rs::gm_backend_starts_with_defaults` |
| TST-UT-08-C002 | REQ-007 §3.1 | BAS-003 §2.1 | DTL-040 §3.4 | **TBD-08-02** v0.2 实装 |
| TST-UT-08-D001~D007 | REQ-007 §3.4 | BAS-003 §3.1-§3.4 | DTL-040 §3.3 | `integration_gm_basic.rs::*_returns_*` |
| TST-UT-08-E001~E004 | REQ-007 §3.1 | BAS-003 §2.1 | DTL-040 §3.2 | `integration_gm_basic.rs::main_router_does_not_accept_*` + `unknown_route_*` + `health_router_does_not_expose_*` |

**总计**：23 测试用例 ID（19 已实现 + 4 边界,1 TBD-08-02 待 v0.2）

---

## 5. 测试执行计划（Test Execution Plan）

| 阶段 | 工具 | 命令 | 触发 |
|---|---|---|---|
| L1 本地 | cargo | `cargo test -p gm-backend` | 每次 commit 必跑 |
| L1 CI | cargo + CI | `.github/workflows/rust-ci.yml` | push to main |
| L1 dev-deps 锁版本 | serial_test 0.5 | env 隔离 | 必跑 |
| 覆盖率 | cargo-llvm-cov | 后续 | TBD |

**已知 bug**：
- 本机 CI 暂未集成 gm-backend 5 域 PG fixture（per 8e5fe38 5 域已集成，gm-backend 暂不需要 DB）
- DDD Review 阶段需补覆盖率门槛

---

## 6. 通过判定标准（Pass Criteria）

| 维度 | 通过阈值 |
|---|---|
| 测试通过率 | 100% |
| 字段级映射覆盖率 | 100%（GmConfig 4 字段 + 7 endpoint × 1-3 字段） |
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
| TBD-08-03 | 5 个 GM endpoint 仍 stub（返回硬编码 202 queued） | P2 | v0.2 实装：admin-service gRPC client + rgs-testkit mock |
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

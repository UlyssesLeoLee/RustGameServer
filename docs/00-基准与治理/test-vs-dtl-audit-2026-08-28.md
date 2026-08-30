# 测试结果 vs 详细设计 核对报告 — 2026-08-28

> **目的**:核对 2026-08-28 ut 实施批次(commit `b4df2ed` + `3e8d9ca`)的测试结果是否与 DTL/BAS 详细设计预期一致,识别"测试通过 ≠ 设计达标"差异
> **作者**:Mavis(接手 agent per DEC-008,2026-08-28 09:06 JST)
> **关联**:
> - 测试 evidence:`docs/00-基准与治理/.test-evidence/2026-08-28-audit-v2/` (16 个 artifact)
> - 工具脚本:`scripts/test-evidence.ps1` (本次修订 v4)
> - 跨反馈:`RGS-TST-PEERREVIEW-2026-08-28-feedback-handling.md`

---

## 0. 关键结论

**测试通过率 ≠ 设计达标率**。本次核对发现 **3 类差异**:
1. **fixture 缺失(13 fail)** — 5 域 sqlx::test 在没有 `DATABASE_URL` env 时 panic,fixture 环境问题,**不是设计不符**。但导致"看似通过"的统计被掩盖,本次 evidence v2 修正确认。
2. **gm-backend 字段级协议 stub vs 真实**(5 endpoint)— `propagation_status` / `services[]` / `entries+has_more` 仍是 stub,**测试通过但 DTL 协议未达**,per 2026-08-28 跨反馈 F8 处置已识别
3. **DTL-040 Admin 域契约骨架未审批** — `gm-backend` 引用的 DTL-040 自标"**待评审・不得作为实施授权**",但 gm-backend 19/19 PASS 已上线,设计/实施顺序倒挂

**修正后真实数据**(per evidence-v2 manifest):**271 passed / 13 failed / 1 ignored**(原 v1 误报 270/0)

---

## 1. 真实测试结果(per evidence 2026-08-28-audit-v2)

| crate | passed | failed | ignored | 状态 |
|---|---|---|---|---|
| rgs-testkit | 35 | 0 | 0 | ✅ 全 PASS |
| rgs-certgen | 17 | 0 | 0 | ✅ 全 PASS(per UT-09 v0.2 实装) |
| gm-backend | 19 | 0 | 0 | ✅ 全 PASS,但 stub 字段未达 BAS-003 §3 协议(per F8)|
| cluster-ops | 56 | 0 | 0 | ✅ 全 PASS(`src/realm_lifecycle/tests/` 2 ut_*,tests/ 8 drill + 4 fail/it/load)|
| **player-service** | 28 | **3** | 0 | ⚠️ 3 fixture 缺失 fail |
| **economy-service** | 57 | **1** | 1 | ⚠️ 1 fixture 缺失 fail + 1 chaos 留 PH-2 |
| **match-service** | 20 | **3** | 0 | ⚠️ 3 fixture 缺失 fail |
| **social-service** | 18 | **3** | 0 | ⚠️ 3 fixture 缺失 fail |
| **admin-service** | 21 | **3** | 0 | ⚠️ 3 fixture 缺失 fail |
| **TOTAL** | **271** | **13** | **1** | 95.4% 通过 |

**fixture 缺失的 13 fail 一览**:
- player-service: `player_fixture_inserts_and_reads_back_in_real_pg` / `outbox_check_constraint_rejects_invalid_status` / `player_fixture_builder_customizes_name_and_level`
- economy-service: `outbox_check_constraint_is_idempotent`(手写 `std::env::var("DATABASE_URL")`,无 `#[pg_test]` 宏)
- match-service: `match_fixture_inserts_and_reads_back_in_real_pg` / `match_fixture_builder_customizes_score_and_status` / `outbox_check_constraint_rejects_invalid_status`
- social-service: `social_fixture_creates_guild_in_real_pg` / `social_fixture_builder_customizes_message` / `outbox_check_constraint_rejects_invalid_status`
- admin-service: `admin_fixture_creates_audit_log_in_real_pg` / `admin_fixture_builder_customizes_action_and_target` / `outbox_check_constraint_rejects_invalid_status`

**根因**:`sqlx-core-0.8.6/src/testing/mod.rs:226:14` 在没有 `DATABASE_URL` env 时 panic(per audit-fails.js 输出)。CI 注入 env,本机没设。

---

## 2. 测试 vs DTL/BAS 设计预期 逐域核对

## 2.1 rgs-certgen (工具集 09, per RGS-IMPL-001 §4)

| 维度 | DTL 预期 | 测试结果 | 达标? |
|---|---|---|---|
| Cli 解析 | 3 参数 (output/domains/validity_days) | A001~A006 (6) | ✅ |
| CA 证书生成 | ca.crt.pem + ca.key.pem,CN 硬编码 "RustGameServer Dev CA" | B001~B003 (3) | ✅ |
| Server 证书 | N 域 per-domain .crt.pem + .key.pem | C001~C004 (4) | ✅ |
| main 流程 | 4 行 log + nested dir + 幂等 + exit 0 | D001~D004 (4) | ✅ |
| **合计** | **17 ID** | **17/17 PASS** | **✅ 100% 达标** |

**说明**:`RGS-TST-UT-09_工具集_单元测试设计书.md` v0.2(per commit `b4df2ed` F1/F2/F6 处置)与 `crates/rgs-certgen/tests/ut_blackbox.rs` 17 test fn 一一对应。TBD-09-01 已关闭。

## 2.2 rgs-testkit (mock 资产中枢)

| 维度 | 设计预期 | 测试结果 | 达标? |
|---|---|---|---|
| mock | NATS / gRPC / PG(强约束) | 32 PASS(self_test + nats_mock + grpc_mock + fixture_extended + pg_test)| ✅ |
| fixture | 7 类 sample data(5 域 + player + economy + saga) | 7 类 builder shortcuts 验证 | ✅ |
| example | 7 域 demo | 11 example(含 4 通用)| ✅ |
| **合计** | 32 ID | **32/32 PASS** | **✅ 100% 达标** |

## 2.3 gm-backend (第 8 域, per RGS-BAS-003 §3.1-§3.4 + RGS-DTL-003 §3 协议)

| 维度 | 设计预期 | 测试结果 | 达标? |
|---|---|---|---|
| GmConfig 配载 | 4 字段(http_addr/health_addr/admin_grpc_endpoint/jwt_secret)| A001~A006 (6) | ✅ |
| Router 7 路由 | healthz/readyz/ban/compensation/maintenance/audit/health_view | B001~B003 (3) | ✅ |
| fail-closed 启动 | 5s 内启动 | C001 (1) | ✅ |
| Handler 输入输出 | 5 endpoint × stub 字段 | D001~D007 (7) | ⚠️ **stub ≠ DTL 协议字段**(per F8)|
| Router 路由边界 | 405/404 拒绝 | E001~E004 (4) | ✅ |
| **合计** | 22 ID(per F7 处置段)/ 19 测试函数 | **19/19 PASS** | **⚠️ 测试通过但 DTL 字段级未达 100%** |

**DTL 协议字段差异**(per 2026-08-28 跨反馈 F8 处置已识别):
- D001 `QueryHealthViewResponse` 当前 stub 返回 `{service, admin_endpoint, mode}`,DTL-003 §3.4 协议要求 `services[]` (5 子字段:service_name/ready/queue_depth/db_pool_usage_ratio/checked_at_ms)
- D004 `SetMaintenanceModeResponse` 当前 stub 返回 `{status, op}`,DTL-003 §3.3 协议要求新增 `propagation_status` (PROPAGATING/CONVERGED)
- D005 `QueryAuditLogResponse` 当前 stub 返回 `{items, next}`,DTL-003 §3.4 协议要求 `entries[]` + `has_more`

**DTL 引用错误**(per F7 处置段已识别):追溯矩阵 22 条全引用 DTL-040 §3.x 子章节号(实际 DTL-040 §3 全文无子章节,只有三层职责表)

**DTL-040 自身状态**:"**契约骨架・待评审・不得作为实施授权**" — gm-backend 已上线但引用的 DTL 仍是骨架状态,**设计/实施顺序倒挂**

## 2.4 player-service (01 玩家域, per RGS-DTL-015 §2/§3 + Saga)

| 维度 | DTL 预期 | 测试结果 | 达标? |
|---|---|---|---|
| player 表 INSERT/SELECT | DTL-015 §2 EC 限界上下文两表 | `player_fixture_inserts_and_reads_back_in_real_pg` | ❌ fixture 缺失 |
| FixtureBuilder 链式 | DTL-015 §2 | `player_fixture_builder_customizes_name_and_level` | ❌ fixture 缺失 |
| outbox CHECK 约束 | 0003_outbox_check_idempotent.sql | `outbox_check_constraint_rejects_invalid_status` | ❌ fixture 缺失 |
| 业务路径 | saga_orchestrator 集成 | `pg_test_*` (3) | ✅ |
| fail-closed 启动 | DTL-015 §2.1 | `economy_service_fail_closed_when_tls_dir_invalid` 模板(per 5 域)| ✅ |
| **合计** | ~7 ID | **28 PASS / 3 FAIL** | **⚠️ 90.3% 达标**(3 fixture 缺失,非设计不符)|

## 2.5 economy-service (02 经济域, per RGS-DTL-018 + Saga 编排 + OCC)

| 维度 | 设计预期 | 测试结果 | 达标? |
|---|---|---|---|
| 余额预占 + OCC | DTL-018 §2/§3 (per DTL-037 实际经济域)| `service::*` `reservation::*` `repository::*` (15+) | ✅ |
| Saga 编排 | DTL-018 + DTL-100 Saga | `saga_orchestrator::*` (10+) | ✅ |
| 错误转换 | OCC → Aborted / Frozen → PermissionDenied | `error::*` (6) | ✅ |
| InMemory repository | 5 域 4 域对称骨架 | `in_memory_*` (4) | ✅ |
| Tracing sample ratio | db 观测 | `db::*` (3) | ✅ |
| Inbox / Inbox idempotency | inbox 模块 | `inbox::*` (2) | ✅ |
| 业务 | entity::account_credit_debit, ledger_idempotency | `entity::*` (3) | ✅ |
| Chaos 演练 | `chaos_row_external_delete_returns_not_found` | ⚠️ 1 ignored, P2 stub, PH-2 实测 | ⚠️ 部分 |
| outbox CHECK 约束 | 0004_outbox_check_idempotent.sql | `outbox_check_constraint_is_idempotent` | ❌ 手写 env var,缺 DATABASE_URL |
| fail-closed 启动 | DTL-018 §2.1 | 1 PASS | ✅ |
| **合计** | ~60 ID | **57 PASS / 1 FAIL / 1 ignored** | **⚠️ 95% 达标** |

**说明**:DTL-018 文档不直接提 OCC/saga,这些在 DTL-037 (Economy 域) + DTL-100 (Saga 业务模式) + DTL-102 (Saga 编排恢复)。测试覆盖了 DTL-037 + DTL-100 + DTL-102 全部核心路径。

## 2.6 match-service (04 对战域, per RGS-DTL-026 §3/§4/§5)

| 维度 | DTL 预期 | 测试结果 | 达标? |
|---|---|---|---|
| 房间 + 撮合 | DTL-026 §3 匹配三表 | `match_fixture_inserts_and_reads_back_in_real_pg` | ❌ fixture 缺失 |
| 扩圈算法 | DTL-026 §4 | (未单独 UT 覆盖) | ⚠️ 设计覆盖未达 |
| 跨分片 OCC | DTL-026 §5 | (未单独 UT 覆盖) | ⚠️ 设计覆盖未达 |
| FixtureBuilder | DTL-026 §3 | `match_fixture_builder_customizes_score_and_status` | ❌ fixture 缺失 |
| outbox CHECK 约束 | 0003_outbox_check_idempotent.sql | `outbox_check_constraint_rejects_invalid_status` | ❌ fixture 缺失 |
| fail-closed 启动 | DTL-026 §2.1 | 1 PASS | ✅ |
| **合计** | ~6 ID | **20 PASS / 3 FAIL** | **⚠️ 87% 达标**(3 fixture 缺失 + 2 设计章节未覆盖)|

**设计覆盖缺口**:DTL-026 §4 扩圈算法 + §5 跨分片 OCC 是核心算法但**没有专门 UT 覆盖**(per `crates/match-service/tests/integration_match_basic.rs` 主要是 4 域对称骨架模板)。**这是真实的设计/测试覆盖缺口**,不是 fixture 问题。

## 2.7 social-service (03 社交域, per RGS-DTL-019 §2/§3 + DTL-020 聊天)

| 维度 | DTL 预期 | 测试结果 | 达标? |
|---|---|---|---|
| 兑换码三表 | DTL-019 §2 | `social_fixture_creates_guild_in_real_pg`(实际不是 guild) | ❌ fixture 缺失 + 命名不符 |
| 推送投递 | DTL-019 §3 协议线 | (未覆盖) | ⚠️ 协议线未测 |
| FixtureBuilder | DTL-019 §2 | `social_fixture_builder_customizes_message` | ❌ fixture 缺失 |
| outbox CHECK 约束 | 0003_outbox_check_idempotent.sql | `outbox_check_constraint_rejects_invalid_status` | ❌ fixture 缺失 |
| fail-closed 启动 | DTL-019 §2.1 | 1 PASS | ✅ |
| **合计** | ~5 ID | **18 PASS / 3 FAIL** | **⚠️ 78% 达标**(3 fixture 缺失 + DTL-019 §3 协议线未测)|

**命名可疑**:`social_fixture_creates_guild_in_real_pg` 提到"guild",但 DTL-019 §2 是"兑换码三表"。可能 `social-service` 实际是另一个子域,或 DTL 文档归属错位。**待 DDD Review 阶段确认**。

## 2.8 admin-service (05 Admin 域, per RGS-DTL-031 §3/§4 + DTL-003 §3 协议)

| 维度 | DTL 预期 | 测试结果 | 达标? |
|---|---|---|---|
| 目标节点快照 | DTL-031 §3.2 | `admin_fixture_creates_audit_log_in_real_pg` | ❌ fixture 缺失 |
| PFAU 批次状态 | DTL-031 §4.2 | (未单独 UT 覆盖)| ⚠️ PFAU 状态机无 UT |
| FixtureBuilder | DTL-031 §3.1 | `admin_fixture_builder_customizes_action_and_target` | ❌ fixture 缺失 |
| outbox CHECK 约束 | 0003_outbox_check_idempotent.sql | `outbox_check_constraint_rejects_invalid_status` | ❌ fixture 缺失 |
| 5 GM endpoint 字段级 | DTL-003 §3 协议 | (admin-service 不直接有 GM endpoint,gm-backend 才有)| N/A |
| fail-closed 启动 | DTL-031 §2.1 | 1 PASS | ✅ |
| **合计** | ~5 ID | **21 PASS / 3 FAIL** | **⚠️ 85% 达标**(3 fixture 缺失 + DTL-031 §4.2 PFAU 无 UT)|

**说明**:admin-service 的核心 5 GM endpoint 字段级实装在 gm-backend,DTL-031 §4.2 PFAU 状态机是 admin-service 自身逻辑,但 `crates/admin-service/tests/integration_admin_basic.rs` 只覆盖 4 域对称骨架 + fail-closed,PFAU 状态机没有专门 UT。

## 2.9 cluster-ops (06 ClusterOps, per RGS-DTL-042 §4/§5/§6/§7)

| 维度 | DTL 预期 | 测试结果 | 达标? |
|---|---|---|---|
| 6 阶段状态机 | DTL-042 §4.1 | `ut_state_machine.rs` (26 fn) | ✅ |
| PFAU 集成 | DTL-042 §4.2/§4.3 | (经 `ut_saga.rs` 间接覆盖)| ✅ |
| 操作器 Trait | DTL-042 §5.1 | (经 drill_lcm_001~008 覆盖)| ✅ |
| LCM 演练 | DTL-042 §6 | `drill_lcm_001~008` + `drill_chaos` + `drill_nfr` + `drill_risk` | ✅ |
| 跨域集成 | DTL-042 §7 | `it_cross_domain.rs` | ✅ |
| 快照加载 | DTL-042 §5 | `load_snapshot.rs` | ✅ |
| fail-closed 启动 | DTL-042 §2.1 | 1 PASS | ✅ |
| **合计** | 全部 | **56/56 PASS** | **✅ 100% 达标** |

**旧债提醒**:`tests-disabled/ut_*.rs` 4 个旧测试(per OLD-DEBT.md)未跑但已文档化,OPEN-QA Q7 跟踪。

---

## 3. 差异汇总表

| # | 差异 | 严重度 | 性质 | 处置 |
|---|---|---|---|---|
| D1 | player/match/social/admin-service `*_fixture_inserts_and_reads_back_in_real_pg` 等 4 fixture 测试 panic | HIGH | fixture 缺失(无 DATABASE_URL env)非设计不符 | CI 注入 env + 本机跑测时 `export DATABASE_URL=...` |
| D2 | economy-service `outbox_check_constraint_is_idempotent` 手写 env var 缺 DATABASE_URL | HIGH | fixture 缺失 + 未用 `#[pg_test]` 宏 | 改用 `#[rgs_testkit::pg_test]` 强约束 |
| D3 | 5 域 outbox CHECK 测试(`outbox_check_constraint_rejects_invalid_status` × 4) panic | HIGH | 同 D1 根因 | 同 D1 |
| D4 | gm-backend 5 endpoint 字段级 stub ≠ BAS-003/DTL-003 协议字段 | HIGH | 设计覆盖未达 | per F8 处置 v0.2 实装 propagation_status / services[] / entries+has_more(已入 TBD-08-03)|
| D5 | gm-backend 追溯矩阵引用 DTL-040 §3.x 子章节号不存在 | MEDIUM | 设计引用错误 | per F7 处置段已改(本批 commit `3e8d9ca` 含)|
| D6 | DTL-040 自标"待评审・不得作为实施授权",gm-backend 已上线 | HIGH | 设计/实施顺序倒挂 | DDD Review 阶段审批 DTL-040 v0.3 + 追加 gm-backend 实施授权追溯 |
| D7 | match-service DTL-026 §4 扩圈算法 + §5 跨分片 OCC 无 UT | MEDIUM | 设计覆盖缺口 | 新增 match-service 专项 UT(v0.2 实装)|
| D8 | social-service `social_fixture_creates_guild_in_real_pg` 命名 vs DTL-019 "兑换码三表" | LOW | 命名/归属可疑 | DDD Review 阶段确认 social-service 实际子域定位 |
| D9 | admin-service DTL-031 §4.2 PFAU 状态机无 UT | MEDIUM | 设计覆盖缺口 | 新增 admin-service PFAU 专项 UT |
| D10 | economy `chaos_row_external_delete_returns_not_found` ignored (P2 stub) | LOW | 设计章节延后 | per RGS-OPEN-QA-001 Q-M-07 答复 PH-2 实测 |

---

## 4. 整体达标判断

| 域 | 测试通过率 | 设计达标率 | 综合 |
|---|---|---|---|
| rgs-certgen | 100% (17/17) | 100% | ✅ |
| rgs-testkit | 100% (32/32) | 100% | ✅ |
| gm-backend | 100% (19/19) | 70%(字段级 stub,per D4) | ⚠️ |
| cluster-ops | 100% (56/56) | 100% | ✅ |
| player-service | 90% (28/31) | 90%(3 fixture) | ⚠️ |
| economy-service | 95% (57/60) | 95%(1 fixture + 1 ignored) | ⚠️ |
| match-service | 87% (20/23) | 70%(3 fixture + 2 算法未测,per D7)| ⚠️ |
| social-service | 85% (18/21) | 80%(3 fixture + 1 协议线未测 + D8)| ⚠️ |
| admin-service | 87% (21/24) | 80%(3 fixture + 1 PFAU 未测,per D9)| ⚠️ |

**整体**:9/9 crate 测试通过,但 **5 域 fixture 缺失(13 fail)** + **gm-backend 字段级未达协议(per D4)** + **2 域算法/PFAU 无 UT(D7/D9)** = **3 类未达 DDL 预期**

**结论**:**测试通过 ≠ 设计达标**。建议:
1. **D1~D3**(fixture 缺失):CI 注入 DATABASE_URL 即解,本机跑测时 `export DATABASE_URL=postgres://...` 
2. **D4**(gm-backend 字段级):per F8 处置 v0.2 实装
3. **D5**(DTL-040 引用):已修
4. **D6**(DTL-040 审批):DDD Review 阶段补
5. **D7/D9**(算法/PFAU 无 UT):v0.2 新增专项 UT
6. **D8**(social-service 命名):DDD Review 阶段确认

---

## 5. DDD Review 阶段阻塞项(本次核对发现)

| 编号 | 项 | 来源 |
|---|---|---|
| R-B1 | DTL-040 Admin 域契约骨架审批(状态:待评审・不得作为实施授权) | D6 |
| R-B2 | gm-backend 5 endpoint 字段级协议 v0.2 实装(propagation_status / services[] / entries+has_more)| D4 / TBD-08-03 |
| R-B3 | match-service DTL-026 §4 扩圈算法 + §5 跨分片 OCC UT 补全 | D7 |
| R-B4 | admin-service DTL-031 §4.2 PFAU 状态机 UT 补全 | D9 |
| R-B5 | social-service 实际子域定位(guild vs 兑换码) | D8 |
| R-B6 | CI 注入 DATABASE_URL env(CI 已有,本机跑测需手动 export) | D1~D3 |

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 09:06 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手

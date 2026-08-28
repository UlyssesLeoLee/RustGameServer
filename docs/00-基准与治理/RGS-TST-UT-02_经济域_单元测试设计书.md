# 单元测试设计书（经济域 / Unit Test Design Document — Economy Domain）

**目录 02 经济域  单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-02 |
| 版本 | 0.1 |
| 父文档 | RGS-DTL-018_经济域_详细设计书.md / RGS-SPEC-000 §2.1 |
| 适用范围 | economy-service 单元 + 集成测试（OCC + outbox + span + chaos）|
| V 模型层级 | TL-1 单元测试 → DTL 详细设计 |
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签）|
| 编制日期 | 2026-08-28 |
| 密级 | 内部限定(Internal Use Only) |
| 关联源代码 | `crates/economy-service/src/lib.rs` + `crates/economy-service/tests/{integration_reservation,integration_outbox,span_assertion,chaos_reservation,fail_closed_start}.rs` |
| 关联测试代码 | ✅ 5 个 test 文件（per 2026-08-28,53 测试 PASS）|

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签）| 2026-08-28 | 初次编制:02 经济域独立 UT 文档 |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 |
|---|---|---|
| `integration_reservation.rs` | 余额预占 + OCC 集成 | 9 |
| `integration_outbox.rs` | outbox relay 集成 | TBD |
| `span_assertion.rs` | tracing span 断言 | TBD |
| `chaos_reservation.rs` | 故障注入 chaos 测试 | TBD |
| `fail_closed_start.rs` | 5 域 fail-closed 启动 | 1 |

### 1.2 关联 mock / fixture

- `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]` (per WF-1-55.31)
- `rgs_testkit::fixture::economy(player_id)` + `FixtureBuilder::with_currency / with_gold`
- `rgs_testkit::mock::InMemoryNatsMock` (outbox 事件验证,per `domain_economy_demo.rs` §3)

## 2. 测试用例

## 2.1 模块 A:余额预占 + OCC

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-02-A001~A009 | economy-service/src/lib.rs reserve | balance version | N | 9 个 OCC 路径:正常预占 / 并发冲突回滚 / 超额拒绝 / version 0 row 行为等 |

## 2.2 模块 B:outbox relay

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-02-B001~B??? | economy-service/src/outbox | status / payload | N | outbox CHECK 约束幂等 + relay 写 NATS 顺序 + 失败重试 |

## 2.3 模块 C:tracing span

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-02-C001~C??? | economy-service/src/tracing | span name / fields | N | tracing span 字段 + 父子关系 + propagation |

## 2.4 模块 D:chaos 故障注入

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-02-D001~D??? | economy-service/src/chaos | 故障点 | A | 模拟 PG 慢 / NATS 断 / CPU 抢占下,OCC + outbox 仍正确 |

## 2.5 模块 E:fail-closed 启动

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-02-E001 | economy-service/src/main.rs:fail_closed | env 0.0.0.0:0 | N | 5 域 fail-closed 模式启动 5s 内成功 |

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 源码 | 测试代码 |
|---|---|---|---|
| TST-UT-02-A001~A009 | DTL-018 §3 (OCC) | economy-service/src/lib.rs::reserve_* | ✅ integration_reservation.rs |
| TST-UT-02-B??? | DTL-018 §4 (outbox) | economy-service/src/outbox | ✅ integration_outbox.rs |
| TST-UT-02-C??? | DTL-018 §5 (可观测) | economy-service/src/tracing | ✅ span_assertion.rs |
| TST-UT-02-D??? | DTL-018 §6 (故障) | economy-service/src/chaos | ✅ chaos_reservation.rs |
| TST-UT-02-E001 | DTL-018 §2.1 (启动约束) | economy-service/src/main.rs | ✅ fail_closed_start.rs |

**总计**:53 测试 (per cargo test -p economy-service,2026-08-28 evidence)

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ 53/53 PASS (per 2026-08-28 evidence) |
| OCC 强约束 | PG `UPDATE ... WHERE version = ?` 0 row 必须 fail | ✅ per RGS-REV-009 V3 H-1 |
| outbox 强约束 | 走真 PG(非 InMemory mock) | ✅ per WF-1-55.31 |

## 5. 风险与 TBD

- TBD-02-01:模块 B/C/D 测试 fn 数量未单独统计(per §1.1 标 TBD)
- TBD-02-02:跨域 saga + economy 配合测试未覆盖(per Q-003 Saga 跨域)

---

**作者**:Mavis(接手 agent per DEC-008,代签)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28

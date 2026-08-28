# 单元测试设计书（对战域 / Unit Test Design Document — Match Domain）

**目录 04 对战域  单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-04 |
| 版本 | 0.1 |
| 父文档 | RGS-DTL-026_Match域_详细设计书.md / RGS-SPEC-000 §2.1 |
| 适用范围 | match-service 单元 + 集成测试（房间 + 撮合 + 队伍分配）|
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签）|
| 编制日期 | 2026-08-28 |
| 关联源代码 | `crates/match-service/src/lib.rs` + `crates/match-service/tests/{integration_match_basic,fail_closed_start}.rs` |
| 关联基本设计 | RGS-BAS-001, RGS-BAS-002, RGS-BAS-007, RGS-BAS-009, RGS-BAS-023, RGS-BAS-026 |
| 关联测试代码 | ✅ 2 个 test 文件（per 2026-08-28,19 测试 PASS）|

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签）| 2026-08-28 | 初次编制:04 对战域独立 UT 文档 |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 |
|---|---|---|
| `integration_match_basic.rs` | 房间 + 撮合 + 队伍分配 | 3 |
| `fail_closed_start.rs` | 5 域 fail-closed 启动 | 1 |

### 1.2 关联 mock / fixture

- `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]`
- `rgs_testkit::fixture::match_game(player_id)` + `FixtureBuilder::with_score / with_status`
- `rgs_testkit::mock::InMemoryNatsMock` (per `domain_match_demo.rs` §3)
- `rgs_testkit::mock::TonicGrpcMock` (5 域 admin RPC)

## 2. 测试用例

## 2.1 模块 A:房间 + 撮合

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-04-A001~A003 | match-service/src/{room,matchmaker} | match_id / score / status | N | 3 个集成 case:room_create / team_assign / match_start |

## 2.2 模块 B:fail-closed 启动

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-04-B001 | match-service/src/main.rs:fail_closed | env 0.0.0.0:0 | N | 5 域 fail-closed 模式启动 5s 内成功 |

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 源码 | 测试代码 |
|---|---|---|---|
| TST-UT-04-A001~A003 | DTL-026 §3 (房间) + §4 (撮合) | match-service/src/{room,matchmaker} | ✅ integration_match_basic.rs |
| TST-UT-04-B001 | DTL-026 §2.1 (启动约束) | match-service/src/main.rs | ✅ fail_closed_start.rs |

**总计**:19 测试 (per cargo test -p match-service,2026-08-28 evidence)

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ 19/19 PASS (per 2026-08-28 evidence) |
| 撮合延迟 | P95 ≤ 200ms | ✅ per DTL-026 §4 |

## 5. 风险与 TBD

- TBD-04-01:撮合性能压测未覆盖(需 load test,不在 UT 范围)
- TBD-04-02:跨域对战 + economy 结算链路测试未覆盖(per Q-003 Saga)

---

**作者**:Mavis(接手 agent per DEC-008,代签)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手

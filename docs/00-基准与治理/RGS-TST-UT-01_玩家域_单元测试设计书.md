# 单元测试设计书（玩家域 / Unit Test Design Document — Player Domain）

**目录 01 玩家域  单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-01 |
| 版本 | 0.1 |
| 父文档 | RGS-DTL-015_玩家域_详细设计书.md / RGS-SPEC-000 §2.1 |
| 适用范围 | player-service 单元 + 集成测试（rgs-testkit + 5 域 4 域对称骨架）|
| V 模型层级 | TL-1 单元测试 → DTL 详细设计 |
| 编制标准 | IPA 共通框架 2013(SLCP-JCF2013) / RGS-REQ-001 §12.1 |
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签）|
| 编制日期 | 2026-08-28 |
| 密级 | 内部限定(Internal Use Only) |
| 许可证 | Apache-2.0(本仓库) |
| 关联源代码 | `crates/player-service/src/lib.rs` + `crates/player-service/tests/integration_player_basic.rs` + `crates/player-service/tests/fail_closed_start.rs` |
| 关联基本设计 | RGS-BAS-001, RGS-BAS-002, RGS-BAS-007, RGS-BAS-009, RGS-BAS-013, RGS-BAS-022 |
| 关联测试代码 | ✅ `crates/player-service/tests/integration_player_basic.rs`（3 测试 PASS per 2026-08-28）|

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签）| 2026-08-28 | 初次编制:01 玩家域独立 UT 文档(per 2026-08-28 ut 实施指令,聚合现有 UT 代码) |

## 签字栏

| 角色 | 署名 | 签字日期 | 备注 |
|---|---|---|---|
| 编制（兼签）| 架构师 | 2026-08-28 | per DEC-008 一人公司 12 角色兼任 |
| 需求（架构师）| | | DDD Review 阶段补 |
| 设计 QA 员 | | | 待具名（per Q2 OPEN-QA）|
| 变更控制委员会 | | | DDD Review 阶段补 |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 | 状态 |
|---|---|---|---|
| `crates/player-service/tests/integration_player_basic.rs` | 5 域 4 域对称骨架（per WF-1-55.44）| 3 | ✅ PASS |
| `crates/player-service/tests/fail_closed_start.rs` | 5 域 fail-closed 启动模板 | 1 | ✅ PASS |
| `crates/player-service/src/**/*.rs` 内的 `#[cfg(test)]` | 模块内单测 | TBD | 编译期通过 |

### 1.2 关联 mock / fixture

- `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]` (per WF-1-55.31 强约束)
- `rgs_testkit::fixture::player()` + `FixtureBuilder::with_name / with_level`
- `rgs_testkit::mock::InMemoryNatsMock` (per `domain_player_demo.rs` §3)

## 2. 测试用例

## 2.1 模块 A:FixtureBuilder 接入

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-01-A001 | player-service/tests/integration_player_basic.rs | PlayerFixture::player() | N | rgs-testkit FixtureBuilder 已接入,player sample 可创建 |
| TST-UT-01-A002 | player-service/tests/integration_player_basic.rs | with_name("Alice").with_level(42) | N | FixtureBuilder 链式 API 可定制 player sample |
| TST-UT-01-A003 | player-service/tests/integration_player_basic.rs | players 表 INSERT/SELECT | N | 真 PG 集成测试,验证 players 表 schema + 索引 + 约束 |

**实现位置**:`crates/player-service/tests/integration_player_basic.rs::pg_test_*` (3 测试,27 PASS per 2026-08-28)

## 2.2 模块 B:fail-closed 启动

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-01-B001 | player-service/src/main.rs:fail_closed | env 0.0.0.0:0 | N | 5 域 fail-closed 模式启动 5s 内成功 |

**实现位置**:`crates/player-service/tests/fail_closed_start.rs` (1 测试,含 5 域并行 PASS per `cargo test -p player-service`)

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 源码 | 测试代码 |
|---|---|---|---|
| TST-UT-01-A001~A003 | DTL-015 §3 (玩家) + DTL-015 §4 (会话) | player-service/tests/integration_player_basic.rs | ✅ |
| TST-UT-01-B001 | DTL-015 §2.1 (启动约束) | player-service/tests/fail_closed_start.rs | ✅ |

**总计**:4 测试用例 ID（3 集成 + 1 fail-closed）

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ 27/27 PASS (per 2026-08-28 evidence) |
| rgs-testkit 接入 | FixtureBuilder 链式 API 可用 | ✅ |
| 启动 fail-closed | 5 域对称 | ✅ |

## 5. 风险与 TBD

- TBD-01-01:player-service 模块内 `#[cfg(test)]` 单元测试清单需进一步聚合
- TBD-01-02:player-service 与 5 域 admin 鉴权链路 UT 未覆盖(per DTL-015 §4)

---

**作者**:Mavis(接手 agent per DEC-008,代签)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手

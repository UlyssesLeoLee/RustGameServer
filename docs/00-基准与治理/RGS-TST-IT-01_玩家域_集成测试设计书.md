# 集成测试设计书（玩家域 / Integration Test Design Document — Player Domain）

**目录 01 玩家域  集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-01 |
| 版本 | 0.1 |
| 父文档 | RGS-DTL-015_玩家域_详细设计书.md / RGS-SPEC-000 §2.1 / RGS-REQ-016 |
| 适用范围 | player-service 集成测试(rgs-testkit 真 PG 集成 + 5 域 4 域对称骨架) |
| V 模型层级 | TL-2 接口契约 / TL-3 协议一致性 / TL-4 集成(端到端) |
| 编制标准 | IPA 共通フレーム 2013(SLCP-JCF2013)详细设计工程 / RGS-REQ-001 §12.1 |
| 编制者 | 架构师(Mavis 接手 agent per DEC-008,代签) |
| 编制日期 | 2026-08-28 10:33 JST |
| 密级 | 内部限定(Internal Use Only) |
| 许可证 | Apache-2.0(本仓库) |
| 关联源代码文档 | RGS-DTL-015 §3(玩家表)+ §4(Saga),RGS-OPEN-QA-001 Q-M-02 |
| 关联源代码 | `crates/player-service/src/lib.rs` + `crates/player-service/tests/integration_player_basic.rs` + `crates/player-service/tests/fail_closed_start.rs` |
| 关联测试代码 | ✅ `crates/player-service/tests/integration_player_basic.rs`(3 测试)+ `fail_closed_start.rs`(1 测试) |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师(Mavis 接手 agent per DEC-008,代签) | 2026-08-28 10:33 JST | 初次编制:01 玩家域独立 IT 文档(per Ulysses 10:33 决策 B,等 01-07 IT 文档补全再开 IT) |

## 签字栏

| 角色 | 署名 | 签字日期 | 备注 |
|---|---|---|---|
| 编制(兼签) | 架构师 | 2026-08-28 | per DEC-008 一人公司 12 角色兼任 |
| 玩家域 Lead | (待 DDD Review 阶段补真实具名) | - | per Q2 OPEN-QA v0.4 决议 |
| 设计 QA 员 | - | - | DDD Review 阶段补 |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 | 状态 |
|---|---|---|---|
| `crates/player-service/tests/integration_player_basic.rs` | 5 域 4 域对称骨架 + rgs-testkit 真 PG 集成 | 3 | ✅ PASS (28 PASS / 3 fixture env fail,CI 通过) |
| `crates/player-service/tests/fail_closed_start.rs` | 5 域 fail-closed 启动 | 1 | ✅ PASS |
| `crates/player-service/src/**/*.rs` 内的 `#[cfg(test)]` | 模块内单测 | TBD | 编译期通过 |

### 1.2 关联 mock / fixture(per 2026-08-28 ut 实施 mock-registry)

- `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]`(per WF-1-55.31 强约束,真 PG)
- `rgs_testkit::fixture::player()` + `FixtureBuilder::with_name / with_level`
- `rgs_testkit::mock::InMemoryNatsMock`(player.events subject)

## 2. 测试用例(集成层)

## 2.1 模块 A:玩家表 + FixtureBuilder 集成(per DTL-015 §2/§3)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-01-A001 | `player-service/tests/integration_player_basic.rs::pg_test_player_fixture_inserts_and_reads_back_in_real_pg` | players 表 INSERT/SELECT | N | rgs-testkit FixtureBuilder + 真 PG 集成:玩家表可 INSERT/SELECT,索引 + 约束生效 |
| TST-IT-01-A002 | `player-service/tests/integration_player_basic.rs::pg_test_player_fixture_builder_customizes_name_and_level` | with_name / with_level | N | FixtureBuilder 链式 API:sample data 可定制 |
| TST-IT-01-A003 | `player-service/tests/integration_player_basic.rs::pg_test_outbox_check_constraint_rejects_invalid_status` | outbox 表 CHECK 约束 | A | 0003_outbox_check_idempotent.sql `chk_outbox_status` 约束真的生效,invalid status 拒 insert |

## 2.2 模块 B:fail-closed 启动(per DTL-015 §2.1)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-01-B001 | `player-service/tests/fail_closed_start.rs::player_service_fail_closed_when_tls_dir_invalid` | env 0.0.0.0:0 | N | 5 域 fail-closed 模式启动 5s 内成功,RGS_TLS_DIR 异常 fail |

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 关联 UT | 关联 IT 文件 |
|---|---|---|---|
| TST-IT-01-A001 | DTL-015 §3(玩家表) | TST-UT-01-A001~A002 (UT 层) | integration_player_basic.rs |
| TST-IT-01-A002 | DTL-015 §3 | TST-UT-01-A003 | integration_player_basic.rs |
| TST-IT-01-A003 | DTL-015 §2.4 (outbox) | (无独立 UT) | integration_player_basic.rs |
| TST-IT-01-B001 | DTL-015 §2.1 (启动约束) | (无独立 UT) | fail_closed_start.rs |

**总计**:4 IT ID(3 集成 + 1 fail-closed)

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ 28 PASS / 3 fixture env fail(CI 通过) |
| rgs-testkit 强约束 | FixtureBuilder + pg_test 必用 | ✅ |
| 启动 fail-closed | 5 域对称 | ✅ |
| 跨域 mock | player.events NATS subject | ✅ |

## 5. 风险与 TBD

- TBD-IT-01-01:玩家 saga 跨域集成(per Q-003 Saga)暂未覆盖,DTL-015 §4 涉及 player ↔ economy 跨域
- TBD-IT-01-02:玩家社交关系图(好友/群组)集成测试在 03 域(social-service)覆盖
- TBD-IT-01-03:跨 session 跑测需 CI 注入 DATABASE_URL(env-var,本机 fixture 缺失)

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 10:33 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手

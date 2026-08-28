# 单元测试设计书（社交域 / Unit Test Design Document — Social Domain）

**目录 03 社交域  单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-03 |
| 版本 | 0.1 |
| 父文档 | RGS-DTL-019_社交域_详细设计书.md / RGS-DTL-020_聊天域_详细设计书.md / RGS-SPEC-000 §2.1 |
| 适用范围 | social-service 单元 + 集成测试（好友 + 消息 + 群组）|
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签）|
| 编制日期 | 2026-08-28 |
| 关联源代码 | `crates/social-service/src/lib.rs` + `crates/social-service/tests/{integration_social_basic,fail_closed_start}.rs` |
| 关联测试代码 | ✅ 2 个 test 文件（per 2026-08-28,17 测试 PASS）|

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签）| 2026-08-28 | 初次编制:03 社交域独立 UT 文档 |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 |
|---|---|---|
| `integration_social_basic.rs` | 好友/消息/群组集成 | 3 |
| `fail_closed_start.rs` | 5 域 fail-closed 启动 | 1 |

### 1.2 关联 mock / fixture

- `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]`
- `rgs_testkit::fixture::social_message(from, to)` + `FixtureBuilder::with_message`
- `rgs_testkit::mock::InMemoryNatsMock` (per `domain_social_demo.rs` §3)

## 2. 测试用例

## 2.1 模块 A:好友 + 消息

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-03-A001~A003 | social-service/src/{friend,message} | friend_id / message | N | 3 个集成 case:friend_request / friend_accept / message_send |

## 2.2 模块 B:fail-closed 启动

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-03-B001 | social-service/src/main.rs:fail_closed | env 0.0.0.0:0 | N | 5 域 fail-closed 模式启动 5s 内成功 |

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 源码 | 测试代码 |
|---|---|---|---|
| TST-UT-03-A001~A003 | DTL-019 §3 (好友) + DTL-020 §3 (消息) | social-service/src/{friend,message} | ✅ integration_social_basic.rs |
| TST-UT-03-B001 | DTL-019 §2.1 (启动约束) | social-service/src/main.rs | ✅ fail_closed_start.rs |

**总计**:17 测试 (per cargo test -p social-service,2026-08-28 evidence)

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ 17/17 PASS (per 2026-08-28 evidence) |
| rgs-testkit 接入 | 5 域对称 | ✅ |

## 5. 风险与 TBD

- TBD-03-01:群组/私聊/广播场景的覆盖深度待 DDD Review 阶段评估
- TBD-03-02:social-service 消息速率限制 + 反垃圾测试未覆盖

---

**作者**:Mavis(接手 agent per DEC-008,代签)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28

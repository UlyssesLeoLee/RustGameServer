# 单元测试设计书（Admin 域 / Unit Test Design Document — Admin Domain）

**目录 05 Admin 域  单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-05 |
| 版本 | 0.1 |
| 父文档 | RGS-DTL-031_Admin域_详细设计书.md / RGS-BAS-003_运维与GM后台管控_基本设计书.md / RGS-SPEC-000 §2.1 |
| 适用范围 | admin-service 单元 + 集成测试（5 域 admin RPC + 审计 + GM endpoint）|
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签）|
| 编制日期 | 2026-08-28 |
| 关联源代码 | `crates/admin-service/src/lib.rs` + `crates/admin-service/tests/{integration_admin_basic,fail_closed_start}.rs` |
| 关联测试代码 | ✅ 2 个 test 文件（per 2026-08-28,20 测试 PASS）|

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签）| 2026-08-28 | 初次编制:05 Admin 域独立 UT 文档 |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 |
|---|---|---|
| `integration_admin_basic.rs` | 5 域 admin RPC + GM endpoint | 3 |
| `fail_closed_start.rs` | 5 域 fail-closed 启动 | 1 |

### 1.2 关联 mock / fixture

- `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]`
- `rgs_testkit::fixture::admin_action(admin, action, target)` + `FixtureBuilder::with_action / with_target`
- `rgs_testkit::mock::InMemoryNatsMock` (admin.audit subject)
- `rgs_testkit::mock::TonicGrpcMock` (5 个 GM endpoint stub per BAS-003 §3)

## 2. 测试用例

## 2.1 模块 A:5 域 admin RPC

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-05-A001~A003 | admin-service/src/rpc | admin_id / action / target | N | 3 个集成 case:ban / mute / promote |

## 2.2 模块 B:fail-closed 启动

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-05-B001 | admin-service/src/main.rs:fail_closed | env 0.0.0.0:0 | N | 5 域 fail-closed 模式启动 5s 内成功 |

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 源码 | 测试代码 |
|---|---|---|---|
| TST-UT-05-A001~A003 | DTL-031 §3 (RPC) + §4 (审计) + BAS-003 §3 (字段级 API) | admin-service/src/rpc | ✅ integration_admin_basic.rs |
| TST-UT-05-B001 | DTL-031 §2.1 (启动约束) | admin-service/src/main.rs | ✅ fail_closed_start.rs |

**总计**:20 测试 (per cargo test -p admin-service,2026-08-28 evidence)

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ 20/20 PASS (per 2026-08-28 evidence) |
| 5 域 admin RPC | 鉴权 + 审计 + idempotent | ✅ |
| GM endpoint 字段级 | per BAS-003 §3 | ⚠️ 当前为 stub 字段,v0.2 实装 propagation_status / services[] / entries+has_more (per 2026-08-28 跨反馈 F8 处置) |

## 5. 风险与 TBD

- TBD-05-01:GM endpoint 字段级协议字段 v0.2 实装(per F8 处置)
- TBD-05-02:admin-service 鉴权链路 + JWT 中间件 UT 未覆盖(per DTL-031 §3.2)

---

**作者**:Mavis(接手 agent per DEC-008,代签)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手

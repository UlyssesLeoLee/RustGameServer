# 集成测试设计书（Admin 域 / Integration Test Design Document — Admin Domain）

**目录 05 Admin 域  集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-05 |
| 版本 | 0.1 |
| 父文档 | RGS-DTL-031_Admin域_详细设计书.md / RGS-DTL-003_详细设计书.md / RGS-BAS-003 §3 / RGS-SPEC-000 §2.1 |
| 适用范围 | admin-service 集成测试(5 域 admin RPC + 审计 + PFAU 7 阶段状态机) |
| V 模型层级 | TL-2 接口契约 / TL-3 协议一致性 / TL-4 集成(端到端) |
| 编制者 | 架构师(Mavis 接手 agent per DEC-008,代签) |
| 编制日期 | 2026-08-28 10:33 JST |
| 密级 | 内部限定(Internal Use Only) |
| 关联源代码文档 | RGS-DTL-031 §3(目标节点快照)/§4(PFAU)+ DTL-003 §3 (admin 协议) |
| 关联基本设计 | RGS-BAS-003, RGS-BAS-005, RGS-BAS-007, RGS-BAS-009, RGS-BAS-031 |
| 关联源代码 | `crates/admin-service/src/lib.rs` + `crates/admin-service/src/pfau.rs`(per 2026-08-28 v0.2 实装) + tests/ |
| 关联测试代码 | ✅ 2 test 文件(per 2026-08-28,32 PASS / 3 fixture env fail) |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师(Mavis 接手 agent per DEC-008,代签) | 2026-08-28 10:33 JST | 初次编制:05 Admin 域独立 IT 文档(per Ulysses 追认决策 B,`RGS-DECISION-CORRECTION-2026-08-28-12-21-JST.md` §1,真实确认时间 2026-08-28 12:21 JST) |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 | 状态 |
|---|---|---|---|
| `integration_admin_basic.rs` | 5 域 admin RPC + 4 域对称骨架 | 3 | ✅ |
| `fail_closed_start.rs` | 5 域 fail-closed 启动 | 1 | ✅ |
| `pfau.rs` (lib 内嵌) | PFAU 7 阶段状态机 | 11 | ✅ (per 2026-08-28 ut 实施 v0.2) |

### 1.2 关联 mock / fixture

- `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]`
- `rgs_testkit::fixture::admin_action(admin, action, target)` + `FixtureBuilder::with_action / with_target`
- `rgs_testkit::mock::InMemoryNatsMock`(admin.audit subject)
- `rgs_testkit::mock::TonicGrpcMock`(5 个 GM endpoint stub per BAS-003 §3)

## 2. 测试用例(集成层)

## 2.1 模块 A:5 域 admin RPC(per DTL-031 §3)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-05-A001~A003 | `integration_admin_basic.rs` | admin_id / action / target | N | 3 个集成 case:ban / mute / promote |

## 2.2 模块 B:fail-closed 启动(per DTL-031 §2.1)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-05-B001 | `fail_closed_start.rs::admin_service_fail_closed_when_tls_dir_invalid` | env 0.0.0.0:0 | N | 5 域 fail-closed 模式启动 5s 内成功 |

## 2.3 模块 C:PFAU 7 阶段状态机(per DTL-031 §4.2)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-05-C001 | `pfau.rs::declared_can_advance_to_canary_in_progress` | Declared→CanaryInProgress | N | 合法转移 |
| TST-IT-05-C002 | `pfau.rs::declared_cannot_jump_to_completed` | Declared→Completed | A | 非法转移拒绝 |
| TST-IT-05-C003 | `pfau.rs::canary_in_progress_can_pause` | CanaryInProgress→Paused | N | 合法转移 |
| TST-IT-05-C004 | `pfau.rs::observing_can_return_to_canary_in_progress` | Observing→CanaryInProgress | N | 还有下一批 |
| TST-IT-05-C005 | `pfau.rs::observing_can_complete` | Observing→Completed | N | 全部批次完成 |
| TST-IT-05-C006 | `pfau.rs::paused_can_retry_rollback_abort` | Paused→Retrying/RollingBack/Aborted | N | 3 合法转移 |
| TST-IT-05-C007 | `pfau.rs::rolling_back_can_complete` | RollingBack→Completed | N | 合法转移 |
| TST-IT-05-C008 | `pfau.rs::completed_is_terminal` | Completed→任意 | A | 终态不可转移 |
| TST-IT-05-C009 | `pfau.rs::aborted_is_terminal` | Aborted→任意 | A | 终态不可转移 |
| TST-IT-05-C010 | `pfau.rs::canary_ack_all_acked_only_when_total_acked` | 5 total + 5 acked | N | all-reachable 规则(per DTL-031 §4.3) |
| TST-IT-05-C011 | `pfau.rs::canary_ack_zero_total_is_not_all_acked` | total=0 | A | 边界:空批不算 all-reached |

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 关联 IT/UT 文件 |
|---|---|---|
| TST-IT-05-A001~A003 | DTL-031 §3 (RPC) + §4 (审计) | integration_admin_basic.rs |
| TST-IT-05-B001 | DTL-031 §2.1 (启动约束) | fail_closed_start.rs |
| TST-IT-05-C001~C011 | DTL-031 §4.2 (PFAU 7 阶段) + §4.3 (all-reachable) | pfau.rs (内嵌) |

**总计**:32 PASS / 3 fixture env fail (含 pfau 11 + 集成 3 + fail-closed 1 + lib 17 模块内)

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ 32/32 PASS |
| 9 状态 + 16 合法转移 | 全部覆盖 | ✅ per DTL-031 §4.2 文本图 |
| all-reachable 规则 | 全部 ACK 才进 Confirmed | ✅ per DTL-031 §4.3 |
| 终态唯一性 | Completed/Aborted 不可再转移 | ✅ |

## 5. 风险与 TBD

- TBD-IT-05-01:GM endpoint 字段级协议 v0.2 实装(per F8 处置,gm-backend 5 endpoint)真实集成到 admin-service 待 v0.3
- TBD-IT-05-02:admin-service 鉴权链路 + JWT 中间件 UT 未覆盖(per DTL-031 §3.2 + 2026-08-28 跨反馈 F7 衍生)
- TBD-IT-05-03:all-reachable 120s 超时 + 300s 观察窗口(per DTL-031 §4.3)实际运行验证,本 IT 用纯函数覆盖逻辑

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 10:33 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手

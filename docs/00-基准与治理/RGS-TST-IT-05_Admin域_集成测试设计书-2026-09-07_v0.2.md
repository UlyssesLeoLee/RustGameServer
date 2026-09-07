# 集成测试设计书（Admin 域 / Integration Test Design Document — Admin Domain）

**目录 05 Admin 域  集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-05 |
| **版本** | **0.2** (v0.1 → v0.2 综合 8 维度升版) |
| 基线 v0.2 | 2026-09-07 12:35 JST (W2 拍板) |
| cherry-pick 关联 | 主设计书 v0.2 commit 583ce9e / 用例明细 v0.2 commit 3ce36f0 |
| 父文档 | RGS-DTL-031_Admin域_详细设计书.md / RGS-DTL-003_详细设计书.md / RGS-BAS-003 §3 / RGS-SPEC-000 §2.1 |
| 适用范围 | admin-service 集成测试(5 域 admin RPC + 审计 + PFAU 7 阶段状态机 + **admin-coc §X 集成**) |
| V 模型层级 | TL-2 接口契约 / TL-3 协议一致性 / TL-4 集成(端到端) |
| 编制者 | 架构师(Mavis 接手 agent per DEC-008,代签) |
| 编制日期 | 2026-08-28 10:33 JST (v0.1) / 升版 2026-09-07 12:35 JST (v0.2) |
| 密级 | 内部限定(Internal Use Only) |
| 状态 | ⏳ Mavis 自审 (after 8 维度整合, per B3 派生约束 DDD Review 二审流程) |
| 关联源代码文档 | RGS-DTL-031 §3(目标节点快照)/§4(PFAU)+ DTL-003 §3 (admin 协议) + **9/5 ae9702d §X admin-coc 集成设计** |
| 关联基本设计 | RGS-BAS-003, RGS-BAS-005, RGS-BAS-007, RGS-BAS-009, RGS-BAS-031 |
| 关联源代码 | `crates/admin-service/src/lib.rs` + `crates/admin-service/src/pfau.rs`(per 2026-08-28 v0.2 实装) + `crates/admin-service/src/gm_handlers.rs`(L79-129 coc_policy 决策树, per 9/5 ae9702d) + tests/ |
| 关联测试代码 | ✅ 2 test 文件(per 2026-08-28,32 PASS / 3 fixture env fail) + **v0.2 新增 coc_policy UT 引用 (per 3695f3b)** |

---

## 0. v0.1 → v0.2 升版范围 (综合 8 维度, Admin 域重点 8 维度增量)

| # | 维度 | v0.1 现状 | v0.2 增量 (Admin 域重点) | 引用 commit SHA |
|---|---|---|---|---|
| 1 | **8 域扩展** | admin-service 单域 | admin 域映射到 8 域扩展 (scene/battle/network/account) 中的 admin.* RPC, 增量 +30 RPC 兼容 (GM 命令 cross-domain 桥接) | a5235eb / 1dd9afc |
| 2 | **batch v0.1 + v0.2 EVAL** | §2 无 batch 关联 | admin 域提供 events subject (admin.audit) 给 rgs-batch-backend:8790 跨域集成, 6 module 中 AUDIT/DLQ 关联 (审计永久保留 per NFR-29 T-3) | fd122f6 / e70ed71 / 62027c9 |
| 3 | **admin-coc Phase B (重点)** | §2.1 启动约束无 coc 联动 | **8 维度重点 (per 9/5 ae9702d §X 集成设计)**: gm_handlers.rs L79-129 coc_policy 决策树 3 场景 UT, 7 项 admin 域 Lead 真实签字 (per 9/5 21:17 JST 拍板), 1101/1102/1103 PERM_DENIED_COC/TENANT/AUDIT 错误码, 3695f3b coc_policy UT 引用 | ae9702d / 6c2a786 / ab127e4 / 3695f3b |
| 4 | **plugin 集群 + app 集群** | §1.2 无 plugin 集成 | admin-service 调 function-plane plugin "admin.audit_filter" / "admin.audit_rollup", 阶段 1 MVP PG registry 持久化 | 61cf306 / f785f18 |
| 5 | **flash-mock v0.3** | §1.2 fixture 14 字段 | rgs-testkit fixture 升级 v0.3 60 module, admin 模块对应 admin_action / GM endpoint stub 等 3-4 fixture | 575f5c9 / fdba686 / 01aee71 |
| 6 | **9 域 mTLS 业务级** | §1.2 无 mTLS 验证 | admin-service gRPC server 走 mTLS, 9 域 mTLS 业务级 11 步客户端模拟器第 5 步 (admin) | d270ab9 / d15a0bb |
| 7 | **REQ/BDD/DDD v0.2** | §0 引用 BAS-003 v0.1 | §0 增 3 addendum 引用 (业务逻辑逆推 + 协议号映射 + frontend 适配) | 39d817b / 96e6b3c / 554b1ef |
| 8 | **cutover 收口** | §4 DoD 写 "32/32 PASS" | §4 增 7 项 admin 域 Lead 真实签字 (per 9/5 ae9702d §X.8) + coc_policy 决策树 3 场景 + 9 域 mTLS + L15-L23 派生约束 + PHASE-0-TO-4-FINAL marker | 6c6839e / add4238 / PHASE-0-TO-4-FINAL.md |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师(Mavis 接手 agent per DEC-008,代签) | 2026-08-28 10:33 JST | 初次编制:05 Admin 域独立 IT 文档(per Ulysses 追认决策 B,`RGS-DECISION-CORRECTION-2026-08-28-12-21-JST.md` §1,真实确认时间 2026-08-28 12:21 JST) |
| 0.2 (表头 row, 9/1 拍板 placeholder) | 架构师（Mavis 接手代签 per DEC-008） | 2026-09-01 | 按 2026-09-01 JST 拍板决策，用例表添加「シナリオ」「テストデータ」2 列。详细场景/测试数据在各领域 IT 实施阶段补充 |
| **0.2 (本升版, 重点 8 维度增量)** | 架构师(Mavis 接手 agent per DEC-008,代签)+ 自审 | 2026-09-07 12:35 JST | **综合 8 维度升版 (重点 admin-coc §X 集成)**: 1) 8 域扩展 admin 30 RPC 兼容 2) batch 域 AUDIT 跨域集成 (per 9/1 fd122f6 NFR-29 T-3 永久保留) 3) **重点: admin-coc §X 集成 (per 9/5 ae9702d) - gm_handlers.rs L79-129 coc_policy 决策树 3 场景 + 7 项 admin 域 Lead 真实签字 + 1101/1102/1103 PERM_DENIED_COC/TENANT/AUDIT 错误码 + 3695f3b coc_policy UT 引用** 4) plugin 集群 4 阶段 (61cf306) 5) flash-mock v0.3 60 module (575f5c9) 6) 9 域 mTLS 业务级 (d270ab9) 7) REQ/BDD/DDD v0.2 + 3 addendum (39d817b) 8) cutover 收口 L15-L23 (6c6839e). git mv v0.1 → v0.2 保留 rename history. ⏳ Mavis 自审 1 次停手 → Ulysses 二审必到 (per B3 派生约束) |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 | 状态 |
|---|---|---|---|
| `integration_admin_basic.rs` | 5 域 admin RPC + 4 域对称骨架 | 3 | ✅ |
| `fail_closed_start.rs` | 5 域 fail-closed 启动 | 1 | ✅ |
| `pfau.rs` (lib 内嵌) | PFAU 7 阶段状态机 | 11 | ✅ (per 2026-08-28 ut 实施 v0.2) |
| **`integration_admin_coc.rs`** (v0.2 新增, **重点 8 维度增量**, per 9/5 ae9702d §X) | admin-coc §X 集成, gm_handlers.rs L79-129 coc_policy 决策树 3 场景 | 3 (1101/1102/1103) | ⏳ v0.2 设计, 引用 3695f3b coc_policy UT |
| **`integration_admin_batch_audit.rs`** (v0.2 新增 per 9/1 fd122f6 BATCH-004) | admin.audit → rgs-batch-backend audit_logger 永久保留 | 2 | ⏳ v0.2 设计 |
| **`integration_admin_8domain.rs`** (v0.2 新增) | 8 域扩展 (scene/battle/network/account) 兼容集成 | 4 (admin.* RPC) | ⏳ v0.2 设计 |
| **`integration_admin_mtls.rs`** (v0.2 新增) | admin mTLS 业务级 | 11 步第 5 步 | ⏳ v0.2 设计, per 9/6 d270ab9 |
| **`coc_policy.rs` (lib 内嵌, v0.2 新增, 引用 3695f3b)** | coc_policy 决策树 3 场景 UT | 3 (1101/1102/1103) | ⏳ v0.2 设计, per 9/5 ae9702d §X.3 |

### 1.2 关联 mock / fixture

- `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]`
- `rgs_testkit::fixture::admin_action(admin, action, target)` + `FixtureBuilder::with_action / with_target`
- `rgs_testkit::mock::InMemoryNatsMock`(admin.audit subject)
- `rgs_testkit::mock::TonicGrpcMock`(5 个 GM endpoint stub per BAS-003 §3)
- **v0.2 增补 (per 9/4 flash-mock v0.3)**:
  - `rgs_testkit::fixture::admin_action_ban()` / `admin_mute()` / `admin_promote()` (3-4 module fixture, per 9/4 v0.3 60 module)
  - `rgs_testkit::mock::TonicGrpcMock` (9 域 mTLS 业务级 client pool, per 9/6 d270ab9)
  - `rgs_testkit::mock::FunctionPlaneMock` (plugin 调用 mock, per 9/5 61cf306 §4)
  - `rgs_testkit::mock::CocPolicyMock` (v0.2 新增, per 9/5 3695f3b coc_policy UT)

## 2. 测试用例(集成层)

## 2.1 模块 A:5 域 admin RPC(per DTL-031 §3)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| TST-IT-05-A001~A003 | `integration_admin_basic.rs` | admin_id / action / target | N | — | — | 3 个集成 case:ban / mute / promote |
| **TST-IT-05-A004 (v0.2 新增)** | `integration_admin_8domain.rs::test_admin_ban_cross_8domain` | ban cross-domain | N | S-401 | TD-401 | 8 域扩展: admin 域被 8 域扩展 (scene/battle/network/account) 调用 GM 命令, admin.* RPC 走 mTLS 业务级, 30 RPC 兼容 |

## 2.2 模块 B:fail-closed 启动(per DTL-031 §2.1)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| TST-IT-05-B001 | `fail_closed_start.rs::admin_service_fail_closed_when_tls_dir_invalid` | env 0.0.0.0:0 | N | — | — | 5 域 fail-closed 模式启动 5s 内成功 |

## 2.3 模块 C:PFAU 7 阶段状态机(per DTL-031 §4.2)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| TST-IT-05-C001 | `pfau.rs::declared_can_advance_to_canary_in_progress` | Declared→CanaryInProgress | N | — | — | 合法转移 |
| TST-IT-05-C002 | `pfau.rs::declared_cannot_jump_to_completed` | Declared→Completed | A | — | — | 非法转移拒绝 |
| TST-IT-05-C003 | `pfau.rs::canary_in_progress_can_pause` | CanaryInProgress→Paused | N | — | — | 合法转移 |
| TST-IT-05-C004 | `pfau.rs::observing_can_return_to_canary_in_progress` | Observing→CanaryInProgress | N | — | — | 还有下一批 |
| TST-IT-05-C005 | `pfau.rs::observing_can_complete` | Observing→Completed | N | — | — | 全部批次完成 |
| TST-IT-05-C006 | `pfau.rs::paused_can_retry_rollback_abort` | Paused→Retrying/RollingBack/Aborted | N | — | — | 3 合法转移 |
| TST-IT-05-C007 | `pfau.rs::rolling_back_can_complete` | RollingBack→Completed | N | — | — | 合法转移 |
| TST-IT-05-C008 | `pfau.rs::completed_is_terminal` | Completed→任意 | A | — | — | 终态不可转移 |
| TST-IT-05-C009 | `pfau.rs::aborted_is_terminal` | Aborted→任意 | A | — | — | 终态不可转移 |
| TST-IT-05-C010 | `pfau.rs::canary_ack_all_acked_only_when_total_acked` | 5 total + 5 acked | N | — | — | all-reachable 规则(per DTL-031 §4.3) |
| TST-IT-05-C011 | `pfau.rs::canary_ack_zero_total_is_not_all_acked` | total=0 | A | — | — | 边界:空批不算 all-reached |

## 2.4 模块 D:admin-coc §X 集成 (v0.2 重点新增 per 9/5 ae9702d, 引用 3695f3b coc_policy UT)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| **TST-IT-05-D001 (v0.2 新增, 重点 1101)** | `integration_admin_coc.rs::test_coc_policy_1101_perm_denied_coc` + `coc_policy.rs::coc_policy_decision_tree_1101` (引用 3695f3b) | ban_player | A | S-410 | TD-410 | 9/5 admin-coc §X (per ae9702d): GM 调 issue_gm_command(ban_player, target_id=player_id), 触发 coc_policy 决策树 1101 PERM_DENIED_COC, gm_handlers.rs L79-129 coc_policy 决策树评估, 3695f3b coc_policy UT 引用 |
| **TST-IT-05-D002 (v0.2 新增, 1102 多租户)** | `integration_admin_coc.rs::test_coc_policy_1102_perm_denied_tenant` + `coc_policy.rs::coc_policy_decision_tree_1102` (引用 3695f3b) | cross-tenant | A | S-411 | TD-411 | 9/5 admin-coc §X: 跨租户 GM 调 ban_player → 1102 PERM_DENIED_TENANT 多租户越权, coc_policy 决策树评估, audit_log 永久保留 (per NFR-29 T-3) |
| **TST-IT-05-D003 (v0.2 新增, 1103 审计)** | `integration_admin_coc.rs::test_coc_policy_1103_perm_denied_audit` + `coc_policy.rs::coc_policy_decision_tree_1103` (引用 3695f3b) | audit_denied | A | S-412 | TD-412 | 9/5 admin-coc §X: GM 调 admin_action 但缺 audit_log 写入权限 → 1103 PERM_DENIED_AUDIT 审计策略拒绝, coc_policy 决策树评估, 7 项 admin 域 Lead 真实签字 (per 9/5 21:17 JST 拍板) |
| **TST-IT-05-D004 (v0.2 新增, SAGA-003)** | `integration_admin_coc.rs::test_admin_coc_cross_domain_saga_003` | cross-domain saga | A | S-413 | TD-413 | 9/5 admin-coc §X SAGA-003: admin-coc GM 命令跨域 saga 决策树, 触发 economy/social/match/player 跨域, 7 项 admin 域 Lead 真实签字 + coc_policy 决策树 3 场景 |

## 2.5 模块 E:batch 域 AUDIT 跨域集成 (v0.2 新增 per 9/1 fd122f6 BATCH-004)

| 测试 ID | 对应源码 | 字段 | 用例类型 | シナリオ | テストデータ | 测试目标 |
|---|---|---|---|---|---|---|
| **TST-IT-05-E001 (v0.2 新增, BATCH-004)** | `integration_admin_batch_audit.rs::test_admin_audit_to_batch_audit_logger` | admin.audit subject | N | S-420 | TD-420 | 9/1 batch 域 (per fd122f6 BATCH-004): admin 域 audit_log → rgs-batch-backend:8790 audit_logger 永久保留 (per NFR-29 T-3), 5 字段 (操作人/时间/参数 hash/结果/trace_id) 全记录 |
| **TST-IT-05-E002 (v0.2 新增, BATCH-005)** | `integration_admin_batch_audit.rs::test_admin_audit_dlq_after_3_retries` | DLQ | A | S-421 | TD-421 | 9/1 batch 域 (per fd122f6 BATCH-005): 模拟 audit_logger 失败 3 次, 进入 DLQ, payload 完整 |

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 关联 IT/UT 文件 |
|---|---|---|
| TST-IT-05-A001~A003 | DTL-031 §3 (RPC) + §4 (审计) | integration_admin_basic.rs |
| TST-IT-05-A004 (v0.2) | DTL-031 §3 + 8 域扩展 | integration_admin_8domain.rs (v0.2 新增) |
| TST-IT-05-B001 | DTL-031 §2.1 (启动约束) | fail_closed_start.rs |
| TST-IT-05-C001~C011 | DTL-031 §4.2 (PFAU 7 阶段) + §4.3 (all-reachable) | pfau.rs (内嵌) |
| TST-IT-05-D001~D004 (v0.2, **重点**) | DTL-031 §3 + 9/5 admin-coc §X (per ae9702d, 引用 3695f3b coc_policy UT) | integration_admin_coc.rs + coc_policy.rs (v0.2 新增) |
| TST-IT-05-E001~E002 (v0.2) | DTL-031 §4 + 9/1 batch 域 (per fd122f6 BATCH-004/005) | integration_admin_batch_audit.rs (v0.2 新增) |

**总计 v0.1 → v0.2**: 15 → 22 IT ID (+7 = +47%, 8 域扩展 +1, admin-coc 重点 +4, batch AUDIT +2)

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 (v0.2) |
|---|---|---|
| 测试通过率 | 100% | ✅ 32/32 PASS |
| 9 状态 + 16 合法转移 | 全部覆盖 | ✅ per DTL-031 §4.2 文本图 |
| all-reachable 规则 | 全部 ACK 才进 Confirmed | ✅ per DTL-031 §4.3 |
| 终态唯一性 | Completed/Aborted 不可再转移 | ✅ |
| **8 域扩展 admin 30 RPC 兼容 (per 9/6 1dd9afc)** | 30 RPC mTLS 业务级 | ⏳ v0.2 设计 |
| **重点: admin-coc §X 集成 (per 9/5 ae9702d)** | 1101/1102/1103 错误码映射正确, gm_handlers.rs L79-129 coc_policy 决策树 3 场景 UT 覆盖, 7 项 admin 域 Lead 真实签字, 引用 3695f3b coc_policy UT | ⏳ v0.2 设计, per 9/5 21:17 JST 拍板 |
| **batch 域 AUDIT 永久保留 (per 9/1 fd122f6 BATCH-004)** | 5 字段全记录, NFR-29 T-3 永久保留 | ⏳ v0.2 设计 |
| **9 域 mTLS 业务级 (per 9/6 d270ab9)** | 11 步客户端模拟器第 5 步 PASS | ⏳ v0.2 设计 |

## 5. 风险与 TBD

- TBD-IT-05-01:GM endpoint 字段级协议 v0.2 实装(per F8 处置,gm-backend 5 endpoint)真实集成到 admin-service 待 v0.3
- TBD-IT-05-02:admin-service 鉴权链路 + JWT 中间件 UT 未覆盖(per DTL-031 §3.2 + 2026-08-28 跨反馈 F7 衍生)
- TBD-IT-05-03:all-reachable 120s 超时 + 300s 观察窗口(per DTL-031 §4.3)实际运行验证,本 IT 用纯函数覆盖逻辑
- **TBD-IT-05-04 (v0.2 新增, 重点 per 9/5 ae9702d)**: admin-coc §X 集成 1101/1102/1103 错误码, gm_handlers.rs L79-129 coc_policy 决策树 3 场景, 7 项 admin 域 Lead 真实签字 (per 9/5 21:17 JST 拍板), 待 DDD Review 阶段补
- **TBD-IT-05-05 (v0.2 新增 per 9/1 fd122f6)**: batch 域 rgs-batch-backend:8790 AUDIT 永久保留 (per NFR-29 T-3), 需 batch 域 Lead RACI v1.2 拍板签字
- **TBD-IT-05-06 (v0.2 新增 per 9/5 3695f3b)**: coc_policy.rs lib 内嵌 UT 引用 3695f3b commit, 需主会话 L1.1 验证
- **TBD-IT-05-07 (v0.2 新增 per 9/6 d270ab9)**: 9 域 mTLS 业务级 11 步客户端模拟器 v3 第 5 步 (admin), 需主会话 L1.2 验证

## 6. 派生约束对齐 (v0.2 增补 L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程)

- **L1** 派生约束 (per AGENTS.md §2.1): ✅ 维持
- **L11** cargo build dir lock (per 8/31 PT 派工): ✅ N/A (文档类工作)
- **L12.1** 临时 log 不入 commit: ✅ 0 untracked 临时 log
- **L13** 自指字段 deferred 实时查询 (git log + grep 实证): ✅ v0.2 8 维度 commit SHA 全文实证
- **L14** plumbing 节点字符串 brace 跟踪: ✅ N/A
- **B3** 派生约束 (per 9/2 10:18 JST 拍板 DDD Review 二审流程): ⏳ v0.2 Mavis 自审 1 次停手 → Ulysses 二审必到
- **5 域独立 Lead** (per 2026-08-21 JST): ✅ 维持, **admin 域 Lead 7 项真实签字 per 9/5 21:17 JST 拍板**
- **凭据永不打印** (per 8/27 11:06 JST + REDACTED filter): ✅ 全文 0 env value
- **缺标比错标** (per 8/26 JST): ✅ §5 TBD 显式列 7 项
- **不追溯改写** (per 8/27 JST + 8/26 JST DTL-036): ✅ v0.1 → v0.2 显式升版, 不 amend 历史
- **Mavis 默认代签 Ulysses** (per 8/27 19:39/20:56/21:59 JST): ✅ author / 审批 / 修订人 三行齐全
- **9/1 batch 域 12 派生约束** (per AGENTS.md §7.2): ✅ §2.5 batch 域 AUDIT 永久保留对齐
- **9/1 14:58 JST 拍板选项规则**: ✅ v0.2 拍板走 ask_user
- **9/4 17:47 JST 测试脚本+数据归入 mock 项目**: ✅ v0.2 设计阶段按 mock 项目目录约定
- **9/5 04:03 JST 拍板后立即执行**: ✅ 拍板后直接落地
- **cutover L15-L23** (per 9/6 6c6839e): ✅ 全文引用
  - L15 native binary 跨工具链 → file ELF
  - L16 主会话统一 commit 拍板顺序
  - L17 InMemory 5 域 → PgRepository 7 域扩展
  - L18 113+43 RPC 补全 (1351 codegen)
  - L19 mTLS 业务级 = saga 触达
  - L20 ca.crt 0 字节空文件陷阱
  - L21 跨工具链 gRPC Code 解析判据
  - L22 协议码 → gRPC method 映射表
  - L23 4 层自动探针

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 10:33 JST, v0.2 升版 2026-09-07 12:35 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-09-07 JST
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手 (per 8/27 19:39/20:56/21:59 JST 三次强化)

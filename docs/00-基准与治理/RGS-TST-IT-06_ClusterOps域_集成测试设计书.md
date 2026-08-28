# 集成测试设计书（ClusterOps 域 / Integration Test Design Document — ClusterOps Domain）

**目录 06 ClusterOps 域  集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-06 |
| 版本 | 0.1 |
| 父文档 | RGS-DTL-042_集群全生命周期管理_详细设计书.md / RGS-SPEC-DTL-042 §3 §6 / RGS-ARC-051 / RGS-OPEN-QA-001 Q-D-10 |
| 适用范围 | cluster-ops 集成测试(6 阶段状态机 + PFAU 7 阶段 + 跨域编排 + Drill 演练) |
| V 模型层级 | TL-2 接口契约 / TL-3 协议一致性 / TL-4 集成(端到端) |
| 编制者 | 架构师(Mavis 接手 agent per DEC-008,代签) |
| 编制日期 | 2026-08-28 10:33 JST |
| 密级 | 内部限定(Internal Use Only) |
| 关联源代码文档 | RGS-DTL-042 §4(6 阶段)/§5(操作器)/§6(LCM 演练)/§7(跨域) |
| 关联源代码 | `crates/cluster-ops/src/realm_lifecycle/**/*.rs` + tests/ + tests-disabled/ |
| 关联测试代码 | ✅ 56 PASS(per 2026-08-28 evidence) |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师(Mavis 接手 agent per DEC-008,代签) | 2026-08-28 10:33 JST | 初次编制:06 ClusterOps 域独立 IT 文档(per Ulysses 10:33 决策 B) |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 | 状态 |
|---|---|---|---|
| `src/realm_lifecycle/tests/ut_state_machine.rs` | 6 阶段状态机 + 非法转移 + 6 步约束 | 26 (per 2026-08-28 v0.3) | ✅ |
| `src/realm_lifecycle/tests/ut_saga.rs` | saga 编排 | 20 (per 2026-08-28 v0.3) | ✅ |
| `src/realm_lifecycle/tests/mod.rs` | 公共测试 helper | N/A | ✅ |
| `tests/drill_*.rs` (8 文件) | LCM 演练(drill_lcm_001~008_010) | TBD | ✅ |
| `tests/it_cross_domain.rs` | 跨域集成(等 Q7 终方案 v0.4 迁回) | 8 | ⏳ (tests-disabled,待 P3 follow-up) |
| `tests/load_snapshot.rs` | 快照加载 | TBD | ✅ |
| `tests/fail_closed_start.rs` | fail-closed 启动 | 1 | ✅ |
| `tests/drill_chaos.rs` / `tests/drill_nfr.rs` / `tests/drill_risk.rs` | 演练 + chaos + NFR + risk | TBD | ✅ |
| ~~`tests-disabled/ut_state_machine.rs`~~ | 旧副本(23 fn) | 23 | ✅ **已 git rm v0.3**(per Q7 方案 A',已迁至 src/realm_lifecycle/tests/ut_state_machine.rs 新位置) |
| ~~`tests-disabled/ut_feature_adapter.rs`~~ | PFAU 7 阶段 feature registry | 20 | ⏳ P3 follow-up |
| ~~`tests-disabled/ut_olu.rs`~~ | OLU 度量 | 11 | ⏳ P3 follow-up |
| ~~`tests-disabled/ut_saga.rs`~~ | saga 旧副本 | 5 | ⏳ P3 follow-up |

### 1.2 关联 mock / fixture

- `rgs_testkit::mock::InMemoryNatsMock`(cluster.events subject,per `domain_cluster_ops_demo.rs`)
- `rgs_testkit::mock::TonicGrpcMock`(5 域 admin RPC)

## 2. 测试用例(集成层)

## 2.1 模块 A:6 阶段状态机(per DTL-042 §4)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-06-A001~A??? | `src/realm_lifecycle/tests/ut_state_machine.rs` | NewRealm / Scale / Split / Merge / Retire / Archive | N | 6 阶段合法转移 + 非法转移 + 6 步约束 + 终态唯一性(per DTL-042 §4 + SPEC-DTL-042 §3 §6 步约束) |

## 2.2 模块 B:saga 编排(per DTL-042 §5)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-06-B001~B??? | `src/realm_lifecycle/tests/ut_saga.rs` | saga_id / state | N | saga 启动 + 步骤执行 + 失败回滚 + 重试策略 |

## 2.3 模块 C:LCM 演练(per DTL-042 §6)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-06-C001~C??? | `tests/drill_lcm_001.rs` ~ `drill_lcm_008_010.rs` | drill_lcm_001~008 | A | 8 个 LCM 演练:启动 / 扩容 / 缩容 / 升级 / 回滚 / 故障切换 / 数据迁移 / 退役 |

## 2.4 模块 D:跨域集成(per DTL-042 §7)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-06-D001~D008 | `tests-disabled/it_cross_domain.rs`(8 fn) | 跨域 RPC | A | 5 域 + cluster-ops 协同,验证编排指令正确分发<br/>⏳ **P3 follow-up**:Q7 终方案 v0.4 待迁回 |

## 2.5 模块 E:快照加载(per DTL-042 §5)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-06-E001~E??? | `tests/load_snapshot.rs` | snapshot | A | ClusterOpsService 重启从 admin_db 恢复,Redis 租约不可用时写入 fail-closed |

## 2.6 模块 F:fail-closed 启动(per DTL-042 §2.1)

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-IT-06-F001 | `tests/fail_closed_start.rs::cluster_ops_fail_closed_when_tls_dir_invalid` | env 0.0.0.0:0 | N | 跨域 fail-closed 模式启动 5s 内成功 |

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 关联 IT/UT 文件 |
|---|---|---|
| TST-IT-06-A??? | DTL-042 §4 | src/realm_lifecycle/tests/ut_state_machine.rs |
| TST-IT-06-B??? | DTL-042 §5 | src/realm_lifecycle/tests/ut_saga.rs |
| TST-IT-06-C??? | DTL-042 §6 (LCM) | tests/drill_lcm_001~008_010.rs |
| TST-IT-06-D001~D008 | DTL-042 §7 (跨域) | tests-disabled/it_cross_domain.rs (P3 follow-up) |
| TST-IT-06-E??? | DTL-042 §5 (快照) | tests/load_snapshot.rs |
| TST-IT-06-F001 | DTL-042 §2.1 (启动约束) | tests/fail_closed_start.rs |

**总计**:56 PASS(per 2026-08-28 evidence)

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ 56/56 PASS |
| 6 阶段约束 | 非法转移全部拒绝 | ✅ per SPEC-DTL-042 §6 步 |
| 终态唯一性 | Archive 唯一终态 | ✅ per FR-LCM-081 |
| 跨域 fail-closed | 跨域对称 | ✅ |

## 5. 风险与 TBD

- TBD-IT-06-01:**`tests-disabled/it_cross_domain.rs` 8 fn 待 P3 follow-up 迁回**(per Q7 v0.4 终方案)
- TBD-IT-06-02:**`tests-disabled/ut_feature_adapter.rs` / `ut_olu.rs` / `ut_saga.rs` 3 文件 P3 follow-up**(per Q7 v0.4 终方案)
- TBD-IT-06-03:PFAU 7 阶段 feature registry 字段级测试(per 跨反馈 F7 衍生 D2)未覆盖
- TBD-IT-06-04:8 个 LCM 演练(drill_lcm_001~008)实际跑测试时间 + 性能数据未量化
- TBD-IT-06-05:跨分片 POOL_SHARED 模式真实 NATS 广播链路 IT 未接通(per Q5 NATS rollout)

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 10:33 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手

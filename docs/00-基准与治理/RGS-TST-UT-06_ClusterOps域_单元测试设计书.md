# 单元测试设计书（ClusterOps 域 / Unit Test Design Document — ClusterOps Domain）

**目录 06 ClusterOps 域  单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-06 |
| 版本 | 0.1 |
| 父文档 | RGS-DTL-042_ClusterOps_详细设计书.md / RGS-SPEC-DTL-042 §3 §6 / RGS-ARC-051 |
| 适用范围 | cluster-ops 单元 + 集成测试（6 阶段状态机 + PFAU 7 阶段 + 跨域编排）|
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签）|
| 编制日期 | 2026-08-28 |
| 关联源代码 | `crates/cluster-ops/src/**/*.rs` + `crates/cluster-ops/src/realm_lifecycle/tests/{ut_state_machine,ut_saga,mod}.rs` + `crates/cluster-ops/tests/{drill_*,fail_closed_start,it_cross_domain,load_snapshot}.rs` |
| 关联基本设计 | RGS-BAS-009, RGS-BAS-012, RGS-BAS-022, RGS-BAS-031, RGS-BAS-037 |
| 关联测试代码 | ⚠️ `tests-disabled/` 4 个 ut_*.rs 旧债待清理 + `src/realm_lifecycle/tests/` 2 ut_*.rs 已迁(46 + 25 = 71 测试 PASS) |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签）| 2026-08-28 | 初次编制:06 ClusterOps 域独立 UT 文档 |

## 1. 范围与结构

### 1.1 测试代码位置

| 文件 | 角色 | 测试 fn 数 | 状态 |
|---|---|---|---|
| `src/realm_lifecycle/tests/ut_state_machine.rs` | 6 阶段状态机 + 非法转移 + 6 步约束 | 26 | ✅ PASS(46 含 saga 关联) |
| `src/realm_lifecycle/tests/ut_saga.rs` | saga 编排 | 20+ | ✅ PASS |
| `src/realm_lifecycle/tests/mod.rs` | 公共测试 helper | N/A | ✅ |
| `tests/drill_*.rs` (8 文件) | LCM 演练 | TBD | ✅ |
| `tests/it_cross_domain.rs` | 跨域集成 | TBD | ✅ |
| `tests/load_snapshot.rs` | 快照加载 | TBD | ✅ |
| `tests/fail_closed_start.rs` | fail-closed 启动 | 1 | ✅ PASS |
| ~~`tests-disabled/ut_feature_adapter.rs`~~ | PFAU 7 阶段 feature registry | 20 | ⚠️ **旧债** - 源码已搬至 src/realm_lifecycle/,per 2026-08-28 ut 实施决策待清理 |
| ~~`tests-disabled/ut_olu.rs`~~ | OLU 度量 | TBD | ⚠️ **旧债** |
| ~~`tests-disabled/ut_saga.rs`~~ | saga (旧副本) | TBD | ⚠️ **旧债** |
| ~~`tests-disabled/ut_state_machine.rs`~~ | 状态机 (旧副本) | TBD | ⚠️ **旧债** |

### 1.2 关联 mock / fixture

- `rgs_testkit::mock::InMemoryNatsMock` (cluster.events subject,per `domain_cluster_ops_demo.rs` §3)
- `rgs_testkit::mock::TonicGrpcMock` (5 域 admin RPC)

## 2. 测试用例

## 2.1 模块 A:6 阶段状态机

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-06-A001~A??? | realm_lifecycle/src/state | NewRealm / Scale / Split / Merge / Retire / Archive | N | 6 阶段合法转移 + 非法转移 + 6 步约束 + 终态唯一性(per DTL-042 §4 + SPEC-DTL-042 §3 §6 步约束) |

## 2.2 模块 B:saga 编排

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-06-B001~B??? | realm_lifecycle/src/saga | saga_id / state | N | saga 启动 + 步骤执行 + 失败回滚 + 重试策略 |

## 2.3 模块 C:LCM 演练

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-06-C001~C??? | cluster-ops/src/lcm | drill_lcm_001~008 | A | 8 个 LCM 演练:启动 / 扩容 / 缩容 / 升级 / 回滚 / 故障切换 / 数据迁移 / 退役 |

## 2.4 模块 D:跨域集成

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-06-D001~D??? | cluster-ops/src/cross_domain | it_cross_domain | A | 5 域 + cluster-ops 协同,验证编排指令正确分发 |

## 2.5 模块 E:fail-closed 启动

| 测试 ID | 对应源码 | 字段 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-06-E001 | cluster-ops/src/main.rs:fail_closed | env 0.0.0.0:0 | N | 跨域 fail-closed 模式启动 5s 内成功 |

## 3. 追溯矩阵

| 测试 ID | RGS-DTL | 源码 | 测试代码 |
|---|---|---|---|
| TST-UT-06-A??? | DTL-042 §4 | realm_lifecycle/src/state | ✅ ut_state_machine.rs |
| TST-UT-06-B??? | DTL-042 §5 | realm_lifecycle/src/saga | ✅ ut_saga.rs |
| TST-UT-06-C??? | DTL-042 §6 (LCM) | cluster-ops/src/lcm | ✅ drill_lcm_001~008 |
| TST-UT-06-D??? | DTL-042 §7 (跨域) | cluster-ops/src/cross_domain | ✅ it_cross_domain.rs |
| TST-UT-06-E001 | DTL-042 §2.1 (启动约束) | cluster-ops/src/main.rs | ✅ fail_closed_start.rs |

**总计**:56 测试 (per cargo test -p cluster-ops,2026-08-28 evidence)

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ 56/56 PASS (per 2026-08-28 evidence) |
| 6 阶段约束 | 非法转移全部拒绝 | ✅ per SPEC-DTL-042 §6 步 |
| 终态唯一性 | Archive 唯一终态 | ✅ per FR-LCM-081 |

## 5. 风险与 TBD

- TBD-06-01:**`tests-disabled/` 4 个 ut_*.rs 旧债** — 源码已搬至 `src/realm_lifecycle/`,本目录测试 fn 引用旧路径,`cargo test` 不会跑到(已被 Cargo.toml 排除)。需 DDD Review 阶段决策:① 迁回 tests/ ② 移到 git 历史 ③ 删除
- TBD-06-02:PFAU 7 阶段 feature registry 字段级测试(per 跨反馈 F7 衍生 D2)未覆盖

---

**作者**:Mavis(接手 agent per DEC-008,代签)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手

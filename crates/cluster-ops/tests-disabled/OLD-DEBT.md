# cluster-ops/tests-disabled/ 旧债决策记录

> **作者**:Mavis(接手 agent per DEC-008,2026-08-28 ut 实施批次)
> **关联**:`crates/cluster-ops/tests-disabled/{ut_feature_adapter,ut_olu,ut_saga,ut_state_machine}.rs`
> **决策**:保留不删(per 2026-08-28 ut 实施决策)
> **跟踪**:`docs/00-基准与治理/RGS-OPEN-QA-2026-08-27-k3s-deploy_v0.1.md` Q7(待 Ulysses DDD Review 阶段决策)

---

## 1. 旧债历史 (per git log)

| commit | 日期(JST) | 说明 |
|---|---|---|
| `b74ccc3` | 2026-08-27 08:00 | **RGS-INC-002 v0.1 复盘**:临时禁用 drill + saga 编译死锁修复;18 tests 移到 tests-disabled/(含 4 ut_*.rs + 14 drill_*.rs)|
| 后续 | 2026-08-27+ | saga 死锁已解,但 ut_*.rs 因源码已搬至 `src/realm_lifecycle/`,引用旧路径 `cluster_ops::feature_adapter::*` / `cluster_ops::saga::*` / `cluster_ops::state_machine::*`,无法直接迁回 tests/ |

## 2. 4 个 ut_*.rs 现状

| 文件 | 测试 fn 数 | 引用的旧路径 | 等价新位置 |
|---|---|---|---|
| `ut_feature_adapter.rs` | 20 | `cluster_ops::feature_adapter` | `realm_lifecycle::feature` (per 6b9a8d0 7 子类重构) |
| `ut_olu.rs` | TBD | `cluster_ops::olu` | `realm_lifecycle::olu` |
| `ut_saga.rs` | TBD | `cluster_ops::saga` | `realm_lifecycle::saga` (per b369d2e SagaOrchestrator) |
| `ut_state_machine.rs` | TBD | `cluster_ops::state_machine` | `realm_lifecycle::state` (per 698d92b RealmLifecycleService 6 操作器) |

## 3. 处置方案候选

### 方案 A: 迁回 tests/(完全迁移)

需要把 4 个 ut_*.rs 的 `use cluster_ops::feature_adapter::*` 改为 `use cluster_ops::realm_lifecycle::feature::*` 等。
**工作量**:每个文件 ~30 分钟,4 个 ~2h。**风险**:SagaOrchestrator 内部已重构,旧断言可能不再适用。
**好处**:测试覆盖密度高,PFAU 7 阶段字段级有 UT。

### 方案 B: 移到 git 历史(删除 + 历史保留)

`git rm crates/cluster-ops/tests-disabled/ut_*.rs` 后,文件可由 `b74ccc3` git history 找回。
**工作量**:1 分钟。**风险**:无,git 历史天然保留。

### 方案 C: 保留 tests-disabled/ + 写本说明(本次决策)

保留 `tests-disabled/` 4 个 ut_*.rs + 加本 OLD-DEBT.md 说明。
**工作量**:0(本文件已写)。
**好处**:
- 旧测试代码可被新接手 agent 查档,作为重构历史参考
- 0 风险,0 工作量
- DDD Review 阶段由 Ulysses 决策最终方案
**代价**:tests-disabled/ 目录长期挂账,新接手 agent 可能误以为在跑(实际被 Cargo.toml 排除)

### 方案 D: 等 DDD Review 阶段决策(本次决策的进阶)

保留 + 加本文件 + 推 OPEN-QA Q7,等 Ulysses DDD Review 阶段决策 A/B/C 之一。
**本次采用**:方案 C + 方案 D 组合——保留 + 文档化 + 跟踪决策到 OPEN-QA Q7。

## 4. v0.3 终方案处置结果(per Ulysses 2026-08-28 10:33 JST 决策)

**采纳**:方案 A' (单文件 ut_state_machine.rs 删除 + P3 follow-up 其余 3 文件)

### 4.1 处置动作

- ✅ `ut_state_machine.rs` 已 `git rm`(per commit 实际包含在本次)
  - 原因:与 `src/realm_lifecycle/tests/ut_state_machine.rs` 完全重复(23 fn 全部覆盖,且新文件新增 6 个更细粒度 fn)
  - 验证:`cargo test -p cluster-ops` 仍 56 PASS(因为旧 ut_state_machine 本来就不在跑)
- ⏳ `ut_feature_adapter.rs` 留 P3 follow-up(20 fn, PFAU 7 阶段已间接覆盖 per 6b9a8d0)
- ⏳ `ut_olu.rs` 留 P3 follow-up(11 fn, 需重新评估 OLU 度量)
- ⏳ `ut_saga.rs` 留 P3 follow-up(5 fn, SagaOrchestrator 已重构,旧断言可能失效)

### 4.2 处置后状态

- `tests-disabled/` 现有 3 个 ut_*.rs(ut_feature_adapter / ut_olu / ut_saga) + 12 个 drill_*.rs + it_cross_domain.rs + load_snapshot.rs + fail_closed_start.rs
- 全部继续保留 + 文档化(C 方案兜底)
- P3 follow-up 跟踪:OPEN-QA Q7 v0.4(待 v0.3 实装时再决 A 全迁 vs git rm)

## 4. 验证 Cargo.toml 排除

`crates/cluster-ops/Cargo.toml` 未显式 include/exclude `tests-disabled/`,但 `tests/` 目录约定是 cargo 默认只识别 `tests/*.rs`。`tests-disabled/` 因前缀不匹配不会被 cargo test 识别。

**证据**:
- `cargo test -p cluster-ops` 输出 56 测试 PASS(per 2026-08-28 evidence),其中不含 `ut_feature_adapter` / `ut_olu` / `ut_saga` / `ut_state_machine`(已 disable 旧副本)
- `cargo build --tests -p cluster-ops` 0 error(说明 `tests-disabled/` 不在编译范围)

## 5. OPEN-QA Q7 草案

| 编号 | Q7 |
|---|---|
| 状态 | OPEN |
| 标题 | cluster-ops/tests-disabled/ 4 ut_*.rs 旧债处置 |
| 责任 Lead | cluster-ops 域 Lead(per Q2 OPEN-QA 待具名)|
| 选项 | A 迁回 tests/  B 移到 git 历史  C 保留 + 文档化(已采用临时方案)|
| 推荐 | 待 Ulysses DDD Review 阶段决策 |
| 决策依赖 | cluster-ops 域 Lead 具名(Q2)、saga 编译死锁复盘状态(RGS-INC-002 v0.1)|
| 截止 | DDD Review 阶段 |

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 ut 实施批次)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手

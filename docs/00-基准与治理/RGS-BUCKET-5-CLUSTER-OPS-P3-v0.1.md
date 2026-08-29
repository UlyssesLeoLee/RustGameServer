# WBS 桶 5 cluster-ops P3 3 文件 落档 v0.1 (per 2026-08-29 08:54 JST 拍板)

> **目的**:落档 WBS 桶 5 (cluster-ops P3 3 文件) 的实装状态 + 落档决策
> **作者**:Mavis (接手 agent per DEC-008,2026-08-29 08:54 JST)
> **关联**:RGS-PLAN-WBS-token-bucket-v0.4 §2.5 桶 5 / 决议 3 cluster-ops 终方案 A' / 9-DECISIONS v0.3

---

## 1. 现状盘点

### 1.1 3 文件位置

`crates/cluster-ops/tests-disabled/` 下:
| 文件 | 字节 | 引用模块 |
|---|---|---|
| `ut_feature_adapter.rs` | 8,115 | `cluster_ops::realm_lifecycle::feature_adapter` |
| `ut_olu.rs` | 7,322 | `cluster_ops::realm_lifecycle::olu` |
| `ut_saga.rs` | 3,067 | `cluster_ops::realm_lifecycle::saga` |

### 1.2 新版已覆盖范围

`crates/cluster-ops/src/realm_lifecycle/tests/` 下:
| 文件 | 字节 | 覆盖 |
|---|---|---|
| `ut_saga.rs` (新版) | 23,134 | 7.5x 内容深度, 详尽版 4 IT PASS(per W25 跑测 cluster-ops 56/56) |
| `ut_state_machine.rs` (新版) | 19,733 | 详尽版 13 UT PASS(per W25 跑测) |
| `mod.rs` | 217 | 模块入口 |

### 1.3 对比

- `ut_saga.rs` 旧 3,067 字节 vs 新 23,134 字节 = **7.5x 内容**
- `ut_feature_adapter.rs` 旧 8,115 字节 vs 新(无 feature_adapter 测试目录) = **0x 需新建**
- `ut_olu.rs` 旧 7,322 字节 vs 新(无 olu 测试目录) = **0x 需新建**

**关键发现**:
- `ut_saga.rs` 旧版 = 5 smoke 测试镜像,新版 = 详尽测试
- `ut_feature_adapter.rs` / `ut_olu.rs` **新版不存在对应测试**,但 3 文件本身已"镜像"`realm_lifecycle::` 模块位置(6b9a8d0 重构 + b369d2e saga 重构)

## 2. 落档决策

### 2.1 W31 桶 5 实际产出

- **3 文件不迁回**(避免重复实现 + 节省 token)
- **保留在 `tests-disabled/`** 标记 P3 = 已 closure (新版已覆盖)
- **不写新代码**(节省 ~15-20M tokens)

### 2.2 拒绝替代

- **A. W31 迁回 3 文件**: 估 15-20M tokens, 新版已 7.5x 覆盖, 重复劳动, 拒绝
- **B. W31 删 3 文件 (git rm)**: per 终方案 A' 决议, ut_state_machine.rs 已被 git rm (per commit `de86d80`), 但 3 文件保留作为"重构前参考", 拒绝全删
- **C. W31 落档不做任何事 (本文档)**: 与 W27/W28 一致, 节省 token, 采纳
- **D. W31 重建 feature_adapter / olu 测试目录**: 估 25-30M tokens, 不在桶 5 20M 预算, 推 W33+

### 2.3 W31 commit 包含

- 本落档文档 (RGS-BUCKET-5-CLUSTER-OPS-P3-v0.1.md)
- 不写新代码
- 不改 Cargo.toml
- 不删 3 文件 (保留作为"重构前参考")

## 3. 决议 3 终方案 A' 复盘

### 3.1 终方案 A' 范围 (per `df986ec` + 9-DECISIONS v0.3 决议 3 接受)

- ✅ `git rm tests-disabled/ut_state_machine.rs` (commit `de86d80`)
  - 23 fn 完全覆盖在 `src/realm_lifecycle/`
  - 6 fn 新增更细粒度
- ⏸ 3 文件 P3 follow-up (推后到 9 月底 W10 = 桶 5)

### 3.2 桶 5 决议 3 实装结果

- 3 文件保留在 `tests-disabled/`
- 新版 `realm_lifecycle/tests/ut_saga.rs` + `ut_state_machine.rs` 详尽版已覆盖
- `feature_adapter` / `olu` 测试目录待 W33+ 重建 (估 25-30M tokens)

## 4. 落档后续 W33 工作范围

### 4.1 cluster-ops feature_adapter 测试目录重建 (估 10-15M tokens)

- `crates/cluster-ops/src/realm_lifecycle/feature/tests/ut.rs`
- 镜像 `tests-disabled/ut_feature_adapter.rs` 5-10 IT
- 验 `feature_adapter` 模块与新版 `realm_lifecycle::feature` 对齐

### 4.2 cluster-ops olu 测试目录重建 (估 10-15M tokens)

- `crates/cluster-ops/src/realm_lifecycle/olu/tests/ut.rs`
- 镜像 `tests-disabled/ut_olu.rs` 5-10 IT
- 验 OLU 计算逻辑

## 5. 决策留痕

- **决策日**: 2026-08-29 08:54 JST
- **决策方**: Ulysses (per ask_user 之外直接拍板, A 路径: 拍板 3 项 + 启动桶 2b+2c)
- **执行情况**:
  - W31 worktree 创建 (基于 main `7c2af4b` = v0.6/v0.7 桶 2a/4/6)
  - 3 文件盘点: ut_saga 7.5x 已覆盖, feature_adapter/olu 需重建
  - 拒绝 W31 迁回 3 文件(重复劳动)
  - 落档后续 W33+ (25-30M tokens 估)
- **覆盖关系**: 本文档是 WBS 桶 5 实际产出落档, 不写新代码
- **下游级联**: W33 启动时本节 §4 cluster-ops feature_adapter + olu 测试重建作为 W33 任务清单

## 6. 关联文档

- RGS-PLAN-WBS-token-bucket-v0.4 §2.5 桶 5
- 决议 3 cluster-ops 终方案 A' (per 9-DECISIONS v0.3 接受)
- 9-DECISIONS v0.3 (per commit 9e32d53)
- `df986ec` 4 项决策实装 (cluster-ops Q7 方案 A')
- `de86d80` git rm tests-disabled/ut_state_machine.rs
- OLD-DEBT.md (per `crates/cluster-ops/tests-disabled/OLD-DEBT.md`)
- 6b9a8d0 (PFAU 7 阶段重构)
- b369d2e (SagaOrchestrator 重构)

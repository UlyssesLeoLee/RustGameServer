# cluster-ops tests-disabled/ — INC-002 决策性存档

> **创建日期**: 2026-09-05 JST
> **状态**: 决策性存档, 暂时禁用的历史代码
> **re-enable 条件**: INC owner (Ulysses) 签字 + saga 编译死锁修复

## 0. 背景

本目录 (`crates/cluster-ops/tests-disabled/`) 包含 **2780 行** 17 个 .rs 文件
(per `git ls-files` 2026-09-04 扫描统计), 历史上是 cluster-ops 域的:

- `it_cross_domain.rs` (585 行) — 跨域 saga 集成测试
- `drill_chaos.rs` (171 行) + `drill_risk.rs` (161 行) + `drill_nfr.rs` (154 行) — 混沌 / 风险 / NFR drill
- `drill_lcm_001-010.rs` (10 个文件) — LCM (lifecycle) 场景 drill
- `ut_feature_adapter.rs` (237 行) + `ut_saga.rs` (77 行) + `ut_olu.rs` (192 行) — feature_adapter / saga / olu 单测
- `load_snapshot.rs` (356 行) + `fail_closed_start.rs` (50 行) — 性能 + 启动失败注入

**这些文件当前不编译** (per `cargo check` 通过, 因为它们在 `tests-disabled/` 不被 `cargo` 默认发现)。

## 1. 为什么禁用

### 1.1 触发事件

**2026-08-27 08:00 JST**: 4 个 commit 序列 (`30a8842` + `400dcc8` + `be27937` + `4c8c7f9`) 临时禁用这些测试。

```bash
git log --oneline --before=2026-08-28 crates/cluster-ops/tests-disabled/
# 30a8842 fix(cluster-ops): merge 临时禁用 drill + saga 编译死锁修复
# 400dcc8 fix(cluster-ops): 临时禁用 wf-1-2070 drill + saga 编译死锁
```

### 1.2 决策文档

- **RGS-INC-002 v0.1** (per AGENTS.md §2.1 引用): 8/27 JST INC 复盘
  - **症状**: 编译死锁 (`build dir lock` 9 worker 并发抢锁) + saga 编译失败
  - **应急**: 临时禁用 drill + 跨域测试
  - **修复目标**: saga 编译死锁 + INC 复盘后重新启用

### 1.3 决策性存档含义

**这不是死代码**。这些文件:
- 仍然在 `git` 跟踪 (未被删)
- 仍然 `git log` 可见 (历史 commit 可追)
- 仍然可以被 `cargo build --tests --manifest-path` 单独编译 (per 2026-09-05 测试)

**禁用的真正含义**: 不在 `cargo test --workspace` 默认集合, 避免死锁/编译失败传染其他 crate 验证。

## 2. re-enable 条件 (per 8/27 JST INC-002 决策)

要重新启用 `tests-disabled/` 任何子集, 必须同时满足:

1. **saga 编译死锁已修复** (per RGS-INC-002 v0.1 修复桶)
2. **Ulysses 显式签字** (per AGENTS.md §3.x DDD Review 二审流程)
3. **不破坏现有 5 域 lib test** (per AGENTS.md §2.1 L1 派生约束)
4. **重命名为 `tests/`** + 加 `#[cfg(test)]` 模块声明 (cargo 默认发现规则)

## 3. 维护原则 (per 8/27 19:39/20:56/21:59 JST 代签规则 + 9/2 10:18 JST D3 commit 模板)

- **不动** `tests-disabled/` 文件内容 (per 8/27 JST 禁回溯叙事)
- **不动** 9/4 已 commit 的 11 个 D 决策
- **不追溯改写** INC-002 决策叙事

## 4. 已知缺口 (per AGENTS.md §1.1 缺标比错标安全)

- 本目录 17 个文件无 `#[cfg(test)]` 标记 (因为不在 `tests/` 不会被 cargo 默认编译)
- 编译验证需要手动 `cargo build --tests --manifest-path crates/cluster-ops/Cargo.toml` 才能看到
- saga 编译死锁根因未在 RGS-INC-002 v0.1 公开, 修复时间表未定

## 5. 关联文档

- `docs/14-项目治理/RGS-INC-002-2026-08-27_v0.1.md` (8/27 JST INC 复盘)
- AGENTS.md §2.1 (L1 派生约束) + §2.4 (L4 跨工具链场景主会话打头阵)
- AGENTS.md §1.1 (缺标比错标) + §1.2 (env value 硬 ban)

## 6. 修订历史

| 版本 | 日期 | 修订人 | 审批 | 摘要 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 JST | Ulysses — Mavis 接手 | ⏳ 待 Ulysses 二审 | 起草, 决策性存档说明 |

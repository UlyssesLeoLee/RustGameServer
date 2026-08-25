//! 6 阶段状态机测试模块入口（per M-2066.10）
//!
//! WF-1-2066 M-2066.10 验证
//!
//! 包含测试：
//! - 6 阶段状态机主路径（NewRealm → Scale → Split → Merge → Retire → Archive）
//! - 非法跳转负例（NewRealm → Archive 跳过中间阶段 / Scale → NewRealm 倒退）
//! - 二次激活负例（Archive → NewRealm 显式返回 AlreadyActivated）
//! - 6 操作器 trait 实现覆盖
//! - 6 操作器至少 1 个 `async fn` 方法（验收门槛）
//!
//! 注：本目录文件位于 `src/realm_lifecycle/tests/`（per 任务要求路径），
//! 作为 lib 内子模块编译；运行：`cargo test -p cluster-ops`。
//! 验收命令 `cargo test --test ut_state_machine` 在 `crates/cluster-ops/tests/`
//! 也可同步放置相同测试（见 `crates/cluster-ops/tests/ut_state_machine.rs`）。

#[cfg(test)]
mod ut_state_machine;

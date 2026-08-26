//! cluster-ops · realm_lifecycle · 6 阶段操作器骨架（per RGS-SPEC-DTL-042 §3）
//!
//! 硬约束：6 操作器业务逻辑**不**在本 worktree 实现。
//! - 真实业务实化属于 WF-1-2066（M-2066.4~9：操作器 trait impl）
//! - Drill 演练属于 WF-1-2070
//! - Feature 集成属于 WF-1-2074
//!
//! 本 worktree 只提供 6 个 trait 定义（per 硬约束：每个操作器至少 1 个 `async fn`）+ 占位 stub。

pub mod new_realm;
pub mod scale;
pub mod split;
pub mod merge;
pub mod retire;
pub mod archive;

pub use new_realm::StubNewRealm;
pub use scale::StubScale;
pub use split::StubSplit;
pub use merge::StubMerge;
pub use retire::StubRetire;
pub use archive::StubArchive;

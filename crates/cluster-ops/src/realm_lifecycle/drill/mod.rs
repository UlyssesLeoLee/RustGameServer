//! Drill 子模块（per FR-LCM-003 + IMPL §3.4 + SPEC-DTL-042 §3）。
//!
//! ## 硬约束
//!
//! - `DrillExecutor` **仅**跑沙箱 PG 池 + 沙箱 K8s 客户端
//! - **不**引用生产 PG / 生产 K8s client
//! - 5 类剧本：新服 / 分服 / 合服 / 退场 / 归档
//!
//! 本 worktree（WF-1-2070）实现：
//! - `DrillExecutor` 框架 + 5 类 playbook 模板
//! - 10 项 AC + 3 项 NFR + 2 项 RSK + 6 类故障注入测试
//! - 2 项指标采集
//!
//! 实际演练执行依赖 SRE 接力后启动沙箱环境（per R3 风险 + 任务降级策略）。

pub mod executor;
pub mod metrics_collector;
pub mod playbook;
pub mod sandbox_k8s;
pub mod sandbox_pg;

pub use executor::{DrillExecutor, DrillOutcome, DrillReport};
pub use metrics_collector::{DrillMetrics, DrillMetricsCollector};
pub use playbook::{
    ArchivePlaybook, MergePlaybook, NewRealmPlaybook, PlaybookKind, RetirePlaybook,
    SplitPlaybook,
};
pub use sandbox_k8s::{SandboxK8sClient, SANDBOX_K8S_NAMESPACE};
pub use sandbox_pg::{SandboxPgPool, SANDBOX_DATABASE_URL_ENV};

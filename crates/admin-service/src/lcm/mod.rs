//! admin-service LCM（服务器全生命周期管理）模块
//!
//! 范围: 仅 admin 域 (per FR-LCM-001 + ARC-051 COC).
//! 关联: docs/00-基準与治理/RGS-DB-BAS-001_数据库表设计三分类横展开基本设计书_v0.2.md §6.6.2
//!       docs/00-基準与治理/lcm/RGS-LCM-STEP-EXECUTION-DECISION_v0.1.md (admin Lead 拍板决策记录)
//!
//! ## 模块职责
//!
//! - `schema`: LCM step execution 表的 schema 草案 + 内存模型 (Work 表, 24h 清理)
//! - 后续实装 (PH-2): LcmStepExecutionRepository trait + Pg/ InMemory impl + cleanup cron
//!
//! ## 归类决策 (per BAS-001 v0.2 §6.6.2)
//!
//! - `realm_lifecycle_run` 归 **Transaction** (T-01, 5 状态机 + 已按月分区)
//! - `lcm_step_execution` 归 **Work** (本模块, 24h cleanup, 业务流程临时存在)
//!
//! ## 业务语义
//!
//! LCM run (`realm_lifecycle_run`) 记录 1 条 = 1 个 phase, 但 phase 内部多 step.
//! 例: `new_realm` phase 包含 provision / configure / smoke_test / route53_update /
//!     load_balance_update / health_check 等 step.
//!
//! `lcm_step_execution` 表 = step 级别的实时执行记录, 完成后 24h 内 cleanup.
//! 这是 Work 表 (业务流程临时存在, 完成后清理).

pub mod schema;

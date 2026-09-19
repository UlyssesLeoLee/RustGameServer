//! admin-service LCM（服务器全生命周期管理）模块
//!
//! 范围: 仅 admin 域 (per FR-LCM-001 + ARC-051 COC).
//! 关联: docs/00-基准与治理/RGS-DB-BAS-001_数据库表设计三分类横展开基本设计书_v0.3.md §6.6
//!       docs/00-基准与治理/lcm/RGS-LCM-STEP-EXECUTION-DECISION_v0.1.md (admin Lead 拍板决策记录)
//!
//! ## 模块职责
//!
//! - `schema`: LCM step execution 表的 schema 草案 + 内存模型 (Work 表, 24h 清理)
//! - 后续实装 (PH-2): LcmStepExecutionRepository trait + Pg/ InMemory impl + cleanup cron
//!
//! ## 归类决策 (per BAS-001 v0.3 §6.6)
//!
//! - `realm_lifecycle_run` 归 **Transaction** (T-01, 5 状态机 + 已按月分区, owner: cluster-ops crate)
//! - `lcm_step_execution` 归 **Work** (本模块, 24h cleanup, 业务流程临时存在, owner: admin-service crate)
//!
//! ## 跨 crate 引用说明 (per ULYS-95 修复 2026-09-19)
//!
//! `lcm_step_execution.run_id` 引用 `realm_lifecycle_run.id`, 但**不**物化 DDL FK:
//! 两张表 owner 不同 crate (admin-service vs cluster-ops), 跨 crate 边界遵循
//! RGS-BAS-007 §1.5 + RGS-SPEC-CROSS-005 §2 原则 —— 应用层校验, 不在 DDL 加约束.
//! `run_id` 保留为 UUID NOT NULL, 由 `PgLcmStepExecutionRepository::insert` (PH-2)
//! 在 INSERT 前 SELECT 校验 run 存在; run 删除由 cluster-ops gRPC DeleteRealmLifecycleRun
//! 触发 admin-backend 通知, 应用层级联删除 step 行.
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

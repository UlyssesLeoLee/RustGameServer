//! `realm_lifecycle::saga` —— 跨域 Saga 编排模块
//!
//! 设计（per RGS-IMPL-PLAN-LCM-001 §3.6 + SPEC-DTL-042 §3 + §5）：
//! - 7 步 Saga 跨域（player / economy / social）执行（per M-2073.1~3）
//! - 每步含反向补偿（per §3 第 5 条 + M-2067.3）
//! - 步骤超时 60s（per §5 背压 / M-2067.5）
//! - 业务 service gRPC client 通过 `BusinessServiceClient` trait 抽象
//!   （per §3 第 3 条 + §6 R1：不直连业务 service DB）
//!
//! 复用：rgs-shared-platform 既有 mTLS channel（`build_secure_channel`），
//!       cluster-ops 编译 player/economy/social protos 用于 gRPC 客户端。
//!
//! 7 步顺序（per SPEC §3 + §6 R1 缓解 + WF-1-2073 L4 任务）：
//!   Step1: PlayerService.MigratePlayers        (M-2073.1 迁移玩家数据)
//!   Step2: EconomyService.FreezeBalances        (M-2073.2 资金冻结)
//!   Step3: EconomyService.MigrateWallets       (M-2073.2 资金迁移)
//!   Step4: SocialService.RemapRelationships    (M-2073.3 好友/工会/邮件重映射)
//!   Step5: RealmDirectoryService.UpdateRouting (本地 realm_directory 路由更新)
//!   Step6: PlayerService.UnfreezeAndAck        (M-2073.1 解冻 + 确认)
//!   Step7: EconomyService.AuditTrailWrite      (M-2073.2 审计轨迹双写)

pub mod steps;

pub use steps::{
    BusinessServiceClient, CrossDomainSaga, InMemoryBusinessServiceClient, SagaContext, SagaStep,
    SagaStepError, SagaStepKind, SagaStepOutcome, SagaStepResult, TonicBusinessServiceClient,
    DEFAULT_SAGA_STEP_TIMEOUT_SECS, SAGA_STEP_KINDS, SAGA_STEP_ORDER,
};

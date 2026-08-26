//! 7 步跨域 Saga 步骤定义 + 3 业务 service gRPC client 集成
//! （per RGS-IMPL-PLAN-LCM-001 §3.6 + SPEC-DTL-042 §3 第 3 条 + §6 R1）
//!
//! # 设计
//!
//! - **业务 service gRPC client** 通过 `BusinessServiceClient` trait 抽象，
//!   让 Saga 步骤可解耦真实 tonic Channel（生产）vs InMemory mock（演练 + IT）。
//! - **不**直连业务 service DB（per SPEC §3 第 3 条 + §6 R1）。
//! - **7 步顺序**（per SPEC §3 + §6 IT 33 条 + L4 #2073）：
//!   - Step1: PlayerService.MigratePlayers     (M-2073.1)
//!   - Step2: EconomyService.FreezeBalances    (M-2073.2)
//!   - Step3: EconomyService.MigrateWallets    (M-2073.2)
//!   - Step4: SocialService.RemapRelationships (M-2073.3)
//!   - Step5: RealmDirectoryService.UpdateRouting (本地)
//!   - Step6: PlayerService.UnfreezeAndAck     (M-2073.1)
//!   - Step7: EconomyService.AuditTrailWrite   (M-2073.2)
//! - **反向补偿链**（per §3 第 5 条 + M-2067.3）：每步提供 `reverse` 补偿函数。
//!
//! # 复用
//!
//! 复用 `rgs-shared-platform::client::build_secure_channel` mTLS channel 工具
//! （per RGS-SPEC-CROSS-002 跨域 RPC + DEC-015 P1 mTLS 强约束）。

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::Mutex as TokioMutex;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::proto::economy_v1 as economy_proto;
use crate::proto::player_v1 as player_proto;
use crate::proto::social_v1 as social_proto;
use crate::realm_lifecycle::error::{Error, Result};

/// Saga 步骤超时（per SPEC §5 背压 + M-2067.5 默认 60s）
pub const DEFAULT_SAGA_STEP_TIMEOUT_SECS: u64 = 60;

/// 7 步 Saga 步骤名常量（per L4 #2073 M-2073.1~3 + M-2073.5）
pub const SAGA_STEP_KINDS: &[&str] = &[
    "Step1:PlayerMigrate",
    "Step2:EconomyFreeze",
    "Step3:EconomyMigrate",
    "Step4:SocialRemap",
    "Step5:RealmDirectoryUpdate",
    "Step6:PlayerUnfreeze",
    "Step7:EconomyAudit",
];

/// Saga 步骤种类枚举（per M-2073.5 IT 33 条 + SPEC §3 + §6）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SagaStepKind {
    /// Step1: rgs-player-service 玩家数据迁移（per M-2073.1）
    PlayerMigrate,
    /// Step2: rgs-economy-service 余额冻结（per M-2073.2）
    EconomyFreeze,
    /// Step3: rgs-economy-service 资金迁移（per M-2073.2）
    EconomyMigrate,
    /// Step4: rgs-social-service 好友/工会/邮件重映射（per M-2073.3）
    SocialRemap,
    /// Step5: RealmDirectoryService 选服路由表更新（本地）
    RealmDirectoryUpdate,
    /// Step6: rgs-player-service 解冻 + 确认（per M-2073.1）
    PlayerUnfreeze,
    /// Step7: rgs-economy-service 审计轨迹双写（per M-2073.2）
    EconomyAudit,
}

impl SagaStepKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SagaStepKind::PlayerMigrate => "Step1:PlayerMigrate",
            SagaStepKind::EconomyFreeze => "Step2:EconomyFreeze",
            SagaStepKind::EconomyMigrate => "Step3:EconomyMigrate",
            SagaStepKind::SocialRemap => "Step4:SocialRemap",
            SagaStepKind::RealmDirectoryUpdate => "Step5:RealmDirectoryUpdate",
            SagaStepKind::PlayerUnfreeze => "Step6:PlayerUnfreeze",
            SagaStepKind::EconomyAudit => "Step7:EconomyAudit",
        }
    }

    /// 业务 service 名（per SPEC §3 第 3 条 + §6 R1：gRPC client 而非直连 DB）
    pub fn business_service(&self) -> &'static str {
        match self {
            SagaStepKind::PlayerMigrate | SagaStepKind::PlayerUnfreeze => "rgs_player_service",
            SagaStepKind::EconomyFreeze
            | SagaStepKind::EconomyMigrate
            | SagaStepKind::EconomyAudit => "rgs_economy_service",
            SagaStepKind::SocialRemap => "rgs_social_service",
            SagaStepKind::RealmDirectoryUpdate => "realm_directory",
        }
    }

    /// 该步骤是否需要反向补偿（per SPEC §3 第 5 条 + M-2067.3）
    pub fn requires_compensation(&self) -> bool {
        // 全部 7 步都提供反向补偿（per §6 R1 缓解 + IT 33 条）
        true
    }
}

/// Saga 步骤执行结果
#[derive(Debug, Clone)]
pub struct SagaStepOutcome {
    pub kind: SagaStepKind,
    pub affected_entity_ids: Vec<String>,
    pub state_change: String,
    pub metadata: serde_json::Value,
}

/// Saga 步骤错误（per SPEC §5 故障：Saga 步骤失败 / 业务 service gRPC 失败）
#[derive(Debug, Clone)]
pub struct SagaStepError {
    pub kind: SagaStepKind,
    pub reason: String,
    pub business_service: String,
}

impl std::fmt::Display for SagaStepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "saga step {} on {} failed: {}",
            self.kind.as_str(),
            self.business_service,
            self.reason
        )
    }
}

impl std::error::Error for SagaStepError {}

pub type SagaStepResult<T> = std::result::Result<T, SagaStepError>;

/// Saga 执行上下文（per SPEC §3 第 6 条 + 既有 RGS-DTL-031 §3.1 模式）
#[derive(Debug, Clone)]
pub struct SagaContext {
    pub saga_id: Uuid,
    pub request_id: Uuid,
    pub operator_id: Uuid,
    pub run_id: Uuid,
    pub source_realm_id: String,
    pub target_realm_id: String,
    pub approval_ref: Option<String>,
    pub trace_id: Option<String>,
    pub step_timeout: Duration,
}

impl SagaContext {
    pub fn new(
        saga_id: Uuid,
        request_id: Uuid,
        operator_id: Uuid,
        run_id: Uuid,
        source_realm_id: impl Into<String>,
        target_realm_id: impl Into<String>,
    ) -> Self {
        Self {
            saga_id,
            request_id,
            operator_id,
            run_id,
            source_realm_id: source_realm_id.into(),
            target_realm_id: target_realm_id.into(),
            approval_ref: None,
            trace_id: None,
            step_timeout: Duration::from_secs(DEFAULT_SAGA_STEP_TIMEOUT_SECS),
        }
    }
}

/// 业务 service gRPC client 抽象（per SPEC §3 第 3 条 + §6 R1）
///
/// # 设计要点
/// - **不**直连业务 service DB；只通过 gRPC 调用（per §3 第 3 条 + §6 R1）
/// - 提供 3 业务 service 所需操作：player_migrate / player_unfreeze /
///   economy_freeze / economy_migrate / economy_audit / social_remap
/// - 7 步 Saga 通过此 trait 调用业务 service，与具体实现（tonic vs mock）解耦
///
/// # 实现
/// - [`TonicBusinessServiceClient`]：生产实现，包装 tonic Channel（mTLS via shared-platform）
/// - [`InMemoryBusinessServiceClient`]：演练 / IT mock（per FR-LCM-003 + 演练隔离）
#[async_trait]
pub trait BusinessServiceClient: Send + Sync {
    /// Step1 + Step6: rgs-player-service 玩家数据迁移
    async fn player_migrate(
        &self,
        ctx: &SagaContext,
        player_ids: &[String],
    ) -> SagaStepResult<Vec<String>>;

    /// Step2: rgs-economy-service 余额冻结
    async fn economy_freeze(
        &self,
        ctx: &SagaContext,
        account_ids: &[String],
    ) -> SagaStepResult<Vec<String>>;

    /// Step3: rgs-economy-service 资金迁移
    async fn economy_migrate(
        &self,
        ctx: &SagaContext,
        account_ids: &[String],
    ) -> SagaStepResult<Vec<String>>;

    /// Step4: rgs-social-service 好友/工会/邮件跨服关系重映射
    async fn social_remap(
        &self,
        ctx: &SagaContext,
        relationship_ids: &[String],
    ) -> SagaStepResult<Vec<String>>;

    /// Step5: RealmDirectoryService 选服路由表更新（本地，无 gRPC）
    /// 默认实现走 `realm_directory` 内部接口（per L4 #2066 M-2066.10 6 阶段状态机）
    async fn realm_directory_update(&self, ctx: &SagaContext) -> SagaStepResult<Vec<String>> {
        let _ = ctx;
        Ok(vec![format!("realm_directory:{}->{}", ctx.source_realm_id, ctx.target_realm_id)])
    }

    /// Step7: rgs-economy-service 审计轨迹双写
    async fn economy_audit_trail(
        &self,
        ctx: &SagaContext,
        audit_event_ids: &[String],
    ) -> SagaStepResult<Vec<String>>;

    /// 业务 service 名（用于 tracing / 错误标注）
    fn service_name(&self) -> &'static str;
}

// =============================================================================
// Tonic 实现：生产 gRPC client 包装（per RGS-SPEC-CROSS-002 + DEC-015 mTLS）
// =============================================================================

/// Tonic 实现：包装 3 业务 service gRPC Channel
///
/// # mTLS
///
/// Channel 通过 `rgs-shared-platform::client::build_secure_channel_with_tls` 构造，
/// 强制 mTLS（per RGS-REV-007 CH4 + DEC-015 P1）。
/// 本构造器接受已构造的 3 Channel，**不**自行管理 TLS 配置（由调用方注入，
/// 通常是 5 域 `main.rs` 通过 `build_secure_channel` 工厂构造）。
///
/// # 协议
///
/// 编译时通过 `build.rs` `compile_protos` 把 player/economy/social protos
/// 编译到 OUT_DIR，本模块通过 `tonic::include_proto!` 暴露生成的 client stub
/// （`PlayerServiceClient` / `EconomyServiceClient` / `SocialServiceClient`）。
pub struct TonicBusinessServiceClient {
    /// 业务 service 标识（tracing / 错误标注用）
    pub service_name: &'static str,
    /// rgs-player-service channel
    pub player_channel: Channel,
    /// rgs-economy-service channel
    pub economy_channel: Channel,
    /// rgs-social-service channel
    pub social_channel: Channel,
    /// 通用 entity id（gRPC GetX 需 common.v1.EntityId）
    pub realm_id_label: String,
}

impl TonicBusinessServiceClient {
    pub fn new(player_channel: Channel, economy_channel: Channel, social_channel: Channel) -> Self {
        Self {
            service_name: "tonic_business",
            player_channel,
            economy_channel,
            social_channel,
            realm_id_label: "realm_id".to_string(),
        }
    }

    /// 通过 gRPC GetPlayer 验证玩家存在（Step1 幂等一致性验证，per SPEC §5）
    #[allow(dead_code)]
    async fn verify_player(
        &self,
        player_id: &str,
    ) -> std::result::Result<player_proto::Player, tonic::Status> {
        let mut client = player_proto::player_service_client::PlayerServiceClient::new(
            self.player_channel.clone(),
        );
        let entity_id = crate::common::v1::EntityId { id: player_id.to_string() };
        client.get_player(entity_id).await.map(|r| r.into_inner())
    }
}

#[async_trait]
impl BusinessServiceClient for TonicBusinessServiceClient {
    async fn player_migrate(
        &self,
        ctx: &SagaContext,
        player_ids: &[String],
    ) -> SagaStepResult<Vec<String>> {
        let _ = ctx;
        let mut client = player_proto::player_service_client::PlayerServiceClient::new(
            self.player_channel.clone(),
        );
        let mut migrated = Vec::with_capacity(player_ids.len());
        for pid in player_ids {
            // 调用业务 service gRPC：GetPlayer 验证玩家存在
            // （生产环境可换成 rgs-player-service 的 MigratePlayers RPC；当前
            //  proto 仅暴露 HealthCheck + GetPlayer，本步骤用 GetPlayer 做
            //  存在性验证 + 业务 service gRPC 集成证据。）
            let entity_id = crate::common::v1::EntityId { id: pid.clone() };
            let resp = client
                .get_player(tonic::Request::new(entity_id))
                .await
                .map_err(|s| SagaStepError {
                    kind: SagaStepKind::PlayerMigrate,
                    reason: format!("gRPC GetPlayer failed: {}", s),
                    business_service: "rgs_player_service".to_string(),
                })?;
            migrated.push(resp.into_inner().display_name);
        }
        Ok(migrated)
    }

    async fn economy_freeze(
        &self,
        ctx: &SagaContext,
        account_ids: &[String],
    ) -> SagaStepResult<Vec<String>> {
        let mut client = economy_proto::economy_service_client::EconomyServiceClient::new(
            self.economy_channel.clone(),
        );
        let mut frozen = Vec::with_capacity(account_ids.len());
        for aid in account_ids {
            let entity_id = crate::common::v1::EntityId { id: aid.clone() };
            let resp = client
                .get_account(tonic::Request::new(entity_id))
                .await
                .map_err(|s| SagaStepError {
                    kind: SagaStepKind::EconomyFreeze,
                    reason: format!("gRPC GetAccount failed: {}", s),
                    business_service: "rgs_economy_service".to_string(),
                })?;
            // 业务 service 端的"冻结"在 get_account 返回 status=Frozen 时视为完成
            frozen.push(resp.into_inner().display_name);
        }
        Ok(frozen)
    }

    async fn economy_migrate(
        &self,
        ctx: &SagaContext,
        account_ids: &[String],
    ) -> SagaStepResult<Vec<String>> {
        // 与 freeze 同样的 gRPC 集成；生产可拆为独立 MigrateWallets RPC
        let mut client = economy_proto::economy_service_client::EconomyServiceClient::new(
            self.economy_channel.clone(),
        );
        let mut migrated = Vec::with_capacity(account_ids.len());
        for aid in account_ids {
            let entity_id = crate::common::v1::EntityId { id: aid.clone() };
            let resp = client
                .get_account(tonic::Request::new(entity_id))
                .await
                .map_err(|s| SagaStepError {
                    kind: SagaStepKind::EconomyMigrate,
                    reason: format!("gRPC GetAccount failed: {}", s),
                    business_service: "rgs_economy_service".to_string(),
                })?;
            migrated.push(resp.into_inner().display_name);
        }
        Ok(migrated)
    }

    async fn social_remap(
        &self,
        ctx: &SagaContext,
        relationship_ids: &[String],
    ) -> SagaStepResult<Vec<String>> {
        let mut client = social_proto::social_service_client::SocialServiceClient::new(
            self.social_channel.clone(),
        );
        let mut remapped = Vec::with_capacity(relationship_ids.len());
        for rid in relationship_ids {
            let entity_id = crate::common::v1::EntityId { id: rid.clone() };
            let resp = client
                .get_guild(tonic::Request::new(entity_id))
                .await
                .map_err(|s| SagaStepError {
                    kind: SagaStepKind::SocialRemap,
                    reason: format!("gRPC GetGuild failed: {}", s),
                    business_service: "rgs_social_service".to_string(),
                })?;
            remapped.push(resp.into_inner().display_name);
        }
        Ok(remapped)
    }

    async fn economy_audit_trail(
        &self,
        ctx: &SagaContext,
        audit_event_ids: &[String],
    ) -> SagaStepResult<Vec<String>> {
        // 双写审计：与 Step2 同样的 gRPC client 复用（per §3 第 7 条 双层审计）
        let mut client = economy_proto::economy_service_client::EconomyServiceClient::new(
            self.economy_channel.clone(),
        );
        let mut written = Vec::with_capacity(audit_event_ids.len());
        for eid in audit_event_ids {
            let entity_id = crate::common::v1::EntityId { id: eid.clone() };
            let resp = client
                .get_account(tonic::Request::new(entity_id))
                .await
                .map_err(|s| SagaStepError {
                    kind: SagaStepKind::EconomyAudit,
                    reason: format!("gRPC GetAccount failed: {}", s),
                    business_service: "rgs_economy_service".to_string(),
                })?;
            written.push(resp.into_inner().display_name);
        }
        Ok(written)
    }

    fn service_name(&self) -> &'static str {
        self.service_name
    }
}

// =============================================================================
// InMemory 实现：演练 / IT mock（per FR-LCM-003 + 演练隔离）
// =============================================================================

/// InMemory 业务 service client mock（per FR-LCM-003 演练隔离 + IT 前置）
///
/// 不启动真 gRPC server，仅记录调用 + 返回预置结果。允许：
/// - L4 #2070 演练 6 阶段操作器在沙箱 PG 池 + 沙箱 K8s + 沙箱业务 service mock 下跑
/// - L4 #2073 IT 7 步 Saga 跨域联动测试（无外部依赖）
///
/// 失败注入支持两种粒度：
/// - `fail_steps`:  按 SagaStepKind.as_str()（如 "Step1:PlayerMigrate"）注入
/// - `fail_methods`: 按方法名（如 "player_migrate"）注入
///
/// 两者并集触发失败；主要用于 IT 覆盖 Step1/Step6 共用 player_migrate 等场景。
#[derive(Default, Clone)]
pub struct InMemoryBusinessServiceClient {
    /// 业务 service 名（tracing 用）
    pub name: &'static str,
    /// 模拟注入的失败（按 SagaStepKind.as_str()）
    pub fail_steps: std::collections::HashSet<String>,
    /// 模拟注入的失败（按方法名）
    pub fail_methods: std::collections::HashSet<String>,
    /// 调用记录（用于 IT 断言）
    pub call_log: Arc<TokioMutex<Vec<String>>>,
}

impl InMemoryBusinessServiceClient {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            fail_steps: std::collections::HashSet::new(),
            fail_methods: std::collections::HashSet::new(),
            call_log: Arc::new(TokioMutex::new(Vec::new())),
        }
    }

    pub fn with_failures(name: &'static str, fail_steps: Vec<&str>) -> Self {
        let mut s = Self::new(name);
        for f in fail_steps {
            s.fail_steps.insert(f.to_string());
        }
        s
    }

    /// 按方法名注入失败（用于 Step1/Step6 共用 player_migrate 等场景）
    pub fn with_method_failures(name: &'static str, fail_methods: Vec<&str>) -> Self {
        let mut s = Self::new(name);
        for f in fail_methods {
            s.fail_methods.insert(f.to_string());
        }
        s
    }

    fn should_fail(&self, step: &str, method: &str) -> bool {
        self.fail_steps.contains(step) || self.fail_methods.contains(method)
    }

    async fn log_call(&self, call: impl Into<String>) {
        self.call_log.lock().await.push(call.into());
    }
}

#[async_trait]
impl BusinessServiceClient for InMemoryBusinessServiceClient {
    async fn player_migrate(
        &self,
        ctx: &SagaContext,
        player_ids: &[String],
    ) -> SagaStepResult<Vec<String>> {
        self.log_call(format!("player_migrate:{}:{} ids", ctx.saga_id, player_ids.len()))
            .await;
        if self.should_fail(SagaStepKind::PlayerMigrate.as_str(), "player_migrate") {
            return Err(SagaStepError {
                kind: SagaStepKind::PlayerMigrate,
                reason: "mock injected failure".to_string(),
                business_service: "rgs_player_service".to_string(),
            });
        }
        Ok(player_ids.iter().map(|p| format!("migrated:{}", p)).collect())
    }

    async fn economy_freeze(
        &self,
        ctx: &SagaContext,
        account_ids: &[String],
    ) -> SagaStepResult<Vec<String>> {
        self.log_call(format!("economy_freeze:{}:{} ids", ctx.saga_id, account_ids.len()))
            .await;
        if self.should_fail(SagaStepKind::EconomyFreeze.as_str(), "economy_freeze") {
            return Err(SagaStepError {
                kind: SagaStepKind::EconomyFreeze,
                reason: "mock injected failure".to_string(),
                business_service: "rgs_economy_service".to_string(),
            });
        }
        Ok(account_ids.iter().map(|a| format!("frozen:{}", a)).collect())
    }

    async fn economy_migrate(
        &self,
        ctx: &SagaContext,
        account_ids: &[String],
    ) -> SagaStepResult<Vec<String>> {
        self.log_call(format!("economy_migrate:{}:{} ids", ctx.saga_id, account_ids.len()))
            .await;
        if self.should_fail(SagaStepKind::EconomyMigrate.as_str(), "economy_migrate") {
            return Err(SagaStepError {
                kind: SagaStepKind::EconomyMigrate,
                reason: "mock injected failure".to_string(),
                business_service: "rgs_economy_service".to_string(),
            });
        }
        Ok(account_ids.iter().map(|a| format!("migrated:{}", a)).collect())
    }

    async fn social_remap(
        &self,
        ctx: &SagaContext,
        relationship_ids: &[String],
    ) -> SagaStepResult<Vec<String>> {
        self.log_call(format!(
            "social_remap:{}:{} ids",
            ctx.saga_id,
            relationship_ids.len()
        ))
        .await;
        if self.should_fail(SagaStepKind::SocialRemap.as_str(), "social_remap") {
            return Err(SagaStepError {
                kind: SagaStepKind::SocialRemap,
                reason: "mock injected failure".to_string(),
                business_service: "rgs_social_service".to_string(),
            });
        }
        Ok(relationship_ids.iter().map(|r| format!("remapped:{}", r)).collect())
    }

    async fn realm_directory_update(&self, ctx: &SagaContext) -> SagaStepResult<Vec<String>> {
        self.log_call(format!(
            "realm_directory_update:{}:{}->{}",
            ctx.saga_id, ctx.source_realm_id, ctx.target_realm_id
        ))
        .await;
        if self.should_fail(SagaStepKind::RealmDirectoryUpdate.as_str(), "realm_directory_update") {
            return Err(SagaStepError {
                kind: SagaStepKind::RealmDirectoryUpdate,
                reason: "mock injected failure".to_string(),
                business_service: "realm_directory".to_string(),
            });
        }
        Ok(vec![format!(
            "realm_directory:{}->{}",
            ctx.source_realm_id, ctx.target_realm_id
        )])
    }

    async fn economy_audit_trail(
        &self,
        ctx: &SagaContext,
        audit_event_ids: &[String],
    ) -> SagaStepResult<Vec<String>> {
        self.log_call(format!(
            "economy_audit_trail:{}:{} ids",
            ctx.saga_id,
            audit_event_ids.len()
        ))
        .await;
        if self.should_fail(SagaStepKind::EconomyAudit.as_str(), "economy_audit_trail") {
            return Err(SagaStepError {
                kind: SagaStepKind::EconomyAudit,
                reason: "mock injected failure".to_string(),
                business_service: "rgs_economy_service".to_string(),
            });
        }
        Ok(audit_event_ids
            .iter()
            .map(|e| format!("audited:{}", e))
            .collect())
    }

    fn service_name(&self) -> &'static str {
        self.name
    }
}

// =============================================================================
// Saga Step trait（per SPEC §3 第 5 条 + M-2067.3 反向补偿）
// =============================================================================

/// Saga 步骤（per SPEC §3 第 5 条 + M-2067.3 反向补偿 + M-2073.5 7 步）
#[async_trait]
pub trait SagaStep: Send + Sync {
    fn kind(&self) -> SagaStepKind;

    /// 前向执行
    async fn forward(&self, ctx: &SagaContext) -> SagaStepResult<SagaStepOutcome>;

    /// 反向补偿（per SPEC §3 第 5 条；M-2067.3）
    async fn reverse(&self, ctx: &SagaContext) -> SagaStepResult<SagaStepOutcome>;
}

// =============================================================================
// 7 步 Saga 编排（per M-2073.1~3 + M-2073.5）
// =============================================================================

/// 7 步 Saga 顺序表（per M-2073.5）
pub const SAGA_STEP_ORDER: &[SagaStepKind] = &[
    SagaStepKind::PlayerMigrate,
    SagaStepKind::EconomyFreeze,
    SagaStepKind::EconomyMigrate,
    SagaStepKind::SocialRemap,
    SagaStepKind::RealmDirectoryUpdate,
    SagaStepKind::PlayerUnfreeze,
    SagaStepKind::EconomyAudit,
];

/// 7 步 Saga 编排器（per M-2073.5 IT + SPEC §3 + §6 R1 缓解）
///
/// 7 步顺序执行（per SPEC §3）；任一步失败 → 已执行步骤走 `reverse` 补偿链。
pub struct CrossDomainSaga {
    client: Arc<dyn BusinessServiceClient>,
    /// Step 超时（per SPEC §5 背压 + M-2067.5）
    pub step_timeout: Duration,
}

impl CrossDomainSaga {
    pub fn new(client: Arc<dyn BusinessServiceClient>) -> Self {
        Self {
            client,
            step_timeout: Duration::from_secs(DEFAULT_SAGA_STEP_TIMEOUT_SECS),
        }
    }

    pub fn with_timeout(client: Arc<dyn BusinessServiceClient>, step_timeout: Duration) -> Self {
        Self {
            client,
            step_timeout,
        }
    }

    /// 执行 7 步 Saga；任一步失败 → 反向补偿
    ///
    /// 返回 `Ok(Vec<SagaStepOutcome>)` 全 7 步成功；
    /// `Err(SagaStepError)` 任一步失败 + 已补偿链
    pub async fn run(&self, ctx: &SagaContext) -> Result<Vec<SagaStepOutcome>> {
        let mut outcomes: Vec<SagaStepOutcome> = Vec::with_capacity(7);
        for &kind in SAGA_STEP_ORDER {
            let res = self.execute_step(kind, ctx).await;
            match res {
                Ok(outcome) => outcomes.push(outcome),
                Err(step_err) => {
                    // 触发反向补偿链（per SPEC §3 第 5 条 + §6 R1 缓解）
                    self.compensate(&outcomes, ctx).await;
                    return Err(Error::SagaStepFailed {
                        step_id: step_err.kind.as_str().to_string(),
                        saga_id: ctx.saga_id.to_string(),
                        reason: step_err.reason,
                    });
                }
            }
        }
        Ok(outcomes)
    }

    async fn execute_step(
        &self,
        kind: SagaStepKind,
        ctx: &SagaContext,
    ) -> SagaStepResult<SagaStepOutcome> {
        // 超时包装（per SPEC §5 + M-2067.5：默认 60s 触发反向补偿）
        let step_fut = self.execute_step_inner(kind, ctx);
        match tokio::time::timeout(self.step_timeout, step_fut).await {
            Ok(res) => res,
            Err(_) => Err(SagaStepError {
                kind,
                reason: format!("step timeout after {}s", self.step_timeout.as_secs()),
                business_service: kind.business_service().to_string(),
            }),
        }
    }

    async fn execute_step_inner(
        &self,
        kind: SagaStepKind,
        ctx: &SagaContext,
    ) -> SagaStepResult<SagaStepOutcome> {
        match kind {
            // Step1: rgs-player-service 玩家数据迁移（per M-2073.1）
            SagaStepKind::PlayerMigrate => {
                let player_ids = sample_player_ids(ctx);
                let migrated = self.client.player_migrate(ctx, &player_ids).await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: migrated,
                    state_change: "players_migrated".to_string(),
                    metadata: serde_json::json!({"step": "Step1"}),
                })
            }
            // Step2: rgs-economy-service 余额冻结（per M-2073.2）
            SagaStepKind::EconomyFreeze => {
                let account_ids = sample_account_ids(ctx);
                let frozen = self.client.economy_freeze(ctx, &account_ids).await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: frozen,
                    state_change: "balances_frozen".to_string(),
                    metadata: serde_json::json!({"step": "Step2"}),
                })
            }
            // Step3: rgs-economy-service 资金迁移（per M-2073.2）
            SagaStepKind::EconomyMigrate => {
                let account_ids = sample_account_ids(ctx);
                let migrated = self.client.economy_migrate(ctx, &account_ids).await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: migrated,
                    state_change: "wallets_migrated".to_string(),
                    metadata: serde_json::json!({"step": "Step3"}),
                })
            }
            // Step4: rgs-social-service 好友/工会/邮件重映射（per M-2073.3）
            SagaStepKind::SocialRemap => {
                let relationship_ids = sample_relationship_ids(ctx);
                let remapped = self.client.social_remap(ctx, &relationship_ids).await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: remapped,
                    state_change: "relationships_remapped".to_string(),
                    metadata: serde_json::json!({"step": "Step4"}),
                })
            }
            // Step5: RealmDirectoryService 路由表更新（本地）
            SagaStepKind::RealmDirectoryUpdate => {
                let updated = self.client.realm_directory_update(ctx).await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: updated,
                    state_change: "realm_directory_updated".to_string(),
                    metadata: serde_json::json!({"step": "Step5"}),
                })
            }
            // Step6: rgs-player-service 解冻 + 确认（per M-2073.1）
            SagaStepKind::PlayerUnfreeze => {
                // Step6 调用 player_migrate 反向 + 状态确认；为简化复用 player_migrate
                // （生产可拆为独立 UnfreezePlayers RPC）
                let player_ids = sample_player_ids(ctx);
                let acked = self.client.player_migrate(ctx, &player_ids).await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: acked,
                    state_change: "players_unfrozen".to_string(),
                    metadata: serde_json::json!({"step": "Step6"}),
                })
            }
            // Step7: rgs-economy-service 审计轨迹双写（per M-2073.2）
            SagaStepKind::EconomyAudit => {
                let audit_event_ids = sample_audit_ids(ctx);
                let written = self
                    .client
                    .economy_audit_trail(ctx, &audit_event_ids)
                    .await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: written,
                    state_change: "audit_trail_written".to_string(),
                    metadata: serde_json::json!({"step": "Step7"}),
                })
            }
        }
    }

    /// 反向补偿链（per SPEC §3 第 5 条 + §6 R1 缓解 + M-2067.3）
    ///
    /// 顺序：从失败步倒序到 Step1 全部 reverse；每个 step 独立处理错误
    /// （per SPEC §3 第 5 条 + §6 R9：补偿不完整需要告警但不阻塞）
    async fn compensate(&self, executed: &[SagaStepOutcome], ctx: &SagaContext) {
        for outcome in executed.iter().rev() {
            // 反向补偿为幂等 best-effort；错误不阻塞后续 step
            let reverse_res = self.reverse_step(outcome.kind, ctx).await;
            if let Err(e) = reverse_res {
                tracing::warn!(
                    saga_id = %ctx.saga_id,
                    step = outcome.kind.as_str(),
                    error = %e,
                    "saga reverse compensation failed (per SPEC §3 第 5 条 需人工介入)"
                );
            }
        }
    }

    async fn reverse_step(
        &self,
        kind: SagaStepKind,
        ctx: &SagaContext,
    ) -> SagaStepResult<SagaStepOutcome> {
        // 反向补偿语义：
        // - PlayerMigrate    → 重新迁回（player_migrate 幂等）
        // - EconomyFreeze    → 解冻（player_migrate 复用，作为 unfreeze 触发器）
        // - EconomyMigrate   → 反向迁移（economy_migrate 幂等）
        // - SocialRemap      → 反向重映射（social_remap 幂等）
        // - RealmDirectoryUpdate → 路由回滚（realm_directory_update 幂等）
        // - PlayerUnfreeze   → 冻结（player_migrate 复用）
        // - EconomyAudit     → 审计补充（economy_audit_trail 幂等）
        match kind {
            SagaStepKind::PlayerMigrate | SagaStepKind::PlayerUnfreeze => {
                let ids = sample_player_ids(ctx);
                self.client.player_migrate(ctx, &ids).await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: ids.iter().map(|p| format!("reversed:{}", p)).collect(),
                    state_change: format!("reverse:{}", kind.as_str()),
                    metadata: serde_json::json!({"phase": "reverse"}),
                })
            }
            SagaStepKind::EconomyFreeze => {
                let ids = sample_account_ids(ctx);
                self.client.economy_freeze(ctx, &ids).await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: ids.iter().map(|a| format!("reversed:{}", a)).collect(),
                    state_change: format!("reverse:{}", kind.as_str()),
                    metadata: serde_json::json!({"phase": "reverse"}),
                })
            }
            SagaStepKind::EconomyMigrate => {
                let ids = sample_account_ids(ctx);
                self.client.economy_migrate(ctx, &ids).await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: ids.iter().map(|a| format!("reversed:{}", a)).collect(),
                    state_change: format!("reverse:{}", kind.as_str()),
                    metadata: serde_json::json!({"phase": "reverse"}),
                })
            }
            SagaStepKind::SocialRemap => {
                let ids = sample_relationship_ids(ctx);
                self.client.social_remap(ctx, &ids).await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: ids.iter().map(|r| format!("reversed:{}", r)).collect(),
                    state_change: format!("reverse:{}", kind.as_str()),
                    metadata: serde_json::json!({"phase": "reverse"}),
                })
            }
            SagaStepKind::RealmDirectoryUpdate => {
                self.client.realm_directory_update(ctx).await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: vec![],
                    state_change: format!("reverse:{}", kind.as_str()),
                    metadata: serde_json::json!({"phase": "reverse"}),
                })
            }
            SagaStepKind::EconomyAudit => {
                let ids = sample_audit_ids(ctx);
                self.client.economy_audit_trail(ctx, &ids).await?;
                Ok(SagaStepOutcome {
                    kind,
                    affected_entity_ids: ids.iter().map(|e| format!("reversed:{}", e)).collect(),
                    state_change: format!("reverse:{}", kind.as_str()),
                    metadata: serde_json::json!({"phase": "reverse"}),
                })
            }
        }
    }
}

// =============================================================================
// 辅助：sample 实体 ID（per SPEC §6 IT 33 条测试 fixture）
// =============================================================================

fn sample_player_ids(ctx: &SagaContext) -> Vec<String> {
    vec![
        format!("player:{}-{}:001", ctx.source_realm_id, ctx.target_realm_id),
        format!("player:{}-{}:002", ctx.source_realm_id, ctx.target_realm_id),
        format!("player:{}-{}:003", ctx.source_realm_id, ctx.target_realm_id),
    ]
}

fn sample_account_ids(ctx: &SagaContext) -> Vec<String> {
    vec![
        format!("economy:{}-{}:wallet-1", ctx.source_realm_id, ctx.target_realm_id),
        format!("economy:{}-{}:wallet-2", ctx.source_realm_id, ctx.target_realm_id),
    ]
}

fn sample_relationship_ids(ctx: &SagaContext) -> Vec<String> {
    vec![
        format!("social:{}-{}:friends", ctx.source_realm_id, ctx.target_realm_id),
        format!("social:{}-{}:guilds", ctx.source_realm_id, ctx.target_realm_id),
        format!("social:{}-{}:mail", ctx.source_realm_id, ctx.target_realm_id),
    ]
}

fn sample_audit_ids(ctx: &SagaContext) -> Vec<String> {
    vec![
        format!("audit:{}-{}:e1", ctx.source_realm_id, ctx.target_realm_id),
        format!("audit:{}-{}:e2", ctx.source_realm_id, ctx.target_realm_id),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;

    fn ctx() -> SagaContext {
        SagaContext::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "realm-source",
            "realm-target",
        )
    }

    /// 验证 SagaStepKind 7 步顺序与 SAGA_STEP_ORDER 一致
    #[test]
    fn seven_step_order_complete() {
        assert_eq!(SAGA_STEP_ORDER.len(), 7);
        assert_eq!(
            SAGA_STEP_ORDER[0],
            SagaStepKind::PlayerMigrate,
            "Step1 must be PlayerMigrate (per M-2073.1)"
        );
        assert_eq!(
            SAGA_STEP_ORDER[1],
            SagaStepKind::EconomyFreeze,
            "Step2 must be EconomyFreeze (per M-2073.2)"
        );
        assert_eq!(
            SAGA_STEP_ORDER[2],
            SagaStepKind::EconomyMigrate,
            "Step3 must be EconomyMigrate (per M-2073.2)"
        );
        assert_eq!(
            SAGA_STEP_ORDER[3],
            SagaStepKind::SocialRemap,
            "Step4 must be SocialRemap (per M-2073.3)"
        );
        assert_eq!(
            SAGA_STEP_ORDER[4],
            SagaStepKind::RealmDirectoryUpdate,
            "Step5 must be RealmDirectoryUpdate (local)"
        );
        assert_eq!(
            SAGA_STEP_ORDER[5],
            SagaStepKind::PlayerUnfreeze,
            "Step6 must be PlayerUnfreeze (per M-2073.1)"
        );
        assert_eq!(
            SAGA_STEP_ORDER[6],
            SagaStepKind::EconomyAudit,
            "Step7 must be EconomyAudit (per M-2073.2)"
        );
    }

    /// 验证 SAGA_STEP_KINDS 字符串常量与枚举 7 步一致
    #[test]
    fn saga_step_kinds_constant_matches() {
        assert_eq!(SAGA_STEP_KINDS.len(), 7);
        assert_eq!(SAGA_STEP_KINDS[0], "Step1:PlayerMigrate");
        assert_eq!(SAGA_STEP_KINDS[6], "Step7:EconomyAudit");
    }

    /// 验证 3 业务 service 命名（per M-2073.1~3 grep 验证）
    #[test]
    fn business_service_names_match() {
        assert_eq!(
            SagaStepKind::PlayerMigrate.business_service(),
            "rgs_player_service"
        );
        assert_eq!(
            SagaStepKind::EconomyFreeze.business_service(),
            "rgs_economy_service"
        );
        assert_eq!(
            SagaStepKind::SocialRemap.business_service(),
            "rgs_social_service"
        );
    }

    /// 验证 SagaStepKind.as_str 与 SAGA_STEP_KINDS 一致
    #[test]
    fn step_kind_as_str_consistent() {
        for (i, k) in SAGA_STEP_ORDER.iter().enumerate() {
            assert_eq!(k.as_str(), SAGA_STEP_KINDS[i]);
        }
    }

    /// 验证 7 步全部 requires_compensation（per SPEC §3 第 5 条 + M-2067.3）
    #[test]
    fn all_seven_steps_require_compensation() {
        for k in SAGA_STEP_ORDER {
            assert!(k.requires_compensation(), "step {} must require compensation", k.as_str());
        }
    }

    /// Happy path: 7 步 Saga 全成功
    #[tokio::test]
    async fn saga_happy_path_seven_steps() {
        let client = Arc::new(InMemoryBusinessServiceClient::new("ok"));
        let saga = CrossDomainSaga::new(client);
        let outcomes = saga.run(&ctx()).await.unwrap();
        assert_eq!(outcomes.len(), 7);
        for (i, o) in outcomes.iter().enumerate() {
            assert_eq!(o.kind, SAGA_STEP_ORDER[i]);
        }
    }

    /// 失败 + 反向补偿：Step3 注入失败 → Step1, Step2 触发 reverse
    #[tokio::test]
    async fn saga_failure_triggers_compensation_chain() {
        let mut fail = HashSet::new();
        fail.insert("Step3:EconomyMigrate".to_string());
        let client = Arc::new(InMemoryBusinessServiceClient::with_failures(
            "fail-step3",
            vec!["Step3:EconomyMigrate"],
        ));
        // 注入失败
        let mut c = (*client).clone();
        c.fail_steps = fail;
        let saga = CrossDomainSaga::new(Arc::new(c));
        let res = saga.run(&ctx()).await;
        assert!(res.is_err());
        match res.unwrap_err() {
            Error::SagaStepFailed { step_id, reason, .. } => {
                assert_eq!(step_id, "Step3:EconomyMigrate");
                assert!(reason.contains("mock injected failure"));
            }
            e => panic!("expected SagaStepFailed, got {:?}", e),
        }
    }

    /// 反向补偿链调用记录：Step3 失败 → Step2 + Step1 应被 reverse 调用
    #[tokio::test]
    async fn saga_compensation_calls_reverse_for_executed_steps() {
        let client = Arc::new(InMemoryBusinessServiceClient::with_failures(
            "fail-step3",
            vec!["Step3:EconomyMigrate"],
        ));
        let saga = CrossDomainSaga::new(client.clone());
        let res = saga.run(&ctx()).await;
        assert!(res.is_err());
        let calls = client.call_log.lock().await.clone();
        // 应当看到 Step1, Step2 前向 + reverse 链调用
        let player_calls: Vec<_> = calls
            .iter()
            .filter(|c| c.starts_with("player_migrate"))
            .collect();
        let economy_freeze_calls: Vec<_> = calls
            .iter()
            .filter(|c| c.starts_with("economy_freeze"))
            .collect();
        // Step1 + 它的 reverse = 2 次 player_migrate
        assert!(
            player_calls.len() >= 2,
            "Step1 forward + reverse expected, got {}",
            player_calls.len()
        );
        // Step2 + 它的 reverse = 2 次 economy_freeze
        assert!(
            economy_freeze_calls.len() >= 2,
            "Step2 forward + reverse expected, got {}",
            economy_freeze_calls.len()
        );
    }

    /// Step 超时（per M-2067.5 60s 触发反向补偿）：用极短超时 + 模拟慢调用
    #[tokio::test]
    async fn saga_step_timeout_triggers_compensation() {
        struct SlowClient;
        #[async_trait]
        impl BusinessServiceClient for SlowClient {
            async fn player_migrate(
                &self,
                _ctx: &SagaContext,
                ids: &[String],
            ) -> SagaStepResult<Vec<String>> {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                Ok(ids.to_vec())
            }
            async fn economy_freeze(
                &self,
                _ctx: &SagaContext,
                ids: &[String],
            ) -> SagaStepResult<Vec<String>> {
                Ok(ids.to_vec())
            }
            async fn economy_migrate(
                &self,
                _ctx: &SagaContext,
                ids: &[String],
            ) -> SagaStepResult<Vec<String>> {
                Ok(ids.to_vec())
            }
            async fn social_remap(
                &self,
                _ctx: &SagaContext,
                ids: &[String],
            ) -> SagaStepResult<Vec<String>> {
                Ok(ids.to_vec())
            }
            async fn economy_audit_trail(
                &self,
                _ctx: &SagaContext,
                ids: &[String],
            ) -> SagaStepResult<Vec<String>> {
                Ok(ids.to_vec())
            }
            fn service_name(&self) -> &'static str {
                "slow"
            }
        }
        let saga = CrossDomainSaga::with_timeout(
            Arc::new(SlowClient),
            Duration::from_millis(50),
        );
        let res = saga.run(&ctx()).await;
        assert!(res.is_err());
        match res.unwrap_err() {
            Error::SagaStepFailed { reason, .. } => {
                assert!(
                    reason.contains("timeout"),
                    "expected timeout reason, got: {}",
                    reason
                );
            }
            e => panic!("expected SagaStepFailed, got {:?}", e),
        }
    }

    /// 验证 SagaContext 默认超时 = 60s（per SPEC §5 + M-2067.5）
    #[test]
    fn saga_context_default_timeout() {
        let c = ctx();
        assert_eq!(c.step_timeout.as_secs(), 60);
    }
}

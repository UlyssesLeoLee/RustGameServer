//! `retire_plan` —— 退场计划表（含 RBAC 查询通道配置，per SPEC-DTL-042 §3 第 8 条）
//!
//! # 关键设计
//!
//! - **退场后 RBAC 查询通道**（per SPEC-DTL-042 §3 第 8 条 + §6 R5）：
//!   - 仅对 `retire_plan.query_channel_rbac` 配置的角色开放
//!   - 默认 `cs_agent` / `sre` / `legal` 3 角色（per SPEC §3 第 8 条）
//!   - 退场后存档可读：客服 / SRE / 法务；其他角色拒绝（per SPEC §6 Security）
//!
//! - **不**分发独立 gRPC（per FR-LCM-004）；本 plan 通过 `RealmLifecycleService`
//!   转发 + `AdminService` 提供查询接口
//!
//! - **不**绕过 PFAU 编排（per SPEC §3 第 2 条）
//!
//! # WF-1-2073 范围
//!
//! - M-2073.4：实现完整 RBAC 配置 + role enum
//! - L4 #2068 M-2068.3：将 `retire_plan` 落到 `migrations/0020_lcm_tables.sql`
//!   （本任务**不**改 migration，由 M-2068.3 独立完成）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::realm_lifecycle::error::{Error, Result};

/// 退场查询通道 RBAC 角色（per SPEC-DTL-042 §3 第 8 条）
///
/// 默认 3 角色：cs_agent / sre / legal。
/// **不**使用 `shared-platform::rbac::Role` 枚举（避免与 5 域 RBAC 耦合；
/// LCM 是 Admin 域内部概念，4 独立 RBAC 平面，per DEC-005）。
///
/// 序列化策略：snake_case `cs_agent` / `sre` / `legal`（per SPEC §3 第 8 条
/// 严格匹配 + 落库 DDL JSONB 一致）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetireChannelRole {
    /// 客服（处理退服后玩家申诉、查询存档）
    CsAgent,
    /// SRE（运维取证、故障复盘、归档验证）
    Sre,
    /// 法务/合规（GDPR 审计、监管查询、纠纷取证）
    Legal,
}

impl RetireChannelRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            RetireChannelRole::CsAgent => "cs_agent",
            RetireChannelRole::Sre => "sre",
            RetireChannelRole::Legal => "legal",
        }
    }

    /// 解析（含兼容性：字符串大小写敏感，per SPEC §3 第 8 条严格匹配）
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "cs_agent" => Ok(RetireChannelRole::CsAgent),
            "sre" => Ok(RetireChannelRole::Sre),
            "legal" => Ok(RetireChannelRole::Legal),
            other => Err(Error::Validation(format!(
                "unknown retire channel role: {} (allowed: cs_agent, sre, legal)",
                other
            ))),
        }
    }

    /// 全部默认角色（per SPEC §3 第 8 条 默认 3 角色）
    pub const ALL_DEFAULT: [RetireChannelRole; 3] = [
        RetireChannelRole::CsAgent,
        RetireChannelRole::Sre,
        RetireChannelRole::Legal,
    ];
}

impl std::fmt::Display for RetireChannelRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 退场查询通道 RBAC 配置（per SPEC-DTL-042 §3 第 8 条 + §6 R5）
///
/// 序列化形态：JSON 数组，例：`["cs_agent", "sre", "legal"]`。
/// 落库列：DDL `query_channel_rbac JSONB NOT NULL`（per M-2068.3 migration 计划）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryChannelRbac {
    /// 允许访问退场后存档的角色列表
    /// 默认：`[cs_agent, sre, legal]`（per SPEC §3 第 8 条）
    pub allowed_roles: Vec<RetireChannelRole>,
    /// 审计开关：true = 每次访问写 `admin_db.operation_audit`
    /// （per SPEC §3 第 7 条 + FR-LCM-002 阶段变更全流程留痕）
    pub audit_on_access: bool,
}

impl Default for QueryChannelRbac {
    fn default() -> Self {
        Self {
            allowed_roles: RetireChannelRole::ALL_DEFAULT.to_vec(),
            audit_on_access: true,
        }
    }
}

impl QueryChannelRbac {
    /// 严格默认：仅 cs_agent / sre / legal（per SPEC §3 第 8 条）
    pub fn strict_default() -> Self {
        Self::default()
    }

    /// 自定义允许角色（运营自定义时使用；运行时不允许覆盖锁定配置）
    pub fn with_roles(roles: Vec<RetireChannelRole>) -> Self {
        Self {
            allowed_roles: roles,
            audit_on_access: true,
        }
    }

    /// 检查某角色是否有权访问退场后存档
    pub fn allows(&self, role: RetireChannelRole) -> bool {
        self.allowed_roles.contains(&role)
    }

    /// 序列化为 JSON 字符串（落库形式，per M-2068.3 DDL JSONB）
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| Error::Internal(anyhow::anyhow!("rbac serialize: {}", e)))
    }

    /// 从 JSON 字符串反序列化（落库读取，per M-2068.3 DDL JSONB）
    pub fn from_json(s: &str) -> Result<Self> {
        serde_json::from_str(s)
            .map_err(|e| Error::Internal(anyhow::anyhow!("rbac deserialize: {}", e)))
    }
}

/// `retire_plan` 实体（per DTL-042 §2 + L4 #2068 M-2068.3 + M-2073.4）
///
/// 字段（per SPEC §3 第 8 条 + DTL §6）：
/// - `plan_id`         UUID 主键
/// - `realm_id`        退场 realm
/// - `retire_at`       退场触发时间
/// - `query_channel_rbac` 退场后查询通道 RBAC 配置
/// - `approval_ref`    三方签字 reference（高危操作必备，per SPEC §5）
/// - `locked_at`       锁定时间（None = 未锁定，可改；Some(_) = 锁定不可改，per FR-LCM-062 类似机制）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetirePlan {
    pub plan_id: Uuid,
    pub realm_id: String,
    pub retire_at: DateTime<Utc>,
    pub query_channel_rbac: QueryChannelRbac,
    pub approval_ref: Option<String>,
    pub locked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RetirePlan {
    pub fn new(
        plan_id: Uuid,
        realm_id: impl Into<String>,
        retire_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            plan_id,
            realm_id: realm_id.into(),
            retire_at,
            query_channel_rbac: QueryChannelRbac::strict_default(),
            approval_ref: None,
            locked_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 自定义 RBAC 配置（仅 unlocked 状态可改，per FR-LCM-062 类似机制）
    pub fn set_rbac(&mut self, rbac: QueryChannelRbac) -> Result<()> {
        if self.locked_at.is_some() {
            return Err(Error::Validation(format!(
                "retire_plan {} is locked; cannot modify rbac (per FR-LCM-062 类似机制)",
                self.plan_id
            )));
        }
        self.query_channel_rbac = rbac;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 锁定（per FR-LCM-062 锁定后不可改）
    pub fn lock(&mut self) -> Result<()> {
        if self.locked_at.is_some() {
            return Err(Error::Conflict(format!(
                "retire_plan {} already locked",
                self.plan_id
            )));
        }
        self.locked_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 检查某角色对当前 plan 的退场后查询权限
    pub fn check_access(&self, role: RetireChannelRole) -> Result<()> {
        if !self.query_channel_rbac.allows(role) {
            return Err(Error::RetireChannelDenied {
                required: self
                    .query_channel_rbac
                    .allowed_roles
                    .iter()
                    .map(|r| r.as_str().to_string())
                    .collect(),
                actual: role.as_str().to_string(),
            });
        }
        Ok(())
    }
}

/// `retire_plan` Repository trait（per L4 #2068 M-2068.7 既有模式）
///
/// 完整 impl（InMemory + Pg）属 L4 #2068；本任务仅声明 trait。
#[async_trait::async_trait]
pub trait RetirePlanConfig: Send + Sync {
    async fn find_by_id(&self, plan_id: Uuid) -> Result<Option<RetirePlan>>;
    async fn find_by_realm(&self, realm_id: &str) -> Result<Option<RetirePlan>>;
    async fn save(&self, plan: &RetirePlan) -> Result<()>;
    async fn list_all(&self) -> Result<Vec<RetirePlan>>;
}

/// InMemory `RetirePlanConfig`（per RGS-IMPL-001 §3 既有模式：Pg + InMemory 双实现）
pub struct InMemoryRetirePlanConfig {
    inner: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<Uuid, RetirePlan>>>,
}

impl Default for InMemoryRetirePlanConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryRetirePlanConfig {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl RetirePlanConfig for InMemoryRetirePlanConfig {
    async fn find_by_id(&self, plan_id: Uuid) -> Result<Option<RetirePlan>> {
        let store = self.inner.lock().await;
        Ok(store.get(&plan_id).cloned())
    }

    async fn find_by_realm(&self, realm_id: &str) -> Result<Option<RetirePlan>> {
        let store = self.inner.lock().await;
        Ok(store.values().find(|p| p.realm_id == realm_id).cloned())
    }

    async fn save(&self, plan: &RetirePlan) -> Result<()> {
        let mut store = self.inner.lock().await;
        store.insert(plan.plan_id, plan.clone());
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<RetirePlan>> {
        let store = self.inner.lock().await;
        Ok(store.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证默认 RBAC 仅 3 角色（per SPEC §3 第 8 条）
    #[test]
    fn default_rbac_is_three_roles() {
        let rbac = QueryChannelRbac::default();
        assert_eq!(rbac.allowed_roles.len(), 3);
        assert!(rbac.allows(RetireChannelRole::CsAgent));
        assert!(rbac.allows(RetireChannelRole::Sre));
        assert!(rbac.allows(RetireChannelRole::Legal));
    }

    /// 验证其他角色（player / super_admin 等）被默认 RBAC 拒绝
    /// （per SPEC §3 第 8 条 仅 cs_agent/sre/legal + §6 R5）
    #[test]
    fn default_rbac_rejects_other_roles() {
        let rbac = QueryChannelRbac::default();
        // 自定义角色模拟：检查未知 role 不在 allowed list
        // 用 parse 失败的角色作为"其他"代表
        let parse_result = RetireChannelRole::parse("player");
        assert!(parse_result.is_err());
        let parse_super = RetireChannelRole::parse("super_admin");
        assert!(parse_super.is_err());
        // 显式 deny：构造 only-cs_agent 列表，验证 Sre 被拒绝
        let rbac_limited = QueryChannelRbac::with_roles(vec![RetireChannelRole::CsAgent]);
        assert!(rbac_limited.allows(RetireChannelRole::CsAgent));
        assert!(!rbac_limited.allows(RetireChannelRole::Sre));
        assert!(!rbac_limited.allows(RetireChannelRole::Legal));
    }

    /// 验证 RetireChannelRole 字符串严格匹配（per SPEC §3 第 8 条）
    #[test]
    fn role_string_strict_match() {
        assert_eq!(RetireChannelRole::CsAgent.as_str(), "cs_agent");
        assert_eq!(RetireChannelRole::Sre.as_str(), "sre");
        assert_eq!(RetireChannelRole::Legal.as_str(), "legal");
        assert_eq!(
            RetireChannelRole::parse("cs_agent").unwrap(),
            RetireChannelRole::CsAgent
        );
        assert_eq!(
            RetireChannelRole::parse("sre").unwrap(),
            RetireChannelRole::Sre
        );
        assert_eq!(
            RetireChannelRole::parse("legal").unwrap(),
            RetireChannelRole::Legal
        );
        // 大小写敏感
        assert!(RetireChannelRole::parse("CS_AGENT").is_err());
        assert!(RetireChannelRole::parse("Sre").is_err());
    }

    /// 验证 RBAC JSON 序列化（含 cs_agent / sre / legal 字符串）
    #[test]
    fn rbac_json_round_trip() {
        let rbac = QueryChannelRbac::default();
        let json = rbac.to_json().unwrap();
        assert!(json.contains("cs_agent"), "JSON must contain cs_agent: {}", json);
        assert!(json.contains("sre"), "JSON must contain sre: {}", json);
        assert!(json.contains("legal"), "JSON must contain legal: {}", json);
        let parsed = QueryChannelRbac::from_json(&json).unwrap();
        assert_eq!(parsed, rbac);
    }

    /// 验证 RetirePlan 锁定后不可改（per FR-LCM-062 类似机制）
    #[tokio::test]
    async fn retire_plan_lock_prevents_rbac_modification() {
        let mut plan = RetirePlan::new(Uuid::new_v4(), "realm-x", Utc::now());
        plan.lock().unwrap();
        let new_rbac = QueryChannelRbac::with_roles(vec![RetireChannelRole::Sre]);
        let res = plan.set_rbac(new_rbac);
        assert!(res.is_err());
        // 二次 lock 拒绝
        let res2 = plan.lock();
        assert!(res2.is_err());
    }

    /// 验证 RetirePlan.check_access：默认 RBAC 允许 3 角色；其他拒绝
    #[tokio::test]
    async fn retire_plan_check_access_default() {
        let plan = RetirePlan::new(Uuid::new_v4(), "realm-y", Utc::now());
        assert!(plan.check_access(RetireChannelRole::CsAgent).is_ok());
        assert!(plan.check_access(RetireChannelRole::Sre).is_ok());
        assert!(plan.check_access(RetireChannelRole::Legal).is_ok());
    }

    /// 验证 InMemoryRetirePlanConfig 增删查
    #[tokio::test]
    async fn in_memory_retire_plan_repo() {
        let repo = InMemoryRetirePlanConfig::new();
        let plan = RetirePlan::new(Uuid::new_v4(), "realm-z", Utc::now());
        repo.save(&plan).await.unwrap();
        let found = repo.find_by_id(plan.plan_id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().realm_id, "realm-z");
        let by_realm = repo.find_by_realm("realm-z").await.unwrap();
        assert!(by_realm.is_some());
        let all = repo.list_all().await.unwrap();
        assert_eq!(all.len(), 1);
    }

    /// 验证 query_channel_rbac JSON 配置的 reject 路径走
    /// Error::RetireChannelDenied → tonic::Status::PermissionDenied（per §6 Security）
    #[test]
    fn retire_channel_denied_to_permission_denied_status() {
        let mut plan = RetirePlan::new(Uuid::new_v4(), "realm-deny", Utc::now());
        plan.set_rbac(QueryChannelRbac::with_roles(vec![RetireChannelRole::Legal]))
            .unwrap();
        let err = plan.check_access(RetireChannelRole::CsAgent).unwrap_err();
        // 先 copy 字段用于 assert，再 into() tonic::Status
        match &err {
            Error::RetireChannelDenied { required, actual } => {
                assert_eq!(required, &vec!["legal".to_string()]);
                assert_eq!(actual, "cs_agent");
            }
            _ => panic!("expected RetireChannelDenied"),
        }
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::PermissionDenied);
    }
}

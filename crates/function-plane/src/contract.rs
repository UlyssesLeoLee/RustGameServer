//! Function Contract types (per RGS-INC-001 v0.2 §15.2 schema, simplified).
//!
//! This is a *mock* of the full registry schema: a faithful subset that supports
//! register / get / list-active / version-select. JSON Schema validation, retry
//! policy, scale policy, security policy, observability, etc. are **not** parsed
//! here — they live in the (planned) PG-backed schema and are intentionally
//! out-of-scope for the mock.
//!
//! See [`crate::error::FunctionPlaneError::ContractInvalid`] for the place
//! where richer validation will land in Phase 1.
#![allow(missing_docs)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Which runtime a function executes on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Runtime {
    /// WASM module executed in-process via Wasmtime.
    Wasm,
    /// Container-based function (mock: never materialized in v0.1).
    Container,
}

/// Lifecycle status of a function version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FunctionStatus {
    /// Newly registered, not yet eligible for invocation.
    Draft,
    /// Live and invokable.
    Active,
    /// Temporarily disabled; invocation returns [`crate::error::FunctionPlaneError::NotActive`].
    Paused,
    /// Retired; not invokable, kept for audit.
    Archived,
}

/// How a function is triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerType {
    /// Synchronous gRPC call.
    Grpc,
    /// Synchronous HTTP call.
    Http,
    /// NATS JetStream subscription (Phase 1+).
    Nats,
    /// Cron tick (mock: not materialized).
    Cron,
}

/// Full metadata for a single `(function_id, version)` record.
///
/// This is the in-memory representation of a row in the
/// `cluster_ops_db.function_registry` table (per RGS-INC-001 v0.2 §15.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    /// Logical function name, e.g. `"achievement.calculate"`.
    pub function_id: String,
    /// SemVer-ish version string, e.g. `"v0.1.0"`. Must be unique per function.
    pub version: String,
    /// Target runtime.
    pub runtime: Runtime,
    /// Trigger style advertised by the function.
    pub trigger_type: TriggerType,
    /// JSON Schema (as JSON value) describing accepted input.
    pub input_schema: serde_json::Value,
    /// JSON Schema describing emitted output.
    pub output_schema: serde_json::Value,
    /// Wall-clock timeout in milliseconds.
    pub timeout_ms: u32,
    /// Wasmtime fuel budget per call.
    pub fuel: u64,
    /// Memory ceiling in MiB.
    pub memory_mib: u32,
    /// Concurrency hint (mock: advisory only, not enforced).
    pub concurrency: u32,
    /// Current lifecycle state.
    pub status: FunctionStatus,
    /// Raw WASM bytes; required when [`Self::runtime`] is [`Runtime::Wasm`].
    pub wasm_bytes: Option<Vec<u8>>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last mutation timestamp.
    pub updated_at: DateTime<Utc>,
}

impl FunctionMetadata {
    /// Convenience constructor for tests and fixtures: produces a metadata record
    /// with sensible defaults for the timestamp fields and the optional `wasm_bytes`.
    pub fn new(
        function_id: impl Into<String>,
        version: impl Into<String>,
        runtime: Runtime,
        trigger_type: TriggerType,
        wasm_bytes: Option<Vec<u8>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            function_id: function_id.into(),
            version: version.into(),
            runtime,
            trigger_type,
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
            timeout_ms: 5_000,
            fuel: 10_000_000,
            memory_mib: 64,
            concurrency: 16,
            status: FunctionStatus::Draft,
            wasm_bytes,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Per-invocation context propagated through the Gateway into the function.
///
/// Mirrors the §3.3 observability business fields (`request_id`, `trace_id`,
/// `saga_id`, `command_id`, `deadline`). `idempotency_key` enables at-least-once
/// delivery semantics on retry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionContext {
    /// Per-call request id (UUID v4); always populated.
    pub request_id: Uuid,
    /// Optional W3C `traceparent` trace id (hex string).
    pub trace_id: Option<String>,
    /// Optional correlation id (UUID).
    pub correlation_id: Option<Uuid>,
    /// Optional saga id (UUID).
    pub saga_id: Option<Uuid>,
    /// Optional event id (UUID) when invoked from an event subscription.
    pub event_id: Option<Uuid>,
    /// Optional user / actor id.
    pub user_id: Option<Uuid>,
    /// Optional tenant id (multi-tenant isolation).
    pub tenant_id: Option<String>,
    /// Optional deadline (UTC) for this call.
    pub deadline: Option<DateTime<Utc>>,
    /// Number of retries so far.
    pub retry_count: u32,
    /// Optional idempotency key.
    pub idempotency_key: Option<String>,
}

impl Default for FunctionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionContext {
    /// Build a fresh context with a new request id.
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4(),
            trace_id: None,
            correlation_id: None,
            saga_id: None,
            event_id: None,
            user_id: None,
            tenant_id: None,
            deadline: None,
            retry_count: 0,
            idempotency_key: None,
        }
    }

    /// Chainable setter for [`Self::trace_id`].
    pub fn with_trace_id(mut self, t: impl Into<String>) -> Self {
        self.trace_id = Some(t.into());
        self
    }

    /// Chainable setter for [`Self::correlation_id`].
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Chainable setter for [`Self::saga_id`].
    pub fn with_saga_id(mut self, id: Uuid) -> Self {
        self.saga_id = Some(id);
        self
    }

    /// Chainable setter for [`Self::event_id`].
    pub fn with_event_id(mut self, id: Uuid) -> Self {
        self.event_id = Some(id);
        self
    }

    /// Chainable setter for [`Self::user_id`].
    pub fn with_user_id(mut self, id: Uuid) -> Self {
        self.user_id = Some(id);
        self
    }

    /// Chainable setter for [`Self::tenant_id`].
    pub fn with_tenant_id(mut self, t: impl Into<String>) -> Self {
        self.tenant_id = Some(t.into());
        self
    }

    /// Chainable setter for [`Self::deadline`].
    pub fn with_deadline(mut self, dl: DateTime<Utc>) -> Self {
        self.deadline = Some(dl);
        self
    }

    /// Chainable setter for [`Self::retry_count`].
    pub fn with_retry_count(mut self, n: u32) -> Self {
        self.retry_count = n;
        self
    }

    /// Chainable setter for [`Self::idempotency_key`].
    pub fn with_idempotency_key(mut self, k: impl Into<String>) -> Self {
        self.idempotency_key = Some(k.into());
        self
    }
}

/// A caller's invocation request to the Gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationRequest {
    /// Target function id.
    pub function_id: String,
    /// Target version; `None` means "latest Active".
    pub version: Option<String>,
    /// JSON input payload.
    pub input: serde_json::Value,
    /// Per-call context (request id, trace id, ...).
    pub context: FunctionContext,
    /// Free-form bag for mock extensions (not persisted).
    pub extra: HashMap<String, serde_json::Value>,
}

/// Outcome of a successful or failed invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationResult {
    /// Resolved function id.
    pub function_id: String,
    /// Resolved version.
    pub version: String,
    /// Function output (JSON). `None` on failure.
    pub output: Option<serde_json::Value>,
    /// Wasmtime fuel consumed (mock: returns configured limit for success).
    pub fuel_consumed: Option<u64>,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
    /// `true` when the function returned normally.
    pub success: bool,
}

// =============================================================================
// coc.policy decision contract (per RGS-INC-001 v0.3 §X.2 决策契约)
// =============================================================================
//
// Phase 0 mock POC: 三态决策 (Allow / RequireSecondReview / Deny) + params_hash
// (SHA-256 of serialized input) for 决策可重放 (per §X.6 护栏 7). Decision logic
// is intentionally trivial (高额 grant + 黑名单). Phase 1+ will replace this
// with full `WasmHost::call(name, json_input)` API and extend the WASM module
// to consult `host_query_db` / `host_get_state` per §8.3 / §X.3.

/// Three-state decision returned by the `coc.policy` WASM module
/// (per RGS-INC-001 v0.3 §X.2 决策 schema).
///
/// 三态语义锁定 (per §X.2):
/// - `Allow`              = 走 Rust 现有 audit_log 落库路径
/// - `RequireSecondReview` = 写 `second_review` 表 + NATS `rgs.ad.review.requested`
///                          异步通知 SuperAdmin，**不**立即执行操作
/// - `Deny`               = 写 audit_log (decision=denied) + 返 `permission_denied`，
///                          **不**写 `second_review`，**不**执行操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CocDecision {
    /// 直接执行（走 Rust 现有 audit_log 落库路径）
    Allow,
    /// 需 SuperAdmin 二审（写 second_review 表 + 异步通知）
    RequireSecondReview,
    /// 拒绝（写 audit_log decision=denied + 返 permission_denied）
    Deny,
}

impl CocDecision {
    /// WAT `compute(a, b) -> i32` decision code encoding used by the Phase 0
    /// mock (per RGS-INC-001 v0.3 §X.1 line 323 `WasmHost.call`):
    /// - 0 = Allow
    /// - 1 = RequireSecondReview
    /// - 2 = Deny
    ///
    /// Phase 1+ will replace this with a real JSON-typed ABI; the encoding
    /// is documented in [`CocDecision::from_wat_code`] for forward symmetry.
    pub fn to_wat_code(self) -> i32 {
        match self {
            Self::Allow => 0,
            Self::RequireSecondReview => 1,
            Self::Deny => 2,
        }
    }

    /// Decode WAT `compute` return code back to a [`CocDecision`].
    /// Returns `None` for any other value (out-of-range or future codes).
    pub fn from_wat_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Allow),
            1 => Some(Self::RequireSecondReview),
            2 => Some(Self::Deny),
            _ => None,
        }
    }

    /// `lowercase` JSON name, mirrors `serde(rename_all = "snake_case")`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::RequireSecondReview => "require_second_review",
            Self::Deny => "deny",
        }
    }
}

/// Input to the `coc.policy` WASM module (per RGS-INC-001 v0.3 §X.2 决策契约).
///
/// Mirrors the gm_handlers `ban_account` line 83-89 解析路径:
/// - `actor_id`  = 操作者 admin_id
/// - `action`    = "player.ban" / "player.unban" / "economy.grant" / ...
/// - `target_id` = account_id / 设备 id / ...
/// - `context`   = 玩家最近 N 次封禁 / 操作时间 / 金额 / ...
/// - `trace_id`  = OTel trace_id (per §3.3 透传)
///
/// POC 字段增量 (per §X.6 简化 POC 决策逻辑, §X.3 集成设计 P0 阶段):
/// - `amount`            高额 grant 检测 (> 1000 货币 → RequireSecondReview)
/// - `target_blacklisted` 黑名单检测 (= true → Deny，黑名单优先级 > 高额)
///
/// Phase 1+ 真实决策逻辑会扩展到完整 `host_query_db` + `host_get_state` 调用
/// (per §8.3 capability 白名单 + §X.3 集成设计).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocPolicyInput {
    /// 操作者 admin_id (gm_handlers line 83 解析)
    pub actor_id: Uuid,
    /// GM action 字符串 (e.g. "player.ban" / "player.unban" / "economy.grant")
    pub action: String,
    /// 目标 account_id / 设备 id
    pub target_id: String,
    /// 决策上下文（玩家最近 N 次封禁 / 操作时间 / 金额 / ...）
    pub context: serde_json::Value,
    /// OTel trace_id (per §3.3 透传)
    pub trace_id: String,
    /// POC 字段：grant 类操作的货币金额；其他 action 填 0
    pub amount: i64,
    /// POC 字段：target_id 是否在 admin-service 黑名单
    pub target_blacklisted: bool,
}

impl CocPolicyInput {
    /// Compute the canonical `params_hash = SHA-256(serde_json::to_vec(self))` hex.
    ///
    /// Per RGS-INC-001 v0.3 §X.4 / §X.5 + §X.6 护栏 7 (决策可重放), this hash
    /// is the decision-replay anchor: every `audit_log` / `second_review` row
    /// carries `coc_params_hash = <this>`, so out-of-band replay of the
    /// decision logic is a byte-exact reproduction of the original call.
    ///
    /// Stability note: callers MUST pass a JSON-stable `context` value (e.g.
    /// primitive, ordered struct, or pre-serialized `Value`). `serde_json`
    /// preserves insertion order for the `Map` variant in `serde_json::Value`,
    /// so building a `Value` via `json!{{}}` / `Value::Object` (BTreeMap
    /// underlying) is stable across runs.
    pub fn params_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let bytes = serde_json::to_vec(self).expect("CocPolicyInput is always serializable");
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        hex::encode(hasher.finalize())
    }
}

/// Output of the `coc.policy` WASM module (per RGS-INC-001 v0.3 §X.2 决策契约).
///
/// 三态决策 + 决策理由 + 4 字段审计锚 (per §X.6 护栏 7 决策可重放):
/// - `module_version` / `module_hash` / `params_hash` 全落 `audit_log` /
///   `second_review` 表 (per §X.5 second_review.coc_module_version /
///   coc_module_hash / coc_params_hash)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocPolicyOutput {
    /// 决策 (3 态: Allow / RequireSecondReview / Deny)
    pub decision: CocDecision,
    /// 决策理由 (落 audit_log 用)
    pub reason: String,
    /// 当前加载 module 版本 (per §X.5 second_review.coc_module_version)
    pub module_version: String,
    /// 当前加载 module SHA-256 (per §X.4 Registry SHA-256 校验 + §X.5 second_review.coc_module_hash)
    pub module_hash: String,
    /// input 序列化 SHA-256 (per §X.4 决策可重放 + §X.5 second_review.coc_params_hash)
    pub params_hash: String,
}

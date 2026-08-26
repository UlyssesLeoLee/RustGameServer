//! NewRealm 操作器占位（per RGS-SPEC-DTL-042 §3.1）
//!
//! 真实业务实化属于 WF-1-2066（M-2066.4）。
//! 本 worktree 仅提供 trait 实现桩（满足 6 操作器 trait 签名约束）+ 单测可注入点。

use async_trait::async_trait;
use uuid::Uuid;

use crate::realm_lifecycle::error::Result;
use crate::realm_lifecycle::service::{LcmOperatorInput, LcmOperatorOutput, NewRealmOperator};

/// NewRealm 操作器桩（per WF-1-2067 PREREQ：trait 签名占位）
///
/// 真实实现由 WF-1-2066 完成。本桩保证：
/// - 满足 `NewRealmOperator` trait 约束（async fn open/reverse）
/// - SagaOrchestrator 可持有 `Arc<dyn NewRealmOperator>`
/// - UT 可注入成功 / 失败场景
pub struct StubNewRealm;

#[async_trait]
impl NewRealmOperator for StubNewRealm {
    async fn open(&self, _input: LcmOperatorInput) -> Result<LcmOperatorOutput> {
        unimplemented!("NewRealmOperator::open 实化属于 WF-1-2066 M-2066.4")
    }

    async fn reverse(&self, _resource_id: Uuid, _reason: String) -> Result<()> {
        unimplemented!("NewRealmOperator::reverse 实化属于 WF-1-2066 M-2066.4")
    }
}

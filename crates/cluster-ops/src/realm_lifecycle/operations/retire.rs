//! Retire 操作器占位（per RGS-SPEC-DTL-042 §3.5）
//!
//! 真实业务实化属于 WF-1-2066（M-2066.8）。

use async_trait::async_trait;
use uuid::Uuid;

use crate::realm_lifecycle::error::Result;
use crate::realm_lifecycle::service::{LcmOperatorInput, LcmOperatorOutput, RetireOperator};

pub struct StubRetire;

#[async_trait]
impl RetireOperator for StubRetire {
    async fn retire(&self, _input: LcmOperatorInput) -> Result<LcmOperatorOutput> {
        unimplemented!("RetireOperator::retire 实化属于 WF-1-2066 M-2066.8")
    }

    async fn reverse(&self, _resource_id: Uuid) -> Result<()> {
        unimplemented!("RetireOperator::reverse 实化属于 WF-1-2066 M-2066.8")
    }
}

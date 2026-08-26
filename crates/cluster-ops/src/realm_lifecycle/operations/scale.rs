//! Scale 操作器占位（per RGS-SPEC-DTL-042 §3.2）
//!
//! 真实业务实化属于 WF-1-2066（M-2066.5）。

use async_trait::async_trait;
use uuid::Uuid;

use crate::realm_lifecycle::error::Result;
use crate::realm_lifecycle::service::{LcmOperatorInput, LcmOperatorOutput, ScaleOperator};

pub struct StubScale;

#[async_trait]
impl ScaleOperator for StubScale {
    async fn scale(&self, _input: LcmOperatorInput, _delta: i32) -> Result<LcmOperatorOutput> {
        unimplemented!("ScaleOperator::scale 实化属于 WF-1-2066 M-2066.5")
    }

    async fn reverse(&self, _resource_id: Uuid, _prior_replicas: i32) -> Result<()> {
        unimplemented!("ScaleOperator::reverse 实化属于 WF-1-2066 M-2066.5")
    }
}

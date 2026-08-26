//! Split 操作器占位（per RGS-SPEC-DTL-042 §3.3）
//!
//! 真实业务实化属于 WF-1-2066（M-2066.6）。

use async_trait::async_trait;
use uuid::Uuid;

use crate::realm_lifecycle::error::Result;
use crate::realm_lifecycle::service::{LcmOperatorInput, LcmOperatorOutput, SplitOperator};

pub struct StubSplit;

#[async_trait]
impl SplitOperator for StubSplit {
    async fn split(
        &self,
        _input: LcmOperatorInput,
        _target_realm_id: Uuid,
    ) -> Result<LcmOperatorOutput> {
        unimplemented!("SplitOperator::split 实化属于 WF-1-2066 M-2066.6")
    }

    async fn reverse(&self, _source_realm_id: Uuid, _child_realm_id: Uuid) -> Result<()> {
        unimplemented!("SplitOperator::reverse 实化属于 WF-1-2066 M-2066.6")
    }
}

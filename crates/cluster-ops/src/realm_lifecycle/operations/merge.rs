//! Merge + MergeRollback 操作器占位（per RGS-SPEC-DTL-042 §3.4）
//!
//! 真实业务实化属于 WF-1-2066（M-2066.7）。

use async_trait::async_trait;
use uuid::Uuid;

use crate::realm_lifecycle::error::Result;
use crate::realm_lifecycle::service::{LcmOperatorInput, LcmOperatorOutput, MergeOperator};

pub struct StubMerge;

#[async_trait]
impl MergeOperator for StubMerge {
    async fn merge(
        &self,
        _input: LcmOperatorInput,
        _target_realm_id: Uuid,
        _source_realm_ids: Vec<Uuid>,
    ) -> Result<LcmOperatorOutput> {
        unimplemented!("MergeOperator::merge 实化属于 WF-1-2066 M-2066.7")
    }

    async fn reverse(
        &self,
        _target_realm_id: Uuid,
        _source_realm_ids: Vec<Uuid>,
    ) -> Result<()> {
        unimplemented!("MergeOperator::reverse 实化属于 WF-1-2066 M-2066.7")
    }

    async fn rollback(&self, _target_realm_id: Uuid, _locked_at_ms: i64) -> Result<()> {
        unimplemented!("MergeOperator::rollback 实化属于 WF-1-2066 M-2066.7")
    }
}

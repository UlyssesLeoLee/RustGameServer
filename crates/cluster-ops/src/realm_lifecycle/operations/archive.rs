//! Archive 操作器占位（per RGS-SPEC-DTL-042 §3.6 + §5 冷热分层）
//!
//! 真实业务实化属于 WF-1-2066（M-2066.9）。

use async_trait::async_trait;
use uuid::Uuid;

use crate::realm_lifecycle::error::Result;
use crate::realm_lifecycle::service::{ArchiveOperator, LcmOperatorInput, LcmOperatorOutput};

pub struct StubArchive;

#[async_trait]
impl ArchiveOperator for StubArchive {
    async fn archive(&self, _input: LcmOperatorInput) -> Result<LcmOperatorOutput> {
        unimplemented!("ArchiveOperator::archive 实化属于 WF-1-2066 M-2066.9")
    }

    async fn reverse(&self, _resource_id: Uuid) -> Result<()> {
        unimplemented!("ArchiveOperator::reverse 实化属于 WF-1-2066 M-2066.9")
    }
}

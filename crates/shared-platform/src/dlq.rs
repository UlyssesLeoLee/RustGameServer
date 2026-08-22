//! DLQ 死信队列（per RGS-DTL-100 §5.3 + RGS-SPEC-CROSS-005）
//!
//! 54.10 实化：DLQ entry + 查询接口
//!
//! 设计：Consumer 超 max_retries 后转发到 rgs.dlq.<source>，由运维侧人工处理或重投。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DLQ 条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DlqEntry {
    /// 原始 subject
    pub original_subject: String,
    /// handler 名
    pub handler: String,
    /// 重试次数
    pub attempts: u32,
    /// 错误信息
    pub error: String,
    /// 业务 command_id
    pub command_id: Option<Uuid>,
    /// 业务 saga_id
    pub saga_id: Option<Uuid>,
    /// 业务 actor_id
    pub actor_id: Option<Uuid>,
    /// payload bytes (base64 in JSON)
    pub payload_base64: String,
    /// 失败时间
    pub failed_at: DateTime<Utc>,
}

impl DlqEntry {
    /// 工厂
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        original_subject: String,
        handler: String,
        attempts: u32,
        error: String,
        command_id: Option<Uuid>,
        saga_id: Option<Uuid>,
        actor_id: Option<Uuid>,
        payload: Vec<u8>,
    ) -> Self {
        use base64::Engine;
        Self {
            original_subject,
            handler,
            attempts,
            error,
            command_id,
            saga_id,
            actor_id,
            payload_base64: base64::engine::general_purpose::STANDARD.encode(&payload),
            failed_at: Utc::now(),
        }
    }

    /// 还原 payload
    pub fn decode_payload(&self) -> Vec<u8> {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(&self.payload_base64)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dlq_entry_round_trip() {
        let entry = DlqEntry::new(
            "rgs.player.registered.v1".to_string(),
            "PlayerHandler".to_string(),
            4,
            "DB timeout".to_string(),
            Some(Uuid::new_v4()),
            None,
            Some(Uuid::new_v4()),
            b"hello".to_vec(),
        );
        assert_eq!(entry.attempts, 4);
        assert_eq!(entry.decode_payload(), b"hello");
    }
}

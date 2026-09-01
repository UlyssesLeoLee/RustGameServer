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

    // ---- 9/1 pt/shared-platform worker 派工 (per PT-WORKER-BRIEFING.md §2) ----
    // DLQ 是 RGS-DTL-100 §5.3 死信队列, payload 编码 / 解码必须有守护, 加 3 单测 + 1 proptest

    #[test]
    fn dlq_entry_serde_round_trip() {
        // DlqEntry 派生 Serialize/Deserialize, JSON 往返必须保真
        let entry = DlqEntry::new(
            "rgs.economy.transferred.v1".to_string(),
            "EconomyHandler".to_string(),
            5,
            "timeout".to_string(),
            Some(Uuid::nil()),
            Some(Uuid::nil()),
            None,
            b"payload-data".to_vec(),
        );
        let json = serde_json::to_string(&entry).expect("serialize");
        let decoded: DlqEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, entry, "DLQ entry JSON round-trip 必须保真");
    }

    #[test]
    fn dlq_entry_decode_empty_payload() {
        let entry = DlqEntry::new(
            "rgs.match.ended.v1".to_string(),
            "MatchHandler".to_string(),
            1,
            "x".to_string(),
            None,
            None,
            None,
            Vec::new(),
        );
        // 空 payload: base64("") = ""; decode 应返空 Vec
        assert_eq!(entry.decode_payload(), Vec::<u8>::new());
    }

    #[test]
    fn dlq_entry_decode_invalid_base64_returns_empty() {
        // 手工构造非法 base64 字段
        let mut entry = DlqEntry::new(
            "rgs.test.x".to_string(),
            "H".to_string(),
            1,
            "x".to_string(),
            None,
            None,
            None,
            b"orig".to_vec(),
        );
        entry.payload_base64 = "!!!invalid-base64!!!".to_string();
        // decode_payload 在 base64 失败时 unwrap_or_default → 空 Vec
        assert_eq!(entry.decode_payload(), Vec::<u8>::new());
    }

    #[test]
    fn dlq_entry_preserves_all_three_ids() {
        let cmd = Some(Uuid::new_v4());
        let saga = Some(Uuid::new_v4());
        let actor = Some(Uuid::new_v4());
        let entry = DlqEntry::new(
            "rgs.x".to_string(),
            "H".to_string(),
            3,
            "err".to_string(),
            cmd,
            saga,
            actor,
            b"p".to_vec(),
        );
        assert_eq!(entry.command_id, cmd);
        assert_eq!(entry.saga_id, saga);
        assert_eq!(entry.actor_id, actor);
    }
}

// ---- 9/1 pt/shared-platform worker 派工 (per PT-WORKER-BRIEFING.md §2) ----
// DLQ payload base64 编码 proptest
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// 任意字节 payload → base64 encode → decode 必须严格保真
    proptest! {
        #[test]
        fn dlq_payload_round_trip(
            payload in proptest::collection::vec(any::<u8>(), 0..256),
        ) {
            let entry = DlqEntry::new(
                "rgs.test.x".to_string(),
                "TestHandler".to_string(),
                1,
                "test".to_string(),
                None, None, None,
                payload.clone(),
            );
            prop_assert_eq!(entry.decode_payload(), payload);
        }
    }
}

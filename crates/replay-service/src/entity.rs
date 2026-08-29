//! replay-service 域 entity 定义
//!
//! 桶 13 (replay) 实装 (per RGS-DTL-038 §3 DEC-038-03 + §7.1 #7):
//! - **ReplayMeta**: 回放元数据 (per DTL-038 §7.1 replays 表)
//! - **ReplayMode**: 模式枚举 (与 common.proto GameMode 数值对应)
//! - **ReplayFilter**: ListReplays 过滤条件
//!
//! 业务约束：
//! - 元数据存 PostgreSQL (replays 表)
//! - 回放数据存对象存储 (cluster-ops S3-兼容, LocalFs mock)
//! - 生命周期: 天梯 90d / 休闲 7d / 房间 30d (custom_ttl_secs 可覆盖)
//! - 跨域引用: match_id / player_a / player_b 不物化 FK (per ARC-008 5 独立 DB)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Result;

// ============================================================================
// 枚举 (与 proto v1 ReplayMode 数值一一对应, 0=unspecified 1=ranked 2=casual 3=room 4=pve_ai)
// ============================================================================

/// 回放模式 (与 common.proto GameMode + replay.proto ReplayMode 数值一一对应)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ReplayMode {
    /// 0 = 未指定
    Unspecified = 0,
    /// 1 = 天梯
    Ranked = 1,
    /// 2 = 休闲
    Casual = 2,
    /// 3 = 房间
    Room = 3,
    /// 4 = PvE AI
    PveAi = 4,
}

impl ReplayMode {
    /// proto int32 -> enum (Unknown -> Unspecified)
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => ReplayMode::Ranked,
            2 => ReplayMode::Casual,
            3 => ReplayMode::Room,
            4 => ReplayMode::PveAi,
            _ => ReplayMode::Unspecified,
        }
    }

    /// enum -> proto int32
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 业务展示名
    pub fn display_name(&self) -> &'static str {
        match self {
            ReplayMode::Unspecified => "Unspecified",
            ReplayMode::Ranked => "Ranked",
            ReplayMode::Casual => "Casual",
            ReplayMode::Room => "Room",
            ReplayMode::PveAi => "PvE_AI",
        }
    }

    /// 默认 TTL (秒) — per 任务书: 天梯 90d / 休闲 7d / 房间 30d
    /// - Ranked: 90 天 = 7,776,000 秒
    /// - Casual: 7 天 = 604,800 秒
    /// - Room: 30 天 = 2,592,000 秒
    /// - PveAi / Unspecified: 30 天 (fallback)
    pub fn default_ttl_secs(&self) -> i64 {
        const DAY: i64 = 24 * 60 * 60;
        match self {
            ReplayMode::Ranked => 90 * DAY,
            ReplayMode::Casual => 7 * DAY,
            ReplayMode::Room => 30 * DAY,
            ReplayMode::PveAi => 30 * DAY,
            ReplayMode::Unspecified => 30 * DAY,
        }
    }
}

impl Default for ReplayMode {
    fn default() -> Self {
        ReplayMode::Casual
    }
}

// ============================================================================
// ReplayMeta (per DTL-038 §7.1 #7 replays 表)
// ============================================================================

/// 回放元数据 (per RGS-DTL-038 §7.1 #7 replays 表)
///
/// 业务含义:
/// - replay_id: UUID, primary key
/// - match_id: 关联 game_sessions.match_id (跨域引用, 不物化 FK)
/// - player_a / player_b: 玩家 ID (UUID 字符串, 不物化 FK)
/// - mode: 模式 (天梯 / 休闲 / 房间 / PvE)
/// - object_key: 对象存储 key (e.g. "replays/2026/08/rp-{uuid}.dat")
/// - object_size: 对象大小 (bytes, 0 = 未知)
/// - duration_secs: 比赛时长 (秒)
/// - created_at / expires_at: 生命周期
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayMeta {
    pub replay_id: Uuid,
    pub match_id: Uuid,
    pub player_a: String,
    pub player_b: Option<String>,
    pub mode: ReplayMode,
    pub object_key: String,
    pub object_size: i64,
    pub duration_secs: u32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl ReplayMeta {
    /// 工厂: 新建回放元数据 (UUID v4, 默认 TTL per mode)
    pub fn new(
        match_id: Uuid,
        player_a: String,
        player_b: Option<String>,
        mode: ReplayMode,
        object_key: String,
    ) -> Self {
        let now = Utc::now();
        let ttl = mode.default_ttl_secs();
        Self {
            replay_id: Uuid::new_v4(),
            match_id,
            player_a,
            player_b,
            mode,
            object_key,
            object_size: 0,
            duration_secs: 0,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(ttl),
        }
    }

    /// 自定义 TTL (per SaveReplayRequest.custom_ttl_secs > 0 时调用)
    pub fn with_custom_ttl(mut self, ttl_secs: i64) -> Self {
        self.expires_at = self.created_at + chrono::Duration::seconds(ttl_secs);
        self
    }

    /// 是否已过期 (per expires_at < now)
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// 业务校验: 必要字段非空
    pub fn validate(&self) -> Result<()> {
        if self.player_a.is_empty() {
            return Err(crate::Error::Validation(
                "player_a must not be empty".to_string(),
            ));
        }
        if self.object_key.is_empty() {
            return Err(crate::Error::Validation(
                "object_key must not be empty".to_string(),
            ));
        }
        if self.match_id.is_nil() {
            return Err(crate::Error::Validation(
                "match_id must not be nil UUID".to_string(),
            ));
        }
        Ok(())
    }
}

// ============================================================================
// 完整 Replay (元数据 + 数据 bytes, GetReplay 一次性返回)
// ============================================================================

/// 完整回放 (元数据 + 数据 bytes, GetReplay 响应)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Replay {
    pub meta: ReplayMeta,
    pub data: Vec<u8>,
}

impl Replay {
    /// 工厂: 新建完整回放
    pub fn new(meta: ReplayMeta, data: Vec<u8>) -> Self {
        let mut m = meta;
        m.object_size = data.len() as i64;
        Self { meta: m, data }
    }
}

// ============================================================================
// 过滤 (per replay.proto ListReplaysRequest)
// ============================================================================

/// Replay 列表过滤 (per replay.proto ListReplaysRequest)
#[derive(Debug, Clone, Default)]
pub struct ReplayFilter {
    /// 按 player_a 过滤
    pub player_a_filter: Option<String>,
    /// 按 player_b 过滤
    pub player_b_filter: Option<String>,
    /// 按 mode 过滤
    pub mode_filter: Option<ReplayMode>,
    /// 是否包含已过期 (false = 仅 hot, 默认 true 便于调试)
    pub include_expired: bool,
}

// ============================================================================
// 流式读取 chunk (per replay.proto ReplayChunk)
// ============================================================================

/// 流式读取 chunk (per ReplayChunk proto)
#[derive(Debug, Clone)]
pub struct ReplayChunk {
    pub replay_id: Uuid,
    pub offset: u64,
    pub payload: Vec<u8>,
    pub is_last: bool,
    pub chunk_index: u32,
}

impl ReplayChunk {
    /// 工厂: 新建 chunk
    pub fn new(
        replay_id: Uuid,
        offset: u64,
        payload: Vec<u8>,
        is_last: bool,
        chunk_index: u32,
    ) -> Self {
        Self {
            replay_id,
            offset,
            payload,
            is_last,
            chunk_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_mode_roundtrip() {
        for m in [
            ReplayMode::Ranked,
            ReplayMode::Casual,
            ReplayMode::Room,
            ReplayMode::PveAi,
        ] {
            assert_eq!(ReplayMode::from_i32(m.as_i32()), m);
        }
        assert_eq!(ReplayMode::from_i32(99), ReplayMode::Unspecified);
    }

    #[test]
    fn replay_mode_default_ttl() {
        // 天梯 90d / 休闲 7d / 房间 30d / PvE 30d
        const DAY: i64 = 24 * 60 * 60;
        assert_eq!(ReplayMode::Ranked.default_ttl_secs(), 90 * DAY);
        assert_eq!(ReplayMode::Casual.default_ttl_secs(), 7 * DAY);
        assert_eq!(ReplayMode::Room.default_ttl_secs(), 30 * DAY);
        assert_eq!(ReplayMode::PveAi.default_ttl_secs(), 30 * DAY);
    }

    #[test]
    fn replay_meta_factory_initializes_uuid_and_ttl() {
        let match_id = Uuid::new_v4();
        let m = ReplayMeta::new(
            match_id,
            "player-a-uuid".to_string(),
            Some("player-b-uuid".to_string()),
            ReplayMode::Ranked,
            "replays/rp-1.dat".to_string(),
        );
        assert_eq!(m.match_id, match_id);
        assert_eq!(m.mode, ReplayMode::Ranked);
        assert_eq!(m.object_size, 0);
        assert_eq!(m.duration_secs, 0);
        // expires_at - created_at 应 = 90 天
        let diff = (m.expires_at - m.created_at).num_seconds();
        assert_eq!(diff, 90 * 24 * 60 * 60);
    }

    #[test]
    fn replay_meta_with_custom_ttl() {
        let m = ReplayMeta::new(
            Uuid::new_v4(),
            "p".to_string(),
            None,
            ReplayMode::Casual,
            "k".to_string(),
        )
        .with_custom_ttl(3600);
        let diff = (m.expires_at - m.created_at).num_seconds();
        assert_eq!(diff, 3600);
    }

    #[test]
    fn replay_meta_is_expired() {
        let mut m = ReplayMeta::new(
            Uuid::new_v4(),
            "p".to_string(),
            None,
            ReplayMode::Casual,
            "k".to_string(),
        );
        assert!(!m.is_expired());
        // 模拟过期
        m.expires_at = Utc::now() - chrono::Duration::seconds(1);
        assert!(m.is_expired());
    }

    #[test]
    fn replay_meta_validate_rejects_empty_player() {
        let m = ReplayMeta::new(
            Uuid::new_v4(),
            String::new(),
            None,
            ReplayMode::Casual,
            "k".to_string(),
        );
        assert!(m.validate().is_err());
    }

    #[test]
    fn replay_meta_validate_rejects_empty_object_key() {
        let m = ReplayMeta::new(
            Uuid::new_v4(),
            "p".to_string(),
            None,
            ReplayMode::Casual,
            String::new(),
        );
        assert!(m.validate().is_err());
    }

    #[test]
    fn replay_meta_validate_rejects_nil_match_id() {
        let m = ReplayMeta::new(
            Uuid::nil(),
            "p".to_string(),
            None,
            ReplayMode::Casual,
            "k".to_string(),
        );
        assert!(m.validate().is_err());
    }

    #[test]
    fn replay_meta_validate_ok() {
        let m = ReplayMeta::new(
            Uuid::new_v4(),
            "p".to_string(),
            Some("q".to_string()),
            ReplayMode::Ranked,
            "k".to_string(),
        );
        assert!(m.validate().is_ok());
    }

    #[test]
    fn replay_factory_sets_object_size() {
        let m = ReplayMeta::new(
            Uuid::new_v4(),
            "p".to_string(),
            None,
            ReplayMode::Casual,
            "k".to_string(),
        );
        let r = Replay::new(m, vec![1, 2, 3, 4, 5]);
        assert_eq!(r.meta.object_size, 5);
        assert_eq!(r.data, vec![1, 2, 3, 4, 5]);
    }
}

//! replay-service 域业务函数单元测试 (per PT-WORKER-BRIEFING §2 必做)
//!
//! 5 UT 覆盖:
//! 1. replay_mode_default_ttl (Ranked 90d / Casual 7d / Room 30d / PveAi 30d)
//! 2. replay_meta_validate_rejects_nil_match_id
//! 3. replay_meta_with_custom_ttl_overrides_default
//! 4. replay_meta_is_expired_boundary
//! 5. replay_factory_sets_object_size

use chrono::Utc;
use replay_service::entity::{Replay, ReplayMeta, ReplayMode};
use uuid::Uuid;

#[test]
fn replay_mode_default_ttl() {
    const DAY: i64 = 24 * 60 * 60;
    assert_eq!(ReplayMode::Ranked.default_ttl_secs(), 90 * DAY);
    assert_eq!(ReplayMode::Casual.default_ttl_secs(), 7 * DAY);
    assert_eq!(ReplayMode::Room.default_ttl_secs(), 30 * DAY);
    assert_eq!(ReplayMode::PveAi.default_ttl_secs(), 30 * DAY);
}

#[test]
fn replay_meta_validate_rejects_nil_match_id() {
    let m = ReplayMeta::new(Uuid::nil(), "p".into(), None, ReplayMode::Casual, "k".into());
    assert!(m.validate().is_err());
}

#[test]
fn replay_meta_with_custom_ttl_overrides_default() {
    let m = ReplayMeta::new(Uuid::new_v4(), "p".into(), None, ReplayMode::Ranked, "k".into())
        .with_custom_ttl(3600);
    let diff = (m.expires_at - m.created_at).num_seconds();
    assert_eq!(diff, 3600);
    // 覆写后不再受 Ranked 90d 影响
    assert_ne!(diff, 90 * 24 * 60 * 60);
}

#[test]
fn replay_meta_is_expired_boundary() {
    let mut m = ReplayMeta::new(Uuid::new_v4(), "p".into(), None, ReplayMode::Casual, "k".into());
    assert!(!m.is_expired());
    m.expires_at = Utc::now() - chrono::Duration::seconds(1);
    assert!(m.is_expired());
}

#[test]
fn replay_factory_sets_object_size() {
    let m = ReplayMeta::new(Uuid::new_v4(), "p".into(), None, ReplayMode::Casual, "k".into());
    let r = Replay::new(m, vec![1, 2, 3, 4, 5]);
    assert_eq!(r.meta.object_size, 5);
    assert_eq!(r.data, vec![1, 2, 3, 4, 5]);
}

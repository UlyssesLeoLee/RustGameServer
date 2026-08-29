//! replay-service v1 proto UT (per RGS-DTL-038 §3 DEC-038-03)
//!
//! 5 UT 验证核心 message / enum 字段:
//! 1. test_replay_meta_basic_fields
//! 2. test_replay_meta_object_size_signed
//! 3. test_replay_mode_enum_values
//! 4. test_replay_chunk_is_last
//! 5. test_replay_list_page_response

use replay_service::proto::v1::*;
use replay_service::common::v1 as common;

#[test]
fn test_replay_meta_basic_fields() {
    let m = ReplayMeta {
        replay_id: "11111111-1111-1111-1111-111111111111".to_string(),
        match_id: "22222222-2222-2222-2222-222222222222".to_string(),
        player_a: "player-a-uuid".to_string(),
        player_b: "player-b-uuid".to_string(),
        mode: ReplayMode::Ranked as i32,
        object_key: "replays/2026/08/rp-1.dat".to_string(),
        object_size: 1024,
        duration_secs: 600,
        created_at: Some(common::Timestamp { seconds: 1700000000, nanos: 0 }),
        expires_at: Some(common::Timestamp { seconds: 1702592000, nanos: 0 }),
    };
    assert_eq!(m.replay_id, "11111111-1111-1111-1111-111111111111");
    assert_eq!(m.player_a, "player-a-uuid");
    assert_eq!(m.player_b, "player-b-uuid".to_string());
    assert_eq!(m.mode, ReplayMode::Ranked as i32);
    assert_eq!(m.object_size, 1024);
    assert_eq!(m.duration_secs, 600);
}

#[test]
fn test_replay_meta_object_size_signed() {
    // object_size 是 int64 (sized 防止溢出, 单 replay 可上 GB)
    let m = ReplayMeta {
        replay_id: String::new(),
        match_id: String::new(),
        player_a: String::new(),
        player_b: String::new(),
        mode: 0,
        object_key: String::new(),
        object_size: 5 * 1024 * 1024 * 1024, // 5 GB
        duration_secs: 0,
        created_at: None,
        expires_at: None,
    };
    assert_eq!(m.object_size, 5 * 1024 * 1024 * 1024);
}

#[test]
fn test_replay_mode_enum_values() {
    // 验证 enum 数值与 common.proto GameMode 对齐: 0=unspecified 1=ranked 2=casual 3=room 4=pve_ai
    assert_eq!(ReplayMode::Unspecified as i32, 0);
    assert_eq!(ReplayMode::Ranked as i32, 1);
    assert_eq!(ReplayMode::Casual as i32, 2);
    assert_eq!(ReplayMode::Room as i32, 3);
    assert_eq!(ReplayMode::PveAi as i32, 4);
}

#[test]
fn test_replay_chunk_is_last() {
    let c = ReplayChunk {
        replay_id: "id".to_string(),
        offset: 1024,
        payload: vec![1, 2, 3, 4, 5],
        is_last: true,
        chunk_index: 7,
    };
    assert_eq!(c.replay_id, "id");
    assert_eq!(c.offset, 1024);
    assert_eq!(c.payload.len(), 5);
    assert!(c.is_last);
    assert_eq!(c.chunk_index, 7);
}

#[test]
fn test_replay_list_page_response() {
    let list = ReplayList {
        items: vec![],
        page: Some(common::PageResponse {
            total: 42,
            has_next: true,
            next_cursor: "2".to_string(),
        }),
    };
    assert_eq!(list.items.len(), 0);
    let page = list.page.as_ref().unwrap();
    assert_eq!(page.total, 42);
    assert!(page.has_next);
    assert_eq!(page.next_cursor, "2");
}

#[test]
fn test_save_replay_request_default_fields() {
    let req = SaveReplayRequest {
        request_id: "req-1".to_string(),
        match_id: "match-1".to_string(),
        player_a: "pa".to_string(),
        player_b: "pb".to_string(),
        mode: ReplayMode::Casual as i32,
        data: vec![0xde, 0xad, 0xbe, 0xef],
        duration_secs: 120,
        custom_ttl_secs: 0,
        saga_id: String::new(),
    };
    assert_eq!(req.player_a, "pa");
    assert_eq!(req.duration_secs, 120);
    assert_eq!(req.data, vec![0xde, 0xad, 0xbe, 0xef]);
    // custom_ttl_secs=0 表示用 mode 默认 (Casual 7d)
    assert_eq!(req.custom_ttl_secs, 0);
}

#[test]
fn test_list_replays_request_include_expired() {
    let req = ListReplaysRequest {
        request_id: "r".to_string(),
        player_a_filter: "p".to_string(),
        player_b_filter: String::new(),
        mode_filter: ReplayMode::Ranked as i32,
        include_expired: true,
        page: None,
    };
    assert!(req.include_expired);
    assert_eq!(req.player_a_filter, "p");
    assert_eq!(req.mode_filter, ReplayMode::Ranked as i32);
}

#[test]
fn test_stream_replay_request_chunk_size() {
    let req = StreamReplayRequest {
        request_id: "r".to_string(),
        replay_id: "rp".to_string(),
        chunk_size: 64 * 1024,
        start_offset: 0,
    };
    assert_eq!(req.chunk_size, 64 * 1024);
    assert_eq!(req.start_offset, 0);
}

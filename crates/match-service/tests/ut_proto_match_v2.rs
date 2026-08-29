//! match-service v2 proto UT (per RGS-DTL-038 §4.2)
//!
//! 1 UT (简化版, 验证 match v2 字段):

use match_service::proto::v1::*;
use match_service::common::v1 as common;

#[test]
fn test_match_v2_fields() {
    // Match v2 扩展: mode / players / board_snapshot_ref / turn_index
    let m = Match {
        id: Some(common::EntityId { id: "match_001".to_string() }),
        status: common::Status::Ok as i32,
        created_at: Some(common::Timestamp { seconds: 1700000000, nanos: 0 }),
        display_name: "Match1".to_string(),
        mode: common::GameMode::Ranked as i32,
        players: vec![],
        board_snapshot_ref: "obj://board/abc".to_string(),
        turn_index: 3,
    };
    assert_eq!(m.id.as_ref().unwrap().id, "match_001");
    assert_eq!(m.mode, common::GameMode::Ranked as i32);
    assert_eq!(m.turn_index, 3);
    assert_eq!(m.board_snapshot_ref, "obj://board/abc");
    assert_eq!(m.display_name, "Match1");
}

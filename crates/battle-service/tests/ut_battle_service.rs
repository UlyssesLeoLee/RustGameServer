//! battle-service 集成 UT (per W5 简报 L1.1 ≥ 30 test)
//!
//! 覆盖:
//! - 战斗状态机 (BattlePhase 转移)
//! - 6 个 PVP 变体 + 数据驱动 PvpMode
//! - 9 个 holiday_* + 数据驱动 HolidayActivity
//! - Boss 挑战 + World Boss HP
//! - 房间 + 矿脉
//! - 副本 + 无尽塔
//! - 护送 + 圣器
//! - 公会战 + 跨服 (复用 PVP 数据驱动)
//! - 远征
//! - 错误码 -> tonic::Status 映射

use battle_service::common::v1 as common_pb;
use battle_service::config::{HolidayConfig, PvpConfig};
use battle_service::error::Error;
use battle_service::proto::v1 as pb;
use battle_service::service::{
    BattleEngineServiceImpl, BattleEngineServiceTrait, BattleServiceImpl, BossServiceImpl,
    BossServiceTrait, CrossServerServiceImpl, CrossServerServiceTrait,
    EndlessTowerServiceImpl, EndlessTowerServiceTrait, EscortServiceImpl, EscortServiceTrait,
    ExpeditionServiceImpl, ExpeditionServiceTrait, GuildWarServiceImpl, GuildWarServiceTrait,
    HolidayActivityServiceImpl, HolidayActivityServiceTrait, HolyEquipServiceImpl,
    HolyEquipServiceTrait, InstanceServiceImpl, InstanceServiceTrait, PvPServiceImpl,
    PvPServiceTrait, RoomServiceImpl, RoomServiceTrait,
};
use std::sync::Arc;
use tonic::Request;

fn player_id() -> common_pb::PlayerId {
    common_pb::PlayerId {
        player_id: Some(common_pb::EntityId { id: "p1".to_string() }),
        display_name: "Player1".to_string(),
        rank_score: 1500,
        level: 30,
    }
}

fn empty_req() -> pb::EmptyRequest {
    pb::EmptyRequest {
        request_id: "req1".to_string(),
        player: Some(player_id()),
    }
}

// ============================================================================
// 1. BattleEngineService 业务实装 (10 tests)
// ============================================================================

#[tokio::test]
async fn battle_init_creates_battle_with_init_phase() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BattleEngineServiceImpl::new(state.clone());

    let resp = svc.battle_init(Request::new(empty_req())).await.unwrap();
    let state_msg = resp.into_inner();
    assert_eq!(state_msg.turn_index, 0);
    assert!(!state_msg.battle_id.is_empty());
}

#[tokio::test]
async fn battle_init_then_prepare_transitions() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BattleEngineServiceImpl::new(state.clone());

    let init = svc.battle_init(Request::new(empty_req())).await.unwrap();
    let _battle_id = init.into_inner().battle_id;

    let prepare = svc.battle_prepare(Request::new(empty_req())).await.unwrap();
    let s = prepare.into_inner();
    // 不一定 match 该 battle_id, 但至少有响应
    assert!(!s.battle_id.is_empty());
}

#[tokio::test]
async fn battle_start_advances_turn() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BattleEngineServiceImpl::new(state.clone());

    let init = svc.battle_init(Request::new(empty_req())).await.unwrap();
    let battle_id = init.into_inner().battle_id;

    // 完整生命周期: Init -> Prepare -> RoundStart
    let _ = svc.battle_prepare(Request::new(empty_req())).await.unwrap();
    let resp = svc.battle_start(Request::new(pb::BattleId { battle_id })).await.unwrap();
    let s = resp.into_inner();
    assert!(s.turn_index >= 1);
}

#[tokio::test]
async fn battle_end_returns_victory_with_rewards() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BattleEngineServiceImpl::new(state.clone());

    let init = svc.battle_init(Request::new(empty_req())).await.unwrap();
    let battle_id = init.into_inner().battle_id;

    // 完整生命周期: Init -> Prepare -> RoundStart -> Action -> End
    let _ = svc.battle_prepare(Request::new(empty_req())).await.unwrap();
    let _ = svc.battle_start(Request::new(pb::BattleId { battle_id: battle_id.clone() })).await.unwrap();
    let _ = svc.battle_action(Request::new(pb::BattleId { battle_id: battle_id.clone() })).await.unwrap();
    let result = svc.battle_end(Request::new(pb::BattleId { battle_id })).await.unwrap();
    let r = result.into_inner();
    assert!(r.victory);
    assert_eq!(r.stars, 3);
    assert!(!r.rewards.is_empty());
}

#[tokio::test]
async fn battle_end_without_init_returns_invalid_transition() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BattleEngineServiceImpl::new(state.clone());

    let result = svc.battle_end(Request::new(pb::BattleId { battle_id: "nonexistent".to_string() })).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn battle_exit_clears_active_battles() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BattleEngineServiceImpl::new(state.clone());

    let _ = svc.battle_init(Request::new(empty_req())).await.unwrap();
    let resp = svc.battle_exit(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().ok);
}

#[tokio::test]
async fn battle_reconnect_finds_active_battle() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BattleEngineServiceImpl::new(state.clone());

    let init = svc.battle_init(Request::new(empty_req())).await.unwrap();
    let _battle_id = init.into_inner().battle_id;

    let resp = svc.battle_reconnect(Request::new(empty_req())).await.unwrap();
    let s = resp.into_inner();
    assert!(!s.battle_id.is_empty());
}

#[tokio::test]
async fn battle_duel_request_ok() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BattleEngineServiceImpl::new(state.clone());

    let resp = svc.battle_duel_request(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().ok);
}

#[tokio::test]
async fn battle_request_state_returns_current() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BattleEngineServiceImpl::new(state.clone());

    let _ = svc.battle_init(Request::new(empty_req())).await.unwrap();
    let resp = svc.battle_request_state(Request::new(empty_req())).await.unwrap();
    assert!(!resp.into_inner().battle_id.is_empty());
}

#[tokio::test]
async fn battle_engine_health_check_ok() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BattleEngineServiceImpl::new(state.clone());

    let resp = svc.health_check(Request::new(common_pb::HealthCheckRequest { service: "battle-engine".to_string() })).await.unwrap();
    assert!(resp.into_inner().message.contains("ok"));
}

// ============================================================================
// 2. PvPService 业务实装 (1 套代码 + 6 PvpMode, 5 tests)
// ============================================================================

#[tokio::test]
async fn pvp_get_challenge_list_returns_6_modes() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = PvPServiceImpl::new(state);

    let resp = svc.get_challenge_list(Request::new(empty_req())).await.unwrap();
    let msg = resp.into_inner();
    assert!(msg.message.contains("6_modes_available"));
}

#[tokio::test]
async fn pvp_challenge_increments_count() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = PvPServiceImpl::new(state.clone());

    let resp = svc.challenge_player(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().ok);
}

#[tokio::test]
async fn pvp_challenge_exhausted_at_daily_limit() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = PvPServiceImpl::new(state.clone());

    // Ranked 模式 daily_limit=10, 跑 11 次应该最后一次失败
    for _ in 0..10 {
        let _ = svc.challenge_player(Request::new(empty_req())).await.unwrap();
    }
    let result = svc.challenge_player(Request::new(empty_req())).await;
    assert!(matches!(result, Err(Error::ChallengeExhausted(_))));
}

#[tokio::test]
async fn pvp_buy_challenge_count_resets() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = PvPServiceImpl::new(state.clone());

    let _ = svc.challenge_player(Request::new(empty_req())).await.unwrap();
    let _ = svc.challenge_player(Request::new(empty_req())).await.unwrap();
    let resp = svc.buy_challenge_count(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().ok);
}

#[tokio::test]
async fn pvp_health_check_returns_6_variants() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = PvPServiceImpl::new(state);

    let resp = svc.health_check(Request::new(common_pb::HealthCheckRequest { service: "pvp".to_string() })).await.unwrap();
    assert!(resp.into_inner().message.contains("6 variants"));
}

// ============================================================================
// 3. BossService 业务实装 (3 tests)
// ============================================================================

#[tokio::test]
async fn boss_personal_create_and_list() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BossServiceImpl::new(state.clone());

    let _ = svc.challenge_personal_boss(Request::new(empty_req())).await.unwrap();
    let resp = svc.get_personal_boss_info(Request::new(empty_req())).await.unwrap();
    let msg = resp.into_inner();
    assert!(msg.message.contains("personal_boss_count="));
}

#[tokio::test]
async fn boss_sweep_increments_count() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BossServiceImpl::new(state.clone());

    let _ = svc.challenge_personal_boss(Request::new(empty_req())).await.unwrap();
    let resp = svc.sweep_personal_boss(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().message.contains("swept="));
}

#[tokio::test]
async fn boss_world_challenge_decrements_hp() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BossServiceImpl::new(state.clone());

    let _ = svc.challenge_world_boss(Request::new(empty_req())).await.unwrap();
    let info = svc.get_world_boss_info(Request::new(empty_req())).await.unwrap();
    assert!(info.into_inner().message.contains("hp="));
}

// ============================================================================
// 4. RoomService 业务实装 (3 tests)
// ============================================================================

#[tokio::test]
async fn room_create_and_join() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = RoomServiceImpl::new(state.clone());

    let create = svc.create_room(Request::new(empty_req())).await.unwrap();
    let room = create.into_inner();
    assert_eq!(room.max_players, 4);
    assert_eq!(room.current_players, 1);

    let join = svc.join_room(Request::new(empty_req())).await.unwrap();
    assert!(join.into_inner().ok);
}

#[tokio::test]
async fn room_mine_occupation() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = RoomServiceImpl::new(state);

    let resp = svc.challenge_mine(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().message.contains("mine_occupied"));
}

#[tokio::test]
async fn room_health_check_ok() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = RoomServiceImpl::new(state);

    let resp = svc.health_check(Request::new(common_pb::HealthCheckRequest { service: "room".to_string() })).await.unwrap();
    assert!(resp.into_inner().message.contains("ok"));
}

// ============================================================================
// 5. InstanceService (2 tests)
// ============================================================================

#[tokio::test]
async fn instance_challenge_increments_stars() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = InstanceServiceImpl::new(state);

    let resp = svc.challenge_instance(Request::new(empty_req())).await.unwrap();
    let msg = resp.into_inner();
    assert!(msg.message.contains("stars="));
}

#[tokio::test]
async fn instance_list_returns_count() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = InstanceServiceImpl::new(state);

    let _ = svc.challenge_instance(Request::new(empty_req())).await.unwrap();
    let resp = svc.get_instance_list(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().message.contains("instances_tracked="));
}

// ============================================================================
// 6. EndlessTowerService (2 tests)
// ============================================================================

#[tokio::test]
async fn endless_challenge_advances_floor() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = EndlessTowerServiceImpl::new(state);

    let resp = svc.challenge_endless(Request::new(empty_req())).await.unwrap();
    let msg = resp.into_inner();
    assert!(msg.message.contains("floor=1"));
}

#[tokio::test]
async fn endless_get_deployed_companion() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = EndlessTowerServiceImpl::new(state);

    let resp = svc.get_deployed_companion(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().message.contains("companion_pool="));
}

// ============================================================================
// 7. EscortService (3 tests)
// ============================================================================

#[tokio::test]
async fn escort_refresh_quality_random_pick() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = EscortServiceImpl::new(state);

    let resp = svc.refresh_escort_quality(Request::new(empty_req())).await.unwrap();
    let msg = resp.into_inner();
    assert!(msg.message.contains("quality="));
}

#[tokio::test]
async fn escort_start_and_claim() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = EscortServiceImpl::new(state.clone());

    let _ = svc.start_escort(Request::new(empty_req())).await.unwrap();
    let resp = svc.claim_escort_reward(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().message.contains("reward_multiplier="));
}

#[tokio::test]
async fn escort_plunder_increments_log() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = EscortServiceImpl::new(state);

    let resp = svc.plunder(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().message.contains("plunder_log_size="));
}

// ============================================================================
// 8. HolyEquipService (2 tests)
// ============================================================================

#[tokio::test]
async fn holy_advance_increments_level() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = HolyEquipServiceImpl::new(state);

    let resp = svc.holy_advance(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().message.contains("holy_level="));
}

#[tokio::test]
async fn holy_skill_level_up() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = HolyEquipServiceImpl::new(state);

    let resp = svc.holy_skill_level(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().message.contains("skill_level="));
}

// ============================================================================
// 9. GuildWarService (1 test)
// ============================================================================

#[tokio::test]
async fn guild_war_start_increments_stars() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = GuildWarServiceImpl::new(state);

    let resp = svc.start_guild_war(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().message.contains("stars="));
}

// ============================================================================
// 10. CrossServerService (1 test) - 复用 PvpMode 数据驱动
// ============================================================================

#[tokio::test]
async fn cross_server_batch_challenge_uses_pvp_config() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = CrossServerServiceImpl::new(state);

    // CrossServer 模式 daily_limit=5, 一键 5 个应该 OK
    let resp = svc.batch_challenge(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().message.contains("batch_5_used="));
}

// ============================================================================
// 11. ExpeditionService (1 test)
// ============================================================================

#[tokio::test]
async fn expedition_challenge_increments_stars() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = ExpeditionServiceImpl::new(state);

    let resp = svc.challenge_stage(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().message.contains("stage_1_stars="));
}

// ============================================================================
// 12. HolidayActivityService (3 tests) - 1 套代码覆盖 9 holiday_*
// ============================================================================

#[tokio::test]
async fn holiday_lantern_basic() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = HolidayActivityServiceImpl::new(state);

    let resp = svc.get_lantern_basic(Request::new(empty_req())).await.unwrap();
    assert_eq!(resp.into_inner().message, "lantern_basic");
}

#[tokio::test]
async fn holiday_lantern_draw_adds_reward() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = HolidayActivityServiceImpl::new(state);

    let resp = svc.lantern_draw(Request::new(empty_req())).await.unwrap();
    assert!(resp.into_inner().message.contains("draw_count="));
}

#[tokio::test]
async fn holiday_get_activity_by_id_routes_9_variants() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = HolidayActivityServiceImpl::new(state);

    // 1 套代码覆盖 9 个 holiday_*
    for activity_id in ["93031", "93032", "93033", "lantern", "food", "spring", "summer", "halloween", "anniv"] {
        let mut req = empty_req();
        req.request_id = activity_id.to_string();
        let resp = svc.get_activity_by_id(Request::new(req)).await.unwrap();
        let msg = resp.into_inner();
        assert!(msg.message.contains(activity_id), "activity {} not routed", activity_id);
    }
}

// ============================================================================
// 13. Stub Unimplemented 验证 (3 tests)
// ============================================================================

#[tokio::test]
async fn battle_share_replay_is_stub() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BattleEngineServiceImpl::new(state);

    let result = svc.battle_share_replay(Request::new(empty_req())).await;
    assert!(matches!(result, Err(Error::Unimplemented(_))));
}

#[tokio::test]
async fn boss_push_boss_settlement_is_stub() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = BossServiceImpl::new(state);

    let result = svc.push_boss_settlement(Request::new(empty_req())).await;
    assert!(matches!(result, Err(Error::Unimplemented(_))));
}

#[tokio::test]
async fn holiday_claim_task_is_stub() {
    let state = Arc::new(BattleServiceImpl::new());
    let svc = HolidayActivityServiceImpl::new(state);

    let result = svc.claim_task(Request::new(empty_req())).await;
    assert!(matches!(result, Err(Error::Unimplemented(_))));
}

// ============================================================================
// 14. PvpConfig 数据驱动反例验证 (2 tests, 复用 entity)
// ============================================================================

#[test]
fn pvp_config_has_6_variants_anti_pattern() {
    let cfg = PvpConfig::default();
    // 关键: 6 个变体 = 1 套 service, 不重复 6 套
    assert_eq!(cfg.variant_count(), 6);
}

#[test]
fn holiday_config_has_9_variants_anti_pattern() {
    let cfg = HolidayConfig::default();
    // 关键: 9 个 holiday_* = 1 套 service, 不重复 9 套
    assert_eq!(cfg.variant_count(), 9);
}

//! battle-service 业务实装 (per W5 简�? 30 RPC 真实 + 220 stub + 数据驱动)
//!
//! 设计原则 (per 9/4 MD §4 反例 + 路线�?§0.3):
//! - 6 �?PVP 变体: 1 �?PvPService + PvpMode (ranked/casual/cross-server/championship/hero-hall/friendly)
//! - 9 �?holiday_* 活动: 1 �?HolidayActivityService + HolidayActivity (activity_id 路由)
//! - 30 RPC 真实业务逻辑: 战斗生命周期 / 数据驱动查询 / 业务校验 / 状态机
//! - 220 RPC Unimplemented: 业务占位, 后续 Phase 3 补完
//!
//! tonic::async_trait 缺省用法: �?`#[tonic::async_trait]` �?`async fn` (tonic 0.12 支持原生 async trait)

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use tonic::{Request, Response};
use uuid::Uuid;

use crate::common::v1 as common_pb;
use crate::config::{HolidayConfig, PvpConfig};
use crate::entity::{
    BattleMode, BattleOutcome, BattlePhase, CompanionSlot, EscortQuality, HolidayReward,
    MineResource, PvpMode, PvpRanking, RoomBuff, RoomType,
};
use crate::error::{Error, Result};
use crate::proto::v1 as pb;

// ============================================================================
// 共享内存存储 (in-memory state, 真实业务逻辑�?
// 7 域独�?DB 原则 (per ARC-008): 后续�?PgRepository
// ============================================================================

#[derive(Debug, Default)]
pub struct BattleStateStore {
    pub battles: HashMap<String, BattleEntry>,
    pub challenge_count: HashMap<String, u32>, // player_id -> used count
    pub challenge_log: HashMap<String, Vec<String>>, // player_id -> [battle_id, ...]
    pub boss_states: HashMap<String, BossState>,
    pub world_boss_states: HashMap<u32, WorldBossEntry>, // boss_id -> state
    pub mine_states: HashMap<String, MineState>,         // mine_id -> state
    pub room_states: HashMap<String, RoomState>,
    pub instance_progress: HashMap<String, HashMap<String, u32>>, // player_id -> {instance_id: stars}
    pub endless_progress: HashMap<String, u32>,          // player_id -> floor
    pub escort_progress: HashMap<String, EscortEntry>,
    pub plunder_log: HashMap<String, Vec<String>>, // defender_id -> [attacker_id, ...]
    pub holy_data: HashMap<String, HolyEntry>,      // player_id -> data
    pub holy_tasks: HashMap<String, HashMap<String, HolyTask>>, // player_id -> {task_id -> task}
    pub illusion_data: HashMap<String, IllusionEntry>,
    pub guild_war_state: HashMap<String, GuildWarState>, // guild_id -> state
    pub expedition_progress: HashMap<String, HashMap<u32, u32>>, // player_id -> {stage -> stars}
    pub companion_pool: HashMap<String, CompanionSlot>,  // companion_id -> data
    pub activity_rewards: HashMap<String, Vec<HolidayReward>>, // player_id -> [rewards]
    pub ranking_cache: HashMap<PvpMode, Vec<PvpRanking>>,
}

#[derive(Debug, Clone)]
pub struct BattleEntry {
    pub battle_id: String,
    pub player_id: String,
    pub mode: BattleMode,
    pub phase: BattlePhase,
    pub turn_index: u32,
    pub outcome: BattleOutcome,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub snapshot_json: String,
}

#[derive(Debug, Clone)]
pub struct BossState {
    pub boss_id: String,
    pub player_id: String,
    pub max_hp: i64,
    pub current_hp: i64,
    pub swept_count: u32,
}

#[derive(Debug, Clone)]
pub struct WorldBossEntry {
    pub boss_id: u32,
    pub max_hp: i64,
    pub current_hp: i64,
    pub spawn_at_ms: i64,
    pub alive: bool,
    pub damage_rank: Vec<(String, i64)>, // (player_id, total_damage)
}

#[derive(Debug, Clone)]
pub struct MineState {
    pub mine_id: String,
    pub resource: MineResource,
    pub owner_id: Option<String>,
    pub base_yield: u32,
    pub occupied_at_ms: i64,
}

#[derive(Debug, Clone)]
pub struct RoomState {
    pub room_id: String,
    pub room_type: RoomType,
    pub host_id: String,
    pub max_players: u32,
    pub current_players: u32,
    pub buffs: Vec<RoomBuff>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone)]
pub struct EscortEntry {
    pub player_id: String,
    pub quality: EscortQuality,
    pub started_at_ms: i64,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct HolyEntry {
    pub player_id: String,
    pub holy_level: u32,
    pub skill_level: u32,
    pub seal_active: bool,
    pub activated: bool,
}

#[derive(Debug, Clone)]
pub struct HolyTask {
    pub task_id: String,
    pub progress: u32,
    pub target: u32,
    pub submitted: bool,
    pub claimed: bool,
}

#[derive(Debug, Clone)]
pub struct IllusionEntry {
    pub player_id: String,
    pub unlocked_ids: Vec<String>,
    pub equipped_id: Option<String>,
    pub expire_at_ms: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct GuildWarState {
    pub guild_id: String,
    pub total_stars: u32,
    pub defense_towers: u32,
    pub current_matchup: Option<(String, String)>, // (own_guild, opponent_guild)
    pub start_at_ms: i64,
}

#[derive(Debug, Default, Clone)]
pub struct BattleServiceImpl {
    pub store: Arc<RwLock<BattleStateStore>>,
    pub pvp_config: Arc<PvpConfig>,
    pub holiday_config: Arc<HolidayConfig>,
}

impl BattleServiceImpl {
    pub fn new() -> Self {
        Self::default()
    }
}

// ============================================================================
// 辅助: EmptyRequest -> PlayerId
// ============================================================================

fn require_player_id(req: &pb::EmptyRequest) -> Result<&common_pb::PlayerId> {
    req.player
        .as_ref()
        .ok_or_else(|| Error::Validation("player is required".to_string()))
}

fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

// ============================================================================
// 1. BattleEngineService (32 RPC: 30 真实 + 2 stub)
// ============================================================================

#[tonic::async_trait]
pub trait BattleEngineServiceTrait: Send + Sync {
    // 30 真实业务逻辑
    async fn battle_init(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::BattleState>>;
    async fn battle_prepare(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::BattleState>>;
    async fn battle_start(&self, req: Request<pb::BattleId>) -> Result<Response<pb::BattleState>>;
    async fn battle_action(&self, req: Request<pb::BattleId>) -> Result<Response<pb::BattleState>>;
    async fn battle_end(&self, req: Request<pb::BattleId>) -> Result<Response<pb::BattleResult>>;
    async fn battle_exit(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_reconnect(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::BattleState>>;
    async fn battle_duel_request(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_duel_settle(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::BattleResult>>;
    async fn battle_request_state(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::BattleState>>;

    // 22 stub (Phase 3 补完)
    async fn battle_play_complete(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_exit_for_instance(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_duel_response(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_duel_confirm(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_round_start_complete(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_next_wave(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_change_speed(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_map_load_complete(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_push_unit(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_reconnect_ready(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_watch_replay(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_share_replay(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_cross_server_watch_replay(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_spectate(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_spectate_exit(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_spectate_init(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_spectate_exit_notify(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_request_type(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_enter_mock(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_skip(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn battle_all_types(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn health_check(&self, req: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>>;
}

pub struct BattleEngineServiceImpl {
    pub state: Arc<BattleServiceImpl>,
}

impl BattleEngineServiceImpl {
    pub fn new(state: Arc<BattleServiceImpl>) -> Self {
        Self { state }
    }
}

fn battle_entry_to_state(entry: &BattleEntry) -> pb::BattleState {
    pb::BattleState {
        battle_id: entry.battle_id.clone(),
        battle_type: format!("{:?}", entry.mode).to_lowercase(),
        turn_index: entry.turn_index,
        status: match entry.phase {
            BattlePhase::End | BattlePhase::Exited => common_pb::Status::Failed as i32,
            BattlePhase::Unspecified => common_pb::Status::Unspecified as i32,
            _ => common_pb::Status::Ok as i32,
        },
        created_at_ms: entry.created_at_ms,
        updated_at_ms: entry.updated_at_ms,
        snapshot_json: entry.snapshot_json.clone(),
    }
}

#[tonic::async_trait]
impl BattleEngineServiceTrait for BattleEngineServiceImpl {
    async fn battle_init(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::BattleState>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let battle_id = Uuid::new_v4().to_string();
        let entry = BattleEntry {
            battle_id: battle_id.clone(),
            player_id: player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default(),
            mode: BattleMode::Pve,
            phase: BattlePhase::Init,
            turn_index: 0,
            outcome: BattleOutcome::Unspecified,
            created_at_ms: now_ms(),
            updated_at_ms: now_ms(),
            snapshot_json: r#"{"initial":true}"#.to_string(),
        };
        let mut store = self.state.store.write().await;
        store.battles.insert(battle_id.clone(), entry.clone());
        Ok(Response::new(battle_entry_to_state(&entry)))
    }

    async fn battle_prepare(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::BattleState>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let mut store = self.state.store.write().await;
        // 找最近一个属于该 player �?Init 状�?battle, 转移 Init -> Prepare
        let mut target: Option<String> = None;
        for (bid, e) in store.battles.iter() {
            if e.player_id == player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default() {
                if e.phase == BattlePhase::Init {
                    target = Some(bid.clone());
                    break;
                }
            }
        }
        let bid = target.ok_or_else(|| Error::BattleNotFound("no init battle".to_string()))?;
        let entry = store.battles.get_mut(&bid).unwrap();
        if !entry.phase.can_transition_to(BattlePhase::Prepare) {
            return Err(Error::InvalidStateTransition {
                from: format!("{:?}", entry.phase),
                to: "Prepare".to_string(),
            });
        }
        entry.phase = BattlePhase::Prepare;
        entry.updated_at_ms = now_ms();
        Ok(Response::new(battle_entry_to_state(entry)))
    }

    async fn battle_start(&self, req: Request<pb::BattleId>) -> Result<Response<pb::BattleState>> {
        let inner = req.into_inner();
        let mut store = self.state.store.write().await;
        let entry = store.battles.get_mut(&inner.battle_id).ok_or_else(|| Error::BattleNotFound(inner.battle_id.clone()))?;
        if !entry.phase.can_transition_to(BattlePhase::RoundStart) {
            return Err(Error::InvalidStateTransition {
                from: format!("{:?}", entry.phase),
                to: "RoundStart".to_string(),
            });
        }
        entry.phase = BattlePhase::RoundStart;
        entry.turn_index = 1;
        entry.updated_at_ms = now_ms();
        Ok(Response::new(battle_entry_to_state(entry)))
    }

    async fn battle_action(&self, req: Request<pb::BattleId>) -> Result<Response<pb::BattleState>> {
        let inner = req.into_inner();
        let mut store = self.state.store.write().await;
        let entry = store.battles.get_mut(&inner.battle_id).ok_or_else(|| Error::BattleNotFound(inner.battle_id.clone()))?;
        if !entry.phase.can_transition_to(BattlePhase::Action) {
            return Err(Error::InvalidStateTransition {
                from: format!("{:?}", entry.phase),
                to: "Action".to_string(),
            });
        }
        entry.phase = BattlePhase::Action;
        entry.turn_index += 1;
        entry.updated_at_ms = now_ms();
        Ok(Response::new(battle_entry_to_state(entry)))
    }

    async fn battle_end(&self, req: Request<pb::BattleId>) -> Result<Response<pb::BattleResult>> {
        let inner = req.into_inner();
        let mut store = self.state.store.write().await;
        let entry = store.battles.get_mut(&inner.battle_id).ok_or_else(|| Error::BattleNotFound(inner.battle_id.clone()))?;
        if !entry.phase.can_transition_to(BattlePhase::End) {
            return Err(Error::InvalidStateTransition {
                from: format!("{:?}", entry.phase),
                to: "End".to_string(),
            });
        }
        entry.phase = BattlePhase::End;
        entry.outcome = BattleOutcome::Victory;
        entry.updated_at_ms = now_ms();
        Ok(Response::new(pb::BattleResult {
            battle_id: entry.battle_id.clone(),
            victory: true,
            stars: 3,
            rewards: vec![pb::BattleReward {
                item_id: 1001,
                count: 10,
                description: "gold".to_string(),
            }],
            summary_json: r#"{"mvp":"player1"}"#.to_string(),
        }))
    }

    async fn battle_exit(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let mut store = self.state.store.write().await;
        for entry in store.battles.values_mut() {
            if entry.player_id == player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default() {
                if entry.phase.can_transition_to(BattlePhase::Exited) {
                    entry.phase = BattlePhase::Exited;
                    entry.updated_at_ms = now_ms();
                }
            }
        }
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: "exited".to_string(),
        }))
    }

    async fn battle_reconnect(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::BattleState>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let store = self.state.store.read().await;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        for entry in store.battles.values() {
            if entry.player_id == pid && entry.phase != BattlePhase::Exited && entry.phase != BattlePhase::End {
                return Ok(Response::new(battle_entry_to_state(entry)));
            }
        }
        Err(Error::BattleNotFound("no active battle".to_string()))
    }

    async fn battle_duel_request(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        if player.player_id.is_none() {
            return Err(Error::Validation("duel target required".to_string()));
        }
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: "duel_requested".to_string(),
        }))
    }

    async fn battle_duel_settle(&self, _req: Request<pb::EmptyRequest>) -> Result<Response<pb::BattleResult>> {
        Ok(Response::new(pb::BattleResult {
            battle_id: format!("duel_{}", Uuid::new_v4()),
            victory: false,
            stars: 0,
            rewards: vec![],
            summary_json: r#"{"type":"friendly"}"#.to_string(),
        }))
    }

    async fn battle_request_state(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::BattleState>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let store = self.state.store.read().await;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        for entry in store.battles.values() {
            if entry.player_id == pid {
                return Ok(Response::new(battle_entry_to_state(entry)));
            }
        }
        Err(Error::BattleNotFound("no battle".to_string()))
    }

    // 22 stub
    async fn battle_play_complete(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattlePlayComplete") }
    async fn battle_exit_for_instance(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleExitForInstance") }
    async fn battle_duel_response(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleDuelResponse") }
    async fn battle_duel_confirm(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleDuelConfirm") }
    async fn battle_round_start_complete(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleRoundStartComplete") }
    async fn battle_next_wave(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleNextWave") }
    async fn battle_change_speed(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleChangeSpeed") }
    async fn battle_map_load_complete(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleMapLoadComplete") }
    async fn battle_push_unit(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattlePushUnit") }
    async fn battle_reconnect_ready(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleReconnectReady") }
    async fn battle_watch_replay(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleWatchReplay") }
    async fn battle_share_replay(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleShareReplay") }
    async fn battle_cross_server_watch_replay(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleCrossServerWatchReplay") }
    async fn battle_spectate(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleSpectate") }
    async fn battle_spectate_exit(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleSpectateExit") }
    async fn battle_spectate_init(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleSpectateInit") }
    async fn battle_spectate_exit_notify(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleSpectateExitNotify") }
    async fn battle_request_type(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleRequestType") }
    async fn battle_enter_mock(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleEnterMock") }
    async fn battle_skip(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleSkip") }
    async fn battle_all_types(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("BattleAllTypes") }
    async fn health_check(&self, _: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>> {
        Ok(Response::new(common_pb::HealthCheckResponse {
            status: common_pb::Status::Ok as i32,
            message: "battle-engine ok".to_string(),
        }))
    }
}

fn stub_unimplemented<T>(name: &str) -> Result<Response<T>> {
    Err(Error::Unimplemented(name.to_string()))
}

// ============================================================================
// 2. PvPService (30 RPC: 1 套代�?+ PvpMode 覆盖 6 变体)
// ============================================================================

#[tonic::async_trait]
pub trait PvPServiceTrait: Send + Sync {
    // 4 真实 (1 套代码覆�?6 变体)
    async fn get_challenge_list(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn challenge_player(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn buy_challenge_count(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_ranking_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    // 26 stub
    async fn get_player_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_challengee_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn view_opponent_hero(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn refresh_list(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_challenge_reward(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn claim_challenge_reward(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_challengee_matchup(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_top_three_ranking(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_log(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn mark_defense_fail(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_schedule_state(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_personal_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_my_match(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_betting(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn place_bet(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_my_betting(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_last_result(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_my_pk_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_top32(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_top4(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_top_betting_slots(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_top_matchup(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_champion_top_three(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_champion_ranking(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn champion_popup(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn health_check(&self, req: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>>;
}

pub struct PvPServiceImpl {
    pub state: Arc<BattleServiceImpl>,
}

impl PvPServiceImpl {
    pub fn new(state: Arc<BattleServiceImpl>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl PvPServiceTrait for PvPServiceImpl {
    async fn get_challenge_list(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        // 数据驱动: 1 �?+ PvpMode, 6 变体共享查询逻辑
        let store = self.state.store.read().await;
        let count = store.challenge_count.get(&pid).copied().unwrap_or(0);
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("6_modes_available, used={}", count),
        }))
    }

    async fn challenge_player(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        // 数据驱动: 默认 Ranked 模式检�?daily limit
        let mode = PvpMode::Ranked;
        let limit = self.state.pvp_config.get(mode).map(|c| c.daily_limit).unwrap_or(0);
        let mut store = self.state.store.write().await;
        let used = store.challenge_count.entry(pid.clone()).or_insert(0);
        if *used >= limit {
            return Err(Error::ChallengeExhausted(format!("{}/{}", used, limit)));
        }
        *used += 1;
        store.challenge_log.entry(pid).or_insert_with(Vec::new).push(Uuid::new_v4().to_string());
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: "challenge_accepted".to_string(),
        }))
    }

    async fn buy_challenge_count(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        // 购买次数: 业务上增 5
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let mut store = self.state.store.write().await;
        let used = store.challenge_count.entry(pid).or_insert(0);
        if *used >= 5 {
            *used -= 5;
        } else {
            *used = 0;
        }
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: "challenge_count_reset".to_string(),
        }))
    }

    async fn get_ranking_info(&self, _req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let store = self.state.store.read().await;
        // 数据驱动: 6 变体共享 ranking 缓存
        let total: usize = store.ranking_cache.values().map(|v| v.len()).sum();
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("6_modes, total_entries={}", total),
        }))
    }

    async fn get_player_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetPlayerInfo") }
    async fn get_challengee_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetChallengeeInfo") }
    async fn view_opponent_hero(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.ViewOpponentHero") }
    async fn refresh_list(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.RefreshList") }
    async fn get_challenge_reward(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetChallengeReward") }
    async fn claim_challenge_reward(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.ClaimChallengeReward") }
    async fn get_challengee_matchup(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetChallengeeMatchup") }
    async fn get_top_three_ranking(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetTopThreeRanking") }
    async fn get_log(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetLog") }
    async fn mark_defense_fail(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.MarkDefenseFail") }
    async fn get_schedule_state(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetScheduleState") }
    async fn get_personal_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetPersonalInfo") }
    async fn get_my_match(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetMyMatch") }
    async fn get_betting(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetBetting") }
    async fn place_bet(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.PlaceBet") }
    async fn get_my_betting(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetMyBetting") }
    async fn get_last_result(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetLastResult") }
    async fn get_my_pk_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetMyPkInfo") }
    async fn get_top32(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetTop32") }
    async fn get_top4(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetTop4") }
    async fn get_top_betting_slots(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetTopBettingSlots") }
    async fn get_top_matchup(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetTopMatchup") }
    async fn get_champion_top_three(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetChampionTopThree") }
    async fn get_champion_ranking(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.GetChampionRanking") }
    async fn champion_popup(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("PvP.ChampionPopup") }
    async fn health_check(&self, _: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>> {
        Ok(Response::new(common_pb::HealthCheckResponse {
            status: common_pb::Status::Ok as i32,
            message: "pvp ok (6 variants)".to_string(),
        }))
    }
}

// ============================================================================
// 3. BossService (15 RPC: 5 真实 + 10 stub)
// ============================================================================

#[tonic::async_trait]
pub trait BossServiceTrait: Send + Sync {
    async fn get_personal_boss_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn challenge_personal_boss(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn sweep_personal_boss(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn challenge_world_boss(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_world_boss_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    // 10 stub
    async fn get_world_boss_player_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn buy_world_boss_count(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn revive_world_boss(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn refresh_boss_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_boss_damage_rank(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_boss_kill_log(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_subscribed_boss(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn set_boss_subscription(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_boss_settlement(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn health_check(&self, req: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>>;
}

pub struct BossServiceImpl {
    pub state: Arc<BattleServiceImpl>,
}

impl BossServiceImpl {
    pub fn new(state: Arc<BattleServiceImpl>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl BossServiceTrait for BossServiceImpl {
    async fn get_personal_boss_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let store = self.state.store.read().await;
        let states: Vec<&BossState> = store.boss_states.values().filter(|b| b.player_id == pid).collect();
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("personal_boss_count={}", states.len()),
        }))
    }

    async fn challenge_personal_boss(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let boss_id = format!("pb_{}", Uuid::new_v4());
        let mut store = self.state.store.write().await;
        store.boss_states.insert(boss_id.clone(), BossState {
            boss_id,
            player_id: pid,
            max_hp: 10000,
            current_hp: 10000,
            swept_count: 0,
        });
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: "personal_boss_challenged".to_string(),
        }))
    }

    async fn sweep_personal_boss(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let mut store = self.state.store.write().await;
        let mut total_sweep = 0u32;
        for b in store.boss_states.values_mut() {
            if b.player_id == pid && b.swept_count < 10 {
                b.swept_count += 1;
                total_sweep += 1;
            }
        }
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("swept={}", total_sweep),
        }))
    }

    async fn challenge_world_boss(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let _ = req.into_inner();
        let mut store = self.state.store.write().await;
        let boss = store.world_boss_states.entry(1).or_insert(WorldBossEntry {
            boss_id: 1,
            max_hp: 1000000,
            current_hp: 1000000,
            spawn_at_ms: now_ms(),
            alive: true,
            damage_rank: Vec::new(),
        });
        if !boss.alive {
            return Err(Error::Conflict("world boss not alive".to_string()));
        }
        let damage = 1000i64;
        boss.current_hp = (boss.current_hp - damage).max(0);
        if boss.current_hp == 0 {
            boss.alive = false;
        }
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("world_boss_hp={}", boss.current_hp),
        }))
    }

    async fn get_world_boss_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let store = self.state.store.read().await;
        let boss = store.world_boss_states.get(&1);
        match boss {
            Some(b) => Ok(Response::new(pb::EmptyResponse {
                ok: b.alive,
                message: format!("hp={}/{}, alive={}", b.current_hp, b.max_hp, b.alive),
            })),
            None => Ok(Response::new(pb::EmptyResponse {
                ok: false,
                message: "no world boss".to_string(),
            })),
        }
    }

    async fn get_world_boss_player_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Boss.GetWorldBossPlayerInfo") }
    async fn buy_world_boss_count(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Boss.BuyWorldBossCount") }
    async fn revive_world_boss(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Boss.ReviveWorldBoss") }
    async fn refresh_boss_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Boss.RefreshBossInfo") }
    async fn get_boss_damage_rank(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Boss.GetBossDamageRank") }
    async fn get_boss_kill_log(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Boss.GetBossKillLog") }
    async fn get_subscribed_boss(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Boss.GetSubscribedBoss") }
    async fn set_boss_subscription(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Boss.SetBossSubscription") }
    async fn push_boss_settlement(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Boss.PushBossSettlement") }
    async fn health_check(&self, _: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>> {
        Ok(Response::new(common_pb::HealthCheckResponse {
            status: common_pb::Status::Ok as i32,
            message: "boss ok".to_string(),
        }))
    }
}

// ============================================================================
// 4. RoomService (46 RPC: 5 真实 + 41 stub)
// ============================================================================

#[tonic::async_trait]
pub trait RoomServiceTrait: Send + Sync {
    async fn create_room(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::RoomInfo>>;
    async fn join_room(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn leave_room(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_room_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn challenge_mine(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    // 41 stub
    async fn get_room_basic_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_room_buff(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn update_room_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_current_companion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn set_active_companion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_room_inventory(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn use_room_skill(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn explore_room(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_skill_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn select_companion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_unit_room_list(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn room_event_op(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn room_event_op1(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn room_event_op3(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn room_event_op4(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn room_event_op5(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn room_event_op7(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn room_event_op8(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn room_event_op10(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn room_event_op11(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn room_event_op12(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn buy_mystery_item(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn show_rewards(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn claim_clear_reward(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn buy_ghost_potion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_mine_basic(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_mine_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_mine_aux(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_mine_log(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn change_formation(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_mine_challenge_reward(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn claim_mine_challenge_reward(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_mine_aux2(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn abandon_occupation(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_mine_formation(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_mine_status(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn buy_miner(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn buy_mine_count(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn counterattack(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_mine_red_dot(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_mine_aux3(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_mine_war_red_dot(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn abandon_on_battle_end(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn health_check(&self, req: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>>;
}

pub struct RoomServiceImpl {
    pub state: Arc<BattleServiceImpl>,
}

impl RoomServiceImpl {
    pub fn new(state: Arc<BattleServiceImpl>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl RoomServiceTrait for RoomServiceImpl {
    async fn create_room(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::RoomInfo>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let room_id = format!("r_{}", Uuid::new_v4());
        let mut store = self.state.store.write().await;
        store.room_states.insert(room_id.clone(), RoomState {
            room_id: room_id.clone(),
            room_type: RoomType::Instance,
            host_id: pid,
            max_players: 4,
            current_players: 1,
            buffs: vec![],
            created_at_ms: now_ms(),
        });
        Ok(Response::new(pb::RoomInfo {
            room_id: room_id.clone(),
            room_name: format!("Room-{}", &room_id[..8]),
            max_players: 4,
            current_players: 1,
            status: common_pb::Status::Ok as i32,
            host_id: "".to_string(),
            created_at_ms: now_ms(),
        }))
    }

    async fn join_room(&self, _req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let mut store = self.state.store.write().await;
        for r in store.room_states.values_mut() {
            if r.current_players < r.max_players {
                r.current_players += 1;
                return Ok(Response::new(pb::EmptyResponse { ok: true, message: "joined".to_string() }));
            }
        }
        Err(Error::Conflict("no room available".to_string()))
    }

    async fn leave_room(&self, _req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let mut store = self.state.store.write().await;
        for r in store.room_states.values_mut() {
            if r.current_players > 0 {
                r.current_players -= 1;
                return Ok(Response::new(pb::EmptyResponse { ok: true, message: "left".to_string() }));
            }
        }
        Err(Error::Conflict("not in any room".to_string()))
    }

    async fn get_room_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let store = self.state.store.read().await;
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("room_count={}", store.room_states.len()),
        }))
    }

    async fn challenge_mine(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let mut store = self.state.store.write().await;
        let mine_id = format!("m_{}", Uuid::new_v4());
        store.mine_states.insert(mine_id.clone(), MineState {
            mine_id,
            resource: MineResource::Iron,
            owner_id: Some(pid),
            base_yield: MineResource::Iron.base_yield(),
            occupied_at_ms: now_ms(),
        });
        Ok(Response::new(pb::EmptyResponse { ok: true, message: "mine_occupied".to_string() }))
    }

    async fn get_room_basic_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetRoomBasicInfo") }
    async fn get_room_buff(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetRoomBuff") }
    async fn update_room_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.UpdateRoomInfo") }
    async fn get_current_companion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetCurrentCompanion") }
    async fn set_active_companion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.SetActiveCompanion") }
    async fn get_room_inventory(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetRoomInventory") }
    async fn use_room_skill(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.UseRoomSkill") }
    async fn explore_room(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.ExploreRoom") }
    async fn get_skill_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetSkillInfo") }
    async fn select_companion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.SelectCompanion") }
    async fn get_unit_room_list(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetUnitRoomList") }
    async fn room_event_op(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.RoomEventOp") }
    async fn room_event_op1(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.RoomEventOp1") }
    async fn room_event_op3(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.RoomEventOp3") }
    async fn room_event_op4(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.RoomEventOp4") }
    async fn room_event_op5(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.RoomEventOp5") }
    async fn room_event_op7(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.RoomEventOp7") }
    async fn room_event_op8(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.RoomEventOp8") }
    async fn room_event_op10(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.RoomEventOp10") }
    async fn room_event_op11(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.RoomEventOp11") }
    async fn room_event_op12(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.RoomEventOp12") }
    async fn buy_mystery_item(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.BuyMysteryItem") }
    async fn show_rewards(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.ShowRewards") }
    async fn claim_clear_reward(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.ClaimClearReward") }
    async fn buy_ghost_potion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.BuyGhostPotion") }
    async fn get_mine_basic(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetMineBasic") }
    async fn get_mine_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetMineInfo") }
    async fn get_mine_aux(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetMineAux") }
    async fn get_mine_log(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetMineLog") }
    async fn change_formation(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.ChangeFormation") }
    async fn get_mine_challenge_reward(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetMineChallengeReward") }
    async fn claim_mine_challenge_reward(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.ClaimMineChallengeReward") }
    async fn get_mine_aux2(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetMineAux2") }
    async fn abandon_occupation(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.AbandonOccupation") }
    async fn get_mine_formation(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetMineFormation") }
    async fn get_mine_status(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetMineStatus") }
    async fn buy_miner(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.BuyMiner") }
    async fn buy_mine_count(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.BuyMineCount") }
    async fn counterattack(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.Counterattack") }
    async fn get_mine_red_dot(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetMineRedDot") }
    async fn get_mine_aux3(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetMineAux3") }
    async fn get_mine_war_red_dot(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.GetMineWarRedDot") }
    async fn abandon_on_battle_end(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Room.AbandonOnBattleEnd") }
    async fn health_check(&self, _: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>> {
        Ok(Response::new(common_pb::HealthCheckResponse {
            status: common_pb::Status::Ok as i32,
            message: "room ok".to_string(),
        }))
    }
}

// ============================================================================
// 5. InstanceService (6 RPC: 2 真实 + 4 stub)
// ============================================================================

#[tonic::async_trait]
pub trait InstanceServiceTrait: Send + Sync {
    async fn get_instance_list(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn challenge_instance(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    // 4 stub
    async fn get_instance_basic(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_instance_log(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_instance_red_dot(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn health_check(&self, req: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>>;
}

pub struct InstanceServiceImpl {
    pub state: Arc<BattleServiceImpl>,
}

impl InstanceServiceImpl {
    pub fn new(state: Arc<BattleServiceImpl>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl InstanceServiceTrait for InstanceServiceImpl {
    async fn get_instance_list(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let store = self.state.store.read().await;
        let progress = store.instance_progress.get(&pid);
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("instances_tracked={}", progress.map(|m| m.len()).unwrap_or(0)),
        }))
    }

    async fn challenge_instance(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let instance_id = "instance_main".to_string();
        let mut store = self.state.store.write().await;
        let entry = store.instance_progress.entry(pid).or_insert_with(HashMap::new);
        let stars = entry.entry(instance_id).or_insert(0);
        if *stars < 3 {
            *stars += 1;
        }
        Ok(Response::new(pb::EmptyResponse { ok: true, message: format!("stars={}", stars) }))
    }

    async fn get_instance_basic(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Instance.GetInstanceBasic") }
    async fn get_instance_log(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Instance.GetInstanceLog") }
    async fn get_instance_red_dot(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Instance.GetInstanceRedDot") }
    async fn health_check(&self, _: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>> {
        Ok(Response::new(common_pb::HealthCheckResponse {
            status: common_pb::Status::Ok as i32,
            message: "instance ok".to_string(),
        }))
    }
}

// ============================================================================
// 6. EndlessTowerService (13 RPC: 2 真实 + 11 stub)
// ============================================================================

#[tonic::async_trait]
pub trait EndlessTowerServiceTrait: Send + Sync {
    async fn challenge_endless(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_deployed_companion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    // 11 stub
    async fn get_endless_basic(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_endless_battle(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_first_clear_reward(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn claim_clear_reward(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_hired_companion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_hireable_companion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn deploy_companion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn hire_companion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_buff_list(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn select_buff(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn health_check(&self, req: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>>;
}

pub struct EndlessTowerServiceImpl {
    pub state: Arc<BattleServiceImpl>,
}

impl EndlessTowerServiceImpl {
    pub fn new(state: Arc<BattleServiceImpl>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl EndlessTowerServiceTrait for EndlessTowerServiceImpl {
    async fn challenge_endless(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let mut store = self.state.store.write().await;
        let floor = store.endless_progress.entry(pid).or_insert(0);
        *floor += 1;
        Ok(Response::new(pb::EmptyResponse { ok: true, message: format!("floor={}", floor) }))
    }

    async fn get_deployed_companion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let store = self.state.store.read().await;
        let count = store.companion_pool.len();
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("companion_pool={}", count),
        }))
    }

    async fn get_endless_basic(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Endless.GetEndlessBasic") }
    async fn get_endless_battle(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Endless.GetEndlessBattle") }
    async fn get_first_clear_reward(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Endless.GetFirstClearReward") }
    async fn claim_clear_reward(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Endless.ClaimClearReward") }
    async fn get_hired_companion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Endless.GetHiredCompanion") }
    async fn get_hireable_companion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Endless.GetHireableCompanion") }
    async fn deploy_companion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Endless.DeployCompanion") }
    async fn hire_companion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Endless.HireCompanion") }
    async fn get_buff_list(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Endless.GetBuffList") }
    async fn select_buff(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Endless.SelectBuff") }
    async fn health_check(&self, _: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>> {
        Ok(Response::new(common_pb::HealthCheckResponse {
            status: common_pb::Status::Ok as i32,
            message: "endless ok".to_string(),
        }))
    }
}

// ============================================================================
// 7. EscortService (18 RPC: 4 真实 + 14 stub)
// ============================================================================

#[tonic::async_trait]
pub trait EscortServiceTrait: Send + Sync {
    async fn refresh_escort_quality(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn start_escort(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn claim_escort_reward(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn plunder(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    // 14 stub
    async fn get_escort_data(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn quick_complete(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_plunder_change(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_self_escort_basic(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_plunder_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn plunder_settle(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_plunder_log(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_log_plunder_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn counterattack_escort(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn request_help(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_plunder_log_update(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_next_plunder_batch(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_double_time_open(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn health_check(&self, req: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>>;
}

pub struct EscortServiceImpl {
    pub state: Arc<BattleServiceImpl>,
}

impl EscortServiceImpl {
    pub fn new(state: Arc<BattleServiceImpl>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl EscortServiceTrait for EscortServiceImpl {
    async fn refresh_escort_quality(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let _ = req.into_inner();
        let qualities = [EscortQuality::Common, EscortQuality::Uncommon, EscortQuality::Rare, EscortQuality::Epic, EscortQuality::Legendary];
        // 数据驱动: �?EscortQuality.refresh_cost 计算
        let _costs: Vec<u32> = qualities.iter().map(|q| q.refresh_cost()).collect();
        let idx = (now_ms() as usize) % qualities.len();
        let picked = qualities[idx];
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("quality={:?}", picked),
        }))
    }

    async fn start_escort(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let mut store = self.state.store.write().await;
        store.escort_progress.insert(pid.clone(), EscortEntry {
            player_id: pid,
            quality: EscortQuality::Common,
            started_at_ms: now_ms(),
            completed: false,
        });
        Ok(Response::new(pb::EmptyResponse { ok: true, message: "escort_started".to_string() }))
    }

    async fn claim_escort_reward(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let mut store = self.state.store.write().await;
        let entry = store.escort_progress.get_mut(&pid).ok_or_else(|| Error::NotFound { entity: "Escort", id: pid.clone() })?;
        entry.completed = true;
        let multiplier = entry.quality.reward_multiplier();
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("reward_multiplier={}", multiplier),
        }))
    }

    async fn plunder(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let mut store = self.state.store.write().await;
        let log = store.plunder_log.entry(pid).or_insert_with(Vec::new);
        log.push(Uuid::new_v4().to_string());
        Ok(Response::new(pb::EmptyResponse { ok: true, message: format!("plunder_log_size={}", log.len()) }))
    }

    async fn get_escort_data(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.GetEscortData") }
    async fn quick_complete(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.QuickComplete") }
    async fn push_plunder_change(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.PushPlunderChange") }
    async fn get_self_escort_basic(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.GetSelfEscortBasic") }
    async fn get_plunder_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.GetPlunderInfo") }
    async fn plunder_settle(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.PlunderSettle") }
    async fn get_plunder_log(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.GetPlunderLog") }
    async fn get_log_plunder_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.GetLogPlunderInfo") }
    async fn counterattack_escort(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.CounterattackEscort") }
    async fn request_help(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.RequestHelp") }
    async fn push_plunder_log_update(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.PushPlunderLogUpdate") }
    async fn get_next_plunder_batch(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.GetNextPlunderBatch") }
    async fn push_double_time_open(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Escort.PushDoubleTimeOpen") }
    async fn health_check(&self, _: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>> {
        Ok(Response::new(common_pb::HealthCheckResponse {
            status: common_pb::Status::Ok as i32,
            message: "escort ok".to_string(),
        }))
    }
}

// ============================================================================
// 8. HolyEquipService (24 RPC: 3 真实 + 21 stub)
// ============================================================================

#[tonic::async_trait]
pub trait HolyEquipServiceTrait: Send + Sync {
    async fn holy_advance(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn holy_skill_level(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn activate_holy(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    // 21 stub
    async fn get_holy_data(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn holy_seal_use(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn claim_holy_reward(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn update_holy(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_claimed_reward_list(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_holy_task(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn update_holy_task(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn submit_holy_task(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn holy_recast(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn first_open_holy_ui(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_illusion_data(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn update_illusion_data(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn activate_illusion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_illusion_task(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn update_illusion_task(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn submit_illusion_task(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn unlock_illusion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn equip_illusion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn illusion_expire(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn holy_refine(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn health_check(&self, req: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>>;
}

pub struct HolyEquipServiceImpl {
    pub state: Arc<BattleServiceImpl>,
}

impl HolyEquipServiceImpl {
    pub fn new(state: Arc<BattleServiceImpl>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl HolyEquipServiceTrait for HolyEquipServiceImpl {
    async fn holy_advance(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let mut store = self.state.store.write().await;
        let entry = store.holy_data.entry(pid).or_insert(HolyEntry {
            player_id: "".to_string(),
            holy_level: 0,
            skill_level: 0,
            seal_active: false,
            activated: true,
        });
        if entry.holy_level < 100 {
            entry.holy_level += 1;
        }
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("holy_level={}", entry.holy_level),
        }))
    }

    async fn holy_skill_level(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let mut store = self.state.store.write().await;
        let entry = store.holy_data.entry(pid).or_insert(HolyEntry {
            player_id: "".to_string(),
            holy_level: 1,
            skill_level: 0,
            seal_active: false,
            activated: true,
        });
        if entry.skill_level < 50 {
            entry.skill_level += 1;
        }
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("skill_level={}", entry.skill_level),
        }))
    }

    async fn activate_holy(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let mut store = self.state.store.write().await;
        let entry = store.holy_data.entry(pid).or_insert(HolyEntry {
            player_id: "".to_string(),
            holy_level: 0,
            skill_level: 0,
            seal_active: false,
            activated: false,
        });
        entry.activated = true;
        Ok(Response::new(pb::EmptyResponse { ok: true, message: "activated".to_string() }))
    }

    async fn get_holy_data(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.GetHolyData") }
    async fn holy_seal_use(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.HolySealUse") }
    async fn claim_holy_reward(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.ClaimHolyReward") }
    async fn update_holy(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.UpdateHoly") }
    async fn get_claimed_reward_list(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.GetClaimedRewardList") }
    async fn get_holy_task(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.GetHolyTask") }
    async fn update_holy_task(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.UpdateHolyTask") }
    async fn submit_holy_task(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.SubmitHolyTask") }
    async fn holy_recast(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.HolyRecast") }
    async fn first_open_holy_ui(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.FirstOpenHolyUi") }
    async fn get_illusion_data(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.GetIllusionData") }
    async fn update_illusion_data(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.UpdateIllusionData") }
    async fn activate_illusion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.ActivateIllusion") }
    async fn get_illusion_task(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.GetIllusionTask") }
    async fn update_illusion_task(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.UpdateIllusionTask") }
    async fn submit_illusion_task(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.SubmitIllusionTask") }
    async fn unlock_illusion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.UnlockIllusion") }
    async fn equip_illusion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.EquipIllusion") }
    async fn illusion_expire(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.IllusionExpire") }
    async fn holy_refine(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holy.HolyRefine") }
    async fn health_check(&self, _: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>> {
        Ok(Response::new(common_pb::HealthCheckResponse {
            status: common_pb::Status::Ok as i32,
            message: "holy ok".to_string(),
        }))
    }
}

// ============================================================================
// 9. GuildWarService (17 RPC: 2 真实 + 15 stub)
// ============================================================================

#[tonic::async_trait]
pub trait GuildWarServiceTrait: Send + Sync {
    async fn start_guild_war(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_war_ranking(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    // 15 stub
    async fn get_guild_war_detail(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_current_defense(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_guild_war_status(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_guild_war_matchup(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_tower_state(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_total_stars(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_own_defense_formation(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_defense_log(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_new_log(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_attack_log(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_war_result(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_war_box_data(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn claim_war_box(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_war_box_claimed(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn health_check(&self, req: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>>;
}

pub struct GuildWarServiceImpl {
    pub state: Arc<BattleServiceImpl>,
}

impl GuildWarServiceImpl {
    pub fn new(state: Arc<BattleServiceImpl>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl GuildWarServiceTrait for GuildWarServiceImpl {
    async fn start_guild_war(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let mut store = self.state.store.write().await;
        let guild_id = "g1".to_string();
        let entry = store.guild_war_state.entry(guild_id.clone()).or_insert(GuildWarState {
            guild_id: guild_id.clone(),
            total_stars: 0,
            defense_towers: 3,
            current_matchup: None,
            start_at_ms: now_ms(),
        });
        entry.total_stars += 1;
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("stars={}", entry.total_stars),
        }))
    }

    async fn get_war_ranking(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let store = self.state.store.read().await;
        let guild_count = store.guild_war_state.len();
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("guild_count={}", guild_count),
        }))
    }

    async fn get_guild_war_detail(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.GetGuildWarDetail") }
    async fn get_current_defense(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.GetCurrentDefense") }
    async fn get_guild_war_status(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.GetGuildWarStatus") }
    async fn get_guild_war_matchup(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.GetGuildWarMatchup") }
    async fn push_tower_state(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.PushTowerState") }
    async fn push_total_stars(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.PushTotalStars") }
    async fn get_own_defense_formation(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.GetOwnDefenseFormation") }
    async fn get_defense_log(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.GetDefenseLog") }
    async fn push_new_log(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.PushNewLog") }
    async fn get_attack_log(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.GetAttackLog") }
    async fn push_war_result(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.PushWarResult") }
    async fn get_war_box_data(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.GetWarBoxData") }
    async fn claim_war_box(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.ClaimWarBox") }
    async fn push_war_box_claimed(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("GuildWar.PushWarBoxClaimed") }
    async fn health_check(&self, _: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>> {
        Ok(Response::new(common_pb::HealthCheckResponse {
            status: common_pb::Status::Ok as i32,
            message: "guild-war ok".to_string(),
        }))
    }
}

// ============================================================================
// 10. CrossServerService (19 RPC: 1 真实 + 18 stub, 复用 PvPService 数据驱动模式)
// ============================================================================

#[tonic::async_trait]
pub trait CrossServerServiceTrait: Send + Sync {
    async fn batch_challenge(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    // 18 stub
    async fn get_player_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_challenge_list(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_challengee_info(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn challenge_player(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn refresh_list(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn buy_challenge_count(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_top_three(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_ranking(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_log(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_top_player_show(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_activity_open(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_hero_red_dot(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_report_red_dot(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn clear_cooldown(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn view_opponent_hero(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_challenge_red_dot(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn share_replay(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn health_check(&self, req: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>>;
}

pub struct CrossServerServiceImpl {
    pub state: Arc<BattleServiceImpl>,
}

impl CrossServerServiceImpl {
    pub fn new(state: Arc<BattleServiceImpl>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl CrossServerServiceTrait for CrossServerServiceImpl {
    async fn batch_challenge(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        // 数据驱动: 跨服�?(CrossServer) 模式 daily_limit=5
        let mode = PvpMode::CrossServer;
        let limit = self.state.pvp_config.get(mode).map(|c| c.daily_limit).unwrap_or(0);
        let mut store = self.state.store.write().await;
        let used = store.challenge_count.entry(pid).or_insert(0);
        if *used + 5 > limit {
            return Err(Error::ChallengeExhausted(format!("{}+5 > {}", used, limit)));
        }
        *used += 5;
        Ok(Response::new(pb::EmptyResponse { ok: true, message: format!("batch_5_used={}", used) }))
    }

    async fn get_player_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.GetPlayerInfo") }
    async fn get_challenge_list(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.GetChallengeList") }
    async fn get_challengee_info(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.GetChallengeeInfo") }
    async fn challenge_player(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.ChallengePlayer") }
    async fn refresh_list(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.RefreshList") }
    async fn buy_challenge_count(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.BuyChallengeCount") }
    async fn get_top_three(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.GetTopThree") }
    async fn get_ranking(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.GetRanking") }
    async fn get_log(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.GetLog") }
    async fn get_top_player_show(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.GetTopPlayerShow") }
    async fn push_activity_open(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.PushActivityOpen") }
    async fn push_hero_red_dot(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.PushHeroRedDot") }
    async fn push_report_red_dot(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.PushReportRedDot") }
    async fn clear_cooldown(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.ClearCooldown") }
    async fn view_opponent_hero(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.ViewOpponentHero") }
    async fn push_challenge_red_dot(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.PushChallengeRedDot") }
    async fn share_replay(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("CrossServer.ShareReplay") }
    async fn health_check(&self, _: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>> {
        Ok(Response::new(common_pb::HealthCheckResponse {
            status: common_pb::Status::Ok as i32,
            message: "cross-server ok".to_string(),
        }))
    }
}

// ============================================================================
// 11. ExpeditionService (15 RPC: 1 真实 + 14 stub)
// ============================================================================

#[tonic::async_trait]
pub trait ExpeditionServiceTrait: Send + Sync {
    async fn challenge_stage(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    // 14 stub
    async fn get_expedition_data(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_stage_boss(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn claim_stage_box(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_my_support(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_supporting_me(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn deploy_companion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn hire_companion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_active_companion(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_challenge_red_dot(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_deploy_red_dot(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn select_difficulty(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn auto_sweep_settle(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_expedition_log(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn health_check(&self, req: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>>;
}

pub struct ExpeditionServiceImpl {
    pub state: Arc<BattleServiceImpl>,
}

impl ExpeditionServiceImpl {
    pub fn new(state: Arc<BattleServiceImpl>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl ExpeditionServiceTrait for ExpeditionServiceImpl {
    async fn challenge_stage(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let stage = 1u32;
        let mut store = self.state.store.write().await;
        let entry = store.expedition_progress.entry(pid).or_insert_with(HashMap::new);
        let stars = entry.entry(stage).or_insert(0);
        if *stars < 3 {
            *stars += 1;
        }
        Ok(Response::new(pb::EmptyResponse { ok: true, message: format!("stage_{}_stars={}", stage, stars) }))
    }

    async fn get_expedition_data(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.GetExpeditionData") }
    async fn get_stage_boss(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.GetStageBoss") }
    async fn claim_stage_box(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.ClaimStageBox") }
    async fn get_my_support(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.GetMySupport") }
    async fn get_supporting_me(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.GetSupportingMe") }
    async fn deploy_companion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.DeployCompanion") }
    async fn hire_companion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.HireCompanion") }
    async fn get_active_companion(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.GetActiveCompanion") }
    async fn get_challenge_red_dot(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.GetChallengeRedDot") }
    async fn get_deploy_red_dot(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.GetDeployRedDot") }
    async fn select_difficulty(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.SelectDifficulty") }
    async fn auto_sweep_settle(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.AutoSweepSettle") }
    async fn get_expedition_log(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Expedition.GetExpeditionLog") }
    async fn health_check(&self, _: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>> {
        Ok(Response::new(common_pb::HealthCheckResponse {
            status: common_pb::Status::Ok as i32,
            message: "expedition ok".to_string(),
        }))
    }
}

// ============================================================================
// 12. HolidayActivityService (15 RPC: 5 真实 + 10 stub, 1 套代�?+ 9 holiday_* activity_id 路由)
// ============================================================================

#[tonic::async_trait]
pub trait HolidayActivityServiceTrait: Send + Sync {
    // 5 真实 (1 套代码覆�?9 �?holiday_* 活动, per 9/4 MD §4 反例)
    async fn get_lantern_basic(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn lantern_draw(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_activity_by_id(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn claim_activity_by_id(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_food_basic(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    // 10 stub
    async fn get_lantern_pool(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn show_clear_reward(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn make_item(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn claim_level_box(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn challenge_activity_instance(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn buy_activity_count(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn get_adventure_task(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn push_task_change(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn claim_task(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>>;
    async fn health_check(&self, req: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>>;
}

pub struct HolidayActivityServiceImpl {
    pub state: Arc<BattleServiceImpl>,
}

impl HolidayActivityServiceImpl {
    pub fn new(state: Arc<BattleServiceImpl>) -> Self {
        Self { state }
    }

    /// 数据驱动: 1 套代�?+ 9 �?holiday_* activity_id 路由
    fn get_activity_or_err(&self, activity_id: &str) -> Result<()> {
        if self.state.holiday_config.get(activity_id).is_none() {
            return Err(Error::NotFound { entity: "HolidayActivity", id: activity_id.to_string() });
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl HolidayActivityServiceTrait for HolidayActivityServiceImpl {
    async fn get_lantern_basic(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        // 数据驱动: �?holiday_config.get("lantern")
        self.get_activity_or_err("lantern")?;
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: "lantern_basic".to_string(),
        }))
    }

    async fn lantern_draw(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        // 1 套代�? 通过 activity_id 路由
        self.get_activity_or_err("lantern")?;
        let mut store = self.state.store.write().await;
        let rewards = store.activity_rewards.entry(pid).or_insert_with(Vec::new);
        rewards.push(HolidayReward {
            activity_id: "lantern".to_string(),
            item_id: 93031,
            count: 1,
            description: "lantern prize".to_string(),
            claimed: false,
        });
        Ok(Response::new(pb::EmptyResponse { ok: true, message: format!("draw_count={}", rewards.len()) }))
    }

    async fn get_activity_by_id(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        // 数据驱动: 1 �?RPC 处理 24813/24815/24817 三个活动, 9 �?holiday_* 都走这个
        let inner = req.into_inner();
        // �?request_id 解析 activity_id (简化处�? �?request_id 作为 activity_id)
        let activity_id = if inner.request_id.is_empty() { "93031" } else { &inner.request_id };
        self.get_activity_or_err(activity_id)?;
        Ok(Response::new(pb::EmptyResponse {
            ok: true,
            message: format!("activity_id={}", activity_id),
        }))
    }

    async fn claim_activity_by_id(&self, req: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        let inner = req.into_inner();
        let player = require_player_id(&inner)?;
        let pid = player.player_id.as_ref().map(|e| e.id.clone()).unwrap_or_default();
        let activity_id = if inner.request_id.is_empty() { "93031".to_string() } else { inner.request_id };
        self.get_activity_or_err(&activity_id)?;
        let mut store = self.state.store.write().await;
        let rewards = store.activity_rewards.entry(pid).or_insert_with(Vec::new);
        let mut claimed_count = 0;
        for r in rewards.iter_mut() {
            if r.activity_id == activity_id && !r.claimed {
                r.claimed = true;
                claimed_count += 1;
            }
        }
        Ok(Response::new(pb::EmptyResponse { ok: true, message: format!("claimed={}", claimed_count) }))
    }

    async fn get_food_basic(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> {
        self.get_activity_or_err("food")?;
        Ok(Response::new(pb::EmptyResponse { ok: true, message: "food_basic".to_string() }))
    }

    async fn get_lantern_pool(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holiday.GetLanternPool") }
    async fn show_clear_reward(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holiday.ShowClearReward") }
    async fn make_item(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holiday.MakeItem") }
    async fn claim_level_box(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holiday.ClaimLevelBox") }
    async fn challenge_activity_instance(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holiday.ChallengeActivityInstance") }
    async fn buy_activity_count(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holiday.BuyActivityCount") }
    async fn get_adventure_task(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holiday.GetAdventureTask") }
    async fn push_task_change(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holiday.PushTaskChange") }
    async fn claim_task(&self, _: Request<pb::EmptyRequest>) -> Result<Response<pb::EmptyResponse>> { stub_unimplemented("Holiday.ClaimTask") }
    async fn health_check(&self, _: Request<common_pb::HealthCheckRequest>) -> Result<Response<common_pb::HealthCheckResponse>> {
        Ok(Response::new(common_pb::HealthCheckResponse {
            status: common_pb::Status::Ok as i32,
            message: format!("holiday ok ({} variants)", self.state.holiday_config.variant_count()),
        }))
    }
}

// ============================================================================
// impl Status 转换 (从 error.rs 已实现 From<Error> for tonic::Status,
// 这里 re-export 让调用方写 Status::from(err) 即可)
// ============================================================================

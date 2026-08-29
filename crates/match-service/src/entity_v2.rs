//! match-service 域 v2 entity 定义 (per RGS-DTL-038 v0.1 §4.2 + §5)
//!
//! 卡牌游戏 session / turn 抽象的 3 个核心 entity:
//! - `GameSession`: 对战 session (根实体, 状态机)
//! - `Move`: 操作日志
//! - `Board`: 战牌状态快照
//!
//! 注意: v1 entity (Match / MatchParticipant) 保留, 不破坏既有 5 域 matchmaker (5v5 等).
//! 详细设计见 RGS-DTL-038 §5.1 状态机图 + §5.2 状态转移表 + §4.2 message.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// 状态机 (per RGS-DTL-038 §5.1)
// ============================================================================

/// session 状态机 (per DTL-038 §5.1)
/// ```
/// Creating → Waiting → Starting → Running → Turn_N → Paused → Ending → Ended
///                              ↘ Canceled
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// 创建中 (host CreateMatch 调用瞬间)
    Creating = 1,
    /// 等待玩家 (ROOM 模式专属)
    Waiting = 2,
    /// 启动中 (加载卡组 / 初始 Board)
    Starting = 3,
    /// 运行中 (主循环)
    Running = 4,
    /// 回合中 (per-turn 状态, 可与 Running 互转, 简化建模为同 Running + turn_index)
    /// 注: per DTL-038 §5.1, Turn_N 是 Running 的子状态; 这里统一记为 Running,
    /// turn_index 区分具体回合.
    // TurnN = 5,  // 不单设 enum, 用 Running + turn_index 表达
    /// 暂停 (断线 / GM 暂停)
    Paused = 6,
    /// 收尾中 (结算 / 保存回放)
    Ending = 7,
    /// 已结束 (终态)
    Ended = 8,
    /// 已取消 (终态, 区别 Ended: 未真正开始)
    Canceled = 9,
}

impl SessionStatus {
    /// 是否终态 (不可再转移)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Ended | Self::Canceled)
    }

    /// 是否可执行 SubmitMove
    pub fn accepts_moves(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// 是否需要玩家活跃 (Running 状态)
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Creating => "creating",
            Self::Waiting => "waiting",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Ending => "ending",
            Self::Ended => "ended",
            Self::Canceled => "canceled",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for SessionStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "creating" => Ok(Self::Creating),
            "waiting" => Ok(Self::Waiting),
            "starting" => Ok(Self::Starting),
            "running" => Ok(Self::Running),
            "paused" => Ok(Self::Paused),
            "ending" => Ok(Self::Ending),
            "ended" => Ok(Self::Ended),
            "canceled" | "cancelled" => Ok(Self::Canceled),
            _ => Err(format!("unknown session status: {}", s)),
        }
    }
}

// ============================================================================
// 模式 / Move 类型 (per common.v1 GameMode / MoveType)
// ============================================================================

/// 游戏模式 (per common.v1.GameMode enum)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    Unspecified = 0,
    Ranked = 1,
    Casual = 2,
    Room = 3,
    PveAi = 4,
}

impl std::fmt::Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Unspecified => "unspecified",
            Self::Ranked => "ranked",
            Self::Casual => "casual",
            Self::Room => "room",
            Self::PveAi => "pve_ai",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for GameMode {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "ranked" => Ok(Self::Ranked),
            "casual" => Ok(Self::Casual),
            "room" => Ok(Self::Room),
            "pve_ai" | "pve-ai" | "pveai" => Ok(Self::PveAi),
            _ => Ok(Self::Unspecified),
        }
    }
}

/// Move 类型 (per match.proto MoveType)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MoveType {
    Unspecified = 0,
    PlayCard = 1,
    Attack = 2,
    EndTurn = 3,
    Surrender = 4,
    UseAbility = 5,
}

impl std::fmt::Display for MoveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Unspecified => "unspecified",
            Self::PlayCard => "play_card",
            Self::Attack => "attack",
            Self::EndTurn => "end_turn",
            Self::Surrender => "surrender",
            Self::UseAbility => "use_ability",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for MoveType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "play_card" | "play-card" | "playcard" => Ok(Self::PlayCard),
            "attack" => Ok(Self::Attack),
            "end_turn" | "end-turn" | "endturn" => Ok(Self::EndTurn),
            "surrender" => Ok(Self::Surrender),
            "use_ability" | "use-ability" | "useability" => Ok(Self::UseAbility),
            _ => Ok(Self::Unspecified),
        }
    }
}

// ============================================================================
// Player 端信息 (per common.v1.PlayerId + CardRef 复合)
// ============================================================================

/// session 内玩家信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionPlayer {
    /// 玩家 ID (string per common.v1.PlayerId.player_id.id, 同时支持 UUID 和自定 ID)
    pub player_id: String,
    /// 显示名
    pub display_name: String,
    /// 天梯积分
    pub rank_score: u32,
    /// 等级
    pub level: u32,
    /// 卡组引用 (per common.v1.CardRef, 可空)
    pub deck_card_id: Option<String>,
    pub deck_instance_id: Option<String>,
    /// 队伍 (1v1/2v2 等, per 通用 v1 Team)
    pub team: i32,
    /// 是否已投降
    pub surrendered: bool,
    /// 是否断线
    pub disconnected: bool,
}

impl SessionPlayer {
    pub fn new(player_id: String, display_name: String) -> Self {
        Self {
            player_id,
            display_name,
            rank_score: 0,
            level: 1,
            deck_card_id: None,
            deck_instance_id: None,
            team: 0,
            surrendered: false,
            disconnected: false,
        }
    }

    pub fn with_deck(mut self, card_id: Option<String>, instance_id: Option<String>) -> Self {
        self.deck_card_id = card_id;
        self.deck_instance_id = instance_id;
        self
    }

    pub fn with_rank(mut self, rank_score: u32, level: u32) -> Self {
        self.rank_score = rank_score;
        self.level = level;
        self
    }

    pub fn with_team(mut self, team: i32) -> Self {
        self.team = team;
        self
    }
}

// ============================================================================
// Board 战牌状态 (per §4.2 GetMatchStateResponse.board_snapshot JSON)
// ============================================================================

/// 战牌状态快照 (JSON-serializable)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Board {
    /// 战牌实例列表 (玩家单位 / 法术 / 装备)
    pub units: Vec<BoardUnit>,
    /// 玩家手牌 (key = player_id, value = [card_id, ...])
    pub hands: std::collections::HashMap<String, Vec<String>>,
    /// 玩家牌库 (key = player_id, value = [card_id, ...])
    pub decks: std::collections::HashMap<String, Vec<String>>,
    /// 玩家墓地 (key = player_id, value = [card_id, ...])
    pub graveyards: std::collections::HashMap<String, Vec<String>>,
    /// 全局计数器 (回合数 / 资源数 / ...)
    pub counters: std::collections::HashMap<String, i32>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            units: Vec::new(),
            hands: std::collections::HashMap::new(),
            decks: std::collections::HashMap::new(),
            graveyards: std::collections::HashMap::new(),
            counters: std::collections::HashMap::new(),
        }
    }
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前在战牌上的单位数
    pub fn unit_count(&self) -> usize {
        self.units.len()
    }
}

/// 战牌单位
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoardUnit {
    pub unit_id: String,
    pub card_id: String,
    pub owner_id: String,
    pub attack: i32,
    pub health: i32,
    pub can_attack: bool,
}

// ============================================================================
// GameSession 根实体 (per §4.2 Match + §5.1 状态机)
// ============================================================================

/// 对战 session (per DTL-038 §5.1 状态机)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameSession {
    /// match_id
    pub match_id: Uuid,

    /// 模式
    pub mode: GameMode,

    /// 状态机
    pub status: SessionStatus,

    /// 玩家列表 (2-N, per §5 状态机进入 Running 需 ≥ min_players)
    pub players: Vec<SessionPlayer>,

    /// 房主 player_id (CreateMatch 触发者)
    pub host_id: Option<String>,

    /// 房间码 (ROOM 模式)
    pub room_code: Option<String>,
    /// 房间密码 hash (不存明文)
    pub room_password_hash: Option<String>,

    /// 玩家上限
    pub max_players: u32,
    /// 玩家下限 (进入 Running 需 ≥ min_players)
    pub min_players: u32,

    /// 当前回合索引 (从 0 开始)
    pub turn_index: u32,
    /// 当前回合玩家
    pub current_player_id: Option<String>,

    /// 当前 turn 截止时间 (epoch ms)
    pub next_turn_deadline_ms: Option<i64>,

    /// 战牌状态
    pub board: Board,
    /// 对象存储引用 (per §4.2 Match.board_snapshot_ref, 可选, 大 snapshot 走对象存储)
    pub board_snapshot_ref: Option<String>,

    /// 胜者 (status=Ended 时填)
    pub winner_id: Option<String>,
    /// 结束原因 (per §5.2: surrender / disconnect / timeout / game_logic / canceled)
    pub end_reason: Option<String>,

    /// AI 难度 (PVE 模式, 0=无)
    pub ai_difficulty: u32,

    /// 当前玩家累计超时次数 (per §5.3 ≥ 3 判负)
    pub timeout_count: u32,

    /// 待执行 move 队列 (per §4.2 GetMatchStateResponse.pending_moves)
    pub pending_moves: Vec<Move>,

    /// 时间戳
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GameSession {
    /// 工厂: 新建 session (默认 status=Creating)
    pub fn new(mode: GameMode, host: SessionPlayer, max_players: u32, min_players: u32) -> Self {
        let now = Utc::now();
        let host_id = Some(host.player_id.clone());
        let mut players = Vec::new();
        players.push(host);
        Self {
            match_id: Uuid::new_v4(),
            mode,
            status: SessionStatus::Creating,
            players,
            host_id,
            room_code: None,
            room_password_hash: None,
            max_players,
            min_players,
            turn_index: 0,
            current_player_id: None,
            next_turn_deadline_ms: None,
            board: Board::new(),
            board_snapshot_ref: None,
            winner_id: None,
            end_reason: None,
            ai_difficulty: 0,
            timeout_count: 0,
            pending_moves: Vec::new(),
            started_at: now,
            ended_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    // ===== 状态转移函数 (per §5.2 状态转移表) =====

    /// CREATING → WAITING (CreateMatch for ROOM 模式)
    pub fn transition_to_waiting(&mut self) -> Result<(), &'static str> {
        if self.status != SessionStatus::Creating {
            return Err("transition_to_waiting: must be Creating");
        }
        self.status = SessionStatus::Waiting;
        self.touch();
        Ok(())
    }

    /// CREATING/WAITING → STARTING (玩家到齐, 加载卡组)
    pub fn transition_to_starting(&mut self) -> Result<(), &'static str> {
        if !matches!(
            self.status,
            SessionStatus::Creating | SessionStatus::Waiting
        ) {
            return Err("transition_to_starting: must be Creating or Waiting");
        }
        if self.players.len() < self.min_players as usize {
            return Err("transition_to_starting: not enough players");
        }
        self.status = SessionStatus::Starting;
        self.touch();
        Ok(())
    }

    /// STARTING → RUNNING (初始 Board 完成)
    pub fn transition_to_running(&mut self) -> Result<(), &'static str> {
        if self.status != SessionStatus::Starting {
            return Err("transition_to_running: must be Starting");
        }
        self.status = SessionStatus::Running;
        self.turn_index = 0;
        // 第 0 回合先手 = host (or first player)
        self.current_player_id = self
            .host_id
            .clone()
            .or_else(|| self.players.first().map(|p| p.player_id.clone()));
        self.touch();
        Ok(())
    }

    /// RUNNING → PAUSED (断线 / GM 暂停)
    pub fn transition_to_paused(&mut self) -> Result<(), &'static str> {
        if self.status != SessionStatus::Running {
            return Err("transition_to_paused: must be Running");
        }
        self.status = SessionStatus::Paused;
        self.touch();
        Ok(())
    }

    /// PAUSED → RUNNING (重连 / GM 恢复)
    pub fn transition_to_resumed(&mut self) -> Result<(), &'static str> {
        if self.status != SessionStatus::Paused {
            return Err("transition_to_resumed: must be Paused");
        }
        self.status = SessionStatus::Running;
        self.touch();
        Ok(())
    }

    /// RUNNING → ENDING (胜负判定 / 投降 / 超时)
    pub fn transition_to_ending(&mut self, winner: Option<String>, reason: String) -> Result<(), &'static str> {
        if !matches!(self.status, SessionStatus::Running | SessionStatus::Paused) {
            return Err("transition_to_ending: must be Running or Paused");
        }
        self.status = SessionStatus::Ending;
        self.winner_id = winner;
        self.end_reason = Some(reason);
        self.touch();
        Ok(())
    }

    /// ENDING → ENDED (回放保存)
    pub fn transition_to_ended(&mut self) -> Result<(), &'static str> {
        if self.status != SessionStatus::Ending {
            return Err("transition_to_ended: must be Ending");
        }
        self.status = SessionStatus::Ended;
        self.ended_at = Some(Utc::now());
        self.touch();
        Ok(())
    }

    /// * → CANCELED (创建中 / 等待中 / 暂停中取消)
    pub fn transition_to_canceled(&mut self, reason: String) -> Result<(), &'static str> {
        if self.status.is_terminal() {
            return Err("transition_to_canceled: already terminal");
        }
        self.status = SessionStatus::Canceled;
        self.end_reason = Some(reason);
        self.ended_at = Some(Utc::now());
        self.touch();
        Ok(())
    }

    /// 切换到下一回合 (per §5.2 RUNNING → TURN_N → RUNNING, 简化为单步)
    pub fn advance_turn(&mut self, deadline_ms: Option<i64>) -> Result<(), &'static str> {
        if self.status != SessionStatus::Running {
            return Err("advance_turn: must be Running");
        }
        if self.players.is_empty() {
            return Err("advance_turn: no players");
        }
        // 轮转: (current_idx + 1) % len
        let current_idx = self
            .current_player_id
            .as_ref()
            .and_then(|pid| self.players.iter().position(|p| &p.player_id == pid))
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.players.len();
        self.current_player_id = Some(self.players[next_idx].player_id.clone());
        self.turn_index = self.turn_index.saturating_add(1);
        self.next_turn_deadline_ms = deadline_ms;
        // 注: 不再重置 timeout_count (per §5.3 累计 3 次判负语义)
        self.touch();
        Ok(())
    }

    /// 玩家加入 (CREATING / WAITING 状态)
    pub fn add_player(&mut self, player: SessionPlayer) -> Result<(), &'static str> {
        if !matches!(self.status, SessionStatus::Creating | SessionStatus::Waiting) {
            return Err("add_player: must be Creating or Waiting");
        }
        if self.players.len() >= self.max_players as usize {
            return Err("add_player: session full");
        }
        if self.players.iter().any(|p| p.player_id == player.player_id) {
            return Err("add_player: player already in session");
        }
        self.players.push(player);
        self.touch();
        Ok(())
    }

    /// 玩家离开 / 投降 (RUNNING / PAUSED)
    pub fn remove_player(&mut self, player_id: &str, surrender: bool) -> Result<bool, &'static str> {
        if let Some(idx) = self.players.iter().position(|p| p.player_id == player_id) {
            self.players[idx].surrendered = surrender;
            // 简化: 标记 disconnected = true 即可, 不真删 (保留回放)
            if !surrender {
                self.players[idx].disconnected = true;
            }
            self.touch();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= self.max_players as usize
    }

    pub fn is_ready_to_start(&self) -> bool {
        self.players.len() >= self.min_players as usize
    }

    pub fn active_player_count(&self) -> usize {
        self.players
            .iter()
            .filter(|p| !p.surrendered && !p.disconnected)
            .count()
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

// ============================================================================
// Move 实体 (per §4.2 Move + §7.1 moves 表)
// ============================================================================

/// 操作日志
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Move {
    pub move_id: Uuid,
    pub match_id: Uuid,
    pub player_id: String,
    pub turn_index: u32,
    pub move_type: MoveType,
    pub payload_json: String,
    pub result_json: Option<String>,
    pub accepted: bool,
    pub reject_reason: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

impl Move {
    pub fn new(match_id: Uuid, player_id: String, turn_index: u32, move_type: MoveType, payload_json: String) -> Self {
        Self {
            move_id: Uuid::new_v4(),
            match_id,
            player_id,
            turn_index,
            move_type,
            payload_json,
            result_json: None,
            accepted: true,
            reject_reason: None,
            occurred_at: Utc::now(),
        }
    }

    pub fn rejected(mut self, reason: String) -> Self {
        self.accepted = false;
        self.reject_reason = Some(reason);
        self
    }

    pub fn with_result(mut self, result_json: String) -> Self {
        self.result_json = Some(result_json);
        self
    }
}

// ============================================================================
// MatchmakingTicket (per §4.2 EnqueueMatchmakingResponse.ticket_id)
// ============================================================================

/// 撮合 ticket
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchmakingTicket {
    pub ticket_id: Uuid,
    pub player_id: String,
    pub mode: GameMode,
    pub rank_score_min: u32,
    pub rank_score_max: u32,
    pub deck_card_id: Option<String>,
    pub deck_instance_id: Option<String>,
    /// TicketStatus: 1=queued 2=matched 3=cancelled 4=expired
    pub status: i32,
    pub match_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub matched_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

impl MatchmakingTicket {
    pub fn new(
        player_id: String,
        mode: GameMode,
        rank_score_min: u32,
        rank_score_max: u32,
        deck_card_id: Option<String>,
        deck_instance_id: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            ticket_id: Uuid::new_v4(),
            player_id,
            mode,
            rank_score_min,
            rank_score_max,
            deck_card_id,
            deck_instance_id,
            status: 1, // queued
            match_id: None,
            created_at: now,
            matched_at: None,
            cancelled_at: None,
            expires_at: now + chrono::Duration::minutes(5),
        }
    }

    pub fn matched(&mut self, match_id: Uuid) {
        self.status = 2;
        self.match_id = Some(match_id);
        self.matched_at = Some(Utc::now());
    }

    pub fn cancelled(&mut self) {
        self.status = 3;
        self.cancelled_at = Some(Utc::now());
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_player(id: &str) -> SessionPlayer {
        SessionPlayer::new(id.to_string(), format!("Player-{}", id)).with_team(1)
    }

    fn make_session(num_players: usize) -> GameSession {
        let host = make_player("p1");
        // max=动态 (num_players+1), min=2 (兼容既有 2 玩家测试)
        let max = (num_players as u32 + 1).max(2);
        let min = 2;
        let mut s = GameSession::new(GameMode::Ranked, host, max, min);
        for i in 2..=num_players {
            s.add_player(make_player(&format!("p{}", i))).unwrap();
        }
        s
    }

    #[test]
    fn session_status_terminal() {
        assert!(!SessionStatus::Creating.is_terminal());
        assert!(!SessionStatus::Running.is_terminal());
        assert!(SessionStatus::Ended.is_terminal());
        assert!(SessionStatus::Canceled.is_terminal());
    }

    #[test]
    fn session_status_accepts_moves() {
        assert!(!SessionStatus::Creating.accepts_moves());
        assert!(SessionStatus::Running.accepts_moves());
        assert!(!SessionStatus::Paused.accepts_moves());
    }

    #[test]
    fn session_lifecycle_happy_path() {
        let mut s = make_session(2);
        assert_eq!(s.status, SessionStatus::Creating);
        s.transition_to_starting().unwrap();
        assert_eq!(s.status, SessionStatus::Starting);
        s.transition_to_running().unwrap();
        assert_eq!(s.status, SessionStatus::Running);
        assert_eq!(s.turn_index, 0);
        s.advance_turn(Some(1000)).unwrap();
        assert_eq!(s.turn_index, 1);
        s.transition_to_ending(Some("p1".to_string()), "game_logic".to_string())
            .unwrap();
        s.transition_to_ended().unwrap();
        assert_eq!(s.status, SessionStatus::Ended);
        assert_eq!(s.winner_id, Some("p1".to_string()));
    }

    #[test]
    fn session_room_mode_waiting_to_starting() {
        let host = make_player("host");
        let mut s = GameSession::new(GameMode::Room, host, 4, 2);
        s.transition_to_waiting().unwrap();
        assert_eq!(s.status, SessionStatus::Waiting);
        s.add_player(make_player("p2")).unwrap();
        s.add_player(make_player("p3")).unwrap();
        s.transition_to_starting().unwrap();
        assert_eq!(s.status, SessionStatus::Starting);
    }

    #[test]
    fn session_full_rejects_add() {
        // 自建 max=2/min=2 session, 第 3 人 add 必须失败
        let host = make_player("p1");
        let mut s = GameSession::new(GameMode::Ranked, host, 2, 2);
        s.add_player(make_player("p2")).unwrap();
        s.add_player(make_player("p3")).unwrap_err();
    }

    #[test]
    fn session_pause_resume() {
        let mut s = make_session(2);
        s.transition_to_starting().unwrap();
        s.transition_to_running().unwrap();
        s.transition_to_paused().unwrap();
        assert_eq!(s.status, SessionStatus::Paused);
        s.transition_to_resumed().unwrap();
        assert_eq!(s.status, SessionStatus::Running);
    }

    #[test]
    fn session_cancel_from_creating() {
        let mut s = GameSession::new(GameMode::Casual, make_player("p1"), 2, 2);
        s.transition_to_canceled("host_left".to_string()).unwrap();
        assert_eq!(s.status, SessionStatus::Canceled);
    }

    #[test]
    fn session_advance_turn_rotates() {
        let mut s = make_session(3);
        s.transition_to_starting().unwrap();
        s.transition_to_running().unwrap();
        let first = s.current_player_id.clone().unwrap();
        s.advance_turn(Some(1000)).unwrap();
        let second = s.current_player_id.clone().unwrap();
        assert_ne!(first, second);
        s.advance_turn(Some(2000)).unwrap();
        let third = s.current_player_id.clone().unwrap();
        assert_ne!(second, third);
        s.advance_turn(Some(3000)).unwrap();
        // 轮转回 first
        assert_eq!(s.current_player_id.clone().unwrap(), first);
    }

    #[test]
    fn session_illegal_transition_rejected() {
        let mut s = GameSession::new(GameMode::Casual, make_player("p1"), 2, 2);
        // Creating → Running 非法, 必须先 starting
        let err = s.transition_to_running();
        assert!(err.is_err());
    }

    #[test]
    fn session_remove_player_marks_disconnect() {
        let mut s = make_session(2);
        s.transition_to_starting().unwrap();
        s.transition_to_running().unwrap();
        let removed = s.remove_player("p2", false).unwrap();
        assert!(removed);
        assert!(s.players.iter().find(|p| p.player_id == "p2").unwrap().disconnected);
    }

    #[test]
    fn session_remove_player_surrender() {
        let mut s = make_session(2);
        s.remove_player("p1", true).unwrap();
        assert!(s.players.iter().find(|p| p.player_id == "p1").unwrap().surrendered);
    }

    #[test]
    fn session_remove_player_not_found() {
        let mut s = make_session(2);
        let removed = s.remove_player("nonexistent", false).unwrap();
        assert!(!removed);
    }

    #[test]
    fn move_creation_default_accepted() {
        let m = Move::new(Uuid::new_v4(), "p1".to_string(), 1, MoveType::EndTurn, "{}".to_string());
        assert!(m.accepted);
        assert!(m.reject_reason.is_none());
    }

    #[test]
    fn move_rejected_with_reason() {
        let m = Move::new(Uuid::new_v4(), "p1".to_string(), 1, MoveType::PlayCard, "{}".to_string())
            .rejected("invalid_state".to_string());
        assert!(!m.accepted);
        assert_eq!(m.reject_reason, Some("invalid_state".to_string()));
    }

    #[test]
    fn ticket_lifecycle() {
        let mut t = MatchmakingTicket::new("p1".to_string(), GameMode::Ranked, 1000, 2000, None, None);
        assert_eq!(t.status, 1);
        let match_id = Uuid::new_v4();
        t.matched(match_id);
        assert_eq!(t.status, 2);
        assert_eq!(t.match_id, Some(match_id));
    }

    #[test]
    fn ticket_cancelled() {
        let mut t = MatchmakingTicket::new("p1".to_string(), GameMode::Casual, 0, 0, None, None);
        t.cancelled();
        assert_eq!(t.status, 3);
    }

    #[test]
    fn game_mode_display() {
        assert_eq!(GameMode::Ranked.to_string(), "ranked");
        assert_eq!(GameMode::PveAi.to_string(), "pve_ai");
    }

    #[test]
    fn session_status_display_round_trip() {
        for s in [
            SessionStatus::Creating,
            SessionStatus::Waiting,
            SessionStatus::Running,
            SessionStatus::Ended,
            SessionStatus::Canceled,
        ] {
            let parsed: SessionStatus = s.to_string().parse().unwrap();
            assert_eq!(parsed, s);
        }
    }
}

//! Sample data 工厂 + DB init/teardown
//!
//! ## 53.3 骨架 (backward compat)
//! - `player()`  生成 sample player
//! - `economy()` 生成 sample economy balance
//! - `saga()`    生成 sample saga 上下文
//!
//! ## 54.x 5 域 fixture (新增)
//! - `match_game()`       5 域 match-service 用 (DTL-026)
//! - `social_message()`   5 域 social-service 用 (DTL-019 / DTL-020)
//! - `admin_action()`     5 域 admin-service 用 (DTL-031)
//!
//! ## FixtureBuilder
//! 链式 API 自定义 sample data (5 域 + Player / Economy 全部支持)
//!
//! ## init_test_db
//! - `testcontainers` feature 启用: 启动真 PG 容器
//! - 默认: 读 `TEST_DATABASE_URL` env var, 否则返回占位 URL
//!
//! 规范: RGS-SPEC-000 §2.4 + RGS-IMPL-001 §3

use serde::{Deserialize, Serialize};

// ============================================================================
// 53.3 fixture: backward compat (Player / Economy / Saga)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerFixture {
    pub id: String,
    pub name: String,
    pub level: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EconomyFixture {
    pub player_id: String,
    pub currency: i64,
    pub gold: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SagaFixture {
    pub saga_id: String,
    pub saga_type: String,
    pub step: u32,
    pub state: String,
}

/// 生成 sample player（53.3 占位：固定值）
pub fn player() -> PlayerFixture {
    PlayerFixture {
        id: "player-test-001".to_string(),
        name: "Test Player".to_string(),
        level: 1,
        created_at: chrono::Utc::now(),
    }
}

/// 生成 sample economy balance
pub fn economy(player_id: &str) -> EconomyFixture {
    EconomyFixture {
        player_id: player_id.to_string(),
        currency: 1000,
        gold: 50,
    }
}

/// 生成 sample saga 上下文
pub fn saga(saga_type: &str) -> SagaFixture {
    SagaFixture {
        saga_id: format!("saga-test-{}", uuid::Uuid::new_v4()),
        saga_type: saga_type.to_string(),
        step: 0,
        state: "Pending".to_string(),
    }
}

// ============================================================================
// 5 域 fixture: match / social / admin (per 5 域 DTL: DTL-026 / DTL-019 / DTL-031)
// ============================================================================

/// Match 域 sample: 5 域 match-service 用
/// DTL-026: match lifecycle (Pending / Active / Completed)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchFixture {
    pub match_id: String,
    pub player_id: String,
    pub score: u32,
    pub status: String, // "Pending" / "Active" / "Completed"
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// Social 域 sample: 5 域 social-service 用
/// DTL-019 (friend) / DTL-020 (message)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SocialFixture {
    pub player_id: String,
    pub friend_id: String,
    pub message: String,
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

/// Admin 域 sample: 5 域 admin-service 用
/// DTL-031: admin action (ban / mute / promote / demote)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdminFixture {
    pub admin_id: String,
    pub action: String, // "ban" / "mute" / "promote" / "demote"
    pub target_id: String,
    pub performed_at: chrono::DateTime<chrono::Utc>,
}

/// 生成 sample match 上下文
pub fn match_game(player_id: &str) -> MatchFixture {
    MatchFixture {
        match_id: format!("match-test-{}", uuid::Uuid::new_v4()),
        player_id: player_id.to_string(),
        score: 0,
        status: "Pending".to_string(),
        started_at: chrono::Utc::now(),
    }
}

/// 生成 sample social 消息
pub fn social_message(from: &str, to: &str) -> SocialFixture {
    SocialFixture {
        player_id: from.to_string(),
        friend_id: to.to_string(),
        message: "Hello from test".to_string(),
        sent_at: chrono::Utc::now(),
    }
}

/// 生成 sample admin 动作
pub fn admin_action(admin: &str, action: &str, target: &str) -> AdminFixture {
    AdminFixture {
        admin_id: admin.to_string(),
        action: action.to_string(),
        target_id: target.to_string(),
        performed_at: chrono::Utc::now(),
    }
}

// ============================================================================
// FixtureBuilder: 链式 API 自定义 sample data
// ============================================================================

/// 链式 API 自定义 sample data
///
/// # Examples
///
/// ```no_run
/// use rgs_testkit::fixture::{self, FixtureBuilder};
/// let p = FixtureBuilder::new(fixture::player())
///     .with_name("Custom")
///     .with_level(99)
///     .build();
/// assert_eq!(p.name, "Custom");
/// assert_eq!(p.level, 99);
/// ```
pub struct FixtureBuilder<T: Clone> {
    fixture: T,
}

impl<T: Clone> FixtureBuilder<T> {
    /// 用 sample fixture 初始化 builder
    pub fn new(fixture: T) -> Self {
        Self { fixture }
    }

    /// 拿 builder 当前 state 的 clone
    pub fn build(&self) -> T {
        self.fixture.clone()
    }
}

// --- PlayerFixture builder shortcuts ---

impl FixtureBuilder<PlayerFixture> {
    pub fn with_name(mut self, name: &str) -> Self {
        self.fixture.name = name.to_string();
        self
    }
    pub fn with_level(mut self, level: u32) -> Self {
        self.fixture.level = level;
        self
    }
}

// --- EconomyFixture builder shortcuts ---

impl FixtureBuilder<EconomyFixture> {
    pub fn with_currency(mut self, currency: i64) -> Self {
        self.fixture.currency = currency;
        self
    }
    pub fn with_gold(mut self, gold: i64) -> Self {
        self.fixture.gold = gold;
        self
    }
}

// --- MatchFixture builder shortcuts ---

impl FixtureBuilder<MatchFixture> {
    pub fn with_score(mut self, score: u32) -> Self {
        self.fixture.score = score;
        self
    }
    pub fn with_status(mut self, status: &str) -> Self {
        self.fixture.status = status.to_string();
        self
    }
}

// --- SocialFixture builder shortcuts ---

impl FixtureBuilder<SocialFixture> {
    pub fn with_message(mut self, message: &str) -> Self {
        self.fixture.message = message.to_string();
        self
    }
}

// --- AdminFixture builder shortcuts ---

impl FixtureBuilder<AdminFixture> {
    pub fn with_action(mut self, action: &str) -> Self {
        self.fixture.action = action.to_string();
        self
    }
    pub fn with_target(mut self, target_id: &str) -> Self {
        self.fixture.target_id = target_id.to_string();
        self
    }
}

// ============================================================================
// init_test_db: feature-gated 双实现
// - testcontainers feature 启用: 启动真 PG 容器 (per 54.x 接入)
// - 默认: 读 TEST_DATABASE_URL env var, 否则返回占位 URL (backward compat)
// ============================================================================

/// 测试 DB 初始化（54.x 接入 testcontainers-rs，feature-gated）
///
/// - `testcontainers` feature 启用时：启动真 PG 容器，返回连接 URL
/// - 默认 fallback：优先 `TEST_DATABASE_URL` env var，否则占位 URL
#[cfg(feature = "testcontainers")]
pub async fn init_test_db(name: &str) -> anyhow::Result<String> {
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;

    let container = Postgres::default().start().await?;
    let port = container.get_host_port_ipv4(5432).await?;
    Ok(format!(
        "postgres://postgres:postgres@localhost:{}/{}",
        port, name
    ))
}

/// 测试 DB 初始化（fallback: env-var 或占位）
///
/// 优先读 `TEST_DATABASE_URL` env var；若无则返回占位 URL（fake host）。
/// 占位 URL 不会真连 PG，仅满足 53.3 self_test 形状不变。
#[cfg(not(feature = "testcontainers"))]
pub async fn init_test_db(name: &str) -> anyhow::Result<String> {
    if let Ok(url) = std::env::var("TEST_DATABASE_URL") {
        return Ok(url);
    }
    // 占位: future PG 容器 (53.3 backward compat)
    Ok(format!("postgres://test:test@localhost:5432/{}", name))
}

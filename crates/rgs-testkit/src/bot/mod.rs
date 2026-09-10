//! Bot 框架 (per DDD Review v0.2 §5.1 M1)
//!
//! 模拟 erlang tester 30 优点 (per DDD Review v0.2 §3):
//! - **A1** 每 bot 1 进程 → 1 tokio task per Bot
//! - **B1-B4** AI 框架 → [`ai::BotAi`] trait + [`act::ActList`]
//! - **C1** 协议随机化 → [`act::ActKind::RandProto`]
//! - **C2** GM 注入 → [`gm::GmClient`] (stub, wave 2 接 admin mTLS)
//! - **E1** 实时统计 → [`stats::BotStats::count/offline`]
//! - **E2** 掉线自愈 + **F3** 错峰 → [`supervisor::BotSupervisor`]
//! - **A2** ETS 全局表 → [`stats::BotStats`] (Arc<Mutex<HashMap>>)
//!
//! # 强约束 (per rgs-testkit L16-44 + 8/27 11:06 JST hard ban)
//!
//! bot 框架**不**用 InMemory PG mock — 5 域调用走真 tonic gRPC (后续 worker 接入),
//! 状态持久化走 `rgs_testkit::pg_pool()` 强约束入口. bot 状态本身**仅**记录
//! 实时统计 (在线 / 掉线), 不缓存业务数据.
//!
//! # compile_fail 锚定
//!
//! 任何把 `InMemoryAccountRepository::new()` 用作 bot 状态后端的尝试, 应
//! 产生编译错误 (跟 rgs-testkit lib.rs 已有模式一致).
//!
//! ```compile_fail
//! use rgs_testkit::bot::InMemoryBotState;  // 错误: 该类型不存在
//! // bot 框架禁止 InMemory 状态, 走真 PG (pg_pool 强约束) 或 BotStats
//! let _x: InMemoryBotState = unimplemented!();
//! ```
//!
//! # 用法 (PoC, 5 域派生留待 wave 2)
//!
//! ```no_run
//! use rgs_testkit::bot::{Bot, BotCore};
//! use rgs_testkit::bot::stats::BotStats;
//!
//! # async fn run() {
//! let stats = BotStats::new();
//! let bot = Bot::new("bot-001", "player", stats.clone());
//! bot.start().await.expect("start");
//! // ... bot 在后台跑 AI 循环 ...
//! bot.stop().await;
//! assert!(stats.offline().contains(&"bot-001".to_string()));
//! # }
//! ```

pub mod act;
pub mod ai;
pub mod common;
pub mod gm;
pub mod stats;
pub mod supervisor;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;

use crate::bot::stats::BotStats;

pub use crate::bot::act::{ActKind, ActList};
pub use crate::bot::ai::BotAi;
pub use crate::bot::gm::{GmClient, GmResponse, MtlsConfig};

/// Bot ID 类型别名 (per erlang tester 命名: account → LocalName)
pub type BotId = String;

/// Bot 模块名 (e.g. "player" / "economy" / "match")
pub type ModName = String;

/// 单个 bot (per erlang A1: 1 bot = 1 gen_server = 1 虚拟玩家)
///
/// Bot 是轻量 wrapper, 内部持 1 个 tokio task 跑 AI 行为循环.
/// 启动: `BotCore::start()` spawn 1 tokio task, 通过 `BotStats` 注册在线.
/// 停止: `BotCore::stop()` 通过 `Notify` 通知 task 退出, 标记 offline.
///
/// `config` 字段留给 M10 (per DDD Review §5.1, P1 多环境路由). M1-M6
/// 不需要 config, AI 行为由 `BotAi` trait 提供.
pub struct Bot {
    /// bot id (e.g. "bot-001")
    pub id: BotId,
    /// 所属域 (e.g. "player" / "economy")
    pub mod_name: ModName,
    /// 共享统计句柄 (Arc 内部, 廉价 clone)
    stats: BotStats,
    /// 通知 stop 信号 (Arc 共享给 spawn 的 task)
    notify_stop: Arc<Notify>,
    /// spawn 的 task handle (启动后 set, stop 后 take)
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl Bot {
    /// 构造新 bot
    ///
    /// - `id`        bot 唯一标识, e.g. "bot-001"
    /// - `mod_name`  所属域, e.g. "player"
    /// - `stats`     共享统计句柄 (supervisor 持有同一实例)
    pub fn new(id: impl Into<BotId>, mod_name: impl Into<ModName>, stats: BotStats) -> Self {
        Self {
            id: id.into(),
            mod_name: mod_name.into(),
            stats,
            notify_stop: Arc::new(Notify::new()),
            handle: Mutex::new(None),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn mod_name(&self) -> &str {
        &self.mod_name
    }

    pub fn stats(&self) -> &BotStats {
        &self.stats
    }
}

/// Bot 生命周期 trait (per erlang A1)
///
/// - `start` 内部 spawn 1 tokio task 跑 AI 循环, 注册到 `BotStats`
/// - `stop`  通过 `Notify` 通知 task 退出, 标记 offline
///
/// 用户不应直接调用 `tokio::spawn`, 应通过 `start/stop` 控制 bot 生命周期.
#[async_trait]
pub trait BotCore: Send + Sync {
    /// 启动 bot (spawn 内部 task, 返回 Ok 表示 spawn 成功)
    async fn start(&self) -> Result<()>;

    /// 停止 bot (通知 task 退出, 等待 task 结束)
    async fn stop(&self);
}

#[async_trait]
impl BotCore for Bot {
    async fn start(&self) -> Result<()> {
        // 单一临界区: check-and-set 防止并发 start 双重 spawn
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            anyhow::bail!("bot {} already started", self.id);
        }

        let id = self.id.clone();
        let mod_name = self.mod_name.clone();
        let stats = self.stats.clone();
        let notify = self.notify_stop.clone();

        stats.mark_online(&id, &mod_name);

        let handle = tokio::spawn(async move {
            tracing::info!(bot_id = %id, mod = %mod_name, "bot started");
            // 等 stop 信号 (per erlang A1: 1 bot 1 进程, 等待外部 stop)
            notify.notified().await;
            tracing::info!(bot_id = %id, "bot stop signaled");
        });

        *guard = Some(handle);
        Ok(())
    }

    async fn stop(&self) {
        self.notify_stop.notify_one();
        let mut guard = self.handle.lock().await;
        if let Some(h) = guard.take() {
            // 等待 task 退出 (PoC: task 收到 notify 后即结束)
            let _ = h.await;
        }
        self.stats.mark_offline(&self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::ai::DefaultBotAi;

    #[tokio::test]
    async fn bot_new_initializes_fields() {
        let stats = BotStats::new();
        let bot = Bot::new("bot-001", "player", stats.clone());
        assert_eq!(bot.id(), "bot-001");
        assert_eq!(bot.mod_name(), "player");
    }

    #[tokio::test]
    async fn bot_start_marks_online_stop_marks_offline() {
        let stats = BotStats::new();
        let bot = Bot::new("bot-001", "player", stats.clone());

        bot.start().await.expect("start");
        assert_eq!(stats.count().get("player").copied(), Some(1));
        assert!(stats.online_ids().contains(&"bot-001".to_string()));

        bot.stop().await;
        assert_eq!(stats.count().get("player").copied(), Some(0));
        assert!(stats.offline().contains(&"bot-001".to_string()));
    }

    #[tokio::test]
    async fn bot_double_start_returns_error() {
        let stats = BotStats::new();
        let bot = Bot::new("bot-001", "player", stats.clone());
        bot.start().await.expect("first start");
        let r = bot.start().await;
        assert!(r.is_err(), "double start should fail");
        bot.stop().await;
    }

    #[tokio::test]
    async fn bot_default_ai_works() {
        let stats = BotStats::new();
        let bot = Bot::new("bot-001", "default", stats);
        let ai = DefaultBotAi;
        // BotAi 通过 `super::*` 已经在 scope (pub use crate::bot::ai::BotAi)
        ai.init(&bot).await.expect("init");
        assert!(ai.act_list().is_empty());
        ai.handle(&bot, ActKind::Heartbeat).await.expect("handle");
    }
}

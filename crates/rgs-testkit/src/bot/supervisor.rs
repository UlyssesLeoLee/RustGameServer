//! Bot 监督者 (per DDD Review v0.2 §5.1 M6 + M11)
//!
//! 模拟 erlang:
//! - **E2** 掉线自愈 (relogin loop 10 min) → [`BotSupervisor::relogin_offline`]
//! - **F3** 错峰启动 (避免峰值) → [`BotSupervisor::spawn_with_stagger`]
//! - **A2** ETS 全局表白名单 4500 → [`BotSupervisor::MAX_BOTS`]
//!
//! PoC: `spawn_with_stagger` 异步 spawn N 个 bot, 每个 bot 间隔 stagger_ms
//! 启动. `relogin_offline` 对离线 bot 调 start() 重启 (PoC stub, 真实
//! 重登录逻辑留 wave 2 接 gRPC).

use std::time::Duration;

use anyhow::Result;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::bot::stats::BotStats;
use crate::bot::{Bot, BotCore};

/// Bot 监督者 (per erlang E2 supervisor)
///
/// 持有 `max` (白名单上限) + `stagger_ms` (错峰间隔) + 实际 bot 列表 + 共享 stats.
/// `spawn_with_stagger` 异步 spawn, `relogin_offline` 同步重启离线 bot.
pub struct BotSupervisor {
    /// 白名单上限 (per erlang A2 4500)
    pub max: usize,
    /// 错峰间隔 (毫秒, per erlang F3)
    pub stagger_ms: u64,
    /// 已 spawn 的 bot 列表
    pub bots: Vec<Bot>,
    /// 共享统计句柄
    pub stats: BotStats,
}

impl BotSupervisor {
    /// 最大 bot 数量白名单 (per erlang tester 同时在线 4500 经验, 防雪崩)
    pub const MAX_BOTS: usize = 4500;

    /// 构造监督者 (PoC: 字段全 pub, 不走 builder)
    ///
    /// - `max`        白名单上限 (硬约束 ≤ [`Self::MAX_BOTS`])
    /// - `stagger_ms` 错峰间隔 (毫秒)
    pub fn new(max: usize, stagger_ms: u64) -> Self {
        let effective_max = if max > Self::MAX_BOTS { Self::MAX_BOTS } else { max };
        if max > Self::MAX_BOTS {
            warn!(
                requested = max,
                effective = effective_max,
                "BotSupervisor::new: max 超 MAX_BOTS 白名单, 截断"
            );
        }
        Self {
            max: effective_max,
            stagger_ms,
            bots: Vec::with_capacity(effective_max),
            stats: BotStats::new(),
        }
    }

    /// 错峰 spawn N 个 bot (per erlang F3 `util:sleep((I-N)*Time)`)
    ///
    /// - spawn 数 = `self.max`
    /// - 每个 bot 间隔 `stagger_ms` 启动
    /// - 全部 bot 启动后, 在线数应 == `self.max` (PoC 不考虑启动失败)
    ///
    /// # 错误
    /// - 任一 bot `start()` 失败 → 立即返回 Err (后续 bot 不再 spawn)
    pub async fn spawn_with_stagger(&mut self) -> Result<()> {
        let total = self.max;
        info!(total, stagger_ms = self.stagger_ms, "spawn_with_stagger begin");

        for i in 0..total {
            let id = format!("bot-{:04}", i);
            let mod_name = "default".to_string(); // PoC: 全部 default, 5 域派生留 wave 2
            let bot = Bot::new(&id, &mod_name, self.stats.clone());
            bot.start().await?;
            self.bots.push(bot);

            if i + 1 < total {
                sleep(Duration::from_millis(self.stagger_ms)).await;
            }
        }

        info!(total, "spawn_with_stagger done");
        Ok(())
    }

    /// 在线 bot 数 (按 `stats.count()` 总和)
    pub fn online_count(&self) -> u32 {
        self.stats.count().values().sum()
    }

    /// 掉线 bot id 列表 (per erlang E1 `offline/0`)
    pub fn offline_ids(&self) -> Vec<String> {
        self.stats.offline()
    }

    /// 掉线自愈 (per erlang E2: 周期 re-login loop)
    ///
    /// PoC: 遍历离线 bot, 重新 `start()`. 真实重登录逻辑 (gRPC 登录流)
    /// 留 wave 2 worker 接入. 当前是 bot 进程重启, 不调 gRPC 登录.
    pub async fn relogin_offline(&mut self) -> Result<usize> {
        let offline_ids = self.stats.offline();
        let mut recovered = 0usize;
        for id in offline_ids {
            if let Some(bot) = self.bots.iter_mut().find(|b| b.id() == id) {
                bot.start().await?;
                recovered += 1;
                info!(bot_id = %id, "bot relogged");
            }
        }
        Ok(recovered)
    }

    /// 停止所有 bot
    pub async fn stop_all(&mut self) {
        for bot in self.bots.drain(..) {
            bot.stop().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_clamped_to_white_list() {
        let sup = BotSupervisor::new(10_000, 50);
        assert_eq!(sup.max, BotSupervisor::MAX_BOTS);
    }

    #[test]
    fn max_within_white_list_unchanged() {
        let sup = BotSupervisor::new(100, 50);
        assert_eq!(sup.max, 100);
    }

    #[tokio::test]
    async fn spawn_with_stagger_brings_all_online() {
        // PoC: 3 bots, 50ms stagger → 总耗时 ≈ 100ms
        let mut sup = BotSupervisor::new(3, 50);
        sup.spawn_with_stagger().await.expect("spawn");
        assert_eq!(sup.online_count(), 3);
        assert_eq!(sup.offline_ids().len(), 0);
        sup.stop_all().await;
        assert_eq!(sup.online_count(), 0);
    }
}

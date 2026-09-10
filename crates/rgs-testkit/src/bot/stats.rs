//! Bot 实时统计 (per DDD Review v0.2 §5.1 M5)
//!
//! 模拟 erlang E1 (`count/0` 按 mod 聚合 + `offline/0`):
//! - `count()` → `HashMap<ModName, u32>` (每 mod 在线数量)
//! - `offline()` → `Vec<BotId>` (掉线 bot id 列表)
//!
//! 实现选型: `Arc<Mutex<HashMap>>` (per crates/rgs-testkit/src/mock.rs:43-44 既有
//! 模式), **不** 引入 dashmap (per L3 跨工具链决策: workspace 依赖 grep 0 引用,
//! 见 D:\rgs-bottest-core\Cargo.toml workspace.dependencies). 后续如要切换 dashmap
//! 需先 `cargo add dashmap -p rgs-testkit`.

use std::collections::HashMap;
use std::sync::Mutex;

/// Mod 名称 (e.g. "player" / "economy" / "match")
pub type ModName = String;

/// Bot 实时统计 (per erlang E1: tester_online ETS 全局表)
///
/// `Arc<Mutex<...>>` 内部, clone 廉价 (只克隆 Arc), 适合在 Bot / Supervisor /
/// 报告 多个 owner 间共享.
#[derive(Clone, Default)]
pub struct BotStats {
    inner: std::sync::Arc<Mutex<BotStatsInner>>,
}

#[derive(Default)]
struct BotStatsInner {
    /// mod_name -> 当前在线数
    online: HashMap<ModName, u32>,
    /// 已注册的 bot id 集合 (用于 offline 查询 + 防重名)
    bots: HashMap<String, BotEntry>,
}

#[derive(Clone, Debug)]
struct BotEntry {
    mod_name: ModName,
    online: bool,
}

impl BotStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册 + 标记 bot 上线 (在 start 内部调用)
    pub fn inc(&self, mod_name: &str) {
        let mut inner = self.inner.lock().expect("BotStats mutex poisoned");
        *inner.online.entry(mod_name.to_string()).or_insert(0) += 1;
        // bots 注册由 start() 调 mark_online 统一处理, inc 仅更新计数
    }

    /// 完整注册一个 bot (id + mod_name, 上线)
    pub fn mark_online(&self, id: &str, mod_name: &str) {
        let mut inner = self.inner.lock().expect("BotStats mutex poisoned");
        *inner.online.entry(mod_name.to_string()).or_insert(0) += 1;
        inner.bots.insert(
            id.to_string(),
            BotEntry {
                mod_name: mod_name.to_string(),
                online: true,
            },
        );
    }

    /// 标记 bot 掉线 (在 stop 内部调用, 不减 online 计数, 由 mark_offline 减)
    pub fn mark_offline(&self, id: &str) {
        let mut inner = self.inner.lock().expect("BotStats mutex poisoned");
        // 拿 mod_name 副本 + 标记 offline, 避免后续 `inner.online.get_mut` 二次借用
        let mod_name = match inner.bots.get_mut(id) {
            Some(entry) if entry.online => {
                entry.online = false;
                Some(entry.mod_name.clone())
            }
            _ => None,
        };
        if let Some(name) = mod_name {
            if let Some(c) = inner.online.get_mut(&name) {
                *c = c.saturating_sub(1);
            }
        }
    }

    /// 当前在线数, 按 mod 聚合 (per erlang E1 `count/0`)
    pub fn count(&self) -> HashMap<ModName, u32> {
        let inner = self.inner.lock().expect("BotStats mutex poisoned");
        inner.online.clone()
    }

    /// 掉线 bot id 列表 (per erlang E1 `offline/0`)
    pub fn offline(&self) -> Vec<String> {
        let inner = self.inner.lock().expect("BotStats mutex poisoned");
        inner
            .bots
            .iter()
            .filter(|(_, e)| !e.online)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// 在线 bot id 列表
    pub fn online_ids(&self) -> Vec<String> {
        let inner = self.inner.lock().expect("BotStats mutex poisoned");
        inner
            .bots
            .iter()
            .filter(|(_, e)| e.online)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// 总 bot 数 (在线 + 离线)
    pub fn total(&self) -> usize {
        let inner = self.inner.lock().expect("BotStats mutex poisoned");
        inner.bots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_aggregates_by_mod() {
        let s = BotStats::new();
        s.mark_online("bot-1", "player");
        s.mark_online("bot-2", "player");
        s.mark_online("bot-3", "economy");
        let c = s.count();
        assert_eq!(c.get("player").copied(), Some(2));
        assert_eq!(c.get("economy").copied(), Some(1));
    }

    #[test]
    fn offline_returns_marked_offline_ids() {
        let s = BotStats::new();
        s.mark_online("bot-1", "player");
        s.mark_online("bot-2", "player");
        s.mark_offline("bot-1");
        let off = s.offline();
        assert_eq!(off, vec!["bot-1".to_string()]);
    }

    #[test]
    fn mark_offline_decrements_mod_count() {
        let s = BotStats::new();
        s.mark_online("bot-1", "player");
        s.mark_online("bot-2", "player");
        s.mark_offline("bot-1");
        assert_eq!(s.count().get("player").copied(), Some(1));
    }

    #[test]
    fn mark_offline_idempotent() {
        let s = BotStats::new();
        s.mark_online("bot-1", "player");
        s.mark_offline("bot-1");
        s.mark_offline("bot-1"); // 重复 offline 不应减到 0 以下
        assert_eq!(s.count().get("player").copied(), Some(0));
    }

    #[test]
    fn total_counts_online_and_offline() {
        let s = BotStats::new();
        s.mark_online("bot-1", "player");
        s.mark_online("bot-2", "player");
        s.mark_offline("bot-2");
        assert_eq!(s.total(), 2);
        assert_eq!(s.online_ids().len(), 1);
    }
}

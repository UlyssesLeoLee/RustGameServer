//! AI 行为序列 (per DDD Review v0.2 §5.1 M3)
//!
//! 模拟 erlang B3 + C1: 行为是序列而非单点 + 协议随机化触发.
//! - `ActKind`        行为枚举 (Init / Heartbeat / RandProto / Guild / Vip / ...)
//! - `ActList`        行为列表 + 概率权重, `pick_random()` 按概率选一个
//!
//! 随机源: `uuid::Uuid::new_v4()` (workspace.dependencies 已声明 uuid v4, 不
//! 引入新 rand crate, per L3 跨工具链决策守门).

use std::collections::HashMap;
use uuid::Uuid;

/// AI 行为类型 (per erlang B3 act_list 序列)
///
/// `RandProto(u32)` 携带概率 (千分位, 0-1000, per erlang C1 `util:rand(1, 1000)`).
/// 例如 `RandProto(100)` = 10% 概率触发.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ActKind {
    /// 初始化 (首次启动时调用)
    Init,
    /// 心跳 (per erlang E4: 60s 触发 restart)
    Heartbeat,
    /// 协议随机化触发 (千分位概率, per erlang C1)
    RandProto(u32),
    /// 随机重登录 (per erlang C4)
    RandRestart,
    /// 工会操作 (per erlang C6)
    Guild,
    /// VIP 操作 (per erlang C6)
    Vip,
    /// 副本 (per erlang C6)
    Dun,
    /// 竞技场 (per erlang C6)
    Arena,
    /// 世界 Boss (per erlang C6)
    Boss,
    /// 伙伴 (per erlang C6)
    Partner,
    /// 自定义扩展点 (5 域派生时用)
    Custom(String),
}

/// 行为列表 + 概率权重 (per erlang B3 + C1)
///
/// `pick_random()` 用 `uuid::Uuid::new_v4()` 拿随机数, 按权重累积区间选 act.
/// `RandProto(p)` 类型的概率 = `p` (千分位), 其它类型概率 = 1.
#[derive(Clone, Debug, Default)]
pub struct ActList {
    /// 所有可选 act
    pub acts: Vec<ActKind>,
    /// 显式权重覆盖 (per act), 未列的 act 默认 1
    pub prob: HashMap<ActKind, u32>,
}

impl ActList {
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加 act, 可选指定概率 (千分位, per erlang C1)
    pub fn push(mut self, act: ActKind) -> Self {
        self.acts.push(act);
        self
    }

    /// 设置某 act 概率 (千分位)
    pub fn with_prob(mut self, act: ActKind, prob: u32) -> Self {
        self.prob.insert(act, prob);
        self
    }

    /// 拿 act 权重 (per erlang C1)
    fn weight_of(&self, act: &ActKind) -> u32 {
        self.prob.get(act).copied().unwrap_or_else(|| {
            // RandProto(u32) 的概率就是携带的 u32 (千分位)
            if let ActKind::RandProto(p) = act {
                *p
            } else {
                1
            }
        })
    }

    /// 按概率随机选一个 act
    ///
    /// 用 uuid v4 派生 [0, u32::MAX] 均匀分布, 映射到累计权重区间.
    /// `total = 0` 时回退到随机索引 (避免空列表 / 全 0 概率的 panic).
    pub fn pick_random(&self) -> Option<ActKind> {
        if self.acts.is_empty() {
            return None;
        }

        let total: u32 = self.acts.iter().map(|a| self.weight_of(a)).sum();
        if total == 0 {
            // 全 0 概率兜底: 均匀随机
            let idx = (Uuid::new_v4().as_u128() as usize) % self.acts.len();
            return self.acts.get(idx).cloned();
        }

        let r = (Uuid::new_v4().as_u128() as u128 % (total as u128)) as u32;
        let mut acc: u32 = 0;
        for a in &self.acts {
            acc = acc.saturating_add(self.weight_of(a));
            if r < acc {
                return Some(a.clone());
            }
        }
        // 浮点边界兜底
        self.acts.last().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_act_list_picks_none() {
        let al = ActList::new();
        assert!(al.pick_random().is_none());
    }

    #[test]
    fn single_act_always_picked() {
        let al = ActList::new().push(ActKind::Heartbeat);
        for _ in 0..20 {
            assert_eq!(al.pick_random(), Some(ActKind::Heartbeat));
        }
    }

    #[test]
    fn rand_proto_carries_probability() {
        // RandProto(1000) = 100% 概率
        let al = ActList::new()
            .push(ActKind::Heartbeat)
            .push(ActKind::RandProto(1000));
        for _ in 0..20 {
            assert_eq!(al.pick_random(), Some(ActKind::RandProto(1000)));
        }
    }

    #[test]
    fn weight_override_takes_precedence() {
        // Heartbeat 默认 weight 1, override 1000 → 总是 Heartbeat
        let al = ActList::new()
            .push(ActKind::Heartbeat)
            .push(ActKind::Guild)
            .with_prob(ActKind::Heartbeat, 1000);
        for _ in 0..20 {
            assert_eq!(al.pick_random(), Some(ActKind::Heartbeat));
        }
    }

    #[test]
    fn all_zero_prob_falls_back_to_uniform() {
        let al = ActList::new()
            .push(ActKind::Heartbeat)
            .push(ActKind::Guild)
            .with_prob(ActKind::Heartbeat, 0)
            .with_prob(ActKind::Guild, 0);
        // 不 panic, 至少返回一个
        assert!(al.pick_random().is_some());
    }
}

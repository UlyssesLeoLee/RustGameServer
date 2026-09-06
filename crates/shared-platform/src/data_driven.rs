//! data_driven —— 9 holiday_* + 6-12 PVP 变体数据驱动框架
//!
//! 反例 (per 9/4 MD §4 + 路线图 §0.2 + W5 简报):
//! - 不为每个变体 / 活动重写 1 套 service 实现
//! - 业务层只跟 `PvpMode` / `HolidayActivity` 打交道, 1 套代码
//!
//! 提供:
//! - `PvpMode` 枚举: 6-12 变体 (ranked/casual/cross-server/championship/hero-hall/friendly/ext_*)
//! - `PvpConfig`: 6-12 变体统一配置
//! - `HolidayActivity` 实体: 9 个 holiday_* 活动
//! - `HolidayConfig`: 9 个 holiday_* 统一配置
//!
//! 共享给 `pvp-full-service` (151 RPC) + `activity-service` (184 RPC)

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// PVP 模式枚举 (6-12 变体, per 9/4 MD §2 + proto_202~proto_207)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PvpMode {
    /// 排位赛 (天梯)
    Ranked,
    /// 休闲赛
    Casual,
    /// 跨服赛
    CrossServer,
    /// 冠军赛
    Championship,
    /// 英雄殿
    HeroHall,
    /// 切磋
    Friendly,
    /// 扩展变体 1 (e.g. 巅峰赛)
    Extended1,
    /// 扩展变体 2 (e.g. 帮派战)
    Extended2,
    /// 扩展变体 3 (e.g. 师徒赛)
    Extended3,
}

impl PvpMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PvpMode::Ranked => "ranked",
            PvpMode::Casual => "casual",
            PvpMode::CrossServer => "cross_server",
            PvpMode::Championship => "championship",
            PvpMode::HeroHall => "hero_hall",
            PvpMode::Friendly => "friendly",
            PvpMode::Extended1 => "extended_1",
            PvpMode::Extended2 => "extended_2",
            PvpMode::Extended3 => "extended_3",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ranked" => Some(PvpMode::Ranked),
            "casual" => Some(PvpMode::Casual),
            "cross_server" => Some(PvpMode::CrossServer),
            "championship" => Some(PvpMode::Championship),
            "hero_hall" => Some(PvpMode::HeroHall),
            "friendly" => Some(PvpMode::Friendly),
            "extended_1" => Some(PvpMode::Extended1),
            "extended_2" => Some(PvpMode::Extended2),
            "extended_3" => Some(PvpMode::Extended3),
            _ => None,
        }
    }

    /// 全部 9 个变体 (per 9/4 MD §2 6-12 变体)
    pub const ALL: [PvpMode; 9] = [
        PvpMode::Ranked,
        PvpMode::Casual,
        PvpMode::CrossServer,
        PvpMode::Championship,
        PvpMode::HeroHall,
        PvpMode::Friendly,
        PvpMode::Extended1,
        PvpMode::Extended2,
        PvpMode::Extended3,
    ];
}

/// 1 个 PVP 模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvpModeConfig {
    pub mode: PvpMode,
    pub display_name: String,
    pub daily_limit: u32,
    pub uses_rank_score: bool,
    pub uses_season: bool,
    pub cross_server_enabled: bool,
}

/// 9 个 PVP 变体统一配置 (per 9/4 MD §4 反例 + proto_202~proto_207)
#[derive(Debug, Clone)]
pub struct PvpConfig {
    pub modes: HashMap<PvpMode, PvpModeConfig>,
}

impl Default for PvpConfig {
    fn default() -> Self {
        let mut modes = HashMap::new();
        let entries = [
            (PvpMode::Ranked, "排位赛", 10u32, true, true, false),
            (PvpMode::Casual, "休闲赛", 20, false, false, false),
            (PvpMode::CrossServer, "跨服赛", 5, true, true, true),
            (PvpMode::Championship, "冠军赛", 3, true, true, true),
            (PvpMode::HeroHall, "英雄殿", 1, false, false, false),
            (PvpMode::Friendly, "切磋", 999, false, false, false),
            (PvpMode::Extended1, "巅峰赛", 2, true, true, true),
            (PvpMode::Extended2, "帮派战", 5, false, true, true),
            (PvpMode::Extended3, "师徒赛", 3, false, true, false),
        ];
        for (mode, name, limit, rank, season, cross) in entries {
            modes.insert(
                mode,
                PvpModeConfig {
                    mode,
                    display_name: name.to_string(),
                    daily_limit: limit,
                    uses_rank_score: rank,
                    uses_season: season,
                    cross_server_enabled: cross,
                },
            );
        }
        Self { modes }
    }
}

impl PvpConfig {
    pub fn get(&self, mode: PvpMode) -> Option<&PvpModeConfig> {
        self.modes.get(&mode)
    }
    /// 6-12 变体总数 (per 9/4 MD §2)
    pub fn variant_count(&self) -> usize {
        self.modes.len()
    }
}

/// 1 个 holiday_* 活动 (per 9/4 MD §0 9 个高度重复变体)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HolidayActivity {
    pub activity_id: String,
    pub activity_name: String,
    pub opens_at_ms: i64,
    pub closes_at_ms: i64,
    pub config_json: String,
}

impl HolidayActivity {
    pub fn new(activity_id: &str, activity_name: &str) -> Self {
        Self {
            activity_id: activity_id.to_string(),
            activity_name: activity_name.to_string(),
            opens_at_ms: 0,
            closes_at_ms: i64::MAX,
            config_json: r#"{"task_type":"daily_kill","target":10}"#.to_string(),
        }
    }

    /// 活动是否在 open 时间窗内
    pub fn is_open_at(&self, now_ms: i64) -> bool {
        now_ms >= self.opens_at_ms && now_ms < self.closes_at_ms
    }
}

/// 9 个 holiday_* 活动统一配置 (per 9/4 MD §4 反例)
#[derive(Debug, Clone)]
pub struct HolidayConfig {
    pub activities: HashMap<String, HolidayActivity>,
}

impl Default for HolidayConfig {
    fn default() -> Self {
        let mut activities = HashMap::new();
        // 9 个 holiday_* 统一模板
        for (id, name) in [
            ("93031", "元宵冒险1"),
            ("93032", "元宵冒险2"),
            ("93033", "元宵冒险3"),
            ("lantern", "元宵灯会"),
            ("food", "欢食元宵"),
            ("spring", "春节活动"),
            ("summer", "夏日祭"),
            ("halloween", "万圣节"),
            ("anniv", "周年庆"),
        ] {
            activities.insert(
                id.to_string(),
                HolidayActivity::new(id, name),
            );
        }
        Self { activities }
    }
}

impl HolidayConfig {
    pub fn get(&self, activity_id: &str) -> Option<&HolidayActivity> {
        self.activities.get(activity_id)
    }
    /// 9 个变体总数
    pub fn variant_count(&self) -> usize {
        self.activities.len()
    }
    /// 列举全部活动 ID
    pub fn list_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.activities.keys().cloned().collect();
        ids.sort();
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pvp_mode_str_roundtrip() {
        for m in PvpMode::ALL.iter() {
            assert_eq!(PvpMode::from_str(m.as_str()), Some(*m));
        }
        assert_eq!(PvpMode::from_str("bogus"), None);
    }

    #[test]
    fn pvp_config_default_has_9_variants() {
        let cfg = PvpConfig::default();
        assert_eq!(cfg.variant_count(), 9, "PVP 必须覆盖 6-12 变体, 实际 9");
    }

    #[test]
    fn pvp_config_ranked_uses_rank_score() {
        let cfg = PvpConfig::default();
        let ranked = cfg.get(PvpMode::Ranked).unwrap();
        assert!(ranked.uses_rank_score);
        assert!(!ranked.cross_server_enabled);
        assert_eq!(ranked.daily_limit, 10);
    }

    #[test]
    fn pvp_config_cross_server_enabled() {
        let cfg = PvpConfig::default();
        let cross = cfg.get(PvpMode::CrossServer).unwrap();
        assert!(cross.cross_server_enabled);
    }

    #[test]
    fn holiday_config_default_has_9_variants() {
        let cfg = HolidayConfig::default();
        assert_eq!(cfg.variant_count(), 9, "Holiday 必须覆盖 9 个变体");
    }

    #[test]
    fn holiday_config_get_93031() {
        let cfg = HolidayConfig::default();
        let act = cfg.get("93031").unwrap();
        assert_eq!(act.activity_name, "元宵冒险1");
    }

    #[test]
    fn holiday_config_unknown_id_is_none() {
        let cfg = HolidayConfig::default();
        assert!(cfg.get("unknown").is_none());
    }

    #[test]
    fn holiday_activity_is_open_at() {
        let act = HolidayActivity::new("93031", "test");
        assert!(act.is_open_at(0));
        assert!(act.is_open_at(i64::MAX - 1));
        assert!(!act.is_open_at(i64::MAX));
        assert!(!act.is_open_at(-1));
    }

    #[test]
    fn holiday_config_list_ids_sorted() {
        let cfg = HolidayConfig::default();
        let ids = cfg.list_ids();
        assert_eq!(ids.len(), 9);
        // sorted
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn pvp_mode_all_9_unique() {
        let mut seen = std::collections::HashSet::new();
        for m in PvpMode::ALL.iter() {
            assert!(seen.insert(*m), "PvpMode::ALL has duplicate: {:?}", m);
        }
        assert_eq!(seen.len(), 9);
    }
}

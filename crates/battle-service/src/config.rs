//! battle-service 配置模块 (数据驱动反例: 6 PVP + 9 holiday_*)
//!
//! 提供:
//! - PvPConfig: 6 个 PVP 变体统一配置
//! - HolidayConfig: 9 个 holiday_* 活动统一配置
//!
//! 反例原则 (per 9/4 MD §4 + 路线图 §0.3 + W5 简报):
//! - 不为每个变体 / 活动重写 1 套 service 实现
//! - 业务层只跟 PvpMode / HolidayActivity 打交道, 1 套代码

use std::collections::HashMap;

use crate::entity::{HolidayActivity, PvpMode};

/// PVP 6 变体配置 (per 9/4 MD §4 反例 + proto_202/proto_243 数据驱动)
#[derive(Debug, Clone)]
pub struct PvpConfig {
    /// 6 个变体 (ranked/casual/cross-server/championship/hero-hall/friendly)
    pub modes: HashMap<PvpMode, PvpModeConfig>,
}

#[derive(Debug, Clone)]
pub struct PvpModeConfig {
    pub mode: PvpMode,
    pub display_name: String,
    pub daily_limit: u32,
    pub uses_rank_score: bool,
    pub uses_season: bool,
    /// 跨服赛跨区允许
    pub cross_server_enabled: bool,
}

impl Default for PvpConfig {
    fn default() -> Self {
        let mut modes = HashMap::new();
        modes.insert(
            PvpMode::Ranked,
            PvpModeConfig {
                mode: PvpMode::Ranked,
                display_name: "排位赛".to_string(),
                daily_limit: 10,
                uses_rank_score: true,
                uses_season: true,
                cross_server_enabled: false,
            },
        );
        modes.insert(
            PvpMode::Casual,
            PvpModeConfig {
                mode: PvpMode::Casual,
                display_name: "休闲赛".to_string(),
                daily_limit: 20,
                uses_rank_score: false,
                uses_season: false,
                cross_server_enabled: false,
            },
        );
        modes.insert(
            PvpMode::CrossServer,
            PvpModeConfig {
                mode: PvpMode::CrossServer,
                display_name: "跨服赛".to_string(),
                daily_limit: 5,
                uses_rank_score: true,
                uses_season: true,
                cross_server_enabled: true,
            },
        );
        modes.insert(
            PvpMode::Championship,
            PvpModeConfig {
                mode: PvpMode::Championship,
                display_name: "冠军赛".to_string(),
                daily_limit: 3,
                uses_rank_score: true,
                uses_season: true,
                cross_server_enabled: true,
            },
        );
        modes.insert(
            PvpMode::HeroHall,
            PvpModeConfig {
                mode: PvpMode::HeroHall,
                display_name: "英雄殿".to_string(),
                daily_limit: 1,
                uses_rank_score: false,
                uses_season: false,
                cross_server_enabled: false,
            },
        );
        modes.insert(
            PvpMode::Friendly,
            PvpModeConfig {
                mode: PvpMode::Friendly,
                display_name: "切磋".to_string(),
                daily_limit: 999,
                uses_rank_score: false,
                uses_season: false,
                cross_server_enabled: false,
            },
        );
        Self { modes }
    }
}

impl PvpConfig {
    pub fn get(&self, mode: PvpMode) -> Option<&PvpModeConfig> {
        self.modes.get(&mode)
    }
    /// 6 个变体总数
    pub fn variant_count(&self) -> usize {
        self.modes.len()
    }
}

/// Holiday 9 变体配置 (per 9/4 MD §4 反例 + proto_248 数据驱动)
///
/// 9 个 holiday_* 活动 (per 9/4 MD §0 / 闪烁之光反例):
/// - 24801-24803: 元宵灯会 (3 RPC: 基础信息/奖池/抽奖)
/// - 24804-24807: 欢食元宵 (4 RPC: 基础/通关/制作/等级)
/// - 24808-24809: 副本 (2 RPC: 挑战/购买)
/// - 24810-24812: 任务 (3 RPC: 任务信息/推送/领取)
/// - 24813-24818: 9 个 bid:93031/93032/93033 元宵冒险 (1+1+1+1+1+1 = 6 RPC, 3 个活动 × 2 RPC)
///   业务层通过 activity_id 路由, 1 套代码覆盖 3 个活动 → 抽象为 9 个 holiday_* 中 3 个
///
/// 完整 9 个 holiday_* 涵盖: 春节/元宵/端午/中秋/圣诞/周年庆/夏日祭/万圣节/感恩节
/// (per 闪烁之光运营活动 9 个高度重复变体)
#[derive(Debug, Clone)]
pub struct HolidayConfig {
    /// 9 个活动配置
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
                HolidayActivity {
                    activity_id: id.to_string(),
                    activity_name: name.to_string(),
                    opens_at_ms: 0,
                    closes_at_ms: i64::MAX,
                    config_json: r#"{"task_type":"daily_kill","target":10}"#.to_string(),
                },
            );
        }
        Self { activities }
    }
}

impl HolidayConfig {
    pub fn get(&self, activity_id: &str) -> Option<&HolidayActivity> {
        self.activities.get(activity_id)
    }
    /// 9 个变体总数 (per W5 简报 + 9/4 MD §4 反例)
    pub fn variant_count(&self) -> usize {
        self.activities.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pvp_config_has_6_variants() {
        let cfg = PvpConfig::default();
        assert_eq!(cfg.variant_count(), 6, "PVP 必须覆盖 6 个变体");
    }

    #[test]
    fn pvp_config_ranked_uses_rank_score() {
        let cfg = PvpConfig::default();
        let ranked = cfg.get(PvpMode::Ranked).unwrap();
        assert!(ranked.uses_rank_score);
        assert!(!ranked.cross_server_enabled);
    }

    #[test]
    fn pvp_config_cross_server_enabled() {
        let cfg = PvpConfig::default();
        let cross = cfg.get(PvpMode::CrossServer).unwrap();
        assert!(cross.cross_server_enabled);
    }

    #[test]
    fn holiday_config_has_9_variants() {
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
}

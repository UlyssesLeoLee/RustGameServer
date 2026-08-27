//! 域抽象 — 5 域业务服务（player / economy / match / social）
//!
//! 强约束（per task §1 / §2.2）：
//! - **不含** admin / cluster-ops（编译期防越界）
//! - 每域独立 subject token / env key，便于按域排障 + 独立消费者组
//!
//! 每域与 .env.example §9 的 `<DOMAIN>_MAX_INFLIGHT` env 一一对应。

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// RGS 业务域（5 域中的 4 个，admin / cluster-ops 不在限流范围）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Domain {
    Player,
    Economy,
    Match,
    Social,
}

impl Domain {
    /// 域小写名（如 `player`），用于日志 / subject / env key 拼接
    pub const fn as_str(self) -> &'static str {
        match self {
            Domain::Player => "player",
            Domain::Economy => "economy",
            Domain::Match => "match",
            Domain::Social => "social",
        }
    }

    /// subject token（与 as_str 保持一致 — 不区分大小写）
    pub const fn subject_token(self) -> &'static str {
        self.as_str()
    }

    /// .env 中对应的 `<DOMAIN>_MAX_INFLIGHT` env key
    pub const fn env_max_inflight(self) -> &'static str {
        match self {
            Domain::Player => "PLAYER_MAX_INFLIGHT",
            Domain::Economy => "ECONOMY_MAX_INFLIGHT",
            Domain::Match => "MATCH_MAX_INFLIGHT",
            Domain::Social => "SOCIAL_MAX_INFLIGHT",
        }
    }

    /// 全部 4 域（迭代顺序 = match arm 顺序，stable）
    pub const ALL: [Domain; 4] = [
        Domain::Player,
        Domain::Economy,
        Domain::Match,
        Domain::Social,
    ];
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Domain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "player" => Ok(Domain::Player),
            "economy" => Ok(Domain::Economy),
            "match" => Ok(Domain::Match),
            "social" => Ok(Domain::Social),
            other => Err(format!(
                "unknown domain '{}' (expected one of: player, economy, match, social)",
                other
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_returns_lowercase() {
        assert_eq!(Domain::Player.as_str(), "player");
        assert_eq!(Domain::Economy.as_str(), "economy");
        assert_eq!(Domain::Match.as_str(), "match");
        assert_eq!(Domain::Social.as_str(), "social");
    }

    #[test]
    fn env_max_inflight_keys_match_dotenv() {
        // 锚定 .env.example §9 实际 env 名（git log -p 实证 2026-08-27 root 提交）
        assert_eq!(Domain::Player.env_max_inflight(), "PLAYER_MAX_INFLIGHT");
        assert_eq!(Domain::Economy.env_max_inflight(), "ECONOMY_MAX_INFLIGHT");
        assert_eq!(Domain::Match.env_max_inflight(), "MATCH_MAX_INFLIGHT");
        assert_eq!(Domain::Social.env_max_inflight(), "SOCIAL_MAX_INFLIGHT");
    }

    #[test]
    fn from_str_round_trip() {
        for d in Domain::ALL {
            let s = d.as_str();
            let parsed: Domain = s.parse().expect("parse domain");
            assert_eq!(parsed, d);
        }
    }

    #[test]
    fn from_str_case_insensitive() {
        assert_eq!("Player".parse::<Domain>().unwrap(), Domain::Player);
        assert_eq!("ECONOMY".parse::<Domain>().unwrap(), Domain::Economy);
    }

    #[test]
    fn from_str_rejects_admin_and_cluster_ops() {
        // 强约束：admin / cluster-ops 不在限流域
        assert!("admin".parse::<Domain>().is_err());
        assert!("cluster-ops".parse::<Domain>().is_err());
        assert!("cluster_ops".parse::<Domain>().is_err());
    }

    #[test]
    fn all_iterates_four_domains() {
        assert_eq!(Domain::ALL.len(), 4);
        // 顺序稳定：player → economy → match → social
        assert_eq!(Domain::ALL[0], Domain::Player);
        assert_eq!(Domain::ALL[3], Domain::Social);
    }
}

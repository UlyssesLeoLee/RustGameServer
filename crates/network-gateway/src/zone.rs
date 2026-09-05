//! Zone 启动 stub (per 9/4 改进路线图 Phase 1 协议网关 + 9/4 MD §3 zone 拓扑)
//!
//! ## 范围
//! 闪烁之光 zone 节点 (per 9/4 MD §3 网络拓扑) 是 center 节点的下属分片.
//! zone 启动脚本 (sname -hidden, e.g. `erl -sname sszg_zone_1 -setcookie XXXX
//! -hidden -s zone start`) 加载 web_conn.erl, 注册到 center, 处理地图分片业务.
//!
//! ## 本骨架 (Phase 1.5 stub)
//! - `ZoneConfig` 配置 (zone_name / center_node / cookie / port / hidden)
//! - `ZoneRole` 枚举 (Center / Zone / Standalone)
//! - `start()` 函数签名 (Phase 1.5 真实实装: 走 rustler 调 BEAM `zone:start/0`)
//! - 不实际启动 BEAM VM (Phase 1.5 + ADR-006 Option A)
//!
//! ## 已知缺口
//! - zone.erl 真实源码未加载 (per 9/4 R2, 评估 rustler)
//! - center↔zone 注册协议未实装 (走 EPMD + dist, Phase 1.5)
//! - k3s StatefulSet + 显式 nodeName (per 9/4 R4 风险)
//!
//! ## 参考
//! - 9/4 MD §3 拓扑: center (端口 8000+) + 多 zone
//! - 9/4 改进路线图.md Phase 4 k3s 部署 (StatefulSet)
//! - ADR-006 Option A (RGS 内嵌 BEAM via rustler)

use std::fmt;

/// 节点角色 (per 9/4 MD §3, center / zone 二分)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneRole {
    /// Center 节点 (中心 zone, 跨服战场)
    Center,
    /// Zone 节点 (地图分片)
    Zone,
    /// 独立 (测试 / 本地开发, 不连 center)
    Standalone,
}

/// Zone 启动配置 (per 9/4 MD §3)
#[derive(Debug, Clone)]
pub struct ZoneConfig {
    /// 节点角色
    pub role: ZoneRole,
    /// zone 名 (e.g. "sszg_zone_1")
    pub zone_name: String,
    /// center 节点全名 (e.g. "sszg_center_6@center.cluster.local")
    pub center_node: String,
    /// 监听端口 (per 9/4 MD §3, web_conn 默认 8000, dist 端口 OS 分配)
    pub listen_port: u16,
    /// 是否 hidden 节点 (per 9/4 MD §3, 闪烁之光全部 -hidden)
    pub hidden: bool,
    /// Erlang cookie (per 8/27 11:06 JST hard ban, 不打印)
    pub cookie: Vec<u8>,
}

impl ZoneConfig {
    /// 默认 center 节点配置 (端口 8000, hidden=true)
    pub fn default_center(name: &str) -> Self {
        Self {
            role: ZoneRole::Center,
            zone_name: name.to_string(),
            center_node: String::new(), // center 自身无 center
            listen_port: 8000,
            hidden: true,
            cookie: Vec::new(),
        }
    }

    /// 默认 zone 节点配置 (向 center 注册, 端口 8001+)
    pub fn default_zone(name: &str, center_full_name: &str, port: u16) -> Self {
        Self {
            role: ZoneRole::Zone,
            zone_name: name.to_string(),
            center_node: center_full_name.to_string(),
            listen_port: port,
            hidden: true,
            cookie: Vec::new(),
        }
    }
}

impl fmt::Display for ZoneConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 凭据 REDACTED 过滤 (per 8/27 11:06 JST hard ban)
        write!(
            f,
            "ZoneConfig {{ role={:?}, zone={}, center={}, port={}, hidden={}, cookie=REDACTED(len={}) }}",
            self.role,
            self.zone_name,
            self.center_node,
            self.listen_port,
            self.hidden,
            self.cookie.len()
        )
    }
}

/// 启动 zone (Phase 1.5 stub)
///
/// 真实实装 (per ADR-006 Option A + 9/4 MD §3):
/// 1. `rustler` 启 BEAM VM
/// 2. 调 `zone:start(ZoneConfig)` 加载 sszg_symlf_3225 + web_conn.erl
/// 3. zone 节点向 center 节点注册 (走 EPMD + dist)
///
/// 当前: 校验配置, 返回 Ok stub
pub async fn start(cfg: ZoneConfig) -> std::io::Result<()> {
    // Phase 1.5 stub: 校验 config
    if cfg.zone_name.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "zone_name must be non-empty",
        ));
    }
    if cfg.role == ZoneRole::Zone && cfg.center_node.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Zone role requires center_node",
        ));
    }
    // 占位: Phase 1.5 替换为真实 BEAM 启动
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_center_config() {
        let cfg = ZoneConfig::default_center("sszg_center_6");
        assert_eq!(cfg.role, ZoneRole::Center);
        assert_eq!(cfg.zone_name, "sszg_center_6");
        assert_eq!(cfg.listen_port, 8000);
        assert!(cfg.hidden);
    }

    #[test]
    fn default_zone_config() {
        let cfg = ZoneConfig::default_zone("sszg_zone_1", "sszg_center_6@center", 8001);
        assert_eq!(cfg.role, ZoneRole::Zone);
        assert_eq!(cfg.zone_name, "sszg_zone_1");
        assert_eq!(cfg.center_node, "sszg_center_6@center");
        assert_eq!(cfg.listen_port, 8001);
        assert!(cfg.hidden);
    }

    #[test]
    fn display_redacts_cookie() {
        let mut cfg = ZoneConfig::default_center("test");
        cfg.cookie = b"secret_zone_cookie".to_vec();
        let s = format!("{}", cfg);
        assert!(!s.contains("secret_zone_cookie"));
        assert!(s.contains("REDACTED"));
    }

    #[tokio::test]
    async fn start_stub_center_ok() {
        let cfg = ZoneConfig::default_center("sszg_center_6");
        assert!(start(cfg).await.is_ok());
    }

    #[tokio::test]
    async fn start_stub_zone_with_center_ok() {
        let cfg = ZoneConfig::default_zone("sszg_zone_1", "sszg_center_6@center", 8001);
        assert!(start(cfg).await.is_ok());
    }

    #[tokio::test]
    async fn start_rejects_empty_zone_name() {
        let cfg = ZoneConfig {
            zone_name: String::new(),
            ..ZoneConfig::default_center("placeholder")
        };
        assert!(start(cfg).await.is_err());
    }

    #[tokio::test]
    async fn start_rejects_zone_without_center() {
        let cfg = ZoneConfig {
            center_node: String::new(),
            ..ZoneConfig::default_zone("sszg_zone_1", "placeholder", 8001)
        };
        // 上面 default_zone 给了 "placeholder", 需手动清空
        let cfg2 = ZoneConfig {
            role: ZoneRole::Zone,
            zone_name: "sszg_zone_1".to_string(),
            center_node: String::new(),
            listen_port: 8001,
            hidden: true,
            cookie: Vec::new(),
        };
        assert!(start(cfg2).await.is_err());
        // 上面的 cfg 不再使用, 避免 warning
        let _ = cfg;
    }
}

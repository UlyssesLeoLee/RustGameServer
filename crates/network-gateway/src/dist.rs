//! Erlang 分布式协议 stub (per 9/4 改进路线图 Phase 1 协议网关)
//!
//! ## 范围
//! Erlang/OTP distributed protocol (dist_proto), 客户端节点互联的二进制协议.
//! 闪烁之光 zone 节点 -hidden 启动 (per 9/4 MD §3), 靠 cluster_srv/cluster_cli
//! 自行管理连接对象, 不加入默认全网广播. RGS 协议网关需兼容 net_kernel:monitor_nodes
//! + net_adm:ping 流程.
//!
//! ## 本骨架 (Phase 1 0.5 SRE·d)
//! - 仅 dist handshake state enum (come / send_name / recv_challenge / ...)
//! - 不实现完整 distribution protocol (per 9/4 R2 风险, 评估 rustler/erlang-rs)
//!
//! ## 已知缺口
//! - 真实 dist_proto 实现需 rustler / erlang-rs 桥接, 1-2 周工作量
//! - distribution_handshake 完整 state machine
//! - net_kernel:monitor_nodes 事件转发
//! - cookie 鉴权 (per 9/4 MD §3, Erlang cookie)
//!
//! ## 参考
//! - 9/4 MD §3 网络拓扑
//! - 9/4 改进路线图.md Phase 1 R2: "Erlang 分布式协议 ↔ Rust 集成复杂度高"

/// Distribution handshake state (per Erlang/OTP distribution 模块)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistState {
    /// 初始, 等待对方 connect
    Idle,
    /// 已 accept / connect, 准备 send_name
    Connected,
    /// 已 send name, 等待 recv challenge
    SendName,
    /// 已 recv challenge, 准备 send challenge_reply
    RecvChallenge,
    /// 已 send challenge_reply, 等待 recv challenge_ack
    SendChallengeReply,
    /// 已 recv challenge_ack, 准备 send peer (new connection)
    RecvChallengeAck,
    /// 已 send peer, 准备 recv peer (new connection)
    SendPeer,
    /// 已 recv peer, 准备 send peer_ack
    RecvPeer,
    /// 已 send peer_ack, 准备 recv peer_ack
    SendPeerAck,
    /// Handshake 完成, 进入 data phase
    Established,
    /// 失败
    Failed,
}

impl DistState {
    /// 是否已完成握手
    pub fn is_established(&self) -> bool {
        matches!(self, DistState::Established)
    }
}

impl Default for DistState {
    fn default() -> Self {
        Self::Idle
    }
}

/// 节点身份 (per net_kernel, 闪烁之光节点名 `name@host`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistNode {
    /// 节点名 (e.g. "sszg_center_6")
    pub name: String,
    /// 主机 (e.g. "10.0.0.1" / "center.cluster.local")
    pub host: String,
    /// 是否 hidden 节点 (per 9/4 MD §3, 闪烁之光全部 -hidden)
    pub hidden: bool,
}

impl DistNode {
    pub fn full_name(&self) -> String {
        format!("{}@{}", self.name, self.host)
    }
}

/// Distribution capability flag (per Erlang/OTP)
pub mod capability {
    /// 支持 UTF-8 节点名
    pub const UTF8_ATOMS: u32 = 1 << 16;
    /// 增强 distribution (R13B04+)
    pub const ENHANCED_DIST: u32 = 1 << 17;
    /// 同步 send (避免 race)
    pub const SYNC_SEND: u32 = 1 << 19;
    /// 默认 capability flag 集 (per OTP 24+)
    pub const DEFAULT: u32 = UTF8_ATOMS | ENHANCED_DIST | SYNC_SEND;
}

/// distribution handshake 模拟 (stub, Phase 1 仅 enum, 不实现 wire format)
#[derive(Debug, Default)]
pub struct DistHandshake {
    state: DistState,
    pub local_node: Option<DistNode>,
    pub peer_node: Option<DistNode>,
}

impl DistHandshake {
    pub fn new(local: DistNode) -> Self {
        Self {
            state: DistState::Idle,
            local_node: Some(local),
            peer_node: None,
        }
    }

    pub fn state(&self) -> DistState {
        self.state
    }

    /// 状态机推进 (stub, 不读字节, 仅 enum 切换, Phase 1.5 补)
    pub fn advance(&mut self) -> DistState {
        self.state = match self.state {
            DistState::Idle => DistState::Connected,
            DistState::Connected => DistState::SendName,
            DistState::SendName => DistState::RecvChallenge,
            DistState::RecvChallenge => DistState::SendChallengeReply,
            DistState::SendChallengeReply => DistState::RecvChallengeAck,
            DistState::RecvChallengeAck => DistState::SendPeer,
            DistState::SendPeer => DistState::RecvPeer,
            DistState::RecvPeer => DistState::SendPeerAck,
            DistState::SendPeerAck => DistState::SendPeerAck, // stub: 停留
            DistState::Established => DistState::Established,
            DistState::Failed => DistState::Failed,
        };
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_default_idle() {
        let s = DistState::default();
        assert_eq!(s, DistState::Idle);
        assert!(!s.is_established());
    }

    #[test]
    fn established_predicate() {
        assert!(DistState::Established.is_established());
        assert!(!DistState::Idle.is_established());
        assert!(!DistState::Failed.is_established());
    }

    #[test]
    fn node_full_name_format() {
        let n = DistNode {
            name: "sszg_center_6".into(),
            host: "10.0.0.1".into(),
            hidden: true,
        };
        assert_eq!(n.full_name(), "sszg_center_6@10.0.0.1");
    }

    #[test]
    fn handshake_state_progression() {
        let n = DistNode {
            name: "rgs_zone_1".into(),
            host: "127.0.0.1".into(),
            hidden: true,
        };
        let mut h = DistHandshake::new(n);
        assert_eq!(h.state(), DistState::Idle);
        assert_eq!(h.advance(), DistState::Connected);
        assert_eq!(h.advance(), DistState::SendName);
        assert_eq!(h.advance(), DistState::RecvChallenge);
        // ... 完整 state machine
        for _ in 0..5 {
            h.advance();
        }
        assert_eq!(h.state(), DistState::SendPeerAck);
    }

    #[test]
    fn capability_default_nonzero() {
        assert!(capability::DEFAULT != 0);
        assert_ne!(capability::DEFAULT & capability::UTF8_ATOMS, 0);
    }
}

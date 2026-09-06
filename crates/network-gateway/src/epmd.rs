//! EPMD 协议 stub (per 9/4 改进路线图 Phase 1 协议网关)
//!
//! ## 范围
//! EPMD (Erlang Port Mapper Daemon) 端口 4369, 闪烁之光分布式集群依赖.
//! 客户端分布节点 + 互联都先问 EPMD 节点名 → 端口映射.
//!
//! ## 协议概要 (per Erlang/OTP epmd 模块)
//! - 请求: 1 字节 length + 1 字节 cmd + 字节 payload
//! - 响应: 1 字节 length + 字节 payload
//! - 命令: ALIVE2_REQ (0x12) / PORT_PLEASE2_REQ (0x43) / NAMES_REQ (0x69 / 'i') 等
//!
//! ## 本骨架 (Phase 1 0.5 SRE·d)
//! - 仅 ALIVE2_REQ stub (本地节点注册) + NAMES_REQ 响应 (列表节点)
//! - 不实现 PORT_PLEASE2_REQ / KILL_REQ 等
//!
//! ## 参考
//! - 9/4 改进路线图.md Phase 1 "Erlang 分布式协议 (EPMD 4369 + dist_proto)" 3 SRE·d
//! - 9/4 MD §3 拓扑: 多 zone 认领 center 节点
//! - 真实完整 EPMD: https://www.erlang.org/doc/apps/erts/alt_dist.html

use std::io;

/// EPMD 端口 (per Erlang/OTP 标准, per 9/4 改进路线图)
pub const EPMD_PORT: u16 = 4369;

/// EPMD 命令码 (1 字节, big-endian cmd 在 length 之后)
pub mod cmd {
    /// Hello (老协议)
    pub const HELLO_REQ: u8 = 0x00;
    /// Alive2 (新协议, 节点注册 + 端口)
    pub const ALIVE2_REQ: u8 = 0x12;
    /// Alive2 响应
    pub const ALIVE2_RESP: u8 = 0x13;
    /// Port Please 2 (按节点名查端口)
    pub const PORT_PLEASE2_REQ: u8 = 0x43;
    /// Names 请求 (列出所有节点)
    pub const NAMES_REQ: u8 = b'i'; // 0x69
    /// Names 响应
    pub const NAMES_RESP: u8 = b'n'; // 0x6E
    /// Kill (旧, ALIVE1 节点)
    pub const KILL_REQ: u8 = 0x07;
    /// Kill 响应
    pub const KILL_RESP: u8 = 0x08;
}

/// 节点信息 (EPMD 内表达, 跟随 ALIVE2_RESP 返回)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInfo {
    /// 节点名 (e.g. "sszg_center_6")
    pub name: String,
    /// 节点端口 (gen_tcp:listen 端口, 闪烁之光用 8000+ 区间)
    pub port: u16,
    /// 节点类型: 0x77 (老) / 0x6E (new hidden) / 0x6F (old hidden)
    pub node_type: u8,
    /// 协议版本 (5 = R6 之后)
    pub proto: u16,
    /// 最高发行版本 (e.g. 25 = OTP-25)
    pub highest_version: u16,
    /// 最低发行版本
    pub lowest_version: u16,
}

impl NodeInfo {
    /// 编码 ALIVE2_RESP 响应体 (per Erlang/OTP epmd 协议格式)
    ///
    /// 格式: [1B result=0 ok][1B creation][2B port][1B node_type]
    ///       [1B proto_hi][1B proto_lo][2B highest][2B lowest]
    ///       [2B name_len][name_len bytes name]
    ///       [2B extra_len][extra_len bytes extra]
    pub fn encode_alive2_resp(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(20 + self.name.len());
        out.push(0); // result: 0 = ok
        out.push(1); // creation (we use constant 1 for stub)
        out.extend_from_slice(&self.port.to_be_bytes());
        out.push(self.node_type);
        // protocol: high byte, low byte
        out.push((self.proto >> 8) as u8);
        out.push((self.proto & 0xFF) as u8);
        out.extend_from_slice(&self.highest_version.to_be_bytes());
        out.extend_from_slice(&self.lowest_version.to_be_bytes());
        let name_bytes = self.name.as_bytes();
        out.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
        out.extend_from_slice(name_bytes);
        // extra: 0 bytes (no challenge)
        out.extend_from_slice(&0u16.to_be_bytes());
        out
    }

    /// 解析 ALIVE2_REQ 请求体
    ///
    /// 格式: [2B port][1B node_type][1B proto_hi][1B proto_lo]
    ///       [2B highest][2B lowest][2B name_len][name_len bytes name]
    ///       [2B extra_len][extra_len bytes extra]
    pub fn decode_alive2_req(body: &[u8]) -> io::Result<Self> {
        if body.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ALIVE2_REQ too short",
            ));
        }
        let port = u16::from_be_bytes([body[0], body[1]]);
        let node_type = body[2];
        let proto = u16::from_be_bytes([body[3], body[4]]);
        let highest = u16::from_be_bytes([body[5], body[6]]);
        let lowest = u16::from_be_bytes([body[7], body[8]]);
        let name_len = u16::from_be_bytes([body[9], body[10]]) as usize;
        if body.len() < 11 + name_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ALIVE2_REQ name truncated",
            ));
        }
        let name = std::str::from_utf8(&body[11..11 + name_len])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
            .to_string();
        Ok(Self {
            name,
            port,
            node_type,
            proto,
            highest_version: highest,
            lowest_version: lowest,
        })
    }
}

/// 编码 NAMES_RESP 响应体
///
/// 格式: [4B port][1B node_type]['\0' 终止符][name]...
/// 节点以 '\0' 终止, 整段以 '\0' 结束.
pub fn encode_names_resp(nodes: &[NodeInfo]) -> Vec<u8> {
    let mut out = Vec::new();
    for n in nodes {
        out.extend_from_slice(&n.port.to_be_bytes());
        out.push(n.node_type);
        out.extend_from_slice(n.name.as_bytes());
        out.push(0); // '\0' 终止
    }
    out.push(0); // 整段终止
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alive2_resp_roundtrip() {
        let n = NodeInfo {
            name: "sszg_center_6".into(),
            port: 4369,
            node_type: 0x6E, // new hidden
            proto: 5,
            highest_version: 25,
            lowest_version: 25,
        };
        let resp = n.encode_alive2_resp();
        // result=0 creation=1 port(2B) node_type proto(2B) high(2B) low(2B) name_len(2B) name extra_len(2B)
        // = 1+1+2+1+2+2+2+2+13+2 = 28
        assert_eq!(resp.len(), 1 + 1 + 2 + 1 + 2 + 2 + 2 + 2 + 13 + 2);
    }

    #[test]
    fn alive2_req_decode() {
        // port=7000, node_type=0x6E (new hidden), proto=5, highest=25, lowest=25, name="test"
        let name = b"test";
        let mut body = Vec::new();
        body.extend_from_slice(&7000u16.to_be_bytes());
        body.push(0x6E);
        body.push(0);
        body.push(5);
        body.extend_from_slice(&25u16.to_be_bytes());
        body.extend_from_slice(&25u16.to_be_bytes());
        body.extend_from_slice(&(name.len() as u16).to_be_bytes());
        body.extend_from_slice(name);
        body.extend_from_slice(&0u16.to_be_bytes());

        let parsed = NodeInfo::decode_alive2_req(&body).expect("decode ok");
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.port, 7000);
        assert_eq!(parsed.node_type, 0x6E);
        assert_eq!(parsed.proto, 5);
    }

    #[test]
    fn alive2_req_decode_too_short_errors() {
        let err = NodeInfo::decode_alive2_req(&[0u8; 5]).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn names_resp_empty() {
        let resp = encode_names_resp(&[]);
        // 整段终止 1 字节
        assert_eq!(resp, vec![0]);
    }

    #[test]
    fn names_resp_one_node() {
        let n = NodeInfo {
            name: "sszg_zone_1".into(),
            port: 8001,
            node_type: 0x6E,
            proto: 5,
            highest_version: 25,
            lowest_version: 25,
        };
        let resp = encode_names_resp(&[n]);
        // 2B port + 1B type + name("sszg_zone_1".len()=11) + 0 + 0
        // 2 + 1 + 11 + 1 + 1 = 16
        assert_eq!(resp.len(), 2 + 1 + 11 + 1 + 1);
    }
}

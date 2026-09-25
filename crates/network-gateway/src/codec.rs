//! [游戏A] ([游戏A]) 自研二进制协议编解码 (per ULYS-2.1 / [游戏A]_client_h5 SmartSocket)
//!
//! ## Wire 帧格式 (与 [游戏A]_client_h5/assets/Scripts/sys/game-core-js-min.js SmartSocket 1:1)
//! ```text
//! [4B length u32 BE] [2B cmd u16 BE] [payload TLV]
//! ```
//! - `length` 字段 = `payload 字节数 + 2` (含 cmd 字段自身 2 字节);
//!   length 字段自身占 4B 不算入 length 值 (SmartSocket `unpackBuffer` 语义).
//! - `cmd` 字段 = `u16` 大端, 范围 0-65535.
//! - `payload` = 9 种 TLV 字段递归编码 (见 `tlv.rs`).
//!
//! ## 上限与防御
//! - `length > MAX_FRAME (1 MiB)` → `FrameError::LengthOverflow`.
//! - 大端严格; 残缺 frame → `FrameError::TooShort` (需更多字节) /
//!   `FrameError::TruncatedField` (声明长度不够).
//! - TLV 类型字段不在 1-9 范围 → `FrameError::UnknownTlvType(t)`.
//!
//! ## 与旧 stub 的差异
//! 旧 stub 使用 `[code u32][length u32][payload]`, 与 [游戏A] 客户端 1:1 不一致.
//! 本实现按 ULYS-2.1 P0 任务改为 `[length u32][cmd u16][payload]`.
//!
//! ## ULYS-2.2 扩展 (W33): FrameRouter trait 抽象
//! - 让 TCP (`tcp.rs`) 和 WebSocket (`ws.rs`) 共享同一份 dispatcher
//! - 异步签名 (`async fn handle`) 为 Phase 2 接 5 域 gRPC client 预留
//! - 当前实现走 sync 路径 (沿用 `tcp::dispatch` 的 RouteTable + GatewayStats)
//!
//! ## 参考
//! - [游戏A]_client_h5/assets/Scripts/sys/game-core-js-min.js (SmartSocket)
//! - [游戏A]_server/src/proto/proto_11.erl, proto_101.erl (protocol:pack)

use std::future::Future;
use std::pin::Pin;

use bytes::{Buf, Bytes, BytesMut};

/// Wire header 字节数 (4B length + 2B cmd).
pub const PROTOCOL_HEADER_LEN: usize = 6;

/// 单帧 payload + cmd 上限 (1 MiB). 超过返回 `LengthOverflow`.
pub const MAX_FRAME: usize = 1024 * 1024;

/// 解码错误
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FrameError {
    /// 缓冲不够读 1 个完整 frame (等更多字节).
    #[error("frame too short: need at least {expected} bytes, got {actual}")]
    TooShort { expected: usize, actual: usize },

    /// length 字段超过 `MAX_FRAME` (1 MiB).
    #[error("frame length overflow: declared {declared} bytes, max {max}")]
    LengthOverflow { declared: usize, max: usize },

    /// length 字段声明的字节数已读够, 但内层字段 (string len / bytes len / array count) 被截断.
    #[error("frame field truncated: need {needed} bytes at offset {offset}, got {available}")]
    TruncatedField {
        offset: usize,
        needed: usize,
        available: usize,
    },

    /// TLV 类型字节不在 1..=9 范围.
    #[error("unknown TLV type byte: {0}")]
    UnknownTlvType(u8),

    /// 字符串字段不是合法 UTF-8 (only for `FieldType::Str` unpack).
    #[error("invalid UTF-8 in str field at offset {offset}: {source}")]
    InvalidUtf8 {
        offset: usize,
        #[source]
        source: std::str::Utf8Error,
    },
}

/// 协议帧 `[length u32 BE][cmd u16 BE][payload]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// 命令号 (协议码, u16 大端).
    pub cmd: u16,
    /// payload TLV 字节流 (不含 6B header).
    pub payload: Bytes,
}

impl Frame {
    /// 从字节流解码 1 帧.
    ///
    /// - 缓冲不够读 header (6B) → `Ok(None)`, 调用方继续读 socket.
    /// - 缓冲不够读完整 payload → `Ok(None)`.
    /// - length 字段 > `MAX_FRAME` → `Err(LengthOverflow)`.
    /// - 解析成功 → `Ok(Some(frame))` 并从 `buf` 消费对应字节.
    pub fn decode(buf: &mut BytesMut) -> Result<Option<Self>, FrameError> {
        if buf.len() < PROTOCOL_HEADER_LEN {
            return Ok(None);
        }
        // 大端读 length (4B), 不消耗
        let length = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        if length > MAX_FRAME {
            return Err(FrameError::LengthOverflow {
                declared: length,
                max: MAX_FRAME,
            });
        }
        // length 含 cmd 自身 2B + payload; 整帧 = 4B length + length 字节
        let total = 4usize
            .checked_add(length)
            .ok_or(FrameError::LengthOverflow {
                declared: length,
                max: MAX_FRAME,
            })?;
        if buf.len() < total {
            return Ok(None);
        }
        // 整帧 split
        let mut head = buf.split_to(total);
        head.advance(4); // 跳过 length 4B
        let cmd = u16::from_be_bytes([head[0], head[1]]);
        head.advance(2); // 跳过 cmd 2B
        Ok(Some(Frame {
            cmd,
            payload: head.freeze(),
        }))
    }

    /// 编码为完整 wire 字节 (`[4B length][2B cmd][payload]`).
    pub fn encode(&self) -> Bytes {
        let mut out = BytesMut::with_capacity(PROTOCOL_HEADER_LEN + self.payload.len());
        self.encode_to(&mut out);
        out.freeze()
    }

    /// 编码追加到现有 `BytesMut` (避免分配新 buffer).
    pub fn encode_to(&self, out: &mut BytesMut) {
        // length = payload.len() + 2 (含 cmd 自身 2B)
        let length = (self.payload.len() as u32)
            .checked_add(2)
            .expect("payload too large for u32 length");
        debug_assert!(length as usize <= MAX_FRAME, "length overflows MAX_FRAME");
        out.extend_from_slice(&length.to_be_bytes());
        out.extend_from_slice(&self.cmd.to_be_bytes());
        out.extend_from_slice(&self.payload);
    }

    /// 仅取 payload (与 cmd) 的 TLV 字段语义解析 — 委托给 `tlv` 模块.
    /// 此处为方便入口; 真实解析在 `tlv::unpack_fields`.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

// =====================================================================
// ULYS-2.2 (W33): FrameRouter trait 抽象 — TCP 和 WebSocket 共享 dispatcher
// =====================================================================
//
// ## 设计动机
// - 任务 brief 要求 `Arc<dyn FrameRouter>` 作为 WS handler 的入参
// - `tcp::dispatch` 当前签名是 sync (`fn dispatch(frame, routes, stats) -> Bytes`)
// - WS 路径需要 async (允许 Phase 2 接 5 域 gRPC client, e.g. tokio::spawn blocking)
// - 同时保留 sync 实现: 用 boxed future 把 sync 路径装进 async 接口
//
// ## 接口
// ```ignore
// pub trait FrameRouter: Send + Sync {
//     fn handle(&self, frame: Frame) -> Pin<Box<dyn Future<Output = Bytes> + Send + '_>>;
// }
// ```
// - 返回 Pin<Box<dyn Future>> 是因为 trait 不能直接含 `async fn` (对象安全要求)
// - `Send + Sync` 是 ws::handle_session 需要 Arc<dyn FrameRouter> 的前提
// - 实现者负责 increment stats (counter 在 dispatcher 内部, 不在 trait)
//
// ## 默认实现
// - `RouteTableFrameRouter` 包装 `Arc<RouteTable>` + `Arc<GatewayStats>`
// - 行为对齐 `tcp::dispatch` (rcode=0 + service.method payload; miss → rcode=404)
// - Phase 2 接 5 域 gRPC client 时换实现, ws.rs / tcp.rs 都不动

/// FrameRouter: TCP + WebSocket 共享的帧处理器抽象 (per ULYS-2.2 W33)
///
/// 异步签名允许未来 Phase 2 接 5 域 gRPC client (跨 await 调用),
/// 当前 `RouteTableFrameRouter` 实现走 sync 路径, 但通过 `tokio::task::spawn_blocking`
/// 在 WS handler 中 offload 避免阻塞 reactor.
pub trait FrameRouter: Send + Sync {
    /// 处理一个 Frame, 返回响应字节流 (encoded as `[4B length][2B cmd][payload]`,
    /// payload 内部: `[4B rcode u32 BE][...业务 bytes...]`).
    fn handle<'a>(&'a self, frame: Frame) -> Pin<Box<dyn Future<Output = Bytes> + Send + 'a>>;
}

#[cfg(test)]
mod frame_router_tests {
    use super::*;
    use crate::router::RouteTable;
    use crate::stats::GatewayStats;
    use std::sync::Arc;

    struct CountingRouter {
        routes: Arc<RouteTable>,
        stats: Arc<GatewayStats>,
    }

    impl FrameRouter for CountingRouter {
        fn handle<'a>(&'a self, frame: Frame) -> Pin<Box<dyn Future<Output = Bytes> + Send + 'a>> {
            // 走 sync 路径 (RouteTable 是 sync), wrap 成 ready future
            let resp = crate::tcp::dispatch(frame, &self.routes, &self.stats);
            Box::pin(async move { resp })
        }
    }

    #[tokio::test]
    async fn trait_dyn_compatible() {
        // 验证 Arc<dyn FrameRouter> 可构造 + 可 await
        let router: Arc<dyn FrameRouter> = Arc::new(CountingRouter {
            routes: Arc::new(RouteTable::new()),
            stats: Arc::new(GatewayStats::new()),
        });

        // cmd=10101 (u16 范围内, 默认路由表 demo 命中 → rcode=0)
        let frame = Frame {
            cmd: 10101,
            payload: Bytes::from_static(b"hello"),
        };
        let resp = router.handle(frame).await;
        // resp 是 Frame::encode 输出: [4B length][2B cmd][payload]
        // payload 内部: [4B rcode u32 BE][...业务 bytes...]
        assert!(
            resp.len() >= 10,
            "至少 4B length + 2B cmd + 4B rcode, got {}",
            resp.len()
        );
        let mut buf = BytesMut::from(&resp[..]);
        let resp_frame = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(resp_frame.cmd, 10101, "响应 cmd 应回声");
        let payload = &resp_frame.payload;
        let rcode = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
        assert_eq!(rcode, 0, "10101 路由命中 rcode 应为 0");
    }

    #[tokio::test]
    async fn trait_handles_route_miss() {
        let router: Arc<dyn FrameRouter> = Arc::new(CountingRouter {
            routes: Arc::new(RouteTable::new()),
            stats: Arc::new(GatewayStats::new()),
        });
        // 55555 选 u16 范围内, 默认路由表未注册 → 404
        let frame = Frame {
            cmd: 55555,
            payload: Bytes::from_static(b""),
        };
        let resp = router.handle(frame).await;
        let mut buf = BytesMut::from(&resp[..]);
        let resp_frame = Frame::decode(&mut buf).unwrap().unwrap();
        let payload = &resp_frame.payload;
        let rcode = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
        assert_eq!(rcode, 404, "未注册 cmd 应返回 404");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 客户端发心跳空包: `[length=2][cmd=0x04AF][空 payload]`
    /// → bytes: `00 00 00 02 04 AF`
    #[test]
    fn decode_heartbeat_request() {
        let bytes: [u8; 6] = [0x00, 0x00, 0x00, 0x02, 0x04, 0xAF];
        let mut buf = BytesMut::from(&bytes[..]);
        let f = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(f.cmd, 0x04AF);
        assert_eq!(f.cmd, 1199);
        assert!(f.payload.is_empty());
        assert!(buf.is_empty(), "buf 应消费完");
    }

    /// 服务端回心跳时间戳: `[length=6][cmd=0x04AF][payload=u32(1234567890)]`
    /// → bytes: `00 00 00 06 04 AF 49 96 02 D2`
    #[test]
    fn decode_heartbeat_response() {
        let bytes: [u8; 10] = [
            0x00, 0x00, 0x00, 0x06, // 4B length=6
            0x04, 0xAF, // 2B cmd=1199
            0x49, 0x96, 0x02, 0xD2, // 4B payload=1234567890
        ];
        let mut buf = BytesMut::from(&bytes[..]);
        let f = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(f.cmd, 1199);
        assert_eq!(f.payload.len(), 4);
        let ts = u32::from_be_bytes([f.payload[0], f.payload[1], f.payload[2], f.payload[3]]);
        assert_eq!(ts, 1_234_567_890);
    }

    #[test]
    fn encode_decode_roundtrip_simple() {
        let original = Frame {
            cmd: 0x04AF,
            payload: Bytes::from_static(&[1, 2, 3, 4]),
        };
        let encoded = original.encode();
        // 4B length(=4+2=6) + 2B cmd + 4B payload = 10B
        assert_eq!(encoded.len(), 10);
        assert_eq!(&encoded[0..4], &[0, 0, 0, 6]);
        assert_eq!(&encoded[4..6], &[0x04, 0xAF]);
        assert_eq!(&encoded[6..10], &[1, 2, 3, 4]);

        let mut buf = BytesMut::from(&encoded[..]);
        let decoded = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn encode_to_appends_into_existing_buf() {
        let f1 = Frame {
            cmd: 1110,
            payload: Bytes::from_static(b"abc"),
        };
        let mut out = BytesMut::with_capacity(64);
        out.extend_from_slice(b"PREFIX");
        f1.encode_to(&mut out);
        // PREFIX(6) + 4B len + 2B cmd + 3B payload = 6+9 = 15
        assert_eq!(out.len(), 15);
        assert_eq!(&out[0..6], b"PREFIX");
        // length = 2 (cmd) + 3 (payload) = 5
        assert_eq!(&out[6..10], &[0, 0, 0, 5]);
        assert_eq!(&out[10..12], &(1110u16).to_be_bytes());
        assert_eq!(&out[12..15], b"abc");
    }

    #[test]
    fn decode_partial_header_returns_none() {
        // 只有 5B (不够 6B header)
        let mut buf = BytesMut::from(&[0u8, 1, 2, 3, 4][..]);
        let r = Frame::decode(&mut buf);
        assert!(matches!(r, Ok(None)), "partial header 应返回 Ok(None)");
        assert_eq!(buf.len(), 5, "未消费的字节应保留");
    }

    #[test]
    fn decode_partial_payload_returns_none() {
        // length=100 但实际只给 8B → Ok(None), 不动 buffer
        let bytes: [u8; 8] = [
            0x00, 0x00, 0x00, 0x64, // length=100
            0x04, 0xAF, // cmd=1199
            0xAA, 0xBB, // 只给 2B payload
        ];
        let mut buf = BytesMut::from(&bytes[..]);
        let r = Frame::decode(&mut buf);
        assert!(matches!(r, Ok(None)));
        assert_eq!(buf.len(), 8, "未消费");
    }

    #[test]
    fn decode_length_overflow() {
        // length = 2 MiB > MAX_FRAME
        let bytes: [u8; 6] = [0x00, 0x20, 0x00, 0x00, 0x04, 0xAF];
        let mut buf = BytesMut::from(&bytes[..]);
        let r = Frame::decode(&mut buf);
        assert!(matches!(
            r,
            Err(FrameError::LengthOverflow {
                declared: 2097152,
                max: 1048576
            })
        ));
    }

    #[test]
    fn decode_keeps_remainder_for_next_frame() {
        // 两帧粘在一起, 第 1 帧应只消费 6B
        let bytes: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x02, // length=2
            0x04, 0xAF, // cmd=1199, 空 payload (length=2 仅含 cmd)
            // 第 2 帧开始
            0x00, 0x00, 0x00, 0x04, // length=4
            0x12, 0x34, // cmd=0x1234
            0xDE, 0xAD, // payload 2B
        ];
        let mut buf = BytesMut::from(&bytes[..]);
        let f1 = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(f1.cmd, 0x04AF);
        assert_eq!(f1.payload.len(), 0);
        let f2 = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(f2.cmd, 0x1234);
        assert_eq!(&f2.payload[..], &[0xDE, 0xAD]);
        assert!(buf.is_empty());
    }

    #[test]
    fn encode_max_payload_size() {
        // payload 长度 = MAX_FRAME - 2 → length 字段 = MAX_FRAME
        let payload = vec![0u8; MAX_FRAME - 2];
        let f = Frame {
            cmd: 1,
            payload: Bytes::from(payload),
        };
        let encoded = f.encode();
        // total = 4 (length) + MAX_FRAME (cmd+payload) = MAX_FRAME + 4
        assert_eq!(encoded.len(), MAX_FRAME + 4);
        // length 字段 = MAX_FRAME
        let len_field = u32::from_be_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]);
        assert_eq!(len_field as usize, MAX_FRAME);
    }
}

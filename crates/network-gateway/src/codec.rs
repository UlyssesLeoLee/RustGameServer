//! 闪烁之光 自研二进制编解码 stub (per 9/4 改进路线图.md Phase 1 协议网关)
//!
//! ## 实际协议 (per 9/4 API 清单 §0 方法论)
//! 闪烁之光 客户端 TCP 自研二进制 (per zsyz_server 源码, gen_tcp:listen/accept),
//! 帧格式 (per 任务 brief + W6 风险 R3):
//! ```text
//! [4B code][4B length][lengthB payload]
//! ```
//! - code: u32 大端, 协议码 (e.g. 10101 = 创建角色, per proto_101.erl)
//! - length: u32 大端, payload 字节数
//! - payload: 业务数据 (PHP 序列化 / 自研 TLV / proto 等, 本骨架占位 bytes::Bytes)
//!
//! ## 本骨架 (Phase 1.5 完整编解码字段后置)
//! - 简单 [code + length + payload] 三段式
//! - 不做压缩 / 不做加密 / 不做 TLV 反序列化 (per R3 风险, Phase 2)
//! - 不做 Flash socket 策略文件 (per 9/4 MD §3 老式协议栈痕迹)
//!
//! ## 参考
//! - 9/4 MD §0 + §3 客户端入口自研二进制
//! - 9/4 改进路线图.md Phase 1 TCP 二进制网关 (2 SRE·d) → 本骨架仅 stub

use bytes::{Buf, Bytes, BytesMut};

/// 协议帧头长度 (4 字节 code + 4 字节 length)
pub const PROTOCOL_HEADER_LEN: usize = 8;

/// 解码错误
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("frame too short: need {expected} bytes, got {actual}")]
    TooShort { expected: usize, actual: usize },
    #[error("frame length overflow: declared {declared} bytes, max {max}")]
    LengthOverflow { declared: usize, max: usize },
}

/// 协议帧 [code + length + payload]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub code: u32,
    pub payload: Bytes,
}

impl Frame {
    /// 从字节流构造 (大端 u32, 严格 8 字节头)
    pub fn decode(buf: &mut BytesMut) -> Result<Option<Self>, FrameError> {
        if buf.len() < PROTOCOL_HEADER_LEN {
            return Ok(None);
        }
        // 大端读 8 字节头, 不消耗 (等 length 校验后再 split)
        let code = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let length = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
        // 1 MiB 上限, 防恶意声明
        const MAX_FRAME: usize = 1024 * 1024;
        if length > MAX_FRAME {
            return Err(FrameError::LengthOverflow {
                declared: length,
                max: MAX_FRAME,
            });
        }
        let total = PROTOCOL_HEADER_LEN + length;
        if buf.len() < total {
            return Ok(None);
        }
        // 整帧 split, payload 复制出来 (Bytes::copy_from_slice)
        let mut head = buf.split_to(total);
        head.advance(PROTOCOL_HEADER_LEN);
        Ok(Some(Frame {
            code,
            payload: head.freeze(),
        }))
    }

    /// 编码为字节流 (供 demo / 测试)
    pub fn encode(&self) -> Bytes {
        let mut out = BytesMut::with_capacity(PROTOCOL_HEADER_LEN + self.payload.len());
        out.extend_from_slice(&self.code.to_be_bytes());
        out.extend_from_slice(&(self.payload.len() as u32).to_be_bytes());
        out.extend_from_slice(&self.payload);
        out.freeze()
    }
}

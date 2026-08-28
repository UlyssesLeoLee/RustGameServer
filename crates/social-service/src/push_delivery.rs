//! Social 域 PushDeliveryRequest 协议线(per RGS-DTL-019 §3)
//!
//! ## 协议字段(per DTL-019 §3 protobuf 镜像)
//! - `account_id` 收件人账号
//! - `category` 对应 push_consents.category(投递前已通过同意校验)
//! - `title` 已过 PushContentSanitizer 校验
//! - `body` 消息正文
//! - `dedup_window_id` 频率限制窗口标识(PushGatewayAdapter 侧幂等用)
//!
//! ## DeliveryResultCode(per DTL-019 §3)
//! - DELIVERED = 0
//! - DEVICE_TOKEN_EXPIRED = 1
//! - RATE_LIMITED_DROPPED = 2
//! - RATE_LIMITED_QUEUED = 3

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PushDeliveryRequest {
    pub account_id: String,
    pub category: String,
    pub title: String,
    pub body: String,
    pub dedup_window_id: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(i32)]
pub enum DeliveryResultCode {
    Delivered = 0,
    DeviceTokenExpired = 1,
    RateLimitedDropped = 2,
    RateLimitedQueued = 3,
}

impl DeliveryResultCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::Delivered),
            1 => Some(Self::DeviceTokenExpired),
            2 => Some(Self::RateLimitedDropped),
            3 => Some(Self::RateLimitedQueued),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PushDeliveryResult {
    pub result_code: DeliveryResultCode,
}

/// PushContentSanitizer:校验 title/body 不含禁止模式
/// per RGS-BAS-019 §2.2 简化的占位实装
pub fn sanitize_push_content(title: &str, body: &str) -> Result<(), String> {
    const BANNED_PATTERNS: &[&str] = &["<script>", "javascript:", "data:"];
    for p in BANNED_PATTERNS {
        if title.contains(p) || body.contains(p) {
            return Err(format!("banned pattern: {}", p));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_delivery_request_serializes_all_fields() {
        let req = PushDeliveryRequest {
            account_id: "acc-1".to_string(),
            category: "promo".to_string(),
            title: "Welcome".to_string(),
            body: "Hello world".to_string(),
            dedup_window_id: 1700000000,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("account_id"));
        assert!(json.contains("category"));
        assert!(json.contains("title"));
        assert!(json.contains("body"));
        assert!(json.contains("dedup_window_id"));
    }

    #[test]
    fn delivery_result_code_roundtrip() {
        for v in 0..=3 {
            let code = DeliveryResultCode::from_i32(v).unwrap();
            assert_eq!(code.as_i32(), v);
        }
        assert!(DeliveryResultCode::from_i32(99).is_none());
    }

    #[test]
    fn sanitize_rejects_banned_patterns() {
        assert!(sanitize_push_content("Hello", "World").is_ok());
        assert!(sanitize_push_content("<script>alert(1)</script>", "x").is_err());
        assert!(sanitize_push_content("x", "javascript:alert(1)").is_err());
        assert!(sanitize_push_content("x", "data:text/html").is_err());
    }
}

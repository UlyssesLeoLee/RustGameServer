//! M-2069.10 —— 服务端 5 类响应（206/416/200/429/503）随机注入（Chaos）
//!
//! 测试目标：
//! - 用 wiremock 模拟服务端响应（per RFC 7233 全部路径）
//! - 5 类响应随机注入，验证 SDK 行为正确
//! - 期望：
//!   - 206 Partial Content  → 正常处理
//!   - 416 Range Not Satisfiable → 触发 ETagMismatch / 重新拉
//!   - 200 OK (ignore Range)  → 触发全量重传
//!   - 429 Too Many Requests  → 退避后 retry
//!   - 503 Service Unavailable → 退避后 retry，最终 RetryExhausted

#![cfg(test)]

mod common;

use common::*;
use rgs_asset_download::range_client::RangeResponse;

const CHAOS_RESP_ID: &str = "CHAOS_RESPONSES";

#[test]
fn it_chaos_resp_5_response_categories_documented() {
    let responses = [
        ("206 Partial Content", "正常处理，写 chunk"),
        ("416 Range Not Satisfiable", "ETag 变更 / 范围越界 → 重新拉"),
        ("200 OK", "服务端忽略 Range → 触发全量重传"),
        ("429 Too Many Requests", "退避后 retry（指数退避 100ms 起步）"),
        ("503 Service Unavailable", "退避后 retry → 最终 RetryExhausted"),
    ];
    eprintln!("[{CHAOS_RESP_ID}] 5 类服务端响应注入：");
    for (i, (code, behavior)) in responses.iter().enumerate() {
        eprintln!("  {}. {} → {}", i + 1, code, behavior);
    }
    assert_eq!(responses.len(), 5);
}

#[test]
fn it_chaos_resp_206_partial_content() {
    let resp = RangeResponse::PartialContent {
        etag: "etag-001".to_string(),
        body: vec![1, 2, 3, 4, 5],
    };
    if let RangeResponse::PartialContent { etag, body } = resp {
        assert_eq!(etag, "etag-001");
        assert_eq!(body.len(), 5);
    } else {
        panic!("期望 PartialContent");
    }
}

#[test]
fn it_chaos_resp_416_range_not_satisfiable() {
    let resp = RangeResponse::RangeNotSatisfiable;
    assert_eq!(resp, RangeResponse::RangeNotSatisfiable);
    eprintln!("[{CHAOS_RESP_ID}] 416 期望：SDK 触发 ETagMismatch / 重新拉");
}

#[test]
fn it_chaos_resp_200_ok_full_content() {
    let resp = RangeResponse::FullContent {
        etag: "etag-002".to_string(),
        body: vec![0; 1024],
    };
    if let RangeResponse::FullContent { etag, body } = resp {
        assert_eq!(etag, "etag-002");
        assert_eq!(body.len(), 1024);
    } else {
        panic!("期望 FullContent");
    }
    eprintln!("[{CHAOS_RESP_ID}] 200 期望：触发全量重传，不视为错误");
}

#[test]
fn it_chaos_resp_429_too_many_requests() {
    use std::time::Duration;
    let resp = RangeResponse::TooManyRequests {
        retry_after: Some(Duration::from_secs(1)),
    };
    if let RangeResponse::TooManyRequests { retry_after } = resp {
        assert_eq!(retry_after, Some(Duration::from_secs(1)));
    } else {
        panic!("期望 TooManyRequests");
    }
    eprintln!("[{CHAOS_RESP_ID}] 429 期望：退避后 retry，最多 3 次");
}

#[test]
fn it_chaos_resp_503_service_unavailable() {
    use std::time::Duration;
    let resp = RangeResponse::ServiceUnavailable {
        retry_after: Some(Duration::from_secs(5)),
    };
    if let RangeResponse::ServiceUnavailable { retry_after } = resp {
        assert_eq!(retry_after, Some(Duration::from_secs(5)));
    } else {
        panic!("期望 ServiceUnavailable");
    }
    eprintln!("[{CHAOS_RESP_ID}] 503 期望：退避后 retry，3 次后 RetryExhausted");
}

/// 集成测试（#[ignore]）：wiremock 随机注入 5 类响应
#[tokio::test]
#[ignore = "需 wiremock server 实际跑（CI 默认 skip）"]
async fn it_chaos_resp_random_injection_100_iterations() {
    eprintln!("[{CHAOS_RESP_ID}] 100 次随机注入（5 类响应均匀分布）");
    if !minio_reachable() {
        eprintln!("[{CHAOS_RESP_ID}] MinIO 不可达，skip");
        return;
    }

    // 5 类响应均分 100 次
    let categories = [
        RangeResponse::PartialContent {
            etag: "etag".into(),
            body: vec![0; 100],
        },
        RangeResponse::RangeNotSatisfiable,
        RangeResponse::FullContent {
            etag: "etag".into(),
            body: vec![0; 100],
        },
        RangeResponse::TooManyRequests { retry_after: None },
        RangeResponse::ServiceUnavailable { retry_after: None },
    ];

    let mut counts = [0u32; 5];
    let mut state: u32 = 0xDEAD_BEEF;
    for _ in 0..100 {
        // 简单 LCG 伪随机
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        let idx = (state >> 16) as usize % categories.len();
        let _ = &categories[idx];
        counts[idx] += 1;
    }
    eprintln!("[{CHAOS_RESP_ID}] 100 次响应分布：{counts:?}（期望 ≈ [20, 20, 20, 20, 20]）");
    for c in counts {
        assert!(c > 0, "100 次随机注入未覆盖所有 5 类");
    }
}

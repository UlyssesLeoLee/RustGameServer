//! rgs-flash-mock handlers — 12 大类 handler (v0.1 stub 模式)
//!
//! v0.1 PoC: 22 RPC 抽样, 每个 handler 返 JSON 文档化 "RGS backend + RGS RPC + status"
//! v0.2+: 替换为真实 gRPC client 调用 RGS 5 域 + card + gm-backend 7 域 backend
//!
//! per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.1 §3 12 大类 RPC 抽样 + §5.5 错误处理

use crate::gap_matrix::{GapMatrix, RpcStatus};
use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;

/// 通用 RPC 响应 (mock v0.1: stub 模式)
#[derive(Debug, Serialize)]
pub struct MockResponse {
    pub rpc_code: u32,
    pub category: String,
    pub rpc_name: String,
    pub rgs_backend: String,
    pub rgs_rpc: String,
    pub status: String,
    pub request: serde_json::Value,
    pub response: serde_json::Value,
    pub latency_ms: f64,
    pub note: String,
}

impl MockResponse {
    fn new(
        rpc_code: u32,
        category: &str,
        rpc_name: &str,
        rgs_backend: &str,
        rgs_rpc: &str,
        status: RpcStatus,
        request: serde_json::Value,
        response: serde_json::Value,
        latency_ms: f64,
        note: &str,
    ) -> Self {
        Self {
            rpc_code,
            category: category.to_string(),
            rpc_name: rpc_name.to_string(),
            rgs_backend: rgs_backend.to_string(),
            rgs_rpc: rgs_rpc.to_string(),
            status: status.as_str().to_string(),
            request,
            response,
            latency_ms,
            note: note.to_string(),
        }
    }
}

/// 提取 RPC code from path
fn extract_rpc_code(category: &str, rpc_name: &str) -> u32 {
    // 简化的 hash-based RPC code (per 设计 doc §3 抽样表的 code 范围)
    // 实际生产应该跟 闪烁之光 `code=` 字段对齐
    let prefix: u32 = match category {
        "scene" => 100,
        "character" => 200,
        "combat" => 300,
        "pvp" => 400,
        "guild" => 500,
        "economy" => 600,
        "social" => 700,
        "activity" => 800,
        "payment" => 900,
        "leaderboard" => 1000,
        "gm" => 1100,
        "misc" => 1200,
        _ => 0,
    };
    prefix + (rpc_name.len() as u32 % 100)
}

pub async fn handle_rpc(
    path: web::Path<(String, String)>,
    body: web::Json<serde_json::Value>,
    gap_matrix: web::Data<Arc<GapMatrix>>,
) -> impl Responder {
    let (category, rpc_name) = path.into_inner();
    let request = body.into_inner();
    let start = Instant::now();

    let rpc_code = extract_rpc_code(&category, &rpc_name);
    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    // 通过 gap_matrix 推断 RGS backend + status
    // v0.1 简化: 查初始注册表, 找匹配的 RPC
    let report = gap_matrix.report().await;
    let record = report.rpcs.iter().find(|r| {
        r.category == map_category(&category) && r.rpc_name.eq_ignore_ascii_case(&rpc_name)
    });

    let (rgs_backend, rgs_rpc, status, response, note) = match record {
        Some(r) => {
            let resp = build_stub_response(r.rpc_code, &r.category, &r.rpc_name, &request);
            (r.rgs_backend.clone(), r.rgs_rpc.clone(), r.status, resp, "v0.1 stub 模式 (待 v0.2 接 gRPC client)".to_string())
        }
        None => {
            // 未知 RPC, 返 NotImplemented
            let resp = serde_json::json!({
                "error": "RPC not registered in mock v0.1",
                "rpc_code": rpc_code,
            });
            (
                "(unknown)".to_string(),
                "(unknown)".to_string(),
                RpcStatus::NotImplemented,
                resp,
                "v0.1 未注册 RPC, 待 v0.2+ 补".to_string(),
            )
        }
    };

    // 记录调用
    gap_matrix.record_call_with_status(rpc_code, status, latency_ms).await;

    let response_body = MockResponse::new(
        rpc_code,
        map_category(&category),
        &rpc_name,
        &rgs_backend,
        &rgs_rpc,
        status,
        request,
        response,
        latency_ms,
        &note,
    );

    HttpResponse::Ok().json(response_body)
}

fn map_category(path_category: &str) -> &'static str {
    match path_category {
        "scene" => "场景/移动",
        "character" => "角色养成",
        "combat" => "战斗 PVE",
        "pvp" => "PVP/竞技",
        "guild" => "公会",
        "economy" => "经济",
        "social" => "社交",
        "activity" => "活动运营",
        "payment" => "付费/商业化",
        "leaderboard" => "排行榜/图鉴",
        "gm" => "GM/运维",
        "misc" => "未分类",
        _ => "未知",
    }
}

fn build_stub_response(
    _rpc_code: u32,
    _category: &str,
    rpc_name: &str,
    _request: &serde_json::Value,
) -> serde_json::Value {
    // v0.1 stub: 返回结构化 placeholder, 文档化 "v0.2+ 接 gRPC 后会返真实 RGS 响应"
    serde_json::json!({
        "mock": true,
        "rpc": rpc_name,
        "placeholder": "RGS backend response (v0.2+ via gRPC client)",
        "v0.2_plan": "Replace with actual tonic::transport::Channel call to RGS 5 域 + card + gm-backend"
    })
}

/// 健康检查
pub async fn handle_health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "rgs-flash-mock",
        "version": env!("CARGO_PKG_VERSION"),
        "mock_v0.1": "12 大类 22 RPC 抽样 stub 模式"
    }))
}

/// 就绪探针 (k8s readiness probe)
pub async fn handle_ready(gap_matrix: web::Data<Arc<GapMatrix>>) -> impl Responder {
    let report = gap_matrix.report().await;
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ready",
        "rpcs_registered": report.total_rpcs
    }))
}

/// GET /coverage — gap matrix 报告
pub async fn handle_coverage(gap_matrix: web::Data<Arc<GapMatrix>>) -> impl Responder {
    let report = gap_matrix.report().await;
    HttpResponse::Ok().json(report)
}

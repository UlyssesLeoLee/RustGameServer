//! reports_handler — 报表 (per ROPE_CS gm_platform/modules/analytics 移植)
//! 2026-09-01 actix-web 重写

use actix_web::{web, HttpResponse};
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportEntry {
    pub id: String,
    pub title: String,
    pub kind: String, // "revenue" / "dau" / "conversion" / "churn"
    pub period: String,
    pub value: f64,
    pub generated_at: String,
}

/// 启动时填充 mock 报表 (5 类 × 14 天)
pub fn seed_reports(state: &AppState) {
    let mut reports = state.reports.lock().unwrap();
    let kinds = ["revenue", "dau", "conversion", "churn"];
    for kind in kinds {
        for d in 0..14 {
            let date = Utc::now() - chrono::Duration::days(d);
            let value: f64 = match kind {
                "revenue" => 10000.0 + (d as f64 * 200.0) + rand::thread_rng().gen_range(-500.0..500.0),
                "dau" => 5000.0 + rand::thread_rng().gen_range(-300.0..300.0),
                "conversion" => 0.05 + rand::thread_rng().gen_range(-0.01..0.01),
                "churn" => 0.02 + rand::thread_rng().gen_range(-0.005..0.005),
                _ => 0.0,
            };
            reports.push(ReportEntry {
                id: format!("{}-{}", kind, date.format("%Y%m%d")),
                title: format!("{} report", kind),
                kind: kind.to_string(),
                period: date.format("%Y-%m-%d").to_string(),
                value: (value * 100.0).round() / 100.0,
                generated_at: date.to_rfc3339(),
            });
        }
    }
}

pub async fn list_reports(state: web::Data<AppState>) -> HttpResponse {
    let reports = state.reports.lock().unwrap();
    HttpResponse::Ok().json(json!({"reports": reports.clone()}))
}

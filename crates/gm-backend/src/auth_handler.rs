//! auth_handler — 登录 + admin 管理 (per ROPE_CS gm_platform/modules/account 移植)
//! 2026-09-01 actix-web 重写 + 内存版 admin store (生产应接 admin_db.gm_users)

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{extract_claims, issue_jwt, AppState, Claims};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminRecord {
    pub username: String,
    pub password_hash: String,
    pub role: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

// ============================================================================
// POST /gm/login
// ============================================================================

pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    let admins = state.admins.lock().unwrap();
    let admin = admins.iter().find(|a| a.username == body.username);
    match admin {
        Some(a) => {
            let verified = bcrypt::verify(&body.password, &a.password_hash).unwrap_or(false);
            if !verified {
                return HttpResponse::Unauthorized().json(json!({"error": "invalid_credentials"}));
            }
            let roles = if a.role == "superadmin" {
                vec!["GM_READ".to_string(), "GM_WRITE".to_string(), "GM_ADMIN".to_string()]
            } else {
                vec!["GM_READ".to_string(), "GM_WRITE".to_string()]
            };
            let token = match issue_jwt(&state.config.jwt_secret, &a.username, roles.clone(), 3600) {
                Ok(t) => t,
                Err(e) => return HttpResponse::InternalServerError().json(json!({"error": "jwt_issue_failed", "detail": e.to_string()})),
            };
            HttpResponse::Ok().json(LoginResponse { token })
        }
        None => HttpResponse::Unauthorized().json(json!({"error": "invalid_credentials"})),
    }
}

// ============================================================================
// POST /gm/admins — superadmin only
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAdminRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub role: Option<String>,
}

pub async fn create_admin(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CreateAdminRequest>,
) -> HttpResponse {
    if !is_superadmin(&req) {
        return HttpResponse::Forbidden().json(json!({"error": "forbidden"}));
    }
    if body.username.trim().is_empty() || body.password.len() < 6 {
        return HttpResponse::BadRequest().json(json!({"error": "missing_or_invalid_fields"}));
    }
    let mut admins = state.admins.lock().unwrap();
    if admins.iter().any(|a| a.username == body.username) {
        return HttpResponse::Conflict().json(json!({"error": "username_exists"}));
    }
    let hashed = bcrypt::hash(&body.password, 12).unwrap_or_default();
    admins.push(AdminRecord {
        username: body.username.clone(),
        password_hash: hashed,
        role: body.role.clone().unwrap_or_else(|| "admin".to_string()),
    });
    HttpResponse::Ok().json(json!({"status": "created", "username": body.username}))
}

pub async fn list_admins(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    if !is_superadmin(&req) {
        return HttpResponse::Forbidden().json(json!({"error": "forbidden"}));
    }
    let admins = state.admins.lock().unwrap();
    let out: Vec<_> = admins.iter().map(|a| json!({
        "id": a.username,
        "username": a.username,
        "role": a.role,
    })).collect();
    HttpResponse::Ok().json(json!({"admins": out}))
}

fn is_superadmin(req: &HttpRequest) -> bool {
    extract_claims(req)
        .map(|c: Claims| c.roles.iter().any(|r| r == "GM_ADMIN"))
        .unwrap_or(false)
}

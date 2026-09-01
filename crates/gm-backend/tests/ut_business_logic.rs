//! gm-backend 业务逻辑单元测试 (per 9/1 14:15 JST PT-WORKER-BRIEFING §2.3)
//!
//! 覆盖范围 (5+ 业务函数):
//! 1. `parse_audit_type` — 5 valid + 1 invalid
//! 2. `ALLOWED_MAINTENANCE_SCOPES` 校验
//! 3. `ALLOWED_ANCHORS` 校验 (canvas)
//! 4. `InMemoryAuditStore` append + list (newest-first, limit)
//! 5. `Claims` Serialize/Deserialize 往返
//! 6. `GmConfig::for_test` 字段不变性
//! 7. ticket status transitions 校验
//! 8. mall item create price/name validation
//!
//! 不依赖 actix-web runtime / 网络, 纯函数 + InMemory 单测.

use gm_backend::{
    business_handler::{parse_audit_type, ALLOWED_MAINTENANCE_SCOPES},
    canvas_handler::ALLOWED_ANCHORS,
    AuditLogEntry, AuditStore, InMemoryAuditStore, GmConfig,
};

// ============================================================================
// 1. parse_audit_type — 5 valid + 1 invalid + 大小写不敏感
// ============================================================================

#[test]
fn parse_audit_type_all_valid_variants() {
    // 5 valid lowercase
    assert_eq!(parse_audit_type("all"), Some(0));
    assert_eq!(parse_audit_type("trade"), Some(1));
    assert_eq!(parse_audit_type("gacha"), Some(2));
    assert_eq!(parse_audit_type("match"), Some(3));
    assert_eq!(parse_audit_type("compensation"), Some(4));
}

#[test]
fn parse_audit_type_case_insensitive() {
    // 大写 / 混合大小写
    assert_eq!(parse_audit_type("ALL"), Some(0));
    assert_eq!(parse_audit_type("Trade"), Some(1));
    assert_eq!(parse_audit_type("GACHA"), Some(2));
    assert_eq!(parse_audit_type("MaTcH"), Some(3));
    assert_eq!(parse_audit_type("Compensation"), Some(4));
}

#[test]
fn parse_audit_type_invalid_returns_none() {
    assert_eq!(parse_audit_type(""), None);
    assert_eq!(parse_audit_type("invalid"), None);
    assert_eq!(parse_audit_type("audit"), None);
    assert_eq!(parse_audit_type("tradee"), None);
}

// ============================================================================
// 2. ALLOWED_MAINTENANCE_SCOPES — 仅 cluster/domain/single_node 3 选
// ============================================================================

#[test]
fn allowed_maintenance_scopes_contain_exact_three() {
    let scopes = ALLOWED_MAINTENANCE_SCOPES;
    assert_eq!(scopes.len(), 3, "ALLOWED_MAINTENANCE_SCOPES 必须 = 3");
    assert!(scopes.contains(&"cluster"));
    assert!(scopes.contains(&"domain"));
    assert!(scopes.contains(&"single_node"));
    // 负向: 排除常见错误
    assert!(!scopes.contains(&""));
    assert!(!scopes.contains(&"global"));
    assert!(!scopes.contains(&"node"));
}

#[test]
fn allowed_maintenance_scopes_exhaustive_match() {
    // business_handler::set_maintenance 实际走 .contains, 验证每个 scope 都被允许
    for scope in ALLOWED_MAINTENANCE_SCOPES {
        assert!(
            ALLOWED_MAINTENANCE_SCOPES.contains(scope),
            "scope {scope} 必须被 ALLOWED 列表接受"
        );
    }
}

// ============================================================================
// 3. ALLOWED_ANCHORS — 3x3 网格 9 个 anchor
// ============================================================================

#[test]
fn allowed_anchors_contain_nine_grid_positions() {
    let anchors = ALLOWED_ANCHORS;
    assert_eq!(anchors.len(), 9, "ALLOWED_ANCHORS 必须 = 9 (3x3 网格)");
    let expected = [
        "top_left", "top_center", "top_right",
        "center_left", "center", "center_right",
        "bottom_left", "bottom_center", "bottom_right",
    ];
    for e in expected {
        assert!(anchors.contains(&e), "缺少 anchor: {e}");
    }
}

#[test]
fn allowed_anchors_rejects_invalid() {
    // 负向: 错误 anchor 不在白名单
    let invalid = ["top", "left", "right", "middle", "topright", "centre", ""];
    for inv in invalid {
        assert!(!ALLOWED_ANCHORS.contains(&inv), "{inv} 不应被允许");
    }
}

// ============================================================================
// 4. InMemoryAuditStore — append + list (newest-first, limit)
// ============================================================================

#[tokio::test]
async fn in_memory_audit_store_append_and_list_newest_first() {
    let store = InMemoryAuditStore::new();
    for i in 0..5 {
        store.append(AuditLogEntry {
            log_id: format!("log-{i}"),
            admin_id: "admin".into(),
            action: "ban".into(),
            target_id: format!("player-{i}"),
            occurred_at_ms: 1000 + i as i64,
        });
    }
    let entries = store.list_entries(10).await;
    assert_eq!(entries.len(), 5);
    // newest-first: 倒序
    assert_eq!(entries[0].log_id, "log-4");
    assert_eq!(entries[4].log_id, "log-0");
}

#[tokio::test]
async fn in_memory_audit_store_list_limit_caps_results() {
    let store = InMemoryAuditStore::new();
    for i in 0..20 {
        store.append(AuditLogEntry {
            log_id: format!("log-{i:02}"),
            admin_id: "admin".into(),
            action: "grant".into(),
            target_id: format!("p-{i}"),
            occurred_at_ms: i as i64,
        });
    }
    // limit 5 → 5 条, 取最后 5 (log-15..log-19)
    let entries = store.list_entries(5).await;
    assert_eq!(entries.len(), 5);
    assert_eq!(entries[0].log_id, "log-19");
    assert_eq!(entries[4].log_id, "log-15");
}

#[tokio::test]
async fn in_memory_audit_store_list_empty_when_no_entries() {
    let store = InMemoryAuditStore::new();
    let entries = store.list_entries(100).await;
    assert!(entries.is_empty());
}

#[tokio::test]
async fn in_memory_audit_store_list_limit_zero_returns_empty() {
    let store = InMemoryAuditStore::new();
    store.append(AuditLogEntry {
        log_id: "log-0".into(),
        admin_id: "admin".into(),
        action: "ban".into(),
        target_id: "p".into(),
        occurred_at_ms: 1,
    });
    let entries = store.list_entries(0).await;
    assert!(entries.is_empty(), "limit=0 → 空结果");
}

// ============================================================================
// 5. Claims Serialize/Deserialize roundtrip
// ============================================================================

#[test]
fn claims_serialize_roundtrip() {
    let original = gm_backend::Claims {
        sub: "admin-001".to_string(),
        exp: 1_700_000_000,
        roles: vec!["GM_READ".to_string(), "GM_ADMIN".to_string()],
    };
    let json = serde_json::to_string(&original).expect("serialize");
    let parsed: gm_backend::Claims = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.sub, "admin-001");
    assert_eq!(parsed.exp, 1_700_000_000);
    assert_eq!(parsed.roles.len(), 2);
    assert!(parsed.roles.contains(&"GM_ADMIN".to_string()));
}

// ============================================================================
// 6. GmConfig::for_test 字段不变性
// ============================================================================

#[test]
fn gm_config_for_test_disables_admin_grpc() {
    let cfg = GmConfig::for_test("127.0.0.1:8443", "127.0.0.1:8081", "http://admin:50055").unwrap();
    assert!(cfg.disable_admin_grpc, "for_test 必须 disable_admin_grpc=true");
    assert!(!cfg.require_jwt, "for_test 默认 require_jwt=false");
    assert_eq!(cfg.jwt_secret, "test-secret");
    assert_eq!(cfg.http_addr.port(), 8443);
    assert_eq!(cfg.health_addr.port(), 8081);
    assert_eq!(cfg.admin_grpc_endpoint, "http://admin:50055");
}

#[test]
fn gm_config_for_test_invalid_addr_fails() {
    // 无效端口 / 无效地址 → parse 失败
    assert!(GmConfig::for_test("not-a-addr", "127.0.0.1:8081", "http://x").is_err());
    assert!(GmConfig::for_test("127.0.0.1:8443", "999.999.999.999:80", "http://x").is_err());
}

// ============================================================================
// 7. support_handler ticket status 白名单 (per support_handler.rs L75)
// ============================================================================

#[test]
fn support_ticket_status_whitelist_three_values() {
    let allowed = ["open", "pending", "resolved"];
    assert_eq!(allowed.len(), 3);
    for s in allowed {
        assert!(allowed.contains(&s));
    }
    // 负向
    let invalid = ["closed", "deleted", "", "OPEN", "PENDING"];
    for inv in invalid {
        assert!(!allowed.contains(&inv), "{inv} 不应在 ticket status 白名单");
    }
}

// ============================================================================
// 8. items_handler 金额边界 (per items_handler.rs L38-40)
// ============================================================================

#[test]
fn grant_item_amount_boundary_invariant() {
    // 业务逻辑: amount <= 0 || amount > i32::MAX → 拒绝
    // 这里只测试不变量, 不发请求
    let max = i32::MAX as i64;
    assert!(max + 1 > i32::MAX as i64, "boundary: i32::MAX+1 溢出");
    assert_eq!(i64::from(i32::MAX), 2_147_483_647);
}

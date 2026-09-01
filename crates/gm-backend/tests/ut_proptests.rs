//! gm-backend proptest invariants (per 9/1 14:15 JST PT-WORKER-BRIEFING §2.3)
//!
//! proptest invariant 列表 (3+):
//! 1. `parse_audit_type` 仅 5 个 magic string 接受, 其他 None
//! 2. `InMemoryAuditStore` append N 条, list_entries 逆序 (新→旧) 返回所有
//! 3. `ALLOWED_ANCHORS` 9 元素不变性
//!
//! 不依赖 actix-web / 网络, 纯函数 + InMemory.
//! proptest 是同步框架, 对 async list_entries 用 tokio runtime 包一层.

use gm_backend::{
    business_handler::parse_audit_type,
    canvas_handler::ALLOWED_ANCHORS,
    AuditLogEntry, AuditStore, InMemoryAuditStore,
};
use proptest::prelude::*;

// ============================================================================
// proptest 1: parse_audit_type — 任意 ascii_lowercase 字符串, 输出 ∈ {Some(0..=4), None}
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn parse_audit_type_output_is_known_variant_or_none(s in "[a-z]{0,32}") {
        let result = parse_audit_type(&s);
        match result {
            Some(v) => {
                prop_assert!(v >= 0 && v <= 4, "valid audit type must be 0..=4, got {v}");
                // 5 个有效 string → 5 个 unique i32
                let canonical = match s.as_str() {
                    "all" => 0,
                    "trade" => 1,
                    "gacha" => 2,
                    "match" => 3,
                    "compensation" => 4,
                    _ => panic!("Some({v}) 但 {s} 不在白名单"),
                };
                prop_assert_eq!(v, canonical);
            }
            None => {
                // 5 个 magic string 之外的输入都应 None
                let magic = ["all", "trade", "gacha", "match", "compensation"];
                prop_assert!(!magic.contains(&s.as_str()), "magic string 不应返 None");
            }
        }
    }
}

// ============================================================================
// proptest 2: InMemoryAuditStore — append N 条, list 返回逆序 (新→旧) 所有元素
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn audit_store_list_returns_subset_of_appended(
        entries in proptest::collection::vec(
            (any::<u32>(), any::<u32>(), any::<u32>()),
            0..30,
        ),
    ) {
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let store = InMemoryAuditStore::new();
        let mut expected_ids: Vec<String> = Vec::with_capacity(entries.len());
        for (i, (admin_seed, action_seed, target_seed)) in entries.iter().enumerate() {
            let id = format!("log-{i}");
            store.append(AuditLogEntry {
                log_id: id.clone(),
                admin_id: format!("admin-{admin_seed}"),
                action: format!("action-{action_seed}"),
                target_id: format!("target-{target_seed}"),
                occurred_at_ms: i as i64,
            });
            expected_ids.push(id);
        }
        let result = rt.block_on(store.list_entries(1000));
        // invariant: result 长度 = expected 长度
        prop_assert_eq!(result.len(), expected_ids.len());
        // invariant: 逆序 (新→旧)
        for (i, entry) in result.iter().enumerate() {
            let expected = &expected_ids[expected_ids.len() - 1 - i];
            prop_assert_eq!(&entry.log_id, expected);
        }
    }

    #[test]
    fn audit_store_list_limit_caps_count(
        n in 0usize..50,
        limit in 0usize..50,
    ) {
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let store = InMemoryAuditStore::new();
        for i in 0..n {
            store.append(AuditLogEntry {
                log_id: format!("log-{i}"),
                admin_id: "a".into(),
                action: "x".into(),
                target_id: "t".into(),
                occurred_at_ms: i as i64,
            });
        }
        let result = rt.block_on(store.list_entries(limit));
        let expected_len = if limit < n { limit } else { n };
        prop_assert_eq!(result.len(), expected_len);
    }
}

// ============================================================================
// proptest 3: ALLOWED_ANCHORS — 固定 9 元素, 无空字符串
// ============================================================================

proptest! {
    #[test]
    fn allowed_anchors_invariant_stable(_unused in 0u32..100) {
        prop_assert_eq!(ALLOWED_ANCHORS.len(), 9);
        for a in ALLOWED_ANCHORS {
            prop_assert!(!a.is_empty());
            prop_assert!(!a.contains(' '), "anchor {a} 不应含空格");
        }
    }
}

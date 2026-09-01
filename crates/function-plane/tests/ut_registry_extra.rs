//! Additional Registry unit tests (per DTL-038 §4.3 FR-001 unit-level coverage).
//!
//! Extends the existing `tests/ut.rs` with edge cases on the InMemoryRegistry
//! business logic — empty id / empty version / contract invariants, key
//! uniqueness, list order, status transitions, list_active semantics.

use function_plane::{
    FunctionMetadata, FunctionPlaneError, FunctionRegistry, FunctionStatus, InMemoryRegistry,
    Runtime, TriggerType,
};

fn meta(id: &str, version: &str, status: FunctionStatus) -> FunctionMetadata {
    let mut m = FunctionMetadata::new(id, version, Runtime::Wasm, TriggerType::Grpc, None);
    m.status = status;
    m
}

#[tokio::test]
async fn ut_registry_register_rejects_empty_function_id() {
    let reg = InMemoryRegistry::new();
    let err = reg
        .register(meta("", "v0.1.0", FunctionStatus::Active))
        .await
        .expect_err("empty id must error");
    assert!(
        matches!(err, FunctionPlaneError::ContractInvalid(_)),
        "expected ContractInvalid, got {err:?}"
    );
    assert_eq!(reg.len().await, 0, "no row should be inserted");
}

#[tokio::test]
async fn ut_registry_register_rejects_empty_version() {
    let reg = InMemoryRegistry::new();
    let err = reg
        .register(meta("fn.x", "", FunctionStatus::Active))
        .await
        .expect_err("empty version must error");
    assert!(matches!(err, FunctionPlaneError::ContractInvalid(_)));
    assert_eq!(reg.len().await, 0);
}

#[tokio::test]
async fn ut_registry_register_wasm_without_bytes_is_rejected() {
    let reg = InMemoryRegistry::new();
    // Wasm runtime + no bytes → ContractInvalid per §15.3
    let err = reg
        .register(meta("fn.bytes", "v0.1.0", FunctionStatus::Active))
        .await
        .expect_err("Wasm without bytes must error");
    match err {
        FunctionPlaneError::ContractInvalid(msg) => {
            assert!(
                msg.contains("wasm_bytes"),
                "msg should mention wasm_bytes, got: {msg}"
            );
        }
        other => panic!("expected ContractInvalid, got {other:?}"),
    }
}

#[tokio::test]
async fn ut_registry_overwrite_same_key_replaces_in_place() {
    let reg = InMemoryRegistry::new();
    reg.register(meta("fn.ow", "v0.1.0", FunctionStatus::Active))
        .await
        .unwrap();
    reg.register(meta("fn.ow", "v0.1.0", FunctionStatus::Paused))
        .await
        .unwrap();
    assert_eq!(reg.len().await, 1, "same key must collapse to one row");
    let got = reg.get("fn.ow", Some("v0.1.0")).await.unwrap();
    assert_eq!(got.status, FunctionStatus::Paused, "latest write wins");
}

#[tokio::test]
async fn ut_registry_list_active_skips_draft_and_archived() {
    let reg = InMemoryRegistry::new();
    for (v, s) in [
        ("v0.1.0", FunctionStatus::Draft),
        ("v0.2.0", FunctionStatus::Active),
        ("v0.3.0", FunctionStatus::Paused),
        ("v0.4.0", FunctionStatus::Archived),
        ("v0.5.0", FunctionStatus::Active),
    ] {
        reg.register(meta("fn.mix", v, s)).await.unwrap();
    }
    let active = reg.list_active().await.unwrap();
    assert_eq!(active.len(), 2, "only the two Active rows");
    let versions: Vec<&str> = active.iter().map(|m| m.version.as_str()).collect();
    assert_eq!(versions, vec!["v0.5.0", "v0.2.0"], "descending by version");
}

#[tokio::test]
async fn ut_registry_list_versions_returns_descending_semver() {
    let reg = InMemoryRegistry::new();
    for v in ["v0.1.0", "v0.10.0", "v0.2.0", "v0.20.0"] {
        reg.register(meta("fn.sort", v, FunctionStatus::Active))
            .await
            .unwrap();
    }
    let rows = reg.list_versions("fn.sort").await.unwrap();
    let versions: Vec<&str> = rows.iter().map(|m| m.version.as_str()).collect();
    assert_eq!(
        versions,
        vec!["v0.20.0", "v0.10.0", "v0.2.0", "v0.1.0"],
        "SemVer tuple-sort descending, not string-sort"
    );
}

#[tokio::test]
async fn ut_registry_set_status_unknown_version_is_versionnotfound() {
    let reg = InMemoryRegistry::new();
    reg.register(meta("fn.s", "v0.1.0", FunctionStatus::Active))
        .await
        .unwrap();
    let err = reg
        .set_status("fn.s", "v9.9.9", FunctionStatus::Paused)
        .await
        .expect_err("unknown version must error");
    match err {
        FunctionPlaneError::VersionNotFound {
            function_id,
            version,
        } => {
            assert_eq!(function_id, "fn.s");
            assert_eq!(version, "v9.9.9");
        }
        other => panic!("expected VersionNotFound, got {other:?}"),
    }
}

#[tokio::test]
async fn ut_registry_set_status_updates_updated_at_field() {
    let reg = InMemoryRegistry::new();
    reg.register(meta("fn.ts", "v0.1.0", FunctionStatus::Active))
        .await
        .unwrap();
    let before = reg.get("fn.ts", Some("v0.1.0")).await.unwrap();
    // Sleep 2ms to guarantee the chrono::Utc::now() ticks.
    tokio::time::sleep(std::time::Duration::from_millis(2)).await;
    reg.set_status("fn.ts", "v0.1.0", FunctionStatus::Paused)
        .await
        .unwrap();
    let after = reg.get("fn.ts", Some("v0.1.0")).await.unwrap();
    assert_eq!(after.status, FunctionStatus::Paused);
    assert!(
        after.updated_at > before.updated_at,
        "updated_at must advance on status change (before={}, after={})",
        before.updated_at,
        after.updated_at
    );
}

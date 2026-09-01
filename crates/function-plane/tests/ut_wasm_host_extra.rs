//! Additional WasmHost unit tests (per DTL-038 §4.3 FR-001).
//!
//! Extends the existing `tests/ut.rs` with engine / module-cache business
//! logic — register_module happy path, re-register replaces cache entry,
//! non-Wasm runtime is rejected, sync invoke contract (compute export) is
//! required.

use function_plane::{FunctionMetadata, FunctionPlaneError, Runtime, TriggerType, WasmHost};
use serde_json::json;

const WAT_SUM: &str = r#"
    (module
      (func (export "compute") (param i32 i32) (result i32)
        local.get 0
        local.get 1
        i32.add
      )
    )
"#;

const WAT_NO_COMPUTE: &str = r#"
    (module
      (func (export "no_compute") (result i32) i32.const 0)
    )
"#;

fn bytes() -> Vec<u8> {
    wat::parse_str(WAT_SUM).expect("wat must parse")
}

fn meta(id: &str, runtime: Runtime) -> FunctionMetadata {
    let mut m = FunctionMetadata::new(id, "v0.1.0", runtime, TriggerType::Grpc, None);
    m.status = function_plane::FunctionStatus::Active;
    m.fuel = 10_000_000;
    m.memory_mib = 64;
    m
}

#[tokio::test]
async fn ut_wasm_register_non_wasm_runtime_is_contractinvalid() {
    let host = WasmHost::new().expect("host");
    let m = meta("fn.c", Runtime::Container);
    let err = host
        .register_module(&m)
        .await
        .expect_err("Container runtime must be rejected");
    assert!(
        matches!(err, FunctionPlaneError::ContractInvalid(_)),
        "expected ContractInvalid, got {err:?}"
    );
    assert_eq!(host.module_count().await, 0, "no module cached on error");
}

#[tokio::test]
async fn ut_wasm_register_wasm_without_bytes_is_contractinvalid() {
    let host = WasmHost::new().expect("host");
    let m = FunctionMetadata::new(
        "fn.b",
        "v0.1.0",
        Runtime::Wasm,
        TriggerType::Grpc,
        None, // no bytes
    );
    let err = host
        .register_module(&m)
        .await
        .expect_err("Wasm without bytes must error");
    assert!(matches!(err, FunctionPlaneError::ContractInvalid(_)));
    assert_eq!(host.module_count().await, 0);
}

#[tokio::test]
async fn ut_wasm_reregister_replaces_cached_module() {
    let host = WasmHost::new().expect("host");
    let mut m1 = meta("fn.r", Runtime::Wasm);
    m1.wasm_bytes = Some(bytes());
    host.register_module(&m1).await.expect("first register");
    assert_eq!(host.module_count().await, 1);

    // Same key — re-register must replace, not append.
    let mut m2 = m1.clone();
    m2.wasm_bytes = Some(bytes());
    host.register_module(&m2).await.expect("re-register");
    assert_eq!(
        host.module_count().await,
        1,
        "re-register of same key must not duplicate the cache entry"
    );
}

#[tokio::test]
async fn ut_wasm_invoke_sync_missing_compute_export_returns_contractinvalid() {
    // Module has no "compute" export — invoke must fail with ContractInvalid
    // (§9.5 export contract violation, not a trap).
    let host = WasmHost::new().expect("host");
    let mut m = meta("fn.nc", Runtime::Wasm);
    m.wasm_bytes = Some(wat::parse_str(WAT_NO_COMPUTE).expect("wat"));
    host.register_module(&m).await.expect("register");

    let host2 = host.clone();
    let m2 = m.clone();
    let res = tokio::task::spawn_blocking(move || host2.invoke_sync(&m2, &json!({"a": 0, "b": 0})))
        .await
        .expect("join");
    let err = res.expect_err("missing compute export must error");
    assert!(
        matches!(err, FunctionPlaneError::ContractInvalid(_)),
        "expected ContractInvalid, got {err:?}"
    );
}

#[tokio::test]
async fn ut_wasm_invoke_sync_missing_module_returns_not_found() {
    // The module is registered through one host, then we attempt to invoke
    // through a *different* host whose cache is empty. The mock surfaces
    // this as NotFound, not WasmInstantiate (cache lookup happens first).
    let host_a = WasmHost::new().expect("host a");
    let host_b = WasmHost::new().expect("host b");
    let mut m = meta("fn.miss", Runtime::Wasm);
    m.wasm_bytes = Some(bytes());
    host_a.register_module(&m).await.expect("register on a");

    let host = host_b.clone();
    let m2 = m.clone();
    let res = tokio::task::spawn_blocking(move || host.invoke_sync(&m2, &json!({"a": 1, "b": 2})))
        .await
        .expect("join");
    let err = res.expect_err("uncached module must error");
    assert!(
        matches!(err, FunctionPlaneError::NotFound(_)),
        "expected NotFound, got {err:?}"
    );
}

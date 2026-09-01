//! Integration tests for the Function Gateway (per DTL-038 §4.3 IT).
//!
//! Cross-module flows exercising the full plane: register → host compile →
//! invoke → status transitions, including multi-version routing and the
//! "Container runtime unsupported" guard. These are integration-level
//! because they drive [`FunctionGateway`] end-to-end, composing
//! [`FunctionRegistry`] (InMemory) + [`WasmHost`] + spawn_blocking bridge.

use function_plane::{
    FunctionContext, FunctionGateway, FunctionMetadata, FunctionPlaneError, FunctionRegistry,
    FunctionStatus, InMemoryRegistry, InvocationRequest, Runtime, TriggerType, WasmHost,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

/// `compute(a, b) -> a + b`
const WAT_SUM: &str = r#"
    (module
      (func (export "compute") (param i32 i32) (result i32)
        local.get 0
        local.get 1
        i32.add
      )
    )
"#;

/// `compute(a, b) -> a * b`
const WAT_MUL: &str = r#"
    (module
      (func (export "compute") (param i32 i32) (result i32)
        local.get 0
        local.get 1
        i32.mul
      )
    )
"#;

fn bytes(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).expect("wat must parse")
}

fn meta_with(
    id: &str,
    version: &str,
    status: FunctionStatus,
    wasm: Vec<u8>,
    fuel: u64,
) -> FunctionMetadata {
    let mut m = FunctionMetadata::new(id, version, Runtime::Wasm, TriggerType::Grpc, Some(wasm));
    m.status = status;
    m.fuel = fuel;
    m.memory_mib = 32;
    m.timeout_ms = 5_000;
    m
}

// =============================================================================
// IT 1 — register → host compile → invoke, end-to-end success
// =============================================================================

#[tokio::test]
async fn it_gateway_register_compile_invoke_full_lifecycle() {
    let gw = FunctionGateway::with_in_memory().expect("gateway");
    let meta = meta_with(
        "fn.calc",
        "v0.1.0",
        FunctionStatus::Active,
        bytes(WAT_SUM),
        1_000_000,
    );
    gw.register(meta).await.expect("register");

    let req = InvocationRequest {
        function_id: "fn.calc".into(),
        version: Some("v0.1.0".into()),
        input: json!({"a": 11, "b": 31}),
        context: FunctionContext::new(),
        extra: Default::default(),
    };
    let res = gw.invoke(req).await.expect("invoke");
    assert!(res.success, "happy-path invoke must succeed");
    assert_eq!(res.function_id, "fn.calc");
    assert_eq!(res.version, "v0.1.0");
    assert_eq!(res.output, Some(json!({"result": 42})));
    assert!(
        res.fuel_consumed.is_some(),
        "fuel_consumed must be reported for the mock"
    );
    assert!(res.duration_ms < 5_000, "happy path stays under timeout");
}

// =============================================================================
// IT 2 — version routing: same function_id, two Active versions, both compiled
// =============================================================================

#[tokio::test]
async fn it_gateway_two_versions_route_by_explicit_selector() {
    // Compiles two Active versions side-by-side; verifies that the Gateway
    // compiles and caches both, and that version selection routes the call
    // to the correct module (sum vs. mul).
    let gw = FunctionGateway::with_in_memory().expect("gateway");
    let m1 = meta_with(
        "fn.arith",
        "v0.1.0",
        FunctionStatus::Active,
        bytes(WAT_SUM),
        1_000_000,
    );
    let m2 = meta_with(
        "fn.arith",
        "v0.2.0",
        FunctionStatus::Active,
        bytes(WAT_MUL),
        1_000_000,
    );
    gw.register(m1).await.expect("register v0.1.0");
    gw.register(m2).await.expect("register v0.2.0");

    // Host module count must reflect both compiled Active versions.
    let active_count = gw.registry().list_active().await.unwrap().len();
    assert_eq!(active_count, 2, "both Active versions should be listed");

    let req_sum = InvocationRequest {
        function_id: "fn.arith".into(),
        version: Some("v0.1.0".into()),
        input: json!({"a": 3, "b": 4}),
        context: FunctionContext::new(),
        extra: Default::default(),
    };
    let res_sum = gw.invoke(req_sum).await.expect("invoke sum");
    assert_eq!(res_sum.output, Some(json!({"result": 7})));

    let req_mul = InvocationRequest {
        function_id: "fn.arith".into(),
        version: Some("v0.2.0".into()),
        input: json!({"a": 3, "b": 4}),
        context: FunctionContext::new(),
        extra: Default::default(),
    };
    let res_mul = gw.invoke(req_mul).await.expect("invoke mul");
    assert_eq!(res_mul.output, Some(json!({"result": 12})));
}

// =============================================================================
// IT 3 — Draft registration does NOT compile; subsequent set_status→Active
//         still requires a host compile, which must happen on the second register.
// =============================================================================

#[tokio::test]
async fn it_gateway_draft_skips_compile_then_active_triggers_compile() {
    let gw = FunctionGateway::with_in_memory().expect("gateway");
    // 1) Register as Draft — host MUST NOT compile (no host module cached).
    let draft = meta_with(
        "fn.d2a",
        "v0.1.0",
        FunctionStatus::Draft,
        bytes(WAT_SUM),
        1_000_000,
    );
    gw.register(draft).await.expect("register draft");
    let active = gw.registry().list_active().await.unwrap();
    assert!(active.is_empty(), "Draft must not be Active-listed");

    // Re-register the same key with Active — host MUST compile this time.
    let active_meta = meta_with(
        "fn.d2a",
        "v0.1.0",
        FunctionStatus::Active,
        bytes(WAT_SUM),
        1_000_000,
    );
    gw.register(active_meta).await.expect("re-register active");
    let req = InvocationRequest {
        function_id: "fn.d2a".into(),
        version: Some("v0.1.0".into()),
        input: json!({"a": 2, "b": 3}),
        context: FunctionContext::new(),
        extra: Default::default(),
    };
    let res = gw.invoke(req).await.expect("invoke after re-register");
    assert_eq!(res.output, Some(json!({"result": 5})));
}

// =============================================================================
// IT 4 — Container runtime is unsupported by the mock Gateway (per §8.4
//         "Phase 1+ work: container backend"). Invoking must surface
//         ContractInvalid, not panic, not silently succeed.
// =============================================================================

#[tokio::test]
async fn it_gateway_container_runtime_invocation_is_contractinvalid() {
    // The registry accepts Container (no bytes required), but the Gateway
    // refuses to dispatch to a non-Wasm runtime.
    let reg: Arc<dyn FunctionRegistry> = Arc::new(InMemoryRegistry::new());
    let host = Arc::new(WasmHost::new().unwrap());
    let gw = FunctionGateway::new(reg.clone(), host);

    let m = FunctionMetadata::new(
        "fn.box",
        "v0.1.0",
        Runtime::Container,
        TriggerType::Grpc,
        None, // Container has no bytes
    );
    // Manually push to the registry — Gateway::register would otherwise
    // short-circuit on the `needs_compile` check (only Wasm Active triggers
    // a host compile; Container falls through to plain registry.register).
    reg.register(m).await.expect("registry write");

    let req = InvocationRequest {
        function_id: "fn.box".into(),
        version: Some("v0.1.0".into()),
        input: json!({}),
        context: FunctionContext::new(),
        extra: Default::default(),
    };
    let err = gw.invoke(req).await.expect_err("Container runtime must error");
    match err {
        FunctionPlaneError::ContractInvalid(msg) => {
            assert!(
                msg.contains("Container") || msg.contains("not supported"),
                "msg should mention Container / not supported, got: {msg}"
            );
        }
        other => panic!("expected ContractInvalid, got {other:?}"),
    }
}

// =============================================================================
// IT 5 — context fields flow into the call path without affecting
//         determinism of the result (per §3.3 observability business fields).
// =============================================================================

#[tokio::test]
async fn it_gateway_full_context_fields_do_not_affect_compute_result() {
    let gw = FunctionGateway::with_in_memory().expect("gateway");
    let m = meta_with(
        "fn.obs",
        "v0.1.0",
        FunctionStatus::Active,
        bytes(WAT_SUM),
        1_000_000,
    );
    gw.register(m).await.expect("register");

    let ctx = FunctionContext::new()
        .with_trace_id("0af7651916cd43dd8448eb211c80319c")
        .with_correlation_id(Uuid::new_v4())
        .with_saga_id(Uuid::new_v4())
        .with_event_id(Uuid::new_v4())
        .with_user_id(Uuid::new_v4())
        .with_tenant_id("tenant-acme")
        .with_retry_count(3)
        .with_idempotency_key("idem-2026-09-01-0001");
    let req = InvocationRequest {
        function_id: "fn.obs".into(),
        version: Some("v0.1.0".into()),
        input: json!({"a": 10, "b": 20}),
        context: ctx,
        extra: Default::default(),
    };
    let res = gw.invoke(req).await.expect("invoke");
    assert!(res.success);
    assert_eq!(res.output, Some(json!({"result": 30})));
}

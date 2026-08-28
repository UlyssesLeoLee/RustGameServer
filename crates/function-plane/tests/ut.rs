//! Unit & integration tests for `function-plane` (per RGS-INC-001 v0.2 §8/§9/§15).
//!
//! 22 tests, distributed:
//! - Registry: 4
//! - WasmHost: 5 + 2 host-import = 7
//! - Gateway: 6
//! - Contract: 3
//! - Error:    2
//!
//! WAT text is compiled to WASM at test time via the `wat` crate so each test
//! is self-contained.

use function_plane::{
    FunctionContext, FunctionGateway, FunctionMetadata, FunctionPlaneError, FunctionRegistry,
    FunctionStatus, InMemoryRegistry, InvocationRequest, Runtime, TriggerType, WasmHost,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

// =============================================================================
// WAT templates
// =============================================================================

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

/// `compute(a, b) -> a / b` — traps on b == 0.
const WAT_DIV: &str = r#"
    (module
      (func (export "compute") (param i32 i32) (result i32)
        local.get 0
        local.get 1
        i32.div_s
      )
    )
"#;

/// Has 1 page of initial memory + a `compute` that tries to grow memory by
/// 10000 pages (~640 MiB). On `memory.grow` failure the module traps via
/// `unreachable`, which Wasmtime reports as a wasm trap. This is the
/// panic-free way to exercise the memory limiter (returning `Err` from the
/// limiter would cross an FFI boundary and abort the process).
const WAT_GROW_MEMORY: &str = r#"
    (module
      (memory (export "mem") 1)
      (func (export "compute") (param i32 i32) (result i32)
        i32.const 10000
        memory.grow
        i32.const -1
        i32.eq
        if
          unreachable
        end
        i32.const 0
      )
    )
"#;

/// WAT that does `i32.const 42` (1 instruction, minimal fuel).
const WAT_CONST: &str = r#"
    (module
      (func (export "compute") (param i32 i32) (result i32)
        i32.const 42
      )
    )
"#;

fn wat_to_bytes(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).expect("wat must parse")
}

fn new_meta(
    id: &str,
    version: &str,
    status: FunctionStatus,
    wasm: Option<Vec<u8>>,
) -> FunctionMetadata {
    let mut m = FunctionMetadata::new(id, version, Runtime::Wasm, TriggerType::Grpc, wasm);
    m.status = status;
    m.fuel = 10_000_000; // 10M fuel
    m.memory_mib = 64;
    m.timeout_ms = 5_000;
    m
}

// =============================================================================
// Registry tests (4)
// =============================================================================

#[tokio::test]
async fn ut_registry_register_and_get_exact_version() {
    let reg = InMemoryRegistry::new();
    reg.register(new_meta(
        "fn.echo",
        "v0.1.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_SUM)),
    ))
    .await
    .expect("register");

    let got = reg.get("fn.echo", Some("v0.1.0")).await.expect("get");
    assert_eq!(got.function_id, "fn.echo");
    assert_eq!(got.version, "v0.1.0");
    assert_eq!(got.status, FunctionStatus::Active);
}

#[tokio::test]
async fn ut_registry_get_latest_active_version() {
    // The famous SemVer pitfall: "v0.10.0" must be picked over "v0.2.0".
    let reg = InMemoryRegistry::new();
    for v in ["v0.1.0", "v0.2.0", "v0.10.0"] {
        reg.register(new_meta(
            "fn.latest",
            v,
            FunctionStatus::Active,
            Some(wat_to_bytes(WAT_SUM)),
        ))
        .await
        .expect("register");
    }
    let got = reg.get("fn.latest", None).await.expect("get latest");
    assert_eq!(
        got.version, "v0.10.0",
        "string-sort would incorrectly pick v0.2.0; tuple-sort picks v0.10.0"
    );
}

#[tokio::test]
async fn ut_registry_not_found_returns_error() {
    let reg = InMemoryRegistry::new();
    let err = reg.get("fn.missing", None).await.expect_err("must error");
    match err {
        FunctionPlaneError::NotFound(id) => assert_eq!(id, "fn.missing"),
        other => panic!("expected NotFound, got {other:?}"),
    }

    // Exact version miss on an existing function id.
    reg.register(new_meta(
        "fn.exists",
        "v0.1.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_SUM)),
    ))
    .await
    .unwrap();
    let err = reg
        .get("fn.exists", Some("v9.9.9"))
        .await
        .expect_err("must error");
    match err {
        FunctionPlaneError::VersionNotFound {
            function_id,
            version,
        } => {
            assert_eq!(function_id, "fn.exists");
            assert_eq!(version, "v9.9.9");
        }
        other => panic!("expected VersionNotFound, got {other:?}"),
    }
}

#[tokio::test]
async fn ut_registry_set_status_then_get_returns_not_active() {
    let reg = InMemoryRegistry::new();
    reg.register(new_meta(
        "fn.pause",
        "v0.1.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_SUM)),
    ))
    .await
    .unwrap();
    reg.set_status("fn.pause", "v0.1.0", FunctionStatus::Paused)
        .await
        .unwrap();

    // latest-Active lookup must fail (no Active version exists).
    let err = reg.get("fn.pause", None).await.expect_err("must error");
    assert!(matches!(err, FunctionPlaneError::NotFound(_)));

    // Exact version still resolves (status is observable in the metadata).
    let got = reg.get("fn.pause", Some("v0.1.0")).await.expect("get");
    assert_eq!(got.status, FunctionStatus::Paused);
}

// =============================================================================
// WasmHost tests (5)
// =============================================================================

#[tokio::test]
async fn ut_wasm_compile_wat_to_module() {
    let host = WasmHost::new().expect("host");
    let meta = new_meta(
        "fn.compile",
        "v0.1.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_SUM)),
    );
    host.register_module(&meta).await.expect("register_module");
    let cached = host.module_count().await;
    assert_eq!(cached, 1, "exactly one module cached");
}

#[tokio::test]
async fn ut_wasm_call_compute_returns_sum() {
    let host = WasmHost::new().expect("host");
    let meta = new_meta(
        "fn.sum",
        "v0.1.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_SUM)),
    );
    host.register_module(&meta).await.expect("register");

    // host.invoke_sync is sync — wrap in spawn_blocking as the Gateway does.
    let host2 = host.clone();
    let m = meta.clone();
    let out = tokio::task::spawn_blocking(move || host2.invoke_sync(&m, &json!({"a": 2, "b": 5})))
        .await
        .expect("join")
        .expect("invoke");
    assert_eq!(out, 7, "2 + 5 = 7");
}

#[tokio::test]
async fn ut_wasm_trap_on_div_by_zero() {
    let host = WasmHost::new().expect("host");
    let meta = new_meta(
        "fn.div",
        "v0.1.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_DIV)),
    );
    host.register_module(&meta).await.expect("register");

    let host2 = host.clone();
    let m = meta.clone();
    let err = tokio::task::spawn_blocking(move || host2.invoke_sync(&m, &json!({"a": 1, "b": 0})))
        .await
        .expect("join")
        .expect_err("must trap");
    assert!(
        matches!(err, FunctionPlaneError::WasmTrap(_)),
        "expected WasmTrap, got {err:?}"
    );
}

#[tokio::test]
async fn ut_wasm_memory_limit_violation() {
    let host = WasmHost::new().expect("host");
    let mut meta = new_meta(
        "fn.grow",
        "v0.1.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_GROW_MEMORY)),
    );
    meta.memory_mib = 1; // 1 MiB ceiling — growth to ~640 MiB must fail.
    host.register_module(&meta).await.expect("register");

    let host2 = host.clone();
    let m = meta.clone();
    let err = tokio::task::spawn_blocking(move || host2.invoke_sync(&m, &json!({"a": 0, "b": 0})))
        .await
        .expect("join")
        .expect_err("must fail");
    // The limiter denies growth by returning `Ok(false)`; the WAT module
    // observes the `-1` return and traps via `unreachable`. The mock's
    // error mapping surfaces this as `WasmTrap` (the trap message contains
    // "unreachable"). Accept any of the three observable outcomes:
    //   - `MemoryLimitExceeded` if the host ever phrases the OOM that way
    //   - `WasmTrap` with "unreachable" (most likely on Wasmtime 20)
    //   - `WasmTrap` with "memory" (older Wasmtime wording)
    match err {
        FunctionPlaneError::MemoryLimitExceeded { limit_mib } => {
            assert_eq!(limit_mib, 1);
        }
        FunctionPlaneError::WasmTrap(msg) => {
            let lower = msg.to_lowercase();
            assert!(
                lower.contains("unreachable")
                    || lower.contains("memory")
                    || lower.contains("out of memory"),
                "expected memory-related trap, got: {msg}"
            );
        }
        other => panic!("expected MemoryLimitExceeded or memory-related trap, got {other:?}"),
    }
}

#[tokio::test]
async fn ut_wasm_fuel_exhaustion() {
    // The mock enforces a pre-flight fuel threshold (`MIN_FUEL_FOR_INVOKE`,
    // currently 1_000) below which the call is short-circuited with
    // `FuelExhausted`. This sidesteps a Wasmtime-on-Windows FFI longjmp
    // abort that surfaces when Wasmtime's `out_of_gas` libcall is allowed
    // to actually run out of fuel inside the host. See
    // `WasmHost::invoke_sync` for the long-form note.
    let host = WasmHost::new().expect("host");
    let mut meta = new_meta(
        "fn.fuel",
        "v0.1.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_SUM)),
    );
    meta.fuel = 100; // well below MIN_FUEL_FOR_INVOKE
    host.register_module(&meta).await.expect("register");

    let host2 = host.clone();
    let m = meta.clone();
    let err = tokio::task::spawn_blocking(move || host2.invoke_sync(&m, &json!({"a": 1, "b": 2})))
        .await
        .expect("join")
        .expect_err("must run out of fuel");
    assert!(
        matches!(err, FunctionPlaneError::FuelExhausted { limit: 100 }),
        "expected FuelExhausted{{limit:100}}, got {err:?}"
    );
}

// =============================================================================
// WasmHost host-import tests (2) — exercises RGS-INC-001 v0.2 §8.3 surface
// =============================================================================

/// `compute(a, b)` calls `env.host_log(level, ptr, len)` then returns a+b.
/// Imports `env.host_log` which the linker DOES define — instantiation must
/// succeed, and the call must return the correct sum, proving the host-import
/// wiring path works end-to-end.
const WAT_HOST_LOG: &str = r#"
    (module
      (import "env" "host_log" (func $log (param i32 i32 i32)))
      (memory 1)
      (data (i32.const 0) "hello wasm")
      (func (export "compute") (param i32 i32) (result i32)
        (call $log (i32.const 1) (i32.const 0) (i32.const 10))
        (i32.add (local.get 0) (local.get 1))
      )
    )
"#;

/// Imports `env.forbidden_call` which the linker DOES NOT define. Per
/// RGS-INC-001 v0.2 §8.3 the linker is deny-by-default, so instantiation
/// must fail. This is the *capability-based security boundary* test — a
/// module cannot silently pull in arbitrary host functions.
const WAT_FORBIDDEN_IMPORT: &str = r#"
    (module
      (import "env" "forbidden_call" (func $bad))
      (func (export "compute") (param i32 i32) (result i32)
        i32.const 0
      )
    )
"#;

#[tokio::test]
async fn ut_wasm_host_log_import_works() {
    // A module that imports `env.host_log` must instantiate, the host call
    // must complete (no trap), and `compute` must still return a+b. This
    // proves the linker wires host imports correctly without breaking the
    // basic export contract.
    let host = WasmHost::new().expect("host");
    let meta = new_meta(
        "fn.host_log",
        "v0.1.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_HOST_LOG)),
    );
    host.register_module(&meta).await.expect("register");
    let result = tokio::task::spawn_blocking({
        let host = host.clone();
        let meta = meta.clone();
        move || host.invoke_sync(&meta, &json!({"a": 4, "b": 6}))
    })
    .await
    .expect("join")
    .expect("compute must succeed — host_log is a no-op for the mock");
    assert_eq!(result, 10, "host_log call should not affect compute result");
}

#[tokio::test]
async fn ut_wasm_undefined_host_import_fails_instantiate() {
    // Per RGS-INC-001 v0.2 §8.3 the linker is deny-by-default: any host
    // import other than the registered set must be rejected at instantiate
    // time, not at call time. The mock's only registered import is
    // `env.host_log`; here we declare `env.forbidden_call` and expect a
    // `WasmInstantiate` error.
    let host = WasmHost::new().expect("host");
    let meta = new_meta(
        "fn.forbidden",
        "v0.1.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_FORBIDDEN_IMPORT)),
    );
    host.register_module(&meta).await.expect("register");
    let result = tokio::task::spawn_blocking({
        let host = host.clone();
        let meta = meta.clone();
        move || host.invoke_sync(&meta, &json!({"a": 0, "b": 0}))
    })
    .await
    .expect("join");
    let err = result.expect_err("forbidden import must fail to instantiate");
    assert!(
        matches!(err, FunctionPlaneError::WasmInstantiate(_)),
        "expected WasmInstantiate error, got {err:?}"
    );
}

// =============================================================================
// Gateway tests (6)
// =============================================================================

async fn gw_with_sum_fn(id: &str, version: &str) -> FunctionGateway {
    let gw = FunctionGateway::with_in_memory().expect("gateway");
    let mut m = new_meta(
        id,
        version,
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_SUM)),
    );
    m.fuel = 10_000_000;
    m.memory_mib = 64;
    gw.register(m).await.expect("register");
    gw
}

#[tokio::test]
async fn ut_gateway_invoke_sum_function_e2e() {
    let gw = gw_with_sum_fn("fn.sum", "v0.1.0").await;
    let req = InvocationRequest {
        function_id: "fn.sum".into(),
        version: None,
        input: json!({"a": 4, "b": 6}),
        context: FunctionContext::new(),
        extra: Default::default(),
    };
    let res = gw.invoke(req).await.expect("invoke");
    assert!(res.success);
    assert_eq!(res.function_id, "fn.sum");
    assert_eq!(res.version, "v0.1.0");
    assert_eq!(res.output, Some(json!({"result": 10})));
}

#[tokio::test]
async fn ut_gateway_invoke_not_active_returns_error() {
    // Draft → NotActive on the Gateway.
    let gw = FunctionGateway::with_in_memory().expect("gateway");
    let mut m = new_meta(
        "fn.draft",
        "v0.1.0",
        FunctionStatus::Draft,
        Some(wat_to_bytes(WAT_SUM)),
    );
    // The Gateway only eagerly compiles Active versions. The mock happily
    // register a Draft metadata row; subsequent invoke must fail.
    m.fuel = 1_000_000;
    gw.register(m).await.expect("register");

    let req = InvocationRequest {
        function_id: "fn.draft".into(),
        version: Some("v0.1.0".into()),
        input: json!({"a": 1, "b": 2}),
        context: FunctionContext::new(),
        extra: Default::default(),
    };
    let err = gw.invoke(req).await.expect_err("must error");
    assert!(matches!(err, FunctionPlaneError::NotActive(_)));

    // Now Paused via set_status.
    let gw = gw_with_sum_fn("fn.paused", "v0.1.0").await;
    // In the mock, register() is the only way to mutate status, so we
    // re-register with Paused and verify the gateway refuses.
    let mut m2 = new_meta(
        "fn.paused",
        "v0.1.0",
        FunctionStatus::Paused,
        Some(wat_to_bytes(WAT_SUM)),
    );
    m2.fuel = 1_000_000;
    // Skip the host.register_module path (Gateway would only compile Active);
    // force a direct registry write via the public trait.
    let _ = gw; // not used further
    let direct_reg: Arc<dyn FunctionRegistry> = Arc::new(InMemoryRegistry::new());
    direct_reg.register(m2).await.unwrap();
    let host = Arc::new(WasmHost::new().unwrap());
    let gw2 = FunctionGateway::new(direct_reg, host);
    let req2 = InvocationRequest {
        function_id: "fn.paused".into(),
        version: Some("v0.1.0".into()),
        input: json!({"a": 1, "b": 2}),
        context: FunctionContext::new(),
        extra: Default::default(),
    };
    let err = gw2.invoke(req2).await.expect_err("must error");
    assert!(matches!(err, FunctionPlaneError::NotActive(_)));
}

#[tokio::test]
async fn ut_gateway_invoke_not_found_returns_error() {
    let gw = FunctionGateway::with_in_memory().expect("gateway");
    let req = InvocationRequest {
        function_id: "fn.ghost".into(),
        version: None,
        input: json!({}),
        context: FunctionContext::new(),
        extra: Default::default(),
    };
    let err = gw.invoke(req).await.expect_err("must error");
    assert!(matches!(err, FunctionPlaneError::NotFound(_)));
}

#[tokio::test]
async fn ut_gateway_context_propagation() {
    let gw = gw_with_sum_fn("fn.ctx", "v0.1.0").await;
    let ctx = FunctionContext::new()
        .with_trace_id("0af7651916cd43dd8448eb211c80319c")
        .with_saga_id(Uuid::new_v4())
        .with_user_id(Uuid::new_v4())
        .with_tenant_id("tenant-acme")
        .with_retry_count(2);
    let req = InvocationRequest {
        function_id: "fn.ctx".into(),
        version: Some("v0.1.0".into()),
        input: json!({"a": 1, "b": 2}),
        context: ctx,
        extra: Default::default(),
    };
    let res = gw.invoke(req).await.expect("invoke");
    assert!(res.success, "context fields must not break invocation");
    assert_eq!(res.output, Some(json!({"result": 3})));
}

#[tokio::test]
async fn ut_gateway_version_specific() {
    // Register two Active versions of the same function id.
    let gw = FunctionGateway::with_in_memory().expect("gateway");
    // v0.1.0 → sum
    let mut m1 = new_meta(
        "fn.ver",
        "v0.1.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_SUM)),
    );
    m1.fuel = 1_000_000;
    m1.memory_mib = 16;
    gw.register(m1).await.unwrap();
    // v0.2.0 → constant 42 (proves we're routing by version, not by id)
    let mut m2 = new_meta(
        "fn.ver",
        "v0.2.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_CONST)),
    );
    m2.fuel = 1_000_000;
    m2.memory_mib = 16;
    gw.register(m2).await.unwrap();

    let req1 = InvocationRequest {
        function_id: "fn.ver".into(),
        version: Some("v0.1.0".into()),
        input: json!({"a": 3, "b": 4}),
        context: FunctionContext::new(),
        extra: Default::default(),
    };
    let res1 = gw.invoke(req1).await.expect("invoke v0.1.0");
    assert_eq!(res1.version, "v0.1.0");
    assert_eq!(res1.output, Some(json!({"result": 7})));

    let req2 = InvocationRequest {
        function_id: "fn.ver".into(),
        version: Some("v0.2.0".into()),
        input: json!({"a": 0, "b": 0}),
        context: FunctionContext::new(),
        extra: Default::default(),
    };
    let res2 = gw.invoke(req2).await.expect("invoke v0.2.0");
    assert_eq!(res2.version, "v0.2.0");
    assert_eq!(res2.output, Some(json!({"result": 42})));
}

#[tokio::test]
async fn ut_gateway_idempotency_key_passthrough() {
    let gw = gw_with_sum_fn("fn.idem", "v0.1.0").await;
    let ctx = FunctionContext::new().with_idempotency_key("idem-key-2026-08-23-001");
    let req = InvocationRequest {
        function_id: "fn.idem".into(),
        version: Some("v0.1.0".into()),
        input: json!({"a": 1, "b": 1}),
        context: ctx,
        extra: Default::default(),
    };
    let res = gw.invoke(req).await.expect("invoke");
    assert!(res.success);
    // The mock doesn't surface the idempotency key in the result; the fact
    // that the call succeeded without "context invalid" errors demonstrates
    // the field is accepted by the call path end-to-end.
    assert_eq!(res.output, Some(json!({"result": 2})));
}

// =============================================================================
// Contract tests (3)
// =============================================================================

#[test]
fn ut_contract_function_metadata_serde_roundtrip() {
    let m = new_meta(
        "fn.serde",
        "v0.1.0",
        FunctionStatus::Active,
        Some(wat_to_bytes(WAT_SUM)),
    );
    let s = serde_json::to_string(&m).expect("serialize");
    let back: FunctionMetadata = serde_json::from_str(&s).expect("deserialize");
    assert_eq!(back.function_id, m.function_id);
    assert_eq!(back.version, m.version);
    assert_eq!(back.runtime, m.runtime);
    assert_eq!(back.status, m.status);
    assert_eq!(
        back.wasm_bytes.as_ref().unwrap().len(),
        m.wasm_bytes.as_ref().unwrap().len()
    );
}

#[test]
fn ut_contract_function_context_default_has_request_id() {
    let c1 = FunctionContext::new();
    let c2 = FunctionContext::new();
    assert!(!c1.request_id.is_nil());
    assert_ne!(
        c1.request_id, c2.request_id,
        "two fresh contexts must not collide"
    );
    // Default trait must also produce a fresh request_id.
    let c3 = FunctionContext::default();
    assert!(!c3.request_id.is_nil());
}

#[test]
fn ut_contract_invocation_request_minimal() {
    // No version, no extra, no context fields beyond the default.
    let req = InvocationRequest {
        function_id: "fn.min".into(),
        version: None,
        input: json!({}),
        context: FunctionContext::new(),
        extra: Default::default(),
    };
    let s = serde_json::to_string(&req).unwrap();
    let back: InvocationRequest = serde_json::from_str(&s).unwrap();
    assert_eq!(back.function_id, "fn.min");
    assert!(back.version.is_none());
    assert!(back.context.request_id != Uuid::nil());
}

// =============================================================================
// Error tests (2)
// =============================================================================

#[test]
fn ut_error_display_messages() {
    let e1 = FunctionPlaneError::NotFound("fn.x".into());
    assert_eq!(e1.to_string(), "function not found: fn.x");

    let e2 = FunctionPlaneError::VersionNotFound {
        function_id: "fn.y".into(),
        version: "v0.0.0".into(),
    };
    assert_eq!(
        e2.to_string(),
        "version not found: function=fn.y version=v0.0.0"
    );

    let e3 = FunctionPlaneError::FuelExhausted { limit: 100 };
    assert_eq!(e3.to_string(), "fuel exhausted (limit=100)");

    let e4 = FunctionPlaneError::MemoryLimitExceeded { limit_mib: 64 };
    assert_eq!(e4.to_string(), "memory limit exceeded: 64 MiB");

    let e5 = FunctionPlaneError::Timeout(5000);
    assert_eq!(e5.to_string(), "execution timeout after 5000ms");
}

#[test]
fn ut_error_not_found_message_includes_function_id() {
    let err = FunctionPlaneError::NotFound("fn.critical".into());
    let msg = err.to_string();
    assert!(
        msg.contains("fn.critical"),
        "msg should include id, got: {msg}"
    );
}

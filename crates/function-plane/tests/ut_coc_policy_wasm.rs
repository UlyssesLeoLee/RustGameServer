//! Unit tests for the `coc.policy` WASM module template
//! (per RGS-INC-001 v0.3 §X.2 决策契约 + §X.6 7 条护栏).
//!
//! 5 scenarios (任务简报要求 4 + 1 边界 case 强化 §X.2 三态优先级):
//! 1. `ut_coc_policy_basic_allow`               — 基础 Allow
//!    input.amount=100, blacklist=false → decision=Allow
//! 2. `ut_coc_policy_high_amount_requires_second_review`
//!    input.amount=5000, blacklist=false → decision=RequireSecondReview
//! 3. `ut_coc_policy_blacklist_target_denied`
//!    input.amount=100, blacklist=true → decision=Deny
//! 4. `ut_coc_policy_blacklist_overrides_high_amount`  (boundary case per §X.2 优先级)
//!    input.amount=5000, blacklist=true → decision=Deny (blacklist 优先级 > 高额)
//! 5. `ut_coc_policy_params_hash_correctness`
//!    SHA-256 长度 = 64 char hex + 确定性 + 不同输入 → 不同 hash
//!
//! ## WAT module decision tree (POC)
//!
//! ```text
//! compute(a, b) -> i32   // a = amount, b = blacklist_flag (0/1)
//!   b == 1   → 2 (Deny)
//!   a > 1000 → 1 (RequireSecondReview)
//!   else     → 0 (Allow)
//! ```
//!
//! ## Adapter to existing Phase 0 mock
//!
//! The Phase 0 mock's `WasmHost::invoke_sync(&meta, &json!({"a":..., "b":...}))`
//! is the only public entry point today. The `evaluate_coc_policy` helper
//! encodes the input → `(a, b)` and decodes the `i32` return → `CocDecision`,
//! mirroring the future `WasmHost::call("coc.policy", json)` API (per
//! RGS-INC-001 v0.3 §X.1 line 323) in shape. Phase 1+ will replace the
//! adapter with the real JSON ABI.

use function_plane::{
    CocDecision, CocPolicyInput, CocPolicyOutput, FunctionMetadata, FunctionStatus, Runtime,
    TriggerType, WasmHost,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

// =============================================================================
// WAT template
// =============================================================================

/// `coc.policy` WASM module template (per RGS-INC-001 v0.3 §X.2 决策 schema).
///
/// - `compute(a, b) -> i32`
///   - `a` = amount (i32; signed)
///   - `b` = blacklist_flag (0 = not blacklisted, 1 = blacklisted)
///   - returns: 0 = Allow, 1 = RequireSecondReview, 2 = Deny
///
/// - `host_log` import is per §8.3 capability 白名单 (allowed).
///   Does **not** import `host_query_db` / `host_publish_event` /
///   `host_get_state` — POC simplified surface (per RGS-INC-001 v0.3 §X.3 POC).
const WAT_COC_POLICY: &str = r#"
    (module
      (import "env" "host_log" (func $log (param i32 i32 i32)))
      (memory 1)
      (data (i32.const 0) "coc_policy_decision")
      (func (export "compute") (param i32 i32) (result i32)
        ;; params: a = amount, b = blacklist_flag (0/1)
        ;; returns: 0=Allow, 1=RequireSecondReview, 2=Deny
        (call $log (i32.const 1) (i32.const 0) (i32.const 20))
        (if (i32.eq (local.get 1) (i32.const 1))
          (then (return (i32.const 2))))
        (if (i32.gt_s (local.get 0) (i32.const 1000))
          (then (return (i32.const 1))))
        (i32.const 0)
      )
    )
"#;

// =============================================================================
// Test helpers
// =============================================================================

fn wat_to_bytes(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).expect("wat must parse")
}

fn new_meta(version: &str, wasm: Option<Vec<u8>>) -> FunctionMetadata {
    let mut m = FunctionMetadata::new(
        "coc.policy",
        version,
        Runtime::Wasm,
        TriggerType::Grpc,
        wasm,
    );
    m.status = FunctionStatus::Active;
    m.fuel = 10_000_000; // well above WasmHost::MIN_FUEL_FOR_INVOKE
    m.memory_mib = 64;
    m.timeout_ms = 5_000;
    m
}

fn module_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Adapter: build a `CocPolicyInput` → invoke the WAT module → decode → `CocPolicyOutput`.
///
/// Mirrors the future `WasmHost::call("coc.policy", json)` API (per RGS-INC-001
/// v0.3 §X.1 line 323) in shape. Today the Phase 0 mock only exposes
/// `compute(a: i32, b: i32) -> i32`, so we encode/decode through that.
async fn evaluate_coc_policy(
    wasm_bytes: &[u8],
    module_version: &str,
    input: &CocPolicyInput,
) -> CocPolicyOutput {
    let host = WasmHost::new().expect("host");
    let meta = new_meta(module_version, Some(wasm_bytes.to_vec()));
    host.register_module(&meta)
        .await
        .expect("register_module");

    // Encode input → (a, b) for the WAT module.
    let amount_i32 = input.amount.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    let blacklist_i32 = if input.target_blacklisted { 1 } else { 0 };

    let host2 = host.clone();
    let m = meta.clone();
    let result_i64 = tokio::task::spawn_blocking(move || {
        host2.invoke_sync(&m, &json!({"a": amount_i32, "b": blacklist_i32}))
    })
    .await
    .expect("join")
    .expect("compute must succeed — WAT decision tree returns one of 0/1/2");

    let result_i32 = result_i64 as i32;
    let decision = CocDecision::from_wat_code(result_i32).unwrap_or_else(|| {
        panic!(
            "WAT module returned out-of-range decision code {result_i32}; \
             expected 0/1/2 per CocDecision encoding"
        )
    });

    // POC reason text — Phase 1+ will move this into the WASM module itself
    // (returning a JSON `{decision, reason}` rather than a bare i32).
    let reason = match decision {
        CocDecision::Allow => "low_amount_normal_target".to_string(),
        CocDecision::RequireSecondReview => "high_amount_grant".to_string(),
        CocDecision::Deny => "target_in_blacklist".to_string(),
    };

    CocPolicyOutput {
        decision,
        reason,
        module_version: module_version.to_string(),
        module_hash: module_hash(wasm_bytes),
        params_hash: input.params_hash(),
    }
}

fn sample_input(amount: i64, target_blacklisted: bool) -> CocPolicyInput {
    CocPolicyInput {
        actor_id: Uuid::new_v4(),
        action: "economy.grant".to_string(),
        target_id: "account-target-1".to_string(),
        context: json!({
            "recent_bans": 0,
            "amount": amount,
            "last_action_at": "2026-09-05T00:00:00Z",
        }),
        trace_id: "0af7651916cd43dd8448eb211c80319c".to_string(),
        amount,
        target_blacklisted,
    }
}

// =============================================================================
// 1. Basic Allow — amount=100, blacklist=false → decision=Allow
// =============================================================================

#[tokio::test]
async fn ut_coc_policy_basic_allow() {
    let wasm = wat_to_bytes(WAT_COC_POLICY);
    let input = sample_input(100, false);

    let out = evaluate_coc_policy(&wasm, "v0.1.0", &input).await;

    assert_eq!(out.decision, CocDecision::Allow, "low amount + non-blacklist = Allow");
    assert_eq!(out.reason, "low_amount_normal_target");
    assert_eq!(out.module_version, "v0.1.0");
    assert_eq!(
        out.module_hash.len(),
        64,
        "module_hash is SHA-256 hex (64 chars); got {}",
        out.module_hash
    );
    assert_eq!(
        out.params_hash.len(),
        64,
        "params_hash is SHA-256 hex (64 chars); got {}",
        out.params_hash
    );
}

// =============================================================================
// 2. High amount → RequireSecondReview
// =============================================================================

#[tokio::test]
async fn ut_coc_policy_high_amount_requires_second_review() {
    let wasm = wat_to_bytes(WAT_COC_POLICY);
    let input = sample_input(5000, false); // amount=5000 > 1000

    let out = evaluate_coc_policy(&wasm, "v0.1.0", &input).await;

    assert_eq!(
        out.decision,
        CocDecision::RequireSecondReview,
        "amount > 1000 + non-blacklist = RequireSecondReview"
    );
    assert_eq!(out.reason, "high_amount_grant");
}

// =============================================================================
// 3. Blacklist → Deny
// =============================================================================

#[tokio::test]
async fn ut_coc_policy_blacklist_target_denied() {
    let wasm = wat_to_bytes(WAT_COC_POLICY);
    let input = sample_input(100, true); // amount=100, blacklist=true

    let out = evaluate_coc_policy(&wasm, "v0.1.0", &input).await;

    assert_eq!(
        out.decision,
        CocDecision::Deny,
        "blacklist = true (regardless of amount) = Deny"
    );
    assert_eq!(out.reason, "target_in_blacklist");
}

// =============================================================================
// 4. Boundary: blacklist overrides high amount (per §X.2 三态优先级)
// =============================================================================

#[tokio::test]
async fn ut_coc_policy_blacklist_overrides_high_amount() {
    // Per RGS-INC-001 v0.3 §X.2 锁定三态语义:
    //   Allow = 走 audit_log
    //   RequireSecondReview = 写 second_review + 异步通知
    //   Deny = 写 audit_log(decision=denied) + 返 permission_denied
    // 黑名单必须先于高额 grant 判定 — 黑名单命中 → Deny，不能 RequireSecondReview
    // (因为 second_review 表是 async 重放，Deny 才是 fail-closed 终点)
    let wasm = wat_to_bytes(WAT_COC_POLICY);
    let input = sample_input(5000, true); // both triggers fire

    let out = evaluate_coc_policy(&wasm, "v0.1.0", &input).await;

    assert_eq!(
        out.decision,
        CocDecision::Deny,
        "blacklist must dominate high_amount (per §X.2 三态优先级)"
    );
    assert_eq!(out.reason, "target_in_blacklist");
}

// =============================================================================
// 5. params_hash correctness (per §X.4 决策可重放 + §X.6 护栏 7)
// =============================================================================

#[tokio::test]
async fn ut_coc_policy_params_hash_correctness() {
    // Build two structurally identical inputs (same actor_id so the hash
    // is byte-equal). Default `sample_input` uses `Uuid::new_v4()`, so
    // we override the actor_id to make the determinism test meaningful.
    let actor = Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap();
    let mut a = sample_input(100, false);
    a.actor_id = actor;
    let mut b = sample_input(100, false);
    b.actor_id = actor;

    let h1 = a.params_hash();
    let h2 = b.params_hash();

    // 5.1 SHA-256 hex is exactly 64 chars.
    assert_eq!(h1.len(), 64, "params_hash is 64-char hex; got {h1}");
    // 5.2 Same logical input → same hash (deterministic).
    assert_eq!(h1, h2, "same logical input → same hash (deterministic)");

    // 5.3 Different amount → different hash.
    let mut c = a.clone();
    c.amount = 200;
    let h3 = c.params_hash();
    assert_ne!(h1, h3, "different amount → different params_hash");

    // 5.4 Different action → different hash.
    let mut d = a.clone();
    d.action = "player.ban".to_string();
    let h4 = d.params_hash();
    assert_ne!(h1, h4, "different action → different params_hash");

    // 5.5 Different target_blacklisted → different hash.
    let mut e = a.clone();
    e.target_blacklisted = true;
    let h5 = e.params_hash();
    assert_ne!(h1, h5, "different target_blacklisted → different params_hash");

    // 5.6 Different trace_id → different hash.
    let mut f = a.clone();
    f.trace_id = "ffffffffffffffffffffffffffffffff".to_string();
    let h6 = f.params_hash();
    assert_ne!(h1, h6, "different trace_id → different params_hash");

    // 5.7 CocPolicyOutput round-trip — params_hash from CocPolicyInput must
    // match the value embedded in CocPolicyOutput (the field is the audit
    // anchor per §X.5 second_review.coc_params_hash).
    let wasm = wat_to_bytes(WAT_COC_POLICY);
    let out = evaluate_coc_policy(&wasm, "v0.1.0", &a).await;
    assert_eq!(
        out.params_hash, h1,
        "CocPolicyOutput.params_hash must equal CocPolicyInput.params_hash"
    );
}

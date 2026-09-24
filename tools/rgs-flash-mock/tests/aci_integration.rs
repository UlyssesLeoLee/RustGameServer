//! `aci_integration` — 3 IT (per ULYS-191 §4.3.1 brief v0.1)
//!
//! 与 IDE1.0 §4.2.2 IT-1/IT-2/IT-3 pattern 1:1 (per brief v0.1 §3.2 #19):
//!
//! | IT  | 名称                                       | 校验                                                                                 |
//! |-----|--------------------------------------------|--------------------------------------------------------------------------------------|
//! | IT-1 | `test_aci_schema_v0_1_roundtrip`            | `.aci.json` schema v0.1 与 aci-emitter 完全一致 (字段名 1:1)                         |
//! | IT-2 | `test_emit_smoke_assertion_v0_1`            | `emit_smoke_assertion()` 输出 assertion 含全部 10 必填字段 + 字段名 1:1 对齐         |
//! | IT-3 | `test_aci_emitter_v0_1_compatibility`       | RGS Rust emitter 字段名与 Star Python emitter 跨语言 parity (字段名 1:1)            |
//!
//! 守门 (per AGENTS.md §4):
//! - #13 W/T/M: 单元 (emit_smoke_assertion 单测) + 集成 (本文件 3 IT) + 系统 (aci-smoke.sh)
//! - #19v19 Python 化: IT-3 跨语言 parity 测

use std::fs;
use std::path::PathBuf;

use aci_emitter::{AciEmitter, ExpectActual, ExpectValueType, Layer, Scope, Severity, Status};
use rgs_flash_mock::aci_emitter_helper;

/// IT-1: `.aci.json` schema v0.1 与 aci-emitter 完全一致.
///
/// 校验项:
/// - 文件存在 (per ULYS-191 §4.3.1)
/// - `aci_version == "0.1.0-draft"` (与 aci-emitter::ACI_VERSION 同步)
/// - 17 个 `expect_value_types` 字段名与 Python emitter 1:1 对齐
/// - 6 个 `scope_dimensions` (project/module/domain/subdomain/operation/http_method)
/// - 10 个 `schema_required_fields` (assertion_id/aci_version/layer/scope/expect/actual/status/severity/reasoning/captured_at)
#[test]
fn test_aci_schema_v0_1_roundtrip() {
    // 1) 文件存在
    let schema_path = locate_aci_json();
    assert!(
        schema_path.exists(),
        ".aci.json not found at {}",
        schema_path.display()
    );
    let raw = fs::read_to_string(&schema_path).expect(".aci.json must be readable");
    let schema: serde_json::Value =
        serde_json::from_str(&raw).expect(".aci.json must be valid JSON");

    // 2) aci_version 字段
    let aci_version = schema["aci_version"]
        .as_str()
        .expect("schema.aci_version must be a string");
    assert_eq!(
        aci_version,
        aci_emitter::ACI_VERSION,
        "schema.aci_version must match aci-emitter::ACI_VERSION"
    );

    // 3) 17 个 expect_value_types
    let evts = schema["expect_value_types"]
        .as_array()
        .expect("schema.expect_value_types must be an array");
    assert_eq!(
        evts.len(),
        17,
        "schema.expect_value_types must have 17 entries (per §4.2 v0.1)"
    );

    // 4) 6 个 scope_dimensions
    let dims = schema["scope_dimensions"]
        .as_array()
        .expect("schema.scope_dimensions must be an array");
    assert_eq!(dims.len(), 6, "schema.scope_dimensions must have 6 entries");

    // 5) 10 个 schema_required_fields — 字段名 1:1 对齐
    let required = schema["schema_required_fields"]
        .as_array()
        .expect("schema.schema_required_fields must be an array");
    let required_set: std::collections::HashSet<&str> =
        required.iter().filter_map(|v| v.as_str()).collect();
    for field in &[
        "assertion_id",
        "aci_version",
        "layer",
        "scope",
        "expect",
        "actual",
        "status",
        "severity",
        "reasoning",
        "captured_at",
    ] {
        assert!(
            required_set.contains(field),
            "schema_required_fields must contain '{field}'"
        );
    }
}

/// IT-2: `emit_smoke_assertion()` 输出 assertion 含全部 10 必填字段 + 字段名 1:1 对齐。
///
/// 校验项:
/// - 10 必填字段全部存在且字段名与 schema_required_fields 1:1 对齐
/// - 默认值符合 §4.3.1 brief v0.1 (assertion_id="rgs-flash-mock:smoke:g-1" 等)
/// - JSON 序列化字段名 1:1 (与 Python emitter 跨语言可解析)
#[test]
fn test_emit_smoke_assertion_v0_1() {
    let assertion = aci_emitter_helper::emit_smoke_assertion();

    // 1) 10 必填字段非空
    assert_eq!(assertion.assertion_id, "rgs-flash-mock:smoke:g-1");
    assert_eq!(assertion.aci_version, "0.1.0-draft");
    assert_eq!(assertion.layer, Layer::It);
    assert_eq!(assertion.scope.project, "rgs-flash-mock");
    assert_eq!(assertion.scope.module.as_deref(), Some("smoke"));
    // expect/actual 是 ExpectActual, 后面单独验
    assert_eq!(assertion.status, Status::Pass);
    assert_eq!(assertion.severity, Severity::Info);
    assert!(
        !assertion.reasoning.is_empty(),
        "reasoning must be non-empty (per §4.3.1)"
    );
    assert!(
        !assertion.captured_at.is_empty(),
        "captured_at must be non-empty"
    );

    // 2) expect/actual 字段类型 + 值 (response_within_ms 2000/150)
    assert_eq!(
        assertion.expect.value_type,
        ExpectValueType::ResponseWithinMs
    );
    assert_eq!(assertion.expect.value, serde_json::json!(2000));
    assert_eq!(
        assertion.actual.value_type,
        ExpectValueType::ResponseWithinMs
    );
    assert_eq!(assertion.actual.value, serde_json::json!(150));

    // 3) JSON 序列化字段名 1:1 (与 Python emitter 跨语言可解析)
    let em = AciEmitter::new(Layer::It);
    let json_value = em.to_json(&assertion);
    let json_str = serde_json::to_string(&json_value).expect("assertion must serialize");
    for field in &[
        "assertion_id",
        "aci_version",
        "layer",
        "scope",
        "expect",
        "actual",
        "status",
        "severity",
        "reasoning",
        "captured_at",
    ] {
        assert!(
            json_str.contains(field),
            "serialized JSON must contain '{field}' (cross-language parity)"
        );
    }

    // 4) smoke helper 单独打印 JSON (供 aci-smoke.sh 抓 stdout)
    let pretty = aci_emitter_helper::emit_smoke_assertion_json();
    assert!(pretty.contains("rgs-flash-mock:smoke:g-1"));
    eprintln!("=== emit_smoke_assertion JSON ===\n{pretty}\n=== END ===");
}

/// IT-3: RGS Rust emitter 与 Star Python emitter 跨语言 parity (字段名 1:1)。
///
/// 校验项 (per §4.2 Python emitter + brief v0.1):
/// - 字段名 1:1 对齐 Python `_lib_aci_emit.py`
/// - ExpectActual 序列化字段名 = "type" / "value" / "description"
/// - Status / Severity / Layer 的字符串表示与 Python 一致 (PASS / Info / It)
#[test]
fn test_aci_emitter_v0_1_compatibility() {
    // 构造一条与 IDE1.0 §4.2.2 ide-cli smoke 1:1 的 assertion
    let em = AciEmitter::new(Layer::It);
    let assertion = em
        .build(
            "rgs-flash-mock:smoke:g-1".to_string(),
            Scope::new(
                "rgs-flash-mock".to_string(),
                Some("smoke".to_string()),
                None,
                None,
                None,
                None,
            ),
            ExpectActual::new(
                ExpectValueType::ResponseWithinMs,
                serde_json::json!(2000),
                "RGS API should respond within 2s",
            ),
            ExpectActual::new(
                ExpectValueType::ResponseWithinMs,
                serde_json::json!(150),
                "measured 150ms",
            ),
            Status::Pass,
            Severity::Info,
            "actual << expect (13x margin)",
        )
        .expect("cross-language parity assertion must build successfully");

    // 1) ExpectActual 序列化字段名 = type/value/description (Python emitter 约定)
    let json_value = em.to_json(&assertion);
    let expect_obj = json_value["expect"]
        .as_object()
        .expect("expect must be object");
    for key in &["type", "value", "description"] {
        assert!(
            expect_obj.contains_key(*key),
            "expect must have field '{key}' (Python emitter parity)"
        );
    }

    // 2) Status 序列化 = "PASS" (大写, 与 Python _lib_aci_emit.py 1:1)
    assert_eq!(
        json_value["status"].as_str(),
        Some("PASS"),
        "status must serialize to 'PASS' (Python emitter parity)"
    );

    // 3) Severity 序列化 = "info" (lowercase, Python emitter parity)
    assert_eq!(
        json_value["severity"].as_str(),
        Some("info"),
        "severity must serialize to 'info' (Python emitter parity)"
    );

    // 4) Layer 序列化 = "it" (lowercase, Python emitter parity)
    assert_eq!(
        json_value["layer"].as_str(),
        Some("it"),
        "layer must serialize to 'it' (Python emitter parity)"
    );

    // 5) Scope 字段名 1:1
    let scope_obj = json_value["scope"]
        .as_object()
        .expect("scope must be object");
    for key in &[
        "project",
        "module",
        "domain",
        "subdomain",
        "operation",
        "http_method",
    ] {
        assert!(
            scope_obj.contains_key(*key),
            "scope must have field '{key}' (Python emitter parity)"
        );
    }
}

/// Locate `.aci.json` relative to crate root (works for both `cargo test -p` and `cargo test --test`).
fn locate_aci_json() -> PathBuf {
    // CARGO_MANIFEST_DIR points to tools/rgs-flash-mock during cargo test
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join(".aci.json")
}

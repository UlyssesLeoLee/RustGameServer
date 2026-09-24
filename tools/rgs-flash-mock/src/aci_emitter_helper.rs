//! `aci_emitter_helper` — RGS ACI emitter helper (per ULYS-191 §4.3.1 brief v0.1)
//!
//! 1 个公开函数 `emit_smoke_assertion()`, 封装 `aci_emitter::AciEmitter` emit 一条
//! placeholder smoke assertion. 与 IDE1.0 §4.2.2 ide-cli 模式一致, 但走 library
//! API (RGS 没有 CLI, 是 HTTP server, smoke 通过 cargo test --nocapture 间接 emit).
//!
//! 守门 (per AGENTS.md §4):
//! - #7 unsafe_code=forbid: aci-emitter workspace lint 已 forbid, 本文件 0 unsafe
//! - #11 缺标比错标: 仅 git dep `aci-emitter`, 字段名 1:1 对齐 Python emitter
//! - #13 W/T/M: emit_smoke_assertion 单测由 IT-2 (`test_emit_smoke_assertion_v0_1`) 覆盖
//!
//! 字段语义 (per `.aci.json` schema v0.1):
//! - `assertion_id`: `rgs-flash-mock:smoke:g-1` (格式 `<project>:<module>:<id>`)
//! - `aci_version`: 自动取自 `aci_emitter::ACI_VERSION` (= "0.1.0-draft")
//! - `layer`: `It` (集成测试, per §4.3.1 brief)
//! - `scope`: project=rgs-flash-mock, module=smoke
//! - `expect`: response_within_ms=2000, "RGS API should respond within 2s"
//! - `actual`: response_within_ms=150, "measured 150ms (placeholder)"
//! - `status`: `Pass` (placeholder per §4.3.1 brief v0.1)
//! - `severity`: `Info`
//! - `reasoning`: 13x margin

use aci_emitter::{AciEmitter, ExpectActual, ExpectValueType, Layer, Scope, Severity, Status};

/// Emit 一条 RGS placeholder smoke assertion (per §4.3.1 brief v0.1).
///
/// 默认值 (与 IDE1.0 §4.2.2 ide-cli smoke 1:1 pattern):
/// - assertion_id: `rgs-flash-mock:smoke:g-1`
/// - scope: project=rgs-flash-mock, module=smoke
/// - expect: response_within_ms = 2000, "RGS API should respond within 2s"
/// - actual: response_within_ms = 150, "measured 150ms (placeholder)"
/// - status: `Pass`
/// - severity: `Info`
/// - reasoning: "actual << expect (13x margin) — placeholder per §4.3.1 brief v0.1"
///
/// # Returns
///
/// `Assertion` (per aci-emitter v0.1.0). 字段名 1:1 对齐 Python
/// [`_lib_aci_emit.py`](https://github.com/UlyssesLeoLee/Star/blob/main/tools/star-flash-mock/scripts/_lib_aci_emit.py).
///
/// # Example
///
/// ```
/// use rgs_flash_mock::aci_emitter_helper::emit_smoke_assertion;
/// let a = emit_smoke_assertion();
/// assert_eq!(a.assertion_id, "rgs-flash-mock:smoke:g-1");
/// assert_eq!(a.aci_version, "0.1.0-draft");
/// ```
#[must_use]
pub fn emit_smoke_assertion() -> aci_emitter::Assertion {
    let em = AciEmitter::new(Layer::It);
    em.build(
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
            "measured 150ms (placeholder)",
        ),
        Status::Pass,
        Severity::Info,
        "actual << expect (13x margin) — placeholder per §4.3.1 brief v0.1",
    )
    .expect("smoke assertion build must succeed (validation is no-op for hardcoded values)")
}

/// Emit smoke assertion 并序列化为 pretty JSON (供 `aci-smoke.sh` 调 cargo test 输出).
///
/// # Returns
///
/// pretty-printed JSON 字符串 (`serde_json::to_string_pretty` 输出).
#[must_use]
pub fn emit_smoke_assertion_json() -> String {
    let assertion = emit_smoke_assertion();
    let em = AciEmitter::new(Layer::It);
    let value = em.to_json(&assertion);
    serde_json::to_string_pretty(&value).expect("assertion must serialize to pretty JSON")
}

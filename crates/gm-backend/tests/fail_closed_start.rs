//! gm-backend fail-closed 启动测试(per 5 域 + cluster-ops 一致模式)
//!
//! gm-backend 当前没有 mTLS 配置(纯 HTTP/8081 health + 8443 main),
//! 不像 5 域 + cluster-ops 需要 RGS_TLS_DIR → fail-closed 验证。
//!
//! 但仍然验证:gm-backend 启动时不会因 mTLS 缺失 panic / 静默 insecure,
//! 跟 5 域不同(5 域 RGS_ALLOW_INSECURE_GRPC=0 默认 fail-closed)。
//!
//! gm-backend 设计:RBAC + mTLS 在 main.rs 不强制(per 2026-08-27 v0.1 dev 妥协,
//! RBAC 留给 v0.2)。本测试只验证基本启动不崩。
//!
//! 跨 session 行为:gRPC client (admin-service 调用) 还没接,暂不能测 mTLS 路径。
//!
//! 实现要点:用 `assert_cmd::Command` 而非 `std::process::Command`,
//! 因后者没有 `.timeout()` 方法。

use assert_cmd::Command;
use std::time::Duration;

#[test]
fn gm_backend_starts_with_defaults() {
    // 用 env 强制 0.0.0.0:0(随机端口,不会冲突)
    let output = Command::cargo_bin("gm-backend")
        .expect("locate gm-backend binary via cargo metadata")
        // TBD-08-02 v0.2:dev 模式跳过 mTLS fail-closed (先 remove 再 env 才生效)
        .env("RGS_ALLOW_INSECURE_GRPC", "1")
        .env("GM_HTTP_ADDR", "127.0.0.1:0")
        .env("GM_HEALTH_ADDR", "127.0.0.1:0")
        .env("RUST_LOG", "info")
        .env_remove("GM_JWT_SECRET")
        .env_remove("ADMIN_GRPC_ENDPOINT")
        .env_remove("RGS_TLS_DIR")
        .timeout(Duration::from_secs(5))
        .output()
        .expect("spawn gm-backend binary");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // 检查启动日志含 "starting GM APIGW"
    // 或 stderr 至少有内容(说明 binary 启动了)
    let started = stderr.contains("starting GM APIGW")
        || stdout.contains("starting GM APIGW")
        || (!stderr.is_empty() && output.status.code().is_none());
    assert!(
        started,
        "gm-backend should start; exit_status={:?} stderr.len={} stdout.len={} stderr={}",
        output.status.code(),
        stderr.len(),
        stdout.len(),
        stderr.chars().take(2000).collect::<String>()
    );
}

/// TBD-08-02 v0.2: mTLS fail-closed 启动路径
/// 生产模式 RGS_ALLOW_INSECURE_GRPC=0 + RGS_TLS_DIR 缺失 → 必须 fail, exit != 0
#[test]
fn gm_backend_fails_closed_when_tls_dir_missing() {
    let output = Command::cargo_bin("gm-backend")
        .expect("locate gm-backend binary")
        .env("GM_HTTP_ADDR", "127.0.0.1:0")
        .env("GM_HEALTH_ADDR", "127.0.0.1:0")
        // 强制 fail-closed
        .env("RGS_ALLOW_INSECURE_GRPC", "0")
        .env_remove("RGS_TLS_DIR")
        .env("RUST_LOG", "info")
        .timeout(Duration::from_secs(5))
        .output()
        .expect("spawn gm-backend binary");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // 必须 fail (exit != 0)
    assert!(
        !output.status.success(),
        "gm-backend should fail-closed when RGS_TLS_DIR missing; got exit_ok=true, stderr={}",
        stderr
    );
    // stderr 应说明 RGS_TLS_DIR fail-closed 原因
    assert!(
        stderr.contains("RGS_TLS_DIR") || stderr.contains("fail-closed"),
        "stderr should explain RGS_TLS_DIR fail-closed; got: {}",
        stderr
    );
}

/// TBD-08-02 v0.2: mTLS fail-closed,RGS_TLS_DIR 指向不存在路径
#[test]
fn gm_backend_fails_closed_when_tls_dir_path_invalid() {
    let output = Command::cargo_bin("gm-backend")
        .expect("locate gm-backend binary")
        .env("GM_HTTP_ADDR", "127.0.0.1:0")
        .env("GM_HEALTH_ADDR", "127.0.0.1:0")
        .env("RGS_ALLOW_INSECURE_GRPC", "0")
        .env("RGS_TLS_DIR", "/nonexistent/tls/dir/should/fail")
        .env("RUST_LOG", "info")
        .timeout(Duration::from_secs(5))
        .output()
        .expect("spawn gm-backend binary");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "gm-backend should fail when RGS_TLS_DIR points to non-existent path"
    );
    assert!(
        stderr.contains("does not exist") || stderr.contains("fail-closed"),
        "stderr should explain TLS path invalid; got: {}",
        stderr
    );
}

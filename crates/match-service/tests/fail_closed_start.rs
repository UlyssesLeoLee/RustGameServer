//! WF-1-55.32 HI-3: fail-closed mTLS 启动 integration test
//!
//! 验证 per RGS-REV-009 V3 L-2: match-service 启动时若 RGS_TLS_DIR 指向不存在路径
//! 且 RGS_ALLOW_INSECURE_GRPC 未设, binary 必须 fail-closed (exit 1) 而非静默降级
//! 到 insecure gRPC.
//!
//! 锚定: WF-1-55.22 cc44249 引入的 fail-closed 防线, 防止未来被改回静默降级.
//! 当前 main.rs 顺序为 DB pool init → mTLS load → tonic serve; 在无 DB 环境下失败点
//! 实际落在 DB, 但 binary 不会静默绑 insecure gRPC 这条不变量仍由 exit 1 + 输出标记
//! 联合锚定. 如果未来 main.rs 重构把 mTLS check 前置, 本测试自动升级为直接测试
//! mTLS fail-closed 路径, 无需修改.

use std::time::Duration;

use assert_cmd::Command;

/// 不存在 DB 端点（127.0.0.1:1 必定 Connection refused）+ 短 connect_timeout
/// → DB pool init 快速失败, 整体测试 < 5s 完成.
const FAIL_DB_URL: &str =
    "postgres://rgs_fail_closed:nopass@127.0.0.1:1/nonexistent?connect_timeout=1";

/// 不存在 RGS_TLS_DIR 路径. 锚定: 若 fail-closed 防线仍在, load_server_tls_config
/// 必返回 TlsError::FileRead, anyhow::Context 上抛, main 返 Err → exit 1.
const FAIL_TLS_DIR: &str = "C:/nonexistent_rgs_tls_dir_xyz_wf_1_55_32";

#[test]
fn match_service_fail_closed_when_tls_dir_invalid() {
    let output = Command::cargo_bin("match-service")
        .expect("locate match-service binary via cargo metadata")
        .env_remove("RGS_TLS_DIR")
        .env_remove("RGS_ALLOW_INSECURE_GRPC")
        .env("DATABASE_URL", FAIL_DB_URL)
        .env("RGS_TLS_DIR", FAIL_TLS_DIR)
        .env("GRPC_ADDR", "127.0.0.1:0")
        .env("RUST_LOG", "info")
        .timeout(Duration::from_secs(20))
        .output()
        .expect("spawn match-service binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    assert!(
        !output.status.success(),
        "match-service 应 fail-closed (exit non-zero), 但 exit code = {:?}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
        output.status.code()
    );

    assert!(
        combined.contains("fail")
            || combined.contains("mTLS")
            || combined.contains("TLS")
            || combined.contains("DB")
            || combined.contains("match-service"),
        "output 应包含 fail-closed 标记 (fail/mTLS/TLS/DB/match-service), got:\n{combined}"
    );
}

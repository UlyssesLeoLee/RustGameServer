//! rgs-hello 单元测试 (per 9/1 14:15 JST 派工 w8-pt-arc-certgen-hello)
//!
//! 3 测试 (G001~G003),黑盒验证 rgs-hello binary 输出:
//! - G001: stdout 包含 "RGS Rust"
//! - G002: stdout 包含 "OK" (per main.rs println!("RGS Rust 1.98 OK"))
//! - G003: 退出码 = 0

use assert_cmd::Command;
use predicates::prelude::*;

/// G001: rgs-hello stdout 包含 "RGS Rust" 标识
#[test]
fn g001_hello_stdout_contains_rgs_rust() {
    Command::cargo_bin("rgs-hello")
        .unwrap()
        .assert()
        .success()
        .stdout(predicate::str::contains("RGS Rust"));
}

/// G002: rgs-hello stdout 包含 "OK" (per main.rs println! 模板)
#[test]
fn g002_hello_stdout_contains_ok() {
    Command::cargo_bin("rgs-hello")
        .unwrap()
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));
}

/// G003: rgs-hello 退出码 = 0 (无 panic + 无错误)
#[test]
fn g003_hello_exit_code_zero() {
    Command::cargo_bin("rgs-hello")
        .unwrap()
        .assert()
        .success()
        .code(0);
}

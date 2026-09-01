//! rgs-hello 集成测试 (per 9/1 14:15 JST 派工 w8-pt-arc-certgen-hello)
//!
//! 2 集成场景:
//! - H001: rgs-hello 在工作目录 (env CWD) 变化下仍能正常输出 (无依赖资源)
//! - H002: rgs-hello stdout 是单行 + 行尾换行 (per println! 约定)

use std::process::Command as StdCommand;
use tempfile::TempDir;

/// H001: rgs-hello 在不同工作目录下都执行成功 (无任何文件依赖)
#[test]
fn h001_hello_runs_in_arbitrary_cwd() {
    let tmp = TempDir::new().unwrap();
    let output = StdCommand::new(env!("CARGO_BIN_EXE_rgs-hello"))
        .current_dir(tmp.path())
        .output()
        .expect("spawn rgs-hello");
    assert!(output.status.success(), "rgs-hello should exit 0 in temp cwd");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("RGS Rust"), "stdout: {}", stdout);
}

/// H002: rgs-hello stdout 是单行 + 行尾有 \n
#[test]
fn h002_hello_stdout_single_line_with_newline() {
    let output = StdCommand::new(env!("CARGO_BIN_EXE_rgs-hello"))
        .output()
        .expect("spawn rgs-hello");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // println! 会在结尾加 \n
    assert!(stdout.ends_with('\n'), "stdout should end with \\n: {:?}", stdout);
    // 只应该有一个 \n
    let newline_count = stdout.matches('\n').count();
    assert_eq!(newline_count, 1, "stdout should be exactly 1 line, got {} newlines", newline_count);
}

//! 测试 helper 集
//!
//! - `init_tracing()`  初始化 tracing subscriber（test 时输出到 stderr + JSON）
//! - `load_test_env()` 读取 .env.test 文件
//! - `assert_eventually!` async 断言 macro

use std::sync::Once;

static INIT_TRACING: Once = Once::new();

/// 初始化 tracing（多次调用幂等）
pub fn init_tracing() {
    INIT_TRACING.call_once(|| {
        use tracing_subscriber::{fmt, EnvFilter};
        let _ = fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info,rgs=debug")),
            )
            .with_test_writer()
            .try_init();
    });
}

/// 读取 .env.test 文件（不存在则返回空）
pub fn load_test_env() -> std::collections::HashMap<String, String> {
    let mut env = std::collections::HashMap::new();
    if let Ok(content) = std::fs::read_to_string(".env.test") {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                env.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }
    env
}

// for_kv_map clippy lint fix：tests 里只取 key，用 .keys() 而非 .iter()

/// async 断言 macro：poll 直到 condition 为 true 或 timeout
#[macro_export]
macro_rules! assert_eventually {
    ($cond:expr, $timeout_ms:expr) => {{
        let timeout = std::time::Duration::from_millis($timeout_ms);
        let start = std::time::Instant::now();
        loop {
            if $cond {
                break;
            }
            if start.elapsed() > timeout {
                panic!("assert_eventually! timed out after {}ms", $timeout_ms);
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }};
}

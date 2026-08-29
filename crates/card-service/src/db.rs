//! card-service DB 模块 (per WBS v0.5 §2A.5 WF-1-54.4 + DTL-038 §7.1)
//!
//! 5 域模板对齐: DATABASE_URL + PgPool 初始化 + migration runner.
//! 桶 10 增量: card-service 独立 card_db (per ARC-008 5 独立 DB 原则).

use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::Result;

/// 55.45 sqlx-tracing 采样率 (per Q-M-03 答复"建议 PH-1 先 10-20% 采样")
///
/// 行为：
/// - 读 `SQLX_TRACING_SAMPLE_RATIO` env, 范围 [0.0, 1.0]
/// - 默认 0.10 (10%) —— 与 Q-M-03 答复"PH-1 10-20% 采样"对齐
/// - 范围校验: clamp 到 [0.0, 1.0], 非法值 (负数/超 1.0/解析失败) 回落到默认
/// - 容错: env 未设置时也返回默认值 (不 panic)
pub fn sqlx_tracing_sample_ratio() -> f64 {
    const DEFAULT: f64 = 0.10; // 10% (per Q-M-03 答复)
    match std::env::var("SQLX_TRACING_SAMPLE_RATIO") {
        Ok(s) => match s.trim().parse::<f64>() {
            Ok(v) if (0.0..=1.0).contains(&v) => v,
            _ => DEFAULT, // 非法或超范围 → 默认
        },
        Err(_) => DEFAULT, // 未设置 → 默认
    }
}

/// 从 DATABASE_URL 环境变量读连接字符串, 初始化 PgPool
pub async fn pool_from_env() -> Result<PgPool> {
    let url = std::env::var("DATABASE_URL").map_err(|_| {
        crate::Error::Validation("DATABASE_URL env required (per ARC-008 card_db)".to_string())
    })?;
    init_pool(&url).await
}

/// 显式 URL 初始化 PgPool
pub async fn init_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Some(Duration::from_secs(300)))
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// 跑 sqlx migration (migrations/ 目录)
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| crate::Error::Internal(e.into()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // 55.45：env::set_var 是 process-global, 多测试并发会互相干扰
    // 用 Mutex 串行化保证测试稳定
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// 55.45 AC1: 默认采样率 10% (Q-M-03 答复)
    #[test]
    fn sqlx_tracing_sample_ratio_default() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var("SQLX_TRACING_SAMPLE_RATIO");
        let ratio = sqlx_tracing_sample_ratio();
        assert!(
            (ratio - 0.10).abs() < 1e-9,
            "默认 10% 采样率 (Q-M-03 答复)"
        );
    }

    /// 55.45 AC2: 合法范围 [0.0, 1.0] 接受
    #[test]
    fn sqlx_tracing_sample_ratio_valid_range() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("SQLX_TRACING_SAMPLE_RATIO", "0.2");
        let r = sqlx_tracing_sample_ratio();
        assert!((r - 0.2).abs() < 1e-9, "0.2 应被接受 (实际={})", r);
        env::remove_var("SQLX_TRACING_SAMPLE_RATIO");
    }

    /// 55.45 AC3: 非法输入回落默认 (容错)
    #[test]
    fn sqlx_tracing_sample_ratio_invalid_falls_back() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("SQLX_TRACING_SAMPLE_RATIO", "not-a-number");
        let r = sqlx_tracing_sample_ratio();
        assert!((r - 0.10).abs() < 1e-9, "非法值回落默认 0.10");
        env::set_var("SQLX_TRACING_SAMPLE_RATIO", "1.5");
        let r = sqlx_tracing_sample_ratio();
        assert!((r - 0.10).abs() < 1e-9, "超范围回落默认 0.10");
        env::set_var("SQLX_TRACING_SAMPLE_RATIO", "-0.1");
        let r = sqlx_tracing_sample_ratio();
        assert!((r - 0.10).abs() < 1e-9, "负数回落默认 0.10");
        env::remove_var("SQLX_TRACING_SAMPLE_RATIO");
    }
}

//! economy-service DB 模块（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-015 §3）
//!
//! 5 域经济域 DATABASE_URL + PgPool 初始化 + migration runner。
//! 55.45 启用 sqlx-tracing 采样率配置（per RGS-OPEN-QA-001 Q-M-03 + WBS WF-1-55.45）

use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::Result;

/// 55.45 sqlx-tracing 采样率（per Q-M-03 答复"建议 PH-1 先 10-20% 采样"）
///
/// 行为：
/// - 读 `SQLX_TRACING_SAMPLE_RATIO` env，范围 [0.0, 1.0]
/// - 默认 0.10（10%）—— 与 Q-M-03 答复"PH-1 10-20% 采样"对齐
/// - 范围校验：clamp 到 [0.0, 1.0]，非法值（负数/超 1.0/解析失败）回落到默认
/// - 容错：env 未设置时也返回默认值（不 panic）
pub fn sqlx_tracing_sample_ratio() -> f64 {
    const DEFAULT: f64 = 0.10; // 10%（per Q-M-03 答复）
    match std::env::var("SQLX_TRACING_SAMPLE_RATIO") {
        Ok(s) => match s.trim().parse::<f64>() {
            Ok(v) if (0.0..=1.0).contains(&v) => v,
            _ => DEFAULT,
        },
        Err(_) => DEFAULT,
    }
}

pub async fn pool_from_env() -> Result<PgPool> {
    let url = std::env::var("DATABASE_URL").map_err(|_| {
        crate::Error::Validation("DATABASE_URL env required (per ARC-008 economy_db)".to_string())
    })?;
    init_pool(&url).await
}

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

    // 55.45：env::set_var 是 process-global，多测试并发会互相干扰
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// 55.45 AC1：默认采样率 10%（Q-M-03 答复）
    #[test]
    fn sqlx_tracing_sample_ratio_default() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var("SQLX_TRACING_SAMPLE_RATIO");
        let ratio = sqlx_tracing_sample_ratio();
        assert!((ratio - 0.10).abs() < 1e-9, "默认 10% 采样率");
    }

    /// 55.45 AC2：合法范围接受
    #[test]
    fn sqlx_tracing_sample_ratio_valid_range() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("SQLX_TRACING_SAMPLE_RATIO", "0.2");
        assert!((sqlx_tracing_sample_ratio() - 0.2).abs() < 1e-9);
        env::remove_var("SQLX_TRACING_SAMPLE_RATIO");
    }

    /// 55.45 AC3：非法输入回落默认
    #[test]
    fn sqlx_tracing_sample_ratio_invalid_falls_back() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("SQLX_TRACING_SAMPLE_RATIO", "garbage");
        assert!((sqlx_tracing_sample_ratio() - 0.10).abs() < 1e-9);
        env::remove_var("SQLX_TRACING_SAMPLE_RATIO");
    }
}

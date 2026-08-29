//! replay-service DB 模块 (per RGS-DTL-038 §3 DEC-038-03 + 桶 13)
//!
//! 5 域模板对齐: DATABASE_URL + PgPool 初始化 + migration runner.
//! 桶 13 增量: replay-service 独立 replay_db (per ARC-008 卡牌游戏 6 域独立 DB).

use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::Result;

/// 从 DATABASE_URL 环境变量读连接字符串, 初始化 PgPool
pub async fn pool_from_env() -> Result<PgPool> {
    let url = std::env::var("DATABASE_URL").map_err(|_| {
        crate::Error::Validation("DATABASE_URL env required (per ARC-008 replay_db)".to_string())
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

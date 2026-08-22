//! player-service DB 模块（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-018 §3）
//!
//! 5 域玩家域 DATABASE_URL + PgPool 初始化 + migration runner。
//! 54.4 适配；54.6 Repository impl 在 trait 上加 sqlx 实现。

use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::Result;

/// 从 DATABASE_URL 环境变量读连接字符串，初始化 PgPool
pub async fn pool_from_env() -> Result<PgPool> {
    let url = std::env::var("DATABASE_URL").map_err(|_| {
        crate::Error::Validation("DATABASE_URL env required (per ARC-008 player_db)".to_string())
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

/// 跑 sqlx migration（migrations/ 目录）
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| crate::Error::Internal(e.into()))?;
    Ok(())
}

//! match-service DB 模块（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-016 §3）
//!
//! 5 域匹配域 DATABASE_URL + PgPool 初始化 + migration runner。

use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::Result;

pub async fn pool_from_env() -> Result<PgPool> {
    let url = std::env::var("DATABASE_URL").map_err(|_| {
        crate::Error::Validation("DATABASE_URL env required (per ARC-008 match_db)".to_string())
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

//! match-service 域 Repository trait
//!
//! 54.1 占位：1 个 find_by_id + 1 个 save trait 方法。
//! 实际实现待 WF-1-54.6 sqlx Repository。

use async_trait::async_trait;
use uuid::Uuid;

use crate::entity::Match;
use crate::Result;

/// match-service 域 Repository trait
#[async_trait]
pub trait MatchRepository: Send + Sync {
    /// 按 id 查询
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Match>>;

    /// 保存
    async fn save(&self, entity: &Match) -> Result<Match>;
}

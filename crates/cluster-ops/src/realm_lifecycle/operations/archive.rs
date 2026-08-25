//! Archive 操作器骨架（per RGS-DTL-042 §5.2 + FR-LCM-080~085）
//!
//! WF-1-2066 M-2066.9 骨架
//!
//! 范围（per FR-LCM-080~085）：
//! - 归档（archive）：退场后数据归档 + 冷热分层 + N+2 存储冗余
//! - **不删除数据**（per FR-LCM-081 硬约束；归档仅迁移存储位置）
//! - GDPR "被遗忘权" 删除通路走 `admin_db.operation_audit` 双层审计
//!   （per NFR-SE-010 既有约束的合规例外）
//! - 冷热分层阈值：3 年热 + 10 年冷（per TBD-DTL-042-01 → M-2074.1 实测填）
//! - Saga 3 步执行（per RGS-DTL-042 §6 归档 3 步）
//!   1. 冷热分层判定（按 archive_policy.threshold 阈值）
//!   2. N+2 存储冗余（per RSK-LCM-005 缓解；3 副本写入）
//!   3. 归档索引写入 + 双层审计
//!
//! 骨架阶段：仅 trait + 冷热分层占位字段
//! 等待 L4 #2067 Saga + L4 #2068 6 表 + L4 #2074 归档冷热分层 + N+2 + GDPR

use async_trait::async_trait;
use uuid::Uuid;

use super::{not_implemented_skeleton, validate_request};
use crate::realm_lifecycle::error::LcmResult;
use crate::realm_lifecycle::service::{RealmLifecycleOperator, RealmLifecycleStage};

/// 冷热分层（per TBD-DTL-042-01 + M-2074.1 实测）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveTier {
    /// 热数据：3 年内
    Hot,
    /// 冷数据：3~13 年
    Cold,
}

impl ArchiveTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hot => "hot",
            Self::Cold => "cold",
        }
    }
}

/// 归档策略骨架（per M-2068.3 archive_policy DDL）
#[derive(Debug, Default, Clone)]
pub struct ArchivePolicy {
    pub target_realm_id: String,
    /// 冷热分层阈值（天数，per TBD-DTL-042-01：3 年 = 1095 天）
    pub hot_threshold_days: u32,
    /// 副本数（per RSK-LCM-005 缓解：N+2 = 3 副本）
    pub replica_count: u8,
    /// 冷热分层（骨架阶段占位；M-2074.1 实测填）
    pub tier: Option<ArchiveTier>,
}

/// Archive 操作器（per FR-LCM-080~085）
#[derive(Debug, Default, Clone)]
pub struct ArchiveOperator;

impl ArchiveOperator {
    pub fn new() -> Self {
        Self
    }

    pub fn new_for_skeleton() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RealmLifecycleOperator for ArchiveOperator {
    fn name(&self) -> &'static str {
        "archive"
    }

    fn stage(&self) -> RealmLifecycleStage {
        RealmLifecycleStage::Archive
    }

    async fn execute(
        &self,
        request_id: Uuid,
        realm_id: &str,
        _operator_id: Uuid,
        _approval_ref: Option<&str>,
    ) -> LcmResult<Uuid> {
        validate_request(request_id, realm_id)?;
        // 骨架阶段占位
        // 后续 L4 #2067 Saga 接入后将替换为：
        //   1. 加载 archive_policy + 判定冷热分层（per TBD-DTL-042-01）
        //   2. 构建 Archive Saga 3 步
        //   3. 调 SagaOrchestrator.execute
        //   4. N+2 副本写入（per RSK-LCM-005 缓解；3 副本跨可用区）
        //   5. GDPR 双层审计写入 admin_db.operation_audit（per NFR-SE-010）
        Err(not_implemented_skeleton(
            "ArchiveOperator",
            "M-2067.2 Archive Saga 3 步 + M-2068.3 archive_policy DDL + M-2074 归档冷热分层 + N+2 + GDPR",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn archive_execute_returns_not_implemented() {
        let op = ArchiveOperator::new_for_skeleton();
        let err = op
            .execute(Uuid::new_v4(), "realm-001", Uuid::new_v4(), None)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("ArchiveOperator"));
    }

    #[test]
    fn archive_operator_metadata() {
        let op = ArchiveOperator::new_for_skeleton();
        assert_eq!(op.name(), "archive");
        assert_eq!(op.stage(), RealmLifecycleStage::Archive);
    }

    #[test]
    fn archive_tier_str_round_trip() {
        assert_eq!(ArchiveTier::Hot.as_str(), "hot");
        assert_eq!(ArchiveTier::Cold.as_str(), "cold");
    }

    #[test]
    fn archive_policy_n_plus_2_default() {
        let policy = ArchivePolicy {
            target_realm_id: "realm-001".to_string(),
            hot_threshold_days: 1095, // 3 年
            replica_count: 3,        // N+2
            tier: Some(ArchiveTier::Hot),
        };
        assert_eq!(policy.hot_threshold_days, 1095);
        assert_eq!(policy.replica_count, 3);
    }

    #[test]
    fn archive_is_terminal_stage() {
        // Archive 是唯一终态（per FR-LCM-081）
        let op = ArchiveOperator::new_for_skeleton();
        assert!(op.stage().is_terminal());
    }
}

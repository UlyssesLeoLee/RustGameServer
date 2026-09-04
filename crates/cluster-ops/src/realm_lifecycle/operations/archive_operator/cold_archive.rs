//! ArchiveOperator::cold_archive_to_object_store helper (P1-3 第二步, 2026-09-05)
//!
//! 冷归档 + N+2 副本验证
//! per RGS-DTL-042 §3 M-2074.2
//!
//! **拆分原因**: impl ArchiveOperator 624 行单文件, 拆 4 个超大 method body 到 helper,
//! archive.rs 主 impl 留 8 行 forward 委派, 总大小 1443 → ~700 行.
//! **pub API 不变**, ArchiveOperator::cold_archive_to_object_store(...) 调用, 内部委托 helper.

use super::super::archive::*;

/// 冷归档 + N+2 副本验证
    pub(super) async fn cold_archive_to_object_store(
    ) -> LcmResult<PutObjectResult> {
        // RSK-LCM-005 硬约束: 业务代码**不**允许写入 N+1 归档
        if policy.storage_redundancy == StorageRedundancy::NPlus1 {
            return Err(LcmError::ColdArchiveFailed {
                realm_id: policy.target_realm_id.clone(),
                replica_count: 0,
                required: StorageRedundancy::NPlus2.required_replica_count(),
                reason: "policy.storage_redundancy = N+1 不允许冷归档（per RSK-LCM-005）".to_string(),
            });
        }

        let required = policy.storage_redundancy.required_replica_count();
        let start = Instant::now();

        // 步骤 1: 写入对象（主副本）
        let put = self
            .object_storage
            .put_object(bucket, key, bytes)
            .await?;

        // 步骤 2: 验证副本数（per RSK-LCM-005 N+2 = 3 副本）
        let replicas = self.object_storage.list_replicas(bucket, key).await?;
        let actual = replicas.len() as u8;

        if actual < required {
            // 副本数不达标 → 冷归档失败，**不**回退（per FR-LCM-081 归档不删数据）
            // 仅记录错误，等待运营手工补副本
            return Err(LcmError::ColdArchiveFailed {
                realm_id: policy.target_realm_id.clone(),
                replica_count: actual,
                required,
                reason: format!(
                    "副本数 {actual} < {required}（per RSK-LCM-005 缓解未生效）"
                ),
            });
        }

        // 步骤 3: 记录 Saga 步骤耗时（M-2074.5 同源指标）
        metrics::observe_saga_step_duration(
            crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
            ArchiveSagaStep::ColdArchive.step_name(),
            start.elapsed().as_secs_f64(),
        );

        Ok(put)
    }

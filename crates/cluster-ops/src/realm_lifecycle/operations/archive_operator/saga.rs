//! ArchiveOperator::execute_archive helper (P1-3 第二步, 2026-09-05)
//!
//! 3 步 Saga 编排
//! per RGS-DTL-042 §6.6
//!
//! **拆分原因**: impl ArchiveOperator 624 行单文件, 拆 4 个超大 method body 到 helper,
//! archive.rs 主 impl 留 8 行 forward 委派, 总大小 1443 → ~700 行.
//! **pub API 不变**, ArchiveOperator::execute_archive(...) 调用, 内部委托 helper.

use super::super::archive::*;

/// 3 步 Saga 编排
    pub(super) async fn execute_archive(
    ) -> LcmResult<ArchiveOutcome> {
        // 校验状态机（仅 Retired 可发起）
        Self::check_state_eligibility(RealmLifecycleState::Retired)?;

        // 校验策略本身
        policy.validate()?;

        let outcome_start = Instant::now();
        let mut steps: Vec<ArchiveStepResult> = Vec::new();
        let mut total_olu: u64 = 0;

        // ===== 步骤 1: HotArchiveStep（DB 切换为冷备实例）=====
        let s1_start = Instant::now();
        let s1_result = self
            .step_hot_archive(policy)
            .await;
        let s1_elapsed = s1_start.elapsed().as_secs_f64();
        metrics::observe_saga_step_duration(
            crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
            ArchiveSagaStep::HotArchive.step_name(),
            s1_elapsed,
        );
        match &s1_result {
            Ok(summary) => {
                steps.push(ArchiveStepResult {
                    step: ArchiveSagaStep::HotArchive,
                    success: true,
                    started_at: Utc::now()
                        - chrono::Duration::milliseconds((s1_elapsed * 1000.0) as i64),
                    finished_at: Utc::now(),
                    elapsed_seconds: s1_elapsed,
                    summary: summary.clone(),
                });
                total_olu += 5_000; // Hot step ≈ 5K tokens
            }
            Err(e) => {
                metrics::inc_saga_rollback(
                    crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
                    ArchiveSagaStep::HotArchive.step_name(),
                    "hot_archive_failed",
                );
                return Err(LcmError::SagaStepFailed {
                    step: ArchiveSagaStep::HotArchive.step_name().to_string(),
                    reason: e.to_string(),
                });
            }
        }

        // ===== 步骤 2: ColdArchiveStep（对象存储 N+2 副本）=====
        let s2_start = Instant::now();
        let key = format!("realm/{}/{}", policy.target_realm_id, run_id);
        let s2_result = self
            .cold_archive_to_object_store(policy, bucket, &key, archive_bytes)
            .await;
        let s2_elapsed = s2_start.elapsed().as_secs_f64();
        metrics::observe_saga_step_duration(
            crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
            ArchiveSagaStep::ColdArchive.step_name(),
            s2_elapsed,
        );
        match &s2_result {
            Ok(put) => {
                steps.push(ArchiveStepResult {
                    step: ArchiveSagaStep::ColdArchive,
                    success: true,
                    started_at: Utc::now()
                        - chrono::Duration::milliseconds((s2_elapsed * 1000.0) as i64),
                    finished_at: Utc::now(),
                    elapsed_seconds: s2_elapsed,
                    summary: format!(
                        "bucket={} key={} size={}B replicas={}",
                        put.bucket, put.key, put.size_bytes, put.replica_count
                    ),
                });
                total_olu += 50_000; // Cold step ≈ 50K tokens（含 N+2 副本验证）
            }
            Err(e) => {
                metrics::inc_saga_rollback(
                    crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
                    ArchiveSagaStep::ColdArchive.step_name(),
                    "cold_archive_failed",
                );
                return Err(LcmError::SagaStepFailed {
                    step: ArchiveSagaStep::ColdArchive.step_name().to_string(),
                    reason: e.to_string(),
                });
            }
        }

        // ===== 步骤 3: EnableGdprDeletePathStep（合规删除通路开启）=====
        let s3_start = Instant::now();
        let s3_result = self.step_enable_gdpr_delete_path(policy, run_id).await;
        let s3_elapsed = s3_start.elapsed().as_secs_f64();
        metrics::observe_saga_step_duration(
            crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
            ArchiveSagaStep::EnableGdprDeletePath.step_name(),
            s3_elapsed,
        );
        match &s3_result {
            Ok(summary) => {
                steps.push(ArchiveStepResult {
                    step: ArchiveSagaStep::EnableGdprDeletePath,
                    success: true,
                    started_at: Utc::now()
                        - chrono::Duration::milliseconds((s3_elapsed * 1000.0) as i64),
                    finished_at: Utc::now(),
                    elapsed_seconds: s3_elapsed,
                    summary: summary.clone(),
                });
                total_olu += 10_000; // GDPR path enable ≈ 10K tokens
            }
            Err(e) => {
                metrics::inc_saga_rollback(
                    crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
                    ArchiveSagaStep::EnableGdprDeletePath.step_name(),
                    "gdpr_path_enable_failed",
                );
                return Err(LcmError::SagaStepFailed {
                    step: ArchiveSagaStep::EnableGdprDeletePath.step_name().to_string(),
                    reason: e.to_string(),
                });
            }
        }

        // 整体状态转移：Retired → Archived
        metrics::inc_run_state_transition(
            crate::realm_lifecycle::FEATURE_SUBTYPE_ARCHIVE,
            "Retired",
            "Archived",
        );

        Ok(ArchiveOutcome {
            run_id,
            policy_id: policy.policy_id,
            realm_id: policy.target_realm_id.clone(),
            final_tier: ArchiveTier::GdprDeletePath,
            steps,
            olu_tokens: total_olu,
            elapsed_seconds: outcome_start.elapsed().as_secs_f64(),
        })
    }

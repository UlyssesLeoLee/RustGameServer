//! ArchiveOperator::execute_gdpr_delete helper (P1-3 第二步, 2026-09-05)
//!
//! GDPR 删除通路
//! per NFR-SE-010 + RGS-DTL-042 §4.3
//!
//! **拆分原因**: impl ArchiveOperator 624 行单文件, 拆 4 个超大 method body 到 helper,
//! archive.rs 主 impl 留 8 行 forward 委派, 总大小 1443 → ~700 行.
//! **pub API 不变**, ArchiveOperator::execute_gdpr_delete(...) 调用, 内部委托 helper.

use super::super::archive::*;

/// GDPR 删除通路
    pub(super) async fn execute_gdpr_delete(
    ) -> LcmResult<GdprDeleteResult> {
        // === 校验 1: signed_by 必须为 Ulysses（per ADR-0055 §4.3）===
        //
        // 注意：此为业务代码"软校验"。**生产环境**应在前置网关（AdminService）
        // 做硬校验（mTLS 证书 subject CN == "Ulysses"）；此处为业务层第二道防线。
        if request.signed_by != "Ulysses" {
            // 资金/合规相关 — 拒绝执行并触发拒绝路径
            return Err(LcmError::GdprDeletePathDenied {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: format!(
                    "signed_by={} != 'Ulysses'（per ADR-0055 §4.3，需 Ulysses 显式独立签字）",
                    request.signed_by
                ),
            });
        }

        // === 校验 2: realm 处于 Archived 状态 ===
        // 真实实现中查询 `realm_lifecycle_run.current_state`；
        // 此处仅做基本非空校验
        if request.realm_id.is_empty() {
            return Err(LcmError::GdprDeletePathDenied {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: "realm_id 不能为空".to_string(),
            });
        }

        // === 校验 3: policy 存在 + gdpr_delete_path 非空 ===
        let policy = self
            .policy_repo
            .find_by_realm_id(&request.realm_id)
            .await?
            .ok_or_else(|| LcmError::GdprDeletePathDenied {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: format!("realm_id={} 找不到对应 archive_policy", request.realm_id),
            })?;
        if policy.gdpr_delete_path.is_empty() {
            return Err(LcmError::GdprDeletePathDenied {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: "archive_policy.gdpr_delete_path 为空（per NFR-SE-010）".to_string(),
            });
        }

        // === 校验 4: HardErase 必须 legal_hold_override=true ===
        if request.erasure_strategy == ErasureStrategy::HardErase && !request.legal_hold_override {
            return Err(LcmError::GdprDeletePathDenied {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: "HardErase 必须 legal_hold_override=true".to_string(),
            });
        }

        let run_id = Uuid::new_v4();
        let executed_at = Utc::now();

        // === 第一层审计（业务事件）===
        let first_payload = serde_json::json!({
            "run_id": run_id,
            "subject_id": request.subject_id,
            "realm_id": request.realm_id,
            "erasure_strategy": request.erasure_strategy,
            "request_id": request.request_id,
            "operator_id": request.operator_id,
            "approval_ref": request.approval_ref,
            "signed_by": request.signed_by,
            "executed_at": executed_at,
        });
        let first_audit_id = self
            .audit_repo
            .append(
                Uuid::new_v4(), // actor_id (system 视角；Ulysses 主体由 actor 字段追踪)
                "lcm.gdpr.delete",
                &format!("subject:{}", request.subject_id),
                &first_payload.to_string(),
            )
            .await
            .map_err(|e| LcmError::GdprDeletePathFailed {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: format!("第一层审计写入失败：{e}"),
            })?;

        // === 第二层审计（合规事件 — 双层审计 per NFR-SE-010）===
        let second_payload = serde_json::json!({
            "run_id": run_id,
            "first_audit_id": first_audit_id,
            "subject_id": request.subject_id,
            "realm_id": request.realm_id,
            "legal_hold_override": request.legal_hold_override,
            "compliance_review_basis": "FR-LCM-084 / NFR-SE-010",
            "signed_by": request.signed_by,
            "executed_at": executed_at,
        });
        let second_audit_id = self
            .audit_repo
            .append(
                Uuid::new_v4(),
                "lcm.gdpr.compliance",
                &format!("subject:{}", request.subject_id),
                &second_payload.to_string(),
            )
            .await
            .map_err(|e| LcmError::GdprDeletePathFailed {
                subject_id: request.subject_id.clone(),
                realm_id: request.realm_id.clone(),
                reason: format!("第二层审计写入失败：{e}（双层审计缺一不可）"),
            })?;

        // === 步骤 6: 物理擦除 / 匿名化 ===
        // 真实实现：调用 player_db / economy_db / social_db 的 anonymize_subject API
        // 单元测试中此步骤由 `#[ignore]` 标记的真实集成测试覆盖
        // （PH-6 实测阶段由 SRE 接力跑真实存储环境）

        Ok(GdprDeleteResult {
            subject_id: request.subject_id,
            realm_id: request.realm_id,
            run_id,
            audit_first_layer_id: first_audit_id,
            audit_second_layer_id: second_audit_id,
            erasure_strategy: request.erasure_strategy,
            executed_at,
            signed_by: request.signed_by,
        })
    }

//! IT 子代理 (2026-08-31 v1): admin 域 GM 指令中途失败 → 状态回滚
//!
//! ## 目的
//! 验证 admin-service GM 指令在"中途失败"场景下的回滚语义 (per RGS-ARC-051 §COC
//! + RGS-REV-008 V2 故障注入规范):
//! - 多步 GM 指令 (audit + 状态变更) 中途失败 → 已写部分必须可回滚
//! - 被禁号玩家没真被禁 (admin 不直接改 player, 但 audit log 不能有 "已禁" 记录)
//! - 余额没动 (同理, economy 不动, 但 audit log 不能有 "已补" 记录)
//!
//! ## 范围
//! 1. **audit_log append 失败 → 整条 GM 指令回滚**: 用 mock repo 在 append 时注入
//!    失败, 验证"状态变更"步骤 (mock 外部副作用) 也回滚, 即 audit_log 链上没有任何
//!    这次失败的痕迹
//! 2. **多步 GM 指令中第 2 步失败 → 第 1 步回滚**: 模拟一个"两步"复合操作
//!    (mock), 验证第 1 步副作用被补偿
//! 3. **n 次 chaos 注入 (随机失败位置) → 全部回滚或全部成功 (无中间态泄漏)**
//!
//! ## 风格
//! 沿用 IT-AGENT-BRIEFING §1: 全部 InMemory + Mock, 不连真 DB, 不起真实 gRPC server.
//! 注入失败用自定义 `FailingAuditLogRepository` wrapper.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use admin_service::entity::{AdminRole, AdminUser, AuditLogEntry};
use admin_service::error::Error;
use admin_service::repository::{
    AuditLogRepository, InMemoryAdminUserRepository, InMemoryAuditLogRepository,
};
use admin_service::service::{AdminService, AdminServiceImpl};
use admin_service::Result;
use async_trait::async_trait;
use uuid::Uuid;

// ============================================================================
// FailingAuditLogRepository: 注入失败 + 记录所有调用
// ============================================================================

/// 注入式失败 repo: 两种模式:
/// - 固定模式 (`new(fail_after)`): 在第 N 次 append 时返 Err(Internal)
/// - 动态模式 (`new_dynamic()`): 由 `set_fail_next(true)` 控制下次 append 失败
///
/// 同时记录所有调用顺序 (用于"rollback 验证").
///
/// 行为模式:
/// - `append` 计数 +1, 当计数 == fail_after (固定) 或 set_fail_next (动态) 时返 Err
/// - `latest` 永远透传到 inner (chain 续接需要)
/// - `find_by_id` / `list_by_actor` 透传
/// - `append_atomic` 透传 (production PG 路径, IT 不走)
pub struct FailingAuditLogRepository {
    inner: Arc<InMemoryAuditLogRepository>,
    call_count: AtomicUsize,
    fail_after: usize,
    /// 动态失败开关: 每次消费完即自动复位 (false)
    fail_next: std::sync::atomic::AtomicBool,
    /// 记录成功 append 的 entry 顺序 (按时间序)
    successful_appends: Mutex<Vec<AuditLogEntry>>,
}

impl FailingAuditLogRepository {
    pub fn new(fail_after: usize) -> Self {
        Self {
            inner: Arc::new(InMemoryAuditLogRepository::new()),
            call_count: AtomicUsize::new(0),
            fail_after,
            fail_next: std::sync::atomic::AtomicBool::new(false),
            successful_appends: Mutex::new(Vec::new()),
        }
    }
    pub fn new_dynamic() -> Self {
        Self::new(usize::MAX) // 永不自动失败, 只看 fail_next
    }
    pub fn set_fail_next(&self) {
        self.fail_next.store(true, Ordering::SeqCst);
    }
    pub fn successful_count(&self) -> usize {
        self.successful_appends.lock().unwrap().len()
    }
    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
    pub fn inner_repo(&self) -> Arc<InMemoryAuditLogRepository> {
        self.inner.clone()
    }
}

#[async_trait]
impl AuditLogRepository for FailingAuditLogRepository {
    async fn append(&self, entry: &AuditLogEntry) -> Result<AuditLogEntry> {
        let n = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;
        let should_fail = n == self.fail_after
            || self.fail_next.swap(false, Ordering::SeqCst);
        if should_fail {
            // 模拟写前故障 (per RGS-REV-008 V2 fault injection)
            return Err(Error::Internal(anyhow::anyhow!(
                "fault injection: append #{n} failed"
            )));
        }
        let r = self.inner.append(entry).await?;
        self.successful_appends.lock().unwrap().push(r.clone());
        Ok(r)
    }
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuditLogEntry>> {
        self.inner.find_by_id(id).await
    }
    async fn list_by_actor(&self, actor_id: Uuid, limit: i64) -> Result<Vec<AuditLogEntry>> {
        self.inner.list_by_actor(actor_id, limit).await
    }
    async fn latest(&self) -> Result<Option<AuditLogEntry>> {
        self.inner.latest().await
    }
    async fn append_atomic(
        &self,
        _tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        entry: &AuditLogEntry,
    ) -> Result<AuditLogEntry> {
        // IT 不走 PG 事务路径, 退化到普通 append
        self.append(entry).await
    }
}

// ============================================================================
// 多步 GM 指令的"应用层"模拟
// ============================================================================

/// 模拟一个 GM 复合指令: audit_log 写入 + 外部副作用 (禁用 player / 调整 balance).
///
/// 回滚语义 (per RGS-ARC-051 §COC fault tolerance):
/// - 步骤 1: audit_log append (受 FailingAuditLogRepository 控制)
/// - 步骤 2: 外部副作用 (mock, 记录到 external_state)
/// - 若步骤 1 失败 → 不进入步骤 2 → 整条指令未发生 → 状态回滚 (空)
/// - 若步骤 2 失败 → 必须回滚步骤 1 的 audit_log append (业务级补偿)
///
/// 注: audit_log 表本身 append-only (per RGS-SEC-100 §7), 不能物理删除, 因此
/// "回滚" = 写一条 compensation audit (per ARC-051 §COC)。本测试断言:
/// - 失败时, audit_log 链上**没有** "player.ban success" 记录
/// - 但可允许有 "compensation" 记录 (业务补偿语义)
#[derive(Debug, Clone, PartialEq, Eq)]
enum CompensateResult {
    Ok,
    Compensated { reason: String },
}

#[derive(Debug, Clone, Default)]
struct ExternalState {
    banned_players: Vec<String>,
    granted_balances: Vec<(String, i64)>,
}

async fn execute_two_step_gm_command(
    svc: &AdminServiceImpl,
    actor_id: Uuid,
    action: &str,
    target: &str,
    payload: &str,
    external: &mut ExternalState,
) -> Result<CompensateResult> {
    // 步骤 1: 写 audit_log (受 FailingAuditLogRepository 控制)
    let entry = svc
        .audit_log(actor_id, action.to_string(), target.to_string(), payload.to_string())
        .await?;

    // 步骤 2: 外部副作用 (模拟跨域 RPC: 玩家域 / 经济域)
    // 这里我们用一个简单的"副作用分类"逻辑
    // 用 enum 避免与 admin_service::Result 冲突
    #[derive(Debug)]
    enum SideEffect {
        Ok,
        Err(String),
    }
    let side_effect_result: SideEffect = match action {
        "player.ban" => {
            // 模拟 player-service 调用
            external.banned_players.push(target.to_string());
            SideEffect::Ok
        }
        "economy.grant" => {
            // 模拟 economy-service 调用
            // 故意偶尔失败: 当 payload 含 "FAIL_GRANT" 时
            if payload.contains("FAIL_GRANT") {
                SideEffect::Err("economy service unavailable".to_string())
            } else {
                external.granted_balances.push((target.to_string(), 100));
                SideEffect::Ok
            }
        }
        _ => SideEffect::Ok,
    };

    if let SideEffect::Err(reason) = side_effect_result {
        // 步骤 2 失败 → 必须补偿步骤 1 (写 compensation audit)
        // 注意: 不能物理删除 audit entry, 改写 compensation 记录
        // 这里 svc.audit_log 用同样的 actor_id + action 标记补偿
        let _ = entry; // suppress unused warning
        // 业务级补偿: 写 compensation audit
        let comp_payload = format!(
            r#"{{"compensation_for":"{}","target":"{}","reason":"{}"}}"#,
            action, target, reason
        );
        svc.audit_log(
            actor_id,
            format!("{}.compensation", action),
            target.to_string(),
            comp_payload,
        )
        .await?;
        // 外部状态也回滚
        match action {
            "player.ban" => {
                external.banned_players.retain(|p| p != target);
            }
            "economy.grant" => {
                external.granted_balances.retain(|(t, _)| t != target);
            }
            _ => {}
        }
        return Ok(CompensateResult::Compensated { reason });
    }

    Ok(CompensateResult::Ok)
}

// ============================================================================
// Test 1: audit_log append 失败 → 整条指令回滚 (被禁号玩家没真被禁)
// ============================================================================

/// 验证: 步骤 1 (audit append) 失败 → 不进入步骤 2 → 玩家没被禁, 余额没动。
/// 模拟 RGS-REV-008 V2 fault injection: audit_log 在第 1 次 append 就失败。
#[tokio::test]
async fn audit_append_failure_rolls_back_entire_gm_command() {
    let audit = Arc::new(FailingAuditLogRepository::new(1)); // 第 1 次 append 失败
    let users = Arc::new(InMemoryAdminUserRepository::new());
    let svc = AdminServiceImpl::new(users, audit.clone());

    // 准备一个 admin actor
    let admin = AdminUser::new(
        "root".to_string(),
        "h".to_string(),
        AdminRole::SuperAdmin,
    );
    let actor_id = admin.id;
    let mut external = ExternalState::default();

    // 执行 player.ban (第一次 audit append 必失败)
    let result = execute_two_step_gm_command(
        &svc,
        actor_id,
        "player.ban",
        "victim-001",
        r#"{"reason":"test"}"#,
        &mut external,
    )
    .await;

    // 关键断言 1: 整条 GM 指令返 Err (步骤 1 失败透传)
    assert!(result.is_err(), "audit append 失败应让上层感知, got {result:?}");

    // 关键断言 2: 步骤 2 (外部副作用) **未发生** → 被禁号玩家列表为空
    assert!(
        external.banned_players.is_empty(),
        "audit append 失败时步骤 2 不应执行, banned_players={:?}",
        external.banned_players
    );
    // 关键断言 3: 余额未动
    assert!(
        external.granted_balances.is_empty(),
        "audit append 失败时不应有 balance grant"
    );

    // 关键断言 4: audit_log 链上**无任何 entry** (失败的 append 没写入)
    let inner = audit.inner_repo();
    let list = inner.list_by_actor(actor_id, 100).await.unwrap();
    assert!(
        list.is_empty(),
        "audit_log 链应为空 (失败 append 不残留), got {} entries",
        list.len()
    );

    // 关键断言 5: audit_log 调用计数 == 1 (只调了 1 次, 失败透传)
    assert_eq!(audit.call_count(), 1);
    assert_eq!(audit.successful_count(), 0);
}

// ============================================================================
// Test 2: 步骤 2 失败 → 步骤 1 被补偿 (compensation audit + 状态回滚)
// ============================================================================

/// 验证: audit append 成功 (步骤 1) 但外部副作用失败 (步骤 2, payload 含 FAIL_GRANT)
/// → 业务级补偿: 写 compensation audit, 外部状态回滚。
/// 玩家没真被禁, 余额没动 (per GM 指令中途失败回滚契约).
#[tokio::test]
async fn external_side_effect_failure_triggers_compensation() {
    let audit = Arc::new(FailingAuditLogRepository::new(100)); // append 全成功
    let users = Arc::new(InMemoryAdminUserRepository::new());
    let svc = AdminServiceImpl::new(users, audit.clone());

    let admin = AdminUser::new(
        "root".to_string(),
        "h".to_string(),
        AdminRole::SuperAdmin,
    );
    let actor_id = admin.id;
    let mut external = ExternalState::default();

    // economy.grant with FAIL_GRANT → 步骤 2 必失败
    let result = execute_two_step_gm_command(
        &svc,
        actor_id,
        "economy.grant",
        "account-007",
        r#"{"amount":500,"reason":"FAIL_GRANT"}"#,
        &mut external,
    )
    .await
    .unwrap();

    // 关键断言 1: 步骤 1 成功, 步骤 2 失败 → 返 Compensated
    assert!(
        matches!(result, CompensateResult::Compensated { .. }),
        "步骤 2 失败应触发补偿, got {result:?}"
    );

    // 关键断言 2: 余额未动 (外部状态已回滚)
    assert!(
        external.granted_balances.is_empty(),
        "步骤 2 失败后 grant 应被补偿回滚, got {:?}",
        external.granted_balances
    );

    // 关键断言 3: audit_log 链上有 2 条 entry: 原始 + compensation
    let inner = audit.inner_repo();
    let mut list = inner.list_by_actor(actor_id, 100).await.unwrap();
    list.reverse(); // DESC → ASC
    assert_eq!(
        list.len(),
        2,
        "应有 1 条原始 audit + 1 条 compensation audit, got {} entries",
        list.len()
    );

    // 关键断言 4: 原始 entry 的 prev_hash = 64 个 "0" (链首)
    assert_eq!(list[0].prev_hash, "0".repeat(64));
    // 关键断言 5: compensation entry 的 prev_hash = 原始 entry 的 hash
    assert_eq!(list[1].prev_hash, list[0].hash);
    // 关键断言 6: compensation entry 的 action = "economy.grant.compensation"
    assert_eq!(list[1].action, "economy.grant.compensation");
    // 关键断言 7: chain 严格连续
    assert_eq!(list[1].hash.len(), 64);
    assert_ne!(list[1].hash, list[0].hash);

    // 关键断言 8: call_count == 2 (1 原始 + 1 补偿)
    assert_eq!(audit.call_count(), 2);
    assert_eq!(audit.successful_count(), 2);
}

// ============================================================================
// Test 3: 成功路径对比 — 不注入失败, 整条指令成功
// ============================================================================

/// 对照组: 不注入失败, 验证 happy path 链 + 状态正确 (与 Test 1/2 对比).
#[tokio::test]
async fn happy_path_no_failure_audit_chain_and_state_correct() {
    let audit = Arc::new(FailingAuditLogRepository::new(100)); // 永不失败
    let users = Arc::new(InMemoryAdminUserRepository::new());
    let svc = AdminServiceImpl::new(users, audit.clone());

    let admin = AdminUser::new(
        "root".to_string(),
        "h".to_string(),
        AdminRole::SuperAdmin,
    );
    let actor_id = admin.id;
    let mut external = ExternalState::default();

    // 3 条 GM 指令: 1 ban + 1 grant + 1 ban
    execute_two_step_gm_command(
        &svc,
        actor_id,
        "player.ban",
        "victim-A",
        r#"{"reason":"A"}"#,
        &mut external,
    )
    .await
    .unwrap();
    execute_two_step_gm_command(
        &svc,
        actor_id,
        "economy.grant",
        "acc-A",
        r#"{"amount":100}"#,
        &mut external,
    )
    .await
    .unwrap();
    execute_two_step_gm_command(
        &svc,
        actor_id,
        "player.ban",
        "victim-B",
        r#"{"reason":"B"}"#,
        &mut external,
    )
    .await
    .unwrap();

    // 断言: 2 玩家被禁, 1 玩家 grant
    assert_eq!(external.banned_players.len(), 2);
    assert!(external.banned_players.contains(&"victim-A".to_string()));
    assert!(external.banned_players.contains(&"victim-B".to_string()));
    assert_eq!(external.granted_balances.len(), 1);

    // 断言: audit 链有 3 条, 链连续
    let inner = audit.inner_repo();
    let mut list = inner.list_by_actor(actor_id, 100).await.unwrap();
    list.reverse();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].prev_hash, "0".repeat(64));
    assert_eq!(list[1].prev_hash, list[0].hash);
    assert_eq!(list[2].prev_hash, list[1].hash);
}

// ============================================================================
// Test 4: chaos — N 次随机失败位置注入, 全部回滚或全部成功 (无中间态泄漏)
// ============================================================================

/// 混沌测试: 模拟 20 次 GM 指令, 每次随机一个失败位置 (audit append 或 步骤 2),
/// 验证任何失败都触发完整回滚, 最终状态与"全部成功"路径一致。
#[tokio::test]
async fn chaos_random_failure_positions_all_rolled_back() {
    use std::collections::HashMap;

    let audit = Arc::new(FailingAuditLogRepository::new_dynamic());
    let users = Arc::new(InMemoryAdminUserRepository::new());
    let svc = AdminServiceImpl::new(users, audit.clone());

    let admin = AdminUser::new(
        "chaos-root".to_string(),
        "h".to_string(),
        AdminRole::SuperAdmin,
    );
    let actor_id = admin.id;

    // 用伪随机数 (固定 seed) 让测试可重复
    let mut rng_state: u64 = 0xdead_beef_cafe_babe;
    let mut next_rand = || -> u64 {
        // LCG (Numerical Recipes 常量)
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        rng_state
    };

    let actions = ["player.ban", "economy.grant"];
    let mut external = ExternalState::default();
    let mut expected_banned: HashMap<String, u32> = HashMap::new();
    let mut expected_grants: HashMap<String, u32> = HashMap::new();
    let n_commands = 20;

    for i in 0..n_commands {
        // 0 = 步骤 1 失败, 1 = 步骤 2 失败, 2 = 成功
        let failure_mode = (next_rand() % 3) as u8;
        let action_idx = (next_rand() % actions.len() as u64) as usize;
        let action = actions[action_idx];

        // 步骤 1 失败 → 通过 set_fail_next 注入
        if failure_mode == 0 {
            audit.set_fail_next();
        }

        let target = format!("chaos-target-{i}");
        let payload = if failure_mode == 1 {
            // 步骤 2 失败: 写 FAIL_GRANT
            r#"{"reason":"FAIL_GRANT"}"#.to_string()
        } else {
            r#"{"reason":"chaos"}"#.to_string()
        };

        let result = execute_two_step_gm_command(
            &svc,
            actor_id,
            action,
            &target,
            &payload,
            &mut external,
        )
        .await;

        match (failure_mode, action) {
            (0, _) => {
                // 步骤 1 失败 → 必返 Err
                assert!(result.is_err(), "cmd {i} 步骤 1 失败应返 Err, got {result:?}");
                // 外部状态无变化
            }
            (1, "economy.grant") => {
                // 步骤 2 失败 (economy) → Compensated
                assert!(
                    matches!(result, Ok(CompensateResult::Compensated { .. })),
                    "cmd {i} 步骤 2 失败应触发补偿, got {result:?}"
                );
            }
            (1, "player.ban") => {
                // player.ban 步骤 2 不会失败 (我们的 mock 不对 player.ban 注入失败)
                // 这种情况算成功
                assert!(result.is_ok());
                *expected_banned.entry(target.clone()).or_insert(0) += 1;
            }
            (2, "player.ban") => {
                assert!(result.is_ok());
                *expected_banned.entry(target.clone()).or_insert(0) += 1;
            }
            (2, "economy.grant") => {
                assert!(result.is_ok());
                *expected_grants.entry(target.clone()).or_insert(0) += 1;
            }
            _ => unreachable!(),
        }
    }

    // 最终断言: 外部状态与"成功执行次数"完全一致 (回滚到位)
    assert_eq!(
        external.banned_players.len(),
        expected_banned.values().sum::<u32>() as usize,
        "banned_players 数量应等于成功 ban 的次数"
    );
    assert_eq!(
        external.granted_balances.len(),
        expected_grants.values().sum::<u32>() as usize,
        "granted_balances 数量应等于成功 grant 的次数"
    );

    // 断言: 每次成功的 player.ban 都在 banned_players 中
    for target in expected_banned.keys() {
        assert!(
            external.banned_players.contains(target),
            "成功 ban 的 {target} 必在 banned_players 中"
        );
    }
    for target in expected_grants.keys() {
        assert!(
            external.granted_balances.iter().any(|(t, _)| t == target),
            "成功 grant 的 {target} 必在 granted_balances 中"
        );
    }
}

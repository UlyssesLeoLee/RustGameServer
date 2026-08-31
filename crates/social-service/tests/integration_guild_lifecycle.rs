//! social 域 IT 1/3 — Guild 完整生命周期集成测试 (最高规格, per RGS-IT-AGENT-BRIEFING §3.4)
//!
//! ## 场景 (per §3.4 #1)
//! 1. create_guild (leader 自动入会, member_count=1)
//! 2. 3 名玩家 join_guild (member_count=4)
//! 3. promote 1 名普通玩家到 officer
//! 4. dissolve_guild (公会 + 所有成员记录清除)
//! 5. dissolve 后再 join_guild → NotFound (公会不存在, 无法再加入)
//! 6. leader 记录的 player_id 也在 dissolve 后查不到 (member 列表空)
//!
//! ## 风格
//! - InMemoryGuildRepository / InMemoryGuildMemberRepository (per §1 既有 IT 风格)
//! - 不连真 DB, 不起真实 gRPC server (per 强制约束 §4)
//! - 走 SocialServiceImpl 业务方法, 不改 src/

use std::sync::Arc;

use social_service::entity::{GuildMember, GuildRole};
use social_service::error::Error;
use social_service::repository::{
    GuildMemberRepository, GuildRepository, InMemoryGuildMemberRepository, InMemoryGuildRepository,
};
use social_service::service::{SocialService, SocialServiceImpl};
use uuid::Uuid;

/// 构造 svc + 把 repos 单独再 clone 一份(Arc::clone)用于跨调用验证
fn new_svc_with_repos() -> (
    SocialServiceImpl,
    Arc<InMemoryGuildRepository>,
    Arc<InMemoryGuildMemberRepository>,
) {
    let guild_repo = Arc::new(InMemoryGuildRepository::new());
    let member_repo = Arc::new(InMemoryGuildMemberRepository::new());
    let svc = SocialServiceImpl::new(
        guild_repo.clone() as Arc<dyn GuildRepository>,
        member_repo.clone() as Arc<dyn GuildMemberRepository>,
    );
    (svc, guild_repo, member_repo)
}

#[tokio::test]
async fn guild_lifecycle_create_join_promote_dissolve_rejoin_fail() {
    let (svc, _guild_repo, member_repo) = new_svc_with_repos();

    // ----- 步骤 1: create_guild -----
    let leader_id = Uuid::new_v4();
    let guild = svc
        .create_guild(
            "Knights of Lifecycle".to_string(),
            "an end-to-end lifecycle test guild".to_string(),
            leader_id,
        )
        .await
        .expect("create_guild 必须成功");
    assert_eq!(guild.name, "Knights of Lifecycle");
    assert_eq!(guild.leader_id, leader_id);
    assert_eq!(guild.member_count, 1, "create 后 leader 自动入会, member_count=1");
    assert_eq!(guild.level, 1);

    // leader 应有对应的 GuildMember 记录, 角色=Leader
    let leader_member_records = member_repo
        .find_by_player(leader_id)
        .await
        .expect("leader member records query");
    assert_eq!(leader_member_records.len(), 1, "leader 自动生成 1 条 member 记录");
    assert_eq!(leader_member_records[0].role, GuildRole::Leader);
    assert_eq!(leader_member_records[0].guild_id, guild.id);

    // ----- 步骤 2: 3 名玩家 join_guild -----
    let player_a = Uuid::new_v4();
    let player_b = Uuid::new_v4();
    let player_c = Uuid::new_v4();

    let member_a = svc
        .join_guild(guild.id, player_a)
        .await
        .expect("player_a join 必须成功");
    let member_b = svc
        .join_guild(guild.id, player_b)
        .await
        .expect("player_b join 必须成功");
    let member_c = svc
        .join_guild(guild.id, player_c)
        .await
        .expect("player_c join 必须成功");

    // 三人角色都是 Member
    for m in [&member_a, &member_b, &member_c] {
        assert_eq!(m.role, GuildRole::Member);
        assert_eq!(m.guild_id, guild.id);
    }

    // member_count 应递增到 4 (1 leader + 3 玩家)
    let after_joins = svc
        .find_guild_by_id(guild.id)
        .await
        .expect("find_guild_by_id")
        .expect("guild 存在");
    assert_eq!(after_joins.member_count, 4, "leader+3 名玩家 = 4 members");

    // ----- 步骤 3: promote player_a 到 officer -----
    let promoted = svc
        .promote_to_officer(member_a.id)
        .await
        .expect("promote_to_officer 必须成功");
    assert_eq!(promoted.id, member_a.id);
    assert_eq!(promoted.role, GuildRole::Officer);
    assert_eq!(promoted.player_id, player_a);

    // leader 自身 promote 必须被拒
    let leader_promote_err = svc
        .promote_to_officer(leader_member_records[0].id)
        .await
        .expect_err("promote leader 必须被拒");
    assert!(
        matches!(leader_promote_err, Error::InsufficientPermission { .. }),
        "promote leader 期望 InsufficientPermission, got {:?}",
        leader_promote_err
    );

    // ----- 步骤 4: dissolve_guild -----
    let dissolved = svc
        .dissolve_guild(guild.id)
        .await
        .expect("dissolve_guild 必须成功");
    assert_eq!(dissolved.id, guild.id);
    assert_eq!(dissolved.name, "Knights of Lifecycle");

    // dissolve 后, guild 本身查不到
    let after_dissolve = svc
        .find_guild_by_id(guild.id)
        .await
        .expect("find_guild_by_id");
    assert!(after_dissolve.is_none(), "dissolve 后 guild 必不存在");

    // dissolve 后, 所有 member 记录被清空 (per service::dissolve_guild 内部循环 delete)
    for pid in [leader_id, player_a, player_b, player_c] {
        let remaining = member_repo
            .find_by_player(pid)
            .await
            .expect("find_by_player after dissolve");
        assert!(
            remaining.is_empty(),
            "dissolve 后 player={} 不应再有 member 记录, got {} 条",
            pid,
            remaining.len()
        );
    }

    // ----- 步骤 5: dissolve 后再 join_guild → NotFound -----
    let newcomer = Uuid::new_v4();
    let rejoin_err = svc
        .join_guild(guild.id, newcomer)
        .await
        .expect_err("dissolve 后 join 必须 NotFound");
    assert!(
        matches!(rejoin_err, Error::NotFound { entity: "Guild", .. }),
        "期望 NotFound {{ entity: Guild, .. }}, got {:?}",
        rejoin_err
    );

    // 确保 newcomer 也没被错误地创建成 ghost member
    let newcomer_records = member_repo
        .find_by_player(newcomer)
        .await
        .expect("find_by_player newcomer");
    assert!(
        newcomer_records.is_empty(),
        "失败的 join 不能留下 ghost member 记录, got {} 条",
        newcomer_records.len()
    );
}

/// 额外覆盖: dissolve_guild 后, 已 promote 的 officer 角色记录也被清 (无孤儿)
#[tokio::test]
async fn guild_lifecycle_dissolve_clears_officer_records() {
    let (svc, guild_repo, member_repo) = new_svc_with_repos();
    let leader_id = Uuid::new_v4();
    let guild = svc
        .create_guild("Officer Cleanup".to_string(), "".to_string(), leader_id)
        .await
        .unwrap();

    let player = Uuid::new_v4();
    let member = svc.join_guild(guild.id, player).await.unwrap();
    let officer = svc.promote_to_officer(member.id).await.unwrap();
    assert_eq!(officer.role, GuildRole::Officer);

    // dissolve
    svc.dissolve_guild(guild.id).await.unwrap();

    // player 的 member 记录(包括 Officer 角色)必须被清
    let player_records = member_repo.find_by_player(player).await.unwrap();
    assert!(
        player_records.is_empty(),
        "dissolve 后 officer 记录也必须清除, got {:?}",
        player_records
    );
    // guild 列表 (按 leader) 也应空
    let leader_guilds = guild_repo.list_by_leader(leader_id).await.unwrap();
    assert!(leader_guilds.is_empty(), "dissolve 后 leader 不再拥有任何 guild");
}

/// 额外覆盖: leader 的 member 记录 GuildMember 字段在 dissolve 前一致性
///   (验证 entity 与 repository 配合完整, 无半成品 state)
#[tokio::test]
async fn guild_lifecycle_leader_member_record_consistent_pre_dissolve() {
    let (svc, _guild_repo, member_repo) = new_svc_with_repos();
    let leader_id = Uuid::new_v4();
    let guild = svc
        .create_guild("Consistency".to_string(), "".to_string(), leader_id)
        .await
        .unwrap();

    // 取出 leader 的 member 记录, 验证字段一致性
    let leader_records = member_repo.find_by_player(leader_id).await.unwrap();
    assert_eq!(leader_records.len(), 1);
    let m: &GuildMember = &leader_records[0];
    assert_eq!(m.guild_id, guild.id);
    assert_eq!(m.player_id, leader_id);
    assert_eq!(m.role, GuildRole::Leader);
    assert_eq!(m.contribution, 0);
    assert!(m.joined_at <= chrono::Utc::now());
}

// ============================================================================
// Q6 leave_guild 集成测试场景 (per RGS-OPEN-QA-2026-08-31 v0.2 §Q6 决策)
// 决策:
//   - leadership 转移 = 加入时间最早剩余成员
//   - 仅剩 leader 一人退出 → 解散公会
//   - leaving player 的 player.profile.guild_id 置空 (跨域事件, 暂 trace 标记)
// ============================================================================

/// Q6 IT 场景 1: 普通成员退出, guild 保留, member_count 减 1
#[tokio::test]
async fn leave_guild_normal_member_does_not_dissolve() {
    let (svc, _guild_repo, member_repo) = new_svc_with_repos();
    let leader_id = Uuid::new_v4();
    let guild = svc
        .create_guild("Leave Normal".to_string(), "".to_string(), leader_id)
        .await
        .unwrap();

    // 2 个普通成员加入
    let player_a = Uuid::new_v4();
    let player_b = Uuid::new_v4();
    svc.join_guild(guild.id, player_a).await.unwrap();
    svc.join_guild(guild.id, player_b).await.unwrap();
    assert_eq!(
        svc.find_guild_by_id(guild.id).await.unwrap().unwrap().member_count,
        3
    );

    // player_a 退出
    svc.leave_guild(guild.id, player_a).await.unwrap();

    // guild 仍存在, member_count = 2
    let after = svc.find_guild_by_id(guild.id).await.unwrap();
    assert!(after.is_some(), "guild 不应解散");
    assert_eq!(after.unwrap().member_count, 2);

    // player_a 的 member 记录已清
    let player_a_records = member_repo.find_by_player(player_a).await.unwrap();
    assert!(
        player_a_records.is_empty(),
        "leave 后 player_a 不应再有 member 记录"
    );

    // leader 没变
    let after_guild = svc.find_guild_by_id(guild.id).await.unwrap().unwrap();
    assert_eq!(after_guild.leader_id, leader_id, "leader 不应被替换");
}

/// Q6 IT 场景 2: leader 退出, leadership 转移给 joined_at 最早剩余成员
#[tokio::test]
async fn leave_guild_leader_transfers_to_oldest_remaining() {
    let (svc, _guild_repo, member_repo) = new_svc_with_repos();
    let leader_id = Uuid::new_v4();
    let guild = svc
        .create_guild("Leader Transfer".to_string(), "".to_string(), leader_id)
        .await
        .unwrap();

    // player_old 先加入（最早）
    let player_old = Uuid::new_v4();
    let _old_member = svc.join_guild(guild.id, player_old).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;

    // player_mid 加入
    let player_mid = Uuid::new_v4();
    svc.join_guild(guild.id, player_mid).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;

    // player_new 最后加入
    let player_new = Uuid::new_v4();
    let _new_member = svc.join_guild(guild.id, player_new).await.unwrap();

    assert_eq!(
        svc.find_guild_by_id(guild.id).await.unwrap().unwrap().member_count,
        4
    );

    // leader 退出
    svc.leave_guild(guild.id, leader_id).await.unwrap();

    // guild 仍存在, member_count = 3
    let after = svc.find_guild_by_id(guild.id).await.unwrap();
    assert!(after.is_some());
    assert_eq!(after.unwrap().member_count, 3);

    // leadership 转移给 player_old (joined_at 最早)
    let after_guild = svc.find_guild_by_id(guild.id).await.unwrap().unwrap();
    assert_eq!(
        after_guild.leader_id, player_old,
        "leadership 应转移给 joined_at 最早剩余成员 (player_old)"
    );

    // 验证 player_old 的 role 变 Leader
    let old_records = member_repo.find_by_player(player_old).await.unwrap();
    assert_eq!(old_records.len(), 1);
    assert_eq!(old_records[0].role, GuildRole::Leader);

    // 验证 player_mid / player_new 角色不变 (仍是 Member)
    let mid_records = member_repo.find_by_player(player_mid).await.unwrap();
    assert_eq!(mid_records[0].role, GuildRole::Member);
    let new_records = member_repo.find_by_player(player_new).await.unwrap();
    assert_eq!(new_records[0].role, GuildRole::Member);

    // leader 原 member 记录已清
    let leader_records = member_repo.find_by_player(leader_id).await.unwrap();
    assert!(leader_records.is_empty());
}

/// Q6 IT 场景 3: 最后一人退出 → 解散公会
#[tokio::test]
async fn leave_guild_last_member_dissolves_guild() {
    let (svc, _guild_repo, member_repo) = new_svc_with_repos();
    let leader_id = Uuid::new_v4();
    let guild = svc
        .create_guild("Solo Guild".to_string(), "".to_string(), leader_id)
        .await
        .unwrap();
    assert_eq!(guild.member_count, 1);

    // leader (唯一成员) 退出
    svc.leave_guild(guild.id, leader_id).await.unwrap();

    // guild 解散, 查不到
    let after = svc.find_guild_by_id(guild.id).await.unwrap();
    assert!(after.is_none(), "最后一人退出应解散 guild");

    // leader 不再有 member 记录
    let leader_records = member_repo.find_by_player(leader_id).await.unwrap();
    assert!(leader_records.is_empty());
}

/// Q6 IT 场景 4: 非成员退出 → NotGuildMember, 状态不变
#[tokio::test]
async fn leave_guild_rejects_non_member_state_unchanged() {
    let (svc, _guild_repo, member_repo) = new_svc_with_repos();
    let leader_id = Uuid::new_v4();
    let guild = svc
        .create_guild("Reject Stranger".to_string(), "".to_string(), leader_id)
        .await
        .unwrap();

    let stranger = Uuid::new_v4();
    let err = svc
        .leave_guild(guild.id, stranger)
        .await
        .expect_err("非成员 leave_guild 应被拒");
    assert!(
        matches!(err, Error::NotGuildMember { .. }),
        "期望 NotGuildMember, got {:?}",
        err
    );

    // guild 状态不变
    let after = svc.find_guild_by_id(guild.id).await.unwrap().unwrap();
    assert_eq!(after.member_count, 1, "拒绝后 guild.member_count 不应变");
    assert_eq!(after.leader_id, leader_id);

    // stranger 也没有 member 记录
    let stranger_records = member_repo.find_by_player(stranger).await.unwrap();
    assert!(stranger_records.is_empty());
}

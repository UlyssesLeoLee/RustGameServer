//! social 域 IT 2/3 — Guild 容量边界集成测试 (最高规格, per RGS-IT-AGENT-BRIEFING §3.4)
//!
//! ## 场景 (per §3.4 #2)
//! 当前 src/ `service::join_guild` 实现硬上限 member_count >= 50 触发 `Error::GuildFull`
//! (per `crates/social-service/src/service.rs` 注释 "// 简单限制:50 人")。
//! 原始 spec 上限 64 与 src 实际值不一致 — 本 IT 按 src 实际行为 50 验证, 不动 src/。
//!
//! 步骤:
//! 1. create_guild (leader 已占 1 席, member_count=1)
//! 2. 补满到 50 (连续 join 49 个新玩家, 全部成功, member_count=50)
//! 3. 第 50 个新玩家 (即 51 总席位) join → 失败 `Error::GuildFull`
//! 4. 模拟 1 名玩家退出 (经 InMemoryGuildMemberRepository 直接 delete 1 个 member,
//!    并手动递减 member_count, 因为 src/ 无 leave_guild 业务方法)
//! 5. 1 名新玩家 join → 成功 (member_count 回到 50)
//! 6. 再次触发满员 (第 51 总席位) → `Error::GuildFull`, 验证边界稳定
//!
//! ## 风格
//! - InMemoryGuildRepository / InMemoryGuildMemberRepository (per §1)
//! - 走 SocialService trait 业务方法 (create/join), 退出通过 repo mock 模拟
//! - 不连真 DB, 不起真实 gRPC server (per §4 强制约束)

use std::sync::Arc;

use social_service::entity::{Guild, GuildRole};
use social_service::error::Error;
use social_service::repository::{
    GuildMemberRepository, GuildRepository, InMemoryGuildMemberRepository, InMemoryGuildRepository,
};
use social_service::service::{SocialService, SocialServiceImpl};
use uuid::Uuid;

/// Guild 实际硬上限 (per src/service.rs: `if guild.member_count >= 50`)
const ACTUAL_MAX_MEMBERS: i32 = 50;

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

/// 直接经 repo 模拟一名玩家退会 (src/ 无 leave_guild 业务方法)
async fn mock_leave(
    guild_repo: &Arc<InMemoryGuildRepository>,
    member_repo: &Arc<InMemoryGuildMemberRepository>,
    guild_id: Uuid,
    member_id: Uuid,
) {
    // 1) 删 member 记录
    let deleted = member_repo
        .delete_by_id(member_id)
        .await
        .expect("delete_by_id");
    assert!(deleted, "mock leave: member 记录必须存在");

    // 2) 递减 guild.member_count
    let mut guild = guild_repo
        .find_by_id(guild_id)
        .await
        .expect("find_by_id")
        .expect("guild 存在");
    assert!(guild.member_count > 1, "member_count 必须 > 1 才能 leave");
    guild.member_count -= 1;
    guild.updated_at = chrono::Utc::now();
    guild_repo.save(&guild).await.expect("save updated guild");
}

#[tokio::test]
async fn guild_capacity_boundary_fill_overflow_leave_recover_overflow() {
    let (svc, guild_repo, member_repo) = new_svc_with_repos();

    // ----- 步骤 1: create_guild (leader 占 1 席) -----
    let leader_id = Uuid::new_v4();
    let guild = svc
        .create_guild("Capacity".to_string(), "boundary test".to_string(), leader_id)
        .await
        .expect("create_guild 必成功");
    assert_eq!(guild.member_count, 1, "create 后 leader 占 1 席");

    // ----- 步骤 2: 补满到 50 (连续 join 49 个新玩家) -----
    let mut new_players: Vec<Uuid> = Vec::with_capacity(49);
    for i in 0..(ACTUAL_MAX_MEMBERS as usize - 1) {
        let p = Uuid::new_v4();
        svc.join_guild(guild.id, p)
            .await
            .unwrap_or_else(|e| panic!("第 {} 个新玩家 join 必成功, got {:?}", i + 1, e));
        new_players.push(p);
    }
    let after_fill = svc.find_guild_by_id(guild.id).await.unwrap().unwrap();
    assert_eq!(
        after_fill.member_count, ACTUAL_MAX_MEMBERS,
        "补满后 member_count={} (leader + {} 玩家)",
        ACTUAL_MAX_MEMBERS,
        ACTUAL_MAX_MEMBERS - 1
    );

    // ----- 步骤 3: 第 50 个新玩家 (即 51 总席位) join → 失败 -----
    let overflow_player = Uuid::new_v4();
    let overflow_err = svc
        .join_guild(guild.id, overflow_player)
        .await
        .expect_err("满员后第 50 个新玩家 join 必须 GuildFull");
    match &overflow_err {
        Error::GuildFull { guild_id } => {
            assert_eq!(guild_id, &guild.id.to_string());
        }
        other => panic!("期望 Error::GuildFull, got {:?}", other),
    }
    // overflow 玩家不应留下 ghost member 记录
    let overflow_records = member_repo.find_by_player(overflow_player).await.unwrap();
    assert!(
        overflow_records.is_empty(),
        "满员被拒的 join 不能留 ghost member, got {} 条",
        overflow_records.len()
    );
    // member_count 仍是 50, 不变
    let after_overflow = svc.find_guild_by_id(guild.id).await.unwrap().unwrap();
    assert_eq!(after_overflow.member_count, ACTUAL_MAX_MEMBERS);

    // ----- 步骤 4: 模拟 1 名玩家退出 (经 repo 直接操作) -----
    // 选第 1 个新玩家作为退出者
    let leaving_player = new_players[0];
    let leaving_member = member_repo
        .find_by_player(leaving_player)
        .await
        .unwrap()
        .into_iter()
        .find(|m| m.guild_id == guild.id)
        .expect("leaving player 必有 member 记录");
    mock_leave(&guild_repo, &member_repo, guild.id, leaving_member.id).await;

    let after_leave = svc.find_guild_by_id(guild.id).await.unwrap().unwrap();
    assert_eq!(
        after_leave.member_count,
        ACTUAL_MAX_MEMBERS - 1,
        "退出后 member_count={}",
        ACTUAL_MAX_MEMBERS - 1
    );
    // leaving player 的 member 记录已删
    let leaving_records = member_repo.find_by_player(leaving_player).await.unwrap();
    assert!(leaving_records.is_empty(), "退出玩家不应再有 member 记录");

    // ----- 步骤 5: 1 名新玩家 join → 成功 (member_count 回到 50) -----
    let newcomer = Uuid::new_v4();
    let joined_member = svc
        .join_guild(guild.id, newcomer)
        .await
        .expect("退出 1 人后, 新玩家 join 必成功");
    assert_eq!(joined_member.role, GuildRole::Member);
    assert_eq!(joined_member.guild_id, guild.id);

    let after_recover = svc.find_guild_by_id(guild.id).await.unwrap().unwrap();
    assert_eq!(
        after_recover.member_count, ACTUAL_MAX_MEMBERS,
        "恢复后 member_count 回到 {}",
        ACTUAL_MAX_MEMBERS
    );

    // ----- 步骤 6: 再次触发满员 → 边界稳定 -----
    let overflow2 = Uuid::new_v4();
    let overflow2_err = svc
        .join_guild(guild.id, overflow2)
        .await
        .expect_err("恢复后满员, 新玩家必须再次 GuildFull");
    assert!(
        matches!(overflow2_err, Error::GuildFull { .. }),
        "期望 GuildFull, got {:?}",
        overflow2_err
    );
    // 仍无 ghost
    let overflow2_records = member_repo.find_by_player(overflow2).await.unwrap();
    assert!(overflow2_records.is_empty());
    let final_state = svc.find_guild_by_id(guild.id).await.unwrap().unwrap();
    assert_eq!(final_state.member_count, ACTUAL_MAX_MEMBERS);
}

/// 额外覆盖: leader 自身尝试 join 自己公会 → AlreadyInGuild (边界外, 但相关)
#[tokio::test]
async fn guild_capacity_leader_cannot_rejoin_own_guild() {
    let (svc, _guild_repo, _member_repo) = new_svc_with_repos();
    let leader_id = Uuid::new_v4();
    let guild = svc
        .create_guild("Leader".to_string(), "".to_string(), leader_id)
        .await
        .unwrap();
    // leader 再 join → 已被记录, 应被拒
    let err = svc.join_guild(guild.id, leader_id).await.unwrap_err();
    assert!(
        matches!(err, Error::AlreadyInGuild { .. }),
        "期望 AlreadyInGuild, got {:?}",
        err
    );
}

/// 额外覆盖: 满员退出后被释放的 player_id 再次 join 应成功 (state 一致性)
#[tokio::test]
async fn guild_capacity_leaving_player_can_rejoin() {
    let (svc, guild_repo, member_repo) = new_svc_with_repos();
    let leader_id = Uuid::new_v4();
    let guild = svc
        .create_guild("Rejoin".to_string(), "".to_string(), leader_id)
        .await
        .unwrap();

    // fill 到 50
    for _ in 0..(ACTUAL_MAX_MEMBERS as usize - 1) {
        svc.join_guild(guild.id, Uuid::new_v4()).await.unwrap();
    }
    // 满员
    assert!(svc.join_guild(guild.id, Uuid::new_v4()).await.is_err());

    // 选一个新加入的玩家退出
    let members = member_repo.list_by_guild(guild.id).await.unwrap();
    // 找一名非 leader 的 Member
    let leaving_member = members
        .iter()
        .find(|m| m.role != GuildRole::Leader)
        .expect("至少有一名非 leader 成员");
    let leaving_player = leaving_member.player_id;
    mock_leave(&guild_repo, &member_repo, guild.id, leaving_member.id).await;

    // 该玩家现在不在任何 guild, 可重新 join 同公会
    let rejoin = svc
        .join_guild(guild.id, leaving_player)
        .await
        .expect("退出后该玩家应能重新 join 同公会");
    assert_eq!(rejoin.player_id, leaving_player);
    assert_eq!(rejoin.guild_id, guild.id);
    assert_eq!(rejoin.role, GuildRole::Member);

    // guild.member_count 回到 50
    let final_state: Guild = svc.find_guild_by_id(guild.id).await.unwrap().unwrap();
    assert_eq!(final_state.member_count, ACTUAL_MAX_MEMBERS);
}

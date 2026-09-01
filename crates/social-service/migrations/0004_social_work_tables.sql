-- social-service migration 0004_social_work_tables (per RGS-DB-BAS-001 v0.2 §3.5 PH-6 + 2026-09-01 21:16 JST 缺口 review 拍板)
-- 5 域社交域 social_db schema 增量
-- 5 张新 Work 表 (per RGS 横展开三分类原则, §2.1)
-- 业务来源: Q6 (8/27 JST) "leave_guild PH-6 下一轮实现" 决策 + 9/1 21:16 JST 缺口 review 拍板 (opt1)
-- 引用: RGS-DB-BAS-001 v0.2 §3.5 PH6-S-01~05 + 14-分区策略 §7 Work 表 cleanup SOP
--
-- ⚠️ MIGRATION_STATUS: DRAFT — 待 social Lead 评审 + PH-6 实装窗口启用
-- ⚠️ 本文件已 commit 到 git (DRAFT 状态), 但**不**在 PH-6 评审前 apply 到生产
-- ⚠️ 评审通过后: 移除本注释 + apply + 同步 RGS-DB-BAS-001 v0.2 §3.5 → v0.3 移表到 §3.3

-- ============================================================
-- sqlx prepare 检查 (per RGS-IMPL-005 BUILD 规范)
-- ============================================================
-- 上线前**必须**在本地 PG 演练环境跑:
--   cargo sqlx prepare --workspace -- --all-targets
-- 然后把生成的 .sqlx/ 目录 commit 进 git, 供 CI 在 SQLX_OFFLINE=true 下编译
-- 演练 PG 池环境变量: DATABASE_URL=postgres://rgs_social:rgs_social@localhost:5544/social_db
-- 若只改本文件 schema、未跑 sqlx prepare, CI 会以 "no cached query for ..." 阻断合并

-- ============================================================
-- 硬约束 (per RGS-BAS-007 + RGS-DB-BAS-001 v0.2)
-- ============================================================
-- RGS-BAS-007 §2: snake_case 字段名 / TIMESTAMPTZ 时间戳 / 不允许 nullable 主键
-- RGS-BAS-007 §4: Work 表不分区, 走 cleanup job (per 14-§7)
-- RGS-BAS-007 §1.5 + RGS-DB-BAS-001 v0.2 §6.1: 跨 DB 禁用外键, 应用层校验
-- RGS-DB-BAS-001 v0.2 §3.5: PH-6 5 候选 Work 表 + §2.1 判定问句
-- RGS-SPEC-CROSS-005 §2: 跨域弱引用 (player_id / target_player_id 都不物化 FK 到 player_db.players)
-- 17-P0-04 + 13-§3.3: CHECK 约束用 DO + EXCEPTION 幂等块 (兼容 fresh DB + 已部署)
-- FR-LCM-002 等价: 5 张表**不**绕过 audit_log (per admin_db.audit_log hash chain), 状态变更全流程留痕

-- ============================================================
-- 表 1/5: guild_invitations (per §3.5 PH6-S-01)
-- 会长 / 官员邀请玩家入会; expires_at + 7 天 TTL
-- ============================================================
CREATE TABLE IF NOT EXISTS guild_invitations (
    id UUID PRIMARY KEY,
    guild_id UUID NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    inviter_id UUID NOT NULL,                -- 跨域弱引用 (player_db.players.id)
    invitee_id UUID NOT NULL,                -- 跨域弱引用 (player_db.players.id)
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'declined', 'expired', 'cancelled')),
    invited_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    responded_at TIMESTAMPTZ,                -- accept/decline/cancel 时填
    expires_at TIMESTAMPTZ NOT NULL,         -- 7 天 TTL (应用层算)
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_guild_invitations_guild_id ON guild_invitations (guild_id);
CREATE INDEX IF NOT EXISTS idx_guild_invitations_invitee_status
    ON guild_invitations (invitee_id, status);
CREATE INDEX IF NOT EXISTS idx_guild_invitations_expires_at
    ON guild_invitations (expires_at) WHERE status = 'pending';

-- ============================================================
-- 表 2/5: guild_join_requests (per §3.5 PH6-S-02)
-- 玩家申请加入公会; expires_at + 7 天 TTL
-- ============================================================
CREATE TABLE IF NOT EXISTS guild_join_requests (
    id UUID PRIMARY KEY,
    guild_id UUID NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    applicant_id UUID NOT NULL,              -- 跨域弱引用
    message TEXT NOT NULL DEFAULT '',        -- 申请留言 (≤ 500 字符, 应用层校验)
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'approved', 'rejected', 'expired', 'cancelled')),
    requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    responded_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_guild_join_requests_guild_id
    ON guild_join_requests (guild_id);
CREATE INDEX IF NOT EXISTS idx_guild_join_requests_applicant_status
    ON guild_join_requests (applicant_id, status);
CREATE INDEX IF NOT EXISTS idx_guild_join_requests_expires_at
    ON guild_join_requests (expires_at) WHERE status = 'pending';

-- ============================================================
-- 表 3/5: guild_applications (per §3.5 PH6-S-03)
-- 跨域私聊入会申请 (公会对玩家主动发起, 与 PH6-S-02 玩家→公会的请求对偶)
-- 业务场景: 某公会向特定玩家发出定向入会申请, 玩家接受后直接入伙
-- ============================================================
CREATE TABLE IF NOT EXISTS guild_applications (
    id UUID PRIMARY KEY,
    guild_id UUID NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    target_player_id UUID NOT NULL,          -- 跨域弱引用
    proposed_role TEXT NOT NULL DEFAULT 'member'
        CHECK (proposed_role IN ('member', 'officer')),
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'declined', 'expired', 'cancelled')),
    sent_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    responded_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_guild_applications_guild_id
    ON guild_applications (guild_id);
CREATE INDEX IF NOT EXISTS idx_guild_applications_target_player_status
    ON guild_applications (target_player_id, status);
CREATE INDEX IF NOT EXISTS idx_guild_applications_expires_at
    ON guild_applications (expires_at) WHERE status = 'pending';

-- ============================================================
-- 表 4/5: friend_requests (per §3.5 PH6-S-04)
-- 玩家间好友请求; expires_at + 14 天 TTL (比 guild 长, 留更多时间)
-- ============================================================
CREATE TABLE IF NOT EXISTS friend_requests (
    id UUID PRIMARY KEY,
    requester_id UUID NOT NULL,              -- 跨域弱引用
    addressee_id UUID NOT NULL,              -- 跨域弱引用 (≠ requester_id, 应用层校验)
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'declined', 'expired', 'cancelled', 'blocked')),
    requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    responded_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,         -- 14 天 TTL
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_friend_requests_requester_status
    ON friend_requests (requester_id, status);
CREATE INDEX IF NOT EXISTS idx_friend_requests_addressee_status
    ON friend_requests (addressee_id, status);
CREATE INDEX IF NOT EXISTS idx_friend_requests_expires_at
    ON friend_requests (expires_at) WHERE status = 'pending';

-- ============================================================
-- 表 5/5: private_messages (per §3.5 PH6-S-05)
-- 玩家间私聊消息; 双方都读后 30 天清理 (per GDPR 规则 + §7 cleanup SOP)
-- 注意: 这是 Work 表 (短期), 非 Master (永久聊天记录属 PH-7 跨域需求)
-- ============================================================
CREATE TABLE IF NOT EXISTS private_messages (
    id UUID PRIMARY KEY,
    sender_id UUID NOT NULL,                 -- 跨域弱引用
    recipient_id UUID NOT NULL,              -- 跨域弱引用 (≠ sender_id, 应用层校验)
    content TEXT NOT NULL,                   -- 消息内容 (≤ 2000 字符, 应用层校验)
    content_type TEXT NOT NULL DEFAULT 'text'
        CHECK (content_type IN ('text', 'image_url', 'system')),
    read_at TIMESTAMPTZ,                     -- 接收方读时间, NULL=未读
    sent_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,         -- sent_at + 30 天, 或 read_at + 30 天, 取晚
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_private_messages_sender_sent
    ON private_messages (sender_id, sent_at DESC);
CREATE INDEX IF NOT EXISTS idx_private_messages_recipient_sent
    ON private_messages (recipient_id, sent_at DESC);
CREATE INDEX IF NOT EXISTS idx_private_messages_recipient_unread
    ON private_messages (recipient_id) WHERE read_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_private_messages_expires_at
    ON private_messages (expires_at);

-- ============================================================
-- 已知缺口 (待 social Lead 评审 + PH-6 实装窗口启用)
-- ============================================================
-- 1. 5 张表都缺应用层 cleanup job (per 14-§7.2 SOP, PH-2 评审), social-service/src/jobs/cleanup_social.rs 待实装
-- 2. 5 张表跨域弱引用 (player_id) 缺应用层校验 SOP (建议在 shared-platform 加 ensure_player_exists_or_get_tombstone helper, per 17-P1-02)
-- 3. private_messages 消息加密 / 端到端 E2EE 属 PH-7+, 本 migration 仅做明文存储
-- 4. guild_invitations 群发场景 (向多个玩家同时发) 当前 1 邀请 1 行, 大批量场景性能待 PH-6 评估
-- 5. friend_requests blocked 状态是否需要单独 blocked_players 表 (master) 待 PH-6 业务确认
-- 6. private_messages content 长度 2000 字符上限与 DTL-026 §3.x 业务规则待 cross-check

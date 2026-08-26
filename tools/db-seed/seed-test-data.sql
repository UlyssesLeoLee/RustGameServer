-- ============================================================================
-- RGS 5 域 + cluster-ops 6 DB 测试数据 seed
-- 任务: WF-1-db-seed-psql-9464 (W3 worker)
-- 创建: 2026-08-26 JST
--
-- 6 DB:
--   player_db / player_user
--   economy_db / economy_user
--   match_db / match_user
--   social_db / social_user
--   admin_db / admin_user
--   cluster_ops_db / cluster_ops_user
--
-- Schema 适配:
--   players.status ∈ {active, banned, disabled, pending}
--   accounts.currency ∈ {gold, diamond, token}
--   matches.mode ∈ {1v1, 2v2, 5v5, battle_royale}
--   match_participants.team ∈ {blue, red, none}
--   admin_users.role ∈ {super_admin, domain_admin, auditor, support}
--   cluster_nodes.role ∈ {primary, replica, candidate}
--   feature_flags.scope ∈ {global, domain, node}
--   guild_members.role ∈ {leader, officer, member}
--
-- 数据规模: 每表 ≥1 条; 关键表 ≥2 条; 跨 DB FK 用合成 UUID（不依赖跨域引用）
-- ============================================================================

-- ----------------------------------------------------------------------------
-- player_db
-- ----------------------------------------------------------------------------
INSERT INTO players (id, name, level, vip_level, status, last_login_at) VALUES
  ('11111111-1111-1111-1111-111111111111', 'Alice',   10, 0, 'active',   NOW() - INTERVAL '1 hour'),
  ('22222222-2222-2222-2222-222222222222', 'Bob',     25, 1, 'active',   NOW() - INTERVAL '30 minutes'),
  ('33333333-3333-3333-3333-333333333333', 'Charlie',  3, 0, 'pending',  NULL),
  ('44444444-4444-4444-4444-444444444444', 'Diana',   42, 2, 'active',   NOW() - INTERVAL '5 minutes'),
  ('55555555-5555-5555-5555-555555555555', 'Eve',      1, 0, 'banned',   NULL);

INSERT INTO player_sessions (id, player_id, device_id, ip, login_at, last_heartbeat_at, expires_at) VALUES
  ('aaaa1111-1111-1111-1111-111111111111', '11111111-1111-1111-1111-111111111111', 'dev-iphone-001',  '10.0.0.1',  NOW() - INTERVAL '30 minutes', NOW(),  NOW() + INTERVAL '1 day'),
  ('aaaa2222-2222-2222-2222-222222222222', '22222222-2222-2222-2222-222222222222', 'dev-android-002', '10.0.0.2',  NOW() - INTERVAL '15 minutes', NOW(),  NOW() + INTERVAL '1 day');

-- ----------------------------------------------------------------------------
-- economy_db  (player_id 用合成 UUID, 跨 DB 不强约束)
-- ----------------------------------------------------------------------------
INSERT INTO accounts (id, player_id, currency, balance, version, status) VALUES
  ('bbbb1111-1111-1111-1111-111111111111', '11111111-1111-1111-1111-111111111111', 'gold',    1000, 1, 'active'),
  ('bbbb2222-2222-2222-2222-222222222222', '22222222-2222-2222-2222-222222222222', 'diamond', 50,   1, 'active'),
  ('bbbb3333-3333-3333-3333-333333333333', '44444444-4444-4444-4444-444444444444', 'token',   100,  1, 'active');

INSERT INTO sagas (id, saga_type, command_id, idempotency_key, current_step, steps, status, completed_at) VALUES
  ('cccc1111-1111-1111-1111-111111111111', 'transfer', 'dddd1111-1111-1111-1111-111111111111', 'idem-saga-001', 2, '[{"step":"reserve","status":"ok"},{"step":"debit","status":"ok"},{"step":"credit","status":"pending"}]'::jsonb, 'running', NULL);

INSERT INTO reservations (id, saga_id, account_id, amount, currency, status, expires_at) VALUES
  ('eeee1111-1111-1111-1111-111111111111', 'cccc1111-1111-1111-1111-111111111111', 'bbbb1111-1111-1111-1111-111111111111', 100, 'gold', 'reserved', NOW() + INTERVAL '1 hour');

INSERT INTO transaction_ledger (id, account_id, idempotency_key, saga_id, command_id, amount, currency, kind, status, memo) VALUES
  ('ffff1111-1111-1111-1111-111111111111', 'bbbb1111-1111-1111-1111-111111111111', 'idem-tx-001', 'cccc1111-1111-1111-1111-111111111111', 'dddd1111-1111-1111-1111-111111111111', -100, 'gold', 'debit',  'committed', 'test transfer debit'),
  ('ffff2222-2222-2222-2222-222222222222', 'bbbb2222-2222-2222-2222-222222222222', 'idem-tx-002', NULL,                                    'dddd2222-2222-2222-2222-222222222222', 50,   'diamond', 'credit', 'pending',   'test diamond grant');

-- ----------------------------------------------------------------------------
-- match_db
-- ----------------------------------------------------------------------------
INSERT INTO matches (id, room_id, mode, status, winner_team, scheduled_at, started_at, ended_at) VALUES
  ('99991111-1111-1111-1111-111111111111', 'room-001', '5v5', 'finished', 'blue', NOW() - INTERVAL '2 hours', NOW() - INTERVAL '2 hours', NOW() - INTERVAL '1 hour 30 minutes'),
  ('99992222-2222-2222-2222-222222222222', 'room-002', '2v2', 'waiting',  NULL,   NOW() + INTERVAL '30 minutes', NULL, NULL);

INSERT INTO match_participants (id, match_id, player_id, team, score, kills, deaths, assists, is_mvp) VALUES
  ('88881111-1111-1111-1111-111111111111', '99991111-1111-1111-1111-111111111111', '11111111-1111-1111-1111-111111111111', 'blue', 12, 8, 3, 4, true),
  ('88882222-2222-2222-2222-222222222222', '99991111-1111-1111-1111-111111111111', '22222222-2222-2222-2222-222222222222', 'blue',  7, 5, 5, 6, false),
  ('88883333-3333-3333-3333-333333333333', '99991111-1111-1111-1111-111111111111', '33333333-3333-3333-3333-333333333333', 'red',   5, 4, 7, 3, false),
  ('88884444-4444-4444-4444-444444444444', '99992222-2222-2222-2222-222222222222', '44444444-4444-4444-4444-444444444444', 'blue',  0, 0, 0, 0, false);

-- ----------------------------------------------------------------------------
-- social_db
-- ----------------------------------------------------------------------------
INSERT INTO guilds (id, name, description, leader_id, level, member_count, experience) VALUES
  ('77771111-1111-1111-1111-111111111111', 'Iron Wolves', 'A test guild for Iron Wolves',  '11111111-1111-1111-1111-111111111111', 5, 3, 12500),
  ('77772222-2222-2222-2222-222222222222', 'Star Falcons','A test guild for Star Falcons', '44444444-4444-4444-4444-444444444444', 3, 2, 4200);

INSERT INTO guild_members (id, guild_id, player_id, role, contribution) VALUES
  ('66661111-1111-1111-1111-111111111111', '77771111-1111-1111-1111-111111111111', '11111111-1111-1111-1111-111111111111', 'leader', 5000),
  ('66662222-2222-2222-2222-222222222222', '77771111-1111-1111-1111-111111111111', '22222222-2222-2222-2222-222222222222', 'officer', 3000),
  ('66663333-3333-3333-3333-333333333333', '77772222-2222-2222-2222-222222222222', '44444444-4444-4444-4444-444444444444', 'leader', 2000);

-- ----------------------------------------------------------------------------
-- admin_db
-- ----------------------------------------------------------------------------
INSERT INTO admin_users (id, username, password_hash, role, domain_scope) VALUES
  ('55551111-1111-1111-1111-111111111111', 'root_admin',     'argon2id$placeholder_hash_root',  'super_admin',  '*'),
  ('55552222-2222-2222-2222-222222222222', 'support_alice',  'argon2id$placeholder_hash_alice', 'support',      'player');

-- audit_log 注意 prev_hash 唯一约束 + 不可删 trigger
INSERT INTO audit_log (id, actor_id, action, target, payload, prev_hash, hash) VALUES
  ('44441111-1111-1111-1111-111111111111', '55551111-1111-1111-1111-111111111111', 'role.assign',     'support_alice', '{"role":"support","scope":"player"}', 'genesis', 'hash-001-audit-bootstrap'),
  ('44442222-2222-2222-2222-222222222222', '55551111-1111-1111-1111-111111111111', 'player.ban',      'Eve',           '{"reason":"test-seed"}',                'hash-001-audit-bootstrap', 'hash-002-audit-ban-eve');

-- ----------------------------------------------------------------------------
-- cluster_ops_db
-- ----------------------------------------------------------------------------
INSERT INTO cluster_nodes (id, hostname, ip, role, status, version) VALUES
  ('33331111-1111-1111-1111-111111111111', 'k3s-server-01', '10.42.0.1', 'primary',   'healthy', 'v1.0.0'),
  ('33332222-2222-2222-2222-222222222222', 'k3s-agent-01',  '10.42.0.7', 'candidate', 'healthy', 'v1.0.0');

INSERT INTO feature_flags (key, scope, scope_value, enabled, version, updated_by) VALUES
  ('metrics.scrape.enabled',     'global', '*',       true,  1, '33331111-1111-1111-1111-111111111111'),
  ('rgs.debug.verbose_logging', 'domain', 'player',  false, 1, '33331111-1111-1111-1111-111111111111');

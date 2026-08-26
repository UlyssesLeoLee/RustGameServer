-- admin_db seed
INSERT INTO admin_users (id, username, password_hash, role, domain_scope) VALUES
  ('55551111-1111-1111-1111-111111111111', 'root_admin',    'argon2id$placeholder_hash_root',  'super_admin', '*'),
  ('55552222-2222-2222-2222-222222222222', 'support_alice', 'argon2id$placeholder_hash_alice', 'support',     'player');

-- audit_log: prev_hash 唯一 + 不可删 trigger
INSERT INTO audit_log (id, actor_id, action, target, payload, prev_hash, hash) VALUES
  ('44441111-1111-1111-1111-111111111111', '55551111-1111-1111-1111-111111111111', 'role.assign', 'support_alice', '{"role":"support","scope":"player"}', 'genesis',                'hash-001-audit-bootstrap'),
  ('44442222-2222-2222-2222-222222222222', '55551111-1111-1111-1111-111111111111', 'player.ban',  'Eve',           '{"reason":"test-seed"}',                'hash-001-audit-bootstrap', 'hash-002-audit-ban-eve');

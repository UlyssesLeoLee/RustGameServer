-- player_db seed
INSERT INTO players (id, name, level, vip_level, status, last_login_at) VALUES
  ('11111111-1111-1111-1111-111111111111', 'Alice',   10, 0, 'active',   NOW() - INTERVAL '1 hour'),
  ('22222222-2222-2222-2222-222222222222', 'Bob',     25, 1, 'active',   NOW() - INTERVAL '30 minutes'),
  ('33333333-3333-3333-3333-333333333333', 'Charlie',  3, 0, 'pending',  NULL),
  ('44444444-4444-4444-4444-444444444444', 'Diana',   42, 2, 'active',   NOW() - INTERVAL '5 minutes'),
  ('55555555-5555-5555-5555-555555555555', 'Eve',      1, 0, 'banned',   NULL);

INSERT INTO player_sessions (id, player_id, device_id, ip, login_at, last_heartbeat_at, expires_at) VALUES
  ('aaaa1111-1111-1111-1111-111111111111', '11111111-1111-1111-1111-111111111111', 'dev-iphone-001',  '10.0.0.1',  NOW() - INTERVAL '30 minutes', NOW(),  NOW() + INTERVAL '1 day'),
  ('aaaa2222-2222-2222-2222-222222222222', '22222222-2222-2222-2222-222222222222', 'dev-android-002', '10.0.0.2',  NOW() - INTERVAL '15 minutes', NOW(),  NOW() + INTERVAL '1 day');

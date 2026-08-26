-- economy_db seed
-- player_id 用合成 UUID（跨 DB 不强约束；player_db 真实 players ID 仅 1111...5555）
INSERT INTO accounts (id, player_id, currency, balance, version, status) VALUES
  ('bbbb1111-1111-1111-1111-111111111111', '11111111-1111-1111-1111-111111111111', 'gold',    1000, 1, 'active'),
  ('bbbb2222-2222-2222-2222-222222222222', '22222222-2222-2222-2222-222222222222', 'diamond',   50, 1, 'active'),
  ('bbbb3333-3333-3333-3333-333333333333', '44444444-4444-4444-4444-444444444444', 'token',    100, 1, 'active');

INSERT INTO sagas (id, saga_type, command_id, idempotency_key, current_step, steps, status, completed_at) VALUES
  ('cccc1111-1111-1111-1111-111111111111', 'transfer', 'dddd1111-1111-1111-1111-111111111111', 'idem-saga-001', 2, '[{"step":"reserve","status":"ok"},{"step":"debit","status":"ok"},{"step":"credit","status":"pending"}]'::jsonb, 'running', NULL);

INSERT INTO reservations (id, saga_id, account_id, amount, currency, status, expires_at) VALUES
  ('eeee1111-1111-1111-1111-111111111111', 'cccc1111-1111-1111-1111-111111111111', 'bbbb1111-1111-1111-1111-111111111111', 100, 'gold', 'reserved', NOW() + INTERVAL '1 hour');

INSERT INTO transaction_ledger (id, account_id, idempotency_key, saga_id, command_id, amount, currency, kind, status, memo) VALUES
  ('ffff1111-1111-1111-1111-111111111111', 'bbbb1111-1111-1111-1111-111111111111', 'idem-tx-001', 'cccc1111-1111-1111-1111-111111111111', 'dddd1111-1111-1111-1111-111111111111', -100, 'gold',    'spend',     'confirmed', 'test transfer debit'),
  ('ffff2222-2222-2222-2222-222222222222', 'bbbb2222-2222-2222-2222-222222222222', 'idem-tx-002', NULL,                                    'dddd2222-2222-2222-2222-222222222222',   50, 'diamond', 'deposit',   'pending',   'test diamond grant');

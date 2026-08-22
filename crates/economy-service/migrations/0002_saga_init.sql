-- economy-service migration 0002_saga_init（per RGS-DTL-100 Saga Q-003）
-- 54.8 实化：sagas + reservations + inbox（per DTL-100 §3 状态机 + §3.2 Reservation + §6 幂等性）

CREATE TABLE IF NOT EXISTS sagas (
    id UUID PRIMARY KEY,
    saga_type TEXT NOT NULL CHECK (saga_type IN ('transfer', 'daily_reward', 'purchase')),
    command_id UUID NOT NULL,
    idempotency_key TEXT NOT NULL,
    current_step INTEGER NOT NULL DEFAULT 0,
    steps JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'running', 'compensating', 'completed', 'failed', 'aborted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_sagas_command_id ON sagas (command_id);
CREATE INDEX IF NOT EXISTS idx_sagas_status ON sagas (status);
CREATE INDEX IF NOT EXISTS idx_sagas_idempotency_key ON sagas (idempotency_key);
CREATE UNIQUE INDEX IF NOT EXISTS uq_sagas_command_id ON sagas (command_id);

-- 资金预留（per DTL-100 §3.2）
CREATE TABLE IF NOT EXISTS reservations (
    id UUID PRIMARY KEY,
    saga_id UUID NOT NULL REFERENCES sagas(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts(id),
    amount BIGINT NOT NULL CHECK (amount > 0),
    currency TEXT NOT NULL CHECK (currency IN ('gold', 'diamond', 'token')),
    status TEXT NOT NULL DEFAULT 'reserved'
        CHECK (status IN ('reserved', 'confirmed', 'compensated', 'expired')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_reservations_saga_id ON reservations (saga_id);
CREATE INDEX IF NOT EXISTS idx_reservations_account_id ON reservations (account_id);
CREATE INDEX IF NOT EXISTS idx_reservations_status ON reservations (status);
CREATE INDEX IF NOT EXISTS idx_reservations_expires_at ON reservations (expires_at);

-- Inbox 幂等（per DTL-100 §6）
CREATE TABLE IF NOT EXISTS inbox (
    id UUID PRIMARY KEY,
    command_id UUID NOT NULL,
    handler TEXT NOT NULL,
    result TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'processed' CHECK (status IN ('processed', 'failed')),
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (command_id, handler)
);

CREATE INDEX IF NOT EXISTS idx_inbox_processed_at ON inbox (processed_at);

-- economy-service migration 0001_init（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-015 §3 + DTL-100 Saga）
-- 5 域经济域 economy_db schema 初始
-- 54.6 实化：accounts（OCC version）+ transaction_ledger（idempotency_key 三件套）

CREATE TABLE IF NOT EXISTS accounts (
    id UUID PRIMARY KEY,
    player_id UUID NOT NULL,
    currency TEXT NOT NULL CHECK (currency IN ('gold', 'diamond', 'token')),
    balance BIGINT NOT NULL DEFAULT 0 CHECK (balance >= 0),
    version BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'frozen', 'closed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (player_id, currency)
);

CREATE INDEX IF NOT EXISTS idx_accounts_player_id ON accounts (player_id);
CREATE INDEX IF NOT EXISTS idx_accounts_status ON accounts (status);

-- 账目（per RGS-DTL-100 §6 幂等性 + Saga 关键能力）
CREATE TABLE IF NOT EXISTS transaction_ledger (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    idempotency_key TEXT NOT NULL UNIQUE,
    saga_id UUID,
    command_id UUID,
    amount BIGINT NOT NULL,
    currency TEXT NOT NULL CHECK (currency IN ('gold', 'diamond', 'token')),
    kind TEXT NOT NULL CHECK (kind IN ('deposit', 'spend', 'transfer', 'refund', 'compensation')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'confirmed', 'reversed', 'failed')),
    memo TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_ledger_saga_id ON transaction_ledger (saga_id);
CREATE INDEX IF NOT EXISTS idx_ledger_account_id ON transaction_ledger (account_id);
CREATE INDEX IF NOT EXISTS idx_ledger_status ON transaction_ledger (status);

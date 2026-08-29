-- economy-service migration 0005_auctions（per RGS-DTL-038 §7.1 #8 + DEC-038-04 trade 域归属）
-- 卡牌 8 桶 / 子桶 1: trade 域 (拍卖 / 私下交易) 复用 economy_db (per ARC-008 5 独立 DB 原则).
--
-- 业务: 公开拍卖 (auction) + 私下交易 (private trade).
--       卡牌实例 (card_instances) 由 card-service 持有, 此处仅存 trade 元数据 +
--       外键引用 (card_instance_id 文本形式, 跨服务边界不强制 FK).
-- 跨域 saga: 拍卖成交需联动 card-service 转移卡牌实例 (per §6.3 ExecuteAuction saga),
--            W36+ 接入, 此迁移仅建 trade 域自有表.
--
-- 字段设计 (per DTL-038 §7.1 #8):
--   - auction_id     UUID PK
--   - seller_id      TEXT (玩家 ID, 跨服务边界不强制 UUID)
--   - card_id        TEXT (catalog 引用)
--   - card_instance_id TEXT (card_instances 引用, 跨域软引用)
--   - min_price      BIGINT (起拍价 / 一口价)
--   - currency_type  SMALLINT (1=soft 2=hard 3=card_value, per common.proto CurrencyType)
--   - highest_bid    BIGINT (当前最高价, 0 = 无人出价)
--   - highest_bidder TEXT (当前最高出价者 player_id, "" = 无人)
--   - status         SMALLINT (1=active 2=sold 3=cancelled 4=expired)
--   - started_at     TIMESTAMPTZ
--   - ends_at        TIMESTAMPTZ
--   - closed_at      TIMESTAMPTZ (成交/撤单/过期时间, NULL=进行中)
--   - winner_id      TEXT (成交时买家 ID, NULL=未成交)
--   - final_price    BIGINT (成交价, 0=未成交)
--
-- 索引 (per DTL-038 §7.1):
--   - status: 列表查询
--   - seller_id: 玩家历史
--   - highest_bidder: 玩家历史
--   - ends_at: 过期扫描 (W36+ 后台任务)

CREATE TABLE IF NOT EXISTS auctions (
    auction_id        UUID PRIMARY KEY,
    seller_id         TEXT NOT NULL,
    card_id           TEXT NOT NULL,
    card_instance_id  TEXT NOT NULL,
    min_price         BIGINT NOT NULL CHECK (min_price >= 0),
    currency_type     SMALLINT NOT NULL CHECK (currency_type IN (1, 2, 3)),
    highest_bid       BIGINT NOT NULL DEFAULT 0 CHECK (highest_bid >= 0),
    highest_bidder    TEXT NOT NULL DEFAULT '',
    status            SMALLINT NOT NULL DEFAULT 1 CHECK (status IN (1, 2, 3, 4)),
    started_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    ends_at           TIMESTAMPTZ NOT NULL,
    closed_at         TIMESTAMPTZ,
    winner_id         TEXT,
    final_price       BIGINT NOT NULL DEFAULT 0,
    -- saga 关联: 跨域 saga (Trade / ExecuteAuction) 启动时记录, 崩溃恢复用
    saga_id           UUID
);

CREATE INDEX IF NOT EXISTS idx_auctions_status ON auctions (status);
CREATE INDEX IF NOT EXISTS idx_auctions_seller ON auctions (seller_id);
CREATE INDEX IF NOT EXISTS idx_auctions_highest_bidder ON auctions (highest_bidder) WHERE highest_bidder <> '';
CREATE INDEX IF NOT EXISTS idx_auctions_ends_at ON auctions (ends_at) WHERE status = 1;

-- 私下交易占位 (per DEC-038-04 A: economy-service v2 内置 trade 域)
-- W36+ 跨域 saga 实装后填 schema 字段; 当前仅建表.
-- 业务: 玩家 A 向玩家 B 提出私下交易, A 出货币 + 1 卡牌实例, B 出货币 + 1 卡牌实例,
--       双方 accept 后执行卡牌 + 货币交换. 涉及 card-service 跨域 (saga 步骤 3/4 per §6.3).
CREATE TABLE IF NOT EXISTS private_trades (
    trade_id          UUID PRIMARY KEY,
    proposer_id       TEXT NOT NULL,
    counterparty_id   TEXT NOT NULL,
    status            SMALLINT NOT NULL DEFAULT 1 CHECK (status IN (1, 2, 3, 4)),
        -- 1=proposed 2=accepted 3=completed 4=cancelled
    proposer_currency_amount BIGINT NOT NULL DEFAULT 0,
    proposer_currency_type   SMALLINT,
    proposer_card_instance_id TEXT,
    counterparty_currency_amount BIGINT NOT NULL DEFAULT 0,
    counterparty_currency_type   SMALLINT,
    counterparty_card_instance_id TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at         TIMESTAMPTZ,
    saga_id           UUID
);

CREATE INDEX IF NOT EXISTS idx_private_trades_proposer ON private_trades (proposer_id);
CREATE INDEX IF NOT EXISTS idx_private_trades_counterparty ON private_trades (counterparty_id);
CREATE INDEX IF NOT EXISTS idx_private_trades_status ON private_trades (status);

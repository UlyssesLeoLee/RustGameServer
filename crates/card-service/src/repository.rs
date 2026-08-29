//! card-service 域 Repository
//!
//! 桶 10 (card catalog) 实化 (per RGS-DTL-038 §4.4 + §7.1):
//! - CardRepository (catalog 静态, per FR-003)
//! - CardSeriesRepository (卡包 / 系列, 含 drop_table, per FR-003 + DEC-038-06)
//! - CardInstanceRepository (玩家收藏, 动态, per FR-006)
//! - 双实现: PgRepository (sqlx, 生产) + InMemoryRepository (单测)
//!
//! 8 个 CRUD 方法 (任务书要求):
//! CardRepository: find_by_id / list / create
//! CardSeriesRepository: find_by_id / list / find_by_id
//! CardInstanceRepository: find_by_id / list_by_owner / add / remove

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity::{
    Card, CardInstance, CardInstanceSource, CardRarity, CardSeries, CardSeriesStatus, CardStats,
    CardType, Currency, CurrencyType, DropEntry, DropTable,
};
use crate::Result;

// ============================================================================
// 分页 (per common.proto PageRequest)
// ============================================================================

/// 分页请求 (per common.proto PageRequest)
#[derive(Debug, Clone, Copy)]
pub struct PageRequest {
    pub page: u32,
    pub page_size: u32,
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

/// 分页响应 (per common.proto PageResponse)
#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

// ============================================================================
// 过滤 (per card.proto ListCardsRequest / GetPlayerCollectionRequest)
// ============================================================================

/// Card 列表过滤 (per card.proto ListCardsRequest)
#[derive(Debug, Clone, Default)]
pub struct CardFilter {
    /// 按 type 过滤 (None = 不过滤)
    pub type_filter: Option<CardType>,
    /// 按 rarity 过滤
    pub rarity_filter: Option<CardRarity>,
    /// 按 series_id 过滤
    pub series_id_filter: Option<String>,
}

/// CardSeries 列表过滤
#[derive(Debug, Clone, Default)]
pub struct CardSeriesFilter {
    /// 按 status 过滤 (None = 不过滤)
    pub status_filter: Option<CardSeriesStatus>,
}

/// CardInstance 列表过滤
#[derive(Debug, Clone, Default)]
pub struct CardInstanceFilter {
    /// 按 rarity 过滤 (跨 cards 表 join)
    pub rarity_filter: Option<CardRarity>,
    /// 按 series_id 过滤
    pub series_id_filter: Option<String>,
    /// 按 source 过滤
    pub source_filter: Option<CardInstanceSource>,
}

// ============================================================================
// Trait 定义
// ============================================================================

/// Card Repository (catalog 静态)
#[async_trait]
pub trait CardRepository: Send + Sync {
    /// 按 card_id 查询
    async fn find_by_id(&self, card_id: &str) -> Result<Option<Card>>;
    /// 分页 + 过滤列出
    async fn list(
        &self,
        filter: &CardFilter,
        page_req: PageRequest,
    ) -> Result<Page<Card>>;
    /// 批量预加载 (按 card_id 列表, 用于 OpenPack 后按抽到的 card_id 一次拉 master)
    async fn find_by_ids(&self, card_ids: &[String]) -> Result<Vec<Card>>;
    /// 创建卡牌 (运营配置入口, 桶 14 后续实装 admin 入口时调用)
    async fn create(&self, card: &Card) -> Result<Card>;
}

/// CardSeries Repository
#[async_trait]
pub trait CardSeriesRepository: Send + Sync {
    /// 按 series_id 查询
    async fn find_by_id(&self, series_id: &str) -> Result<Option<CardSeries>>;
    /// 分页 + 过滤列出
    async fn list(
        &self,
        filter: &CardSeriesFilter,
        page_req: PageRequest,
    ) -> Result<Page<CardSeries>>;
    /// 创建 / 更新系列 (运营配置入口, 桶 14 后续实装)
    async fn upsert(&self, series: &CardSeries) -> Result<CardSeries>;
}

/// CardInstance Repository (玩家收藏, 动态)
#[async_trait]
pub trait CardInstanceRepository: Send + Sync {
    /// 按 instance_id 查询
    async fn find_by_id(&self, instance_id: Uuid) -> Result<Option<CardInstance>>;
    /// 分页 + 过滤按 owner_id 列出 (per GetPlayerCollectionRequest)
    async fn list_by_owner(
        &self,
        owner_id: Uuid,
        filter: &CardInstanceFilter,
        page_req: PageRequest,
    ) -> Result<Page<CardInstance>>;
    /// 批量添加 (OpenPack 一次性插入 N 张)
    async fn add_many(&self, instances: &[CardInstance]) -> Result<Vec<CardInstance>>;
    /// 按 instance_id 删除 (RemoveCardFromCollection)
    async fn remove(&self, instance_id: Uuid) -> Result<bool>;
    /// 按 owner 统计总数 (收藏统计用)
    async fn count_by_owner(&self, owner_id: Uuid) -> Result<u64>;
}

// ============================================================================
// SQL 序列化 helper (JSONB 字段)
// ============================================================================

fn hashmap_to_jsonb(m: &HashMap<String, String>) -> Result<serde_json::Value> {
    serde_json::to_value(m)
        .map_err(|e| crate::Error::Internal(anyhow::anyhow!("serialize map to JSONB: {}", e)))
}

fn hashmap_to_jsonb_i32(m: &HashMap<String, i32>) -> Result<serde_json::Value> {
    serde_json::to_value(m)
        .map_err(|e| crate::Error::Internal(anyhow::anyhow!("serialize map to JSONB: {}", e)))
}

fn jsonb_to_hashmap(v: serde_json::Value) -> Result<HashMap<String, String>> {
    if v.is_null() {
        return Ok(HashMap::new());
    }
    serde_json::from_value(v)
        .map_err(|e| crate::Error::Internal(anyhow::anyhow!("deserialize map from JSONB: {}", e)))
}

fn jsonb_to_hashmap_i32(v: serde_json::Value) -> Result<HashMap<String, i32>> {
    if v.is_null() {
        return Ok(HashMap::new());
    }
    serde_json::from_value(v)
        .map_err(|e| crate::Error::Internal(anyhow::anyhow!("deserialize map from JSONB: {}", e)))
}

fn stats_to_jsonb(s: &CardStats) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "attack": s.attack,
        "health": s.health,
        "mana": s.mana,
        "custom": s.custom,
    }))
}

fn jsonb_to_stats(v: serde_json::Value) -> Result<CardStats> {
    if v.is_null() {
        return Ok(CardStats::default());
    }
    let attack = v.get("attack").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
    let health = v.get("health").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
    let mana = v.get("mana").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
    let custom_val = v
        .get("custom")
        .cloned()
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let custom = jsonb_to_hashmap_i32(custom_val)?;
    Ok(CardStats {
        attack,
        health,
        mana,
        custom,
    })
}

fn drop_table_to_jsonb(dt: &DropTable) -> Result<serde_json::Value> {
    let entries: Vec<serde_json::Value> = dt
        .entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "rarity": e.rarity.as_i32(),
                "count": e.count,
                "probability": e.probability,
                "card_id": e.card_id,
            })
        })
        .collect();
    Ok(serde_json::json!({
        "version": dt.version,
        "snapshot_at": dt.snapshot_at.to_rfc3339(),
        "entries": entries,
    }))
}

fn jsonb_to_drop_table(v: serde_json::Value) -> Result<DropTable> {
    if v.is_null() {
        return Ok(DropTable::new(Vec::new()));
    }
    let version = v.get("version").and_then(|x| x.as_u64()).unwrap_or(1) as u32;
    let snapshot_at_str = v
        .get("snapshot_at")
        .and_then(|x| x.as_str())
        .unwrap_or("");
    let snapshot_at = if snapshot_at_str.is_empty() {
        Utc::now()
    } else {
        DateTime::parse_from_rfc3339(snapshot_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now())
    };
    let entries_val = v
        .get("entries")
        .cloned()
        .unwrap_or(serde_json::Value::Array(Vec::new()));
    let entries_arr = entries_val
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut entries = Vec::with_capacity(entries_arr.len());
    for e in entries_arr {
        let rarity_int = e.get("rarity").and_then(|x| x.as_i64()).unwrap_or(0) as i32;
        let count = e.get("count").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
        let probability = e
            .get("probability")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0);
        let card_id = e
            .get("card_id")
            .and_then(|x| x.as_str())
            .map(String::from);
        entries.push(DropEntry {
            rarity: CardRarity::from_i32(rarity_int),
            count,
            probability,
            card_id,
        });
    }
    Ok(DropTable {
        version,
        snapshot_at,
        entries,
    })
}

// ============================================================================
// Pg Repository (sqlx 实现, 生产用)
// ============================================================================

pub struct PgCardRepository {
    pool: PgPool,
}

impl PgCardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_card(row: sqlx::postgres::PgRow) -> Result<Card> {
    let type_int: i16 = row.get("type");
    let rarity_int: i16 = row.get("rarity");
    let name_i18n: serde_json::Value = row.get("name_i18n");
    let desc_i18n: serde_json::Value = row.get("description_i18n");
    let stats: serde_json::Value = row.get("stats");
    Ok(Card {
        card_id: row.get("card_id"),
        series_id: row.get("series_id"),
        name_default: row.get("name_default"),
        name_i18n: jsonb_to_hashmap(name_i18n)?,
        card_type: CardType::from_i32(i32::from(type_int)),
        rarity: CardRarity::from_i32(i32::from(rarity_int)),
        base_cost: row.get::<i32, _>("base_cost") as u32,
        description_i18n: jsonb_to_hashmap(desc_i18n)?,
        effect_ref: row.get("effect_ref"),
        stats: jsonb_to_stats(stats)?,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

#[async_trait]
impl CardRepository for PgCardRepository {
    async fn find_by_id(&self, card_id: &str) -> Result<Option<Card>> {
        let row = sqlx::query(
            "SELECT card_id, series_id, name_default, name_i18n, type, rarity, base_cost, \
             description_i18n, effect_ref, stats, created_at, updated_at \
             FROM cards WHERE card_id = $1",
        )
        .bind(card_id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(row_to_card(r)?)),
            None => Ok(None),
        }
    }

    async fn list(
        &self,
        filter: &CardFilter,
        page_req: PageRequest,
    ) -> Result<Page<Card>> {
        let offset = ((page_req.page.saturating_sub(1)) * page_req.page_size) as i64;
        let limit = page_req.page_size as i64;

        // 动态 WHERE 构造 (filter 三选 N)
        let mut where_clauses: Vec<String> = Vec::new();
        if let Some(t) = filter.type_filter {
            where_clauses.push(format!("type = {}", t.as_i32()));
        }
        if let Some(r) = filter.rarity_filter {
            where_clauses.push(format!("rarity = {}", r.as_i32()));
        }
        if let Some(ref s) = filter.series_id_filter {
            where_clauses.push(format!("series_id = '{}'", s.replace('\'', "''")));
        }
        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM cards {}", where_sql);
        let total: i64 = sqlx::query_scalar(&count_sql).fetch_one(&self.pool).await?;

        let list_sql = format!(
            "SELECT card_id, series_id, name_default, name_i18n, type, rarity, base_cost, \
             description_i18n, effect_ref, stats, created_at, updated_at \
             FROM cards {} ORDER BY card_id ASC OFFSET {} LIMIT {}",
            where_sql, offset, limit
        );
        let rows = sqlx::query(&list_sql).fetch_all(&self.pool).await?;
        let items: Vec<Card> = rows
            .into_iter()
            .map(row_to_card)
            .collect::<Result<Vec<_>>>()?;

        let has_next = (offset + items.len() as i64) < total;
        Ok(Page {
            items,
            total,
            page: page_req.page,
            page_size: page_req.page_size,
            has_next,
        })
    }

    async fn find_by_ids(&self, card_ids: &[String]) -> Result<Vec<Card>> {
        if card_ids.is_empty() {
            return Ok(Vec::new());
        }
        // 用 ANY($1) 数组查询
        let rows = sqlx::query(
            "SELECT card_id, series_id, name_default, name_i18n, type, rarity, base_cost, \
             description_i18n, effect_ref, stats, created_at, updated_at \
             FROM cards WHERE card_id = ANY($1)",
        )
        .bind(card_ids)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_card).collect()
    }

    async fn create(&self, card: &Card) -> Result<Card> {
        let name_i18n = hashmap_to_jsonb(&card.name_i18n)?;
        let desc_i18n = hashmap_to_jsonb(&card.description_i18n)?;
        let stats = stats_to_jsonb(&card.stats)?;
        let type_num: i16 = card.card_type.as_i32() as i16;
        let rarity_num: i16 = card.rarity.as_i32() as i16;

        sqlx::query(
            "INSERT INTO cards \
             (card_id, series_id, name_default, name_i18n, type, rarity, base_cost, \
              description_i18n, effect_ref, stats, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) \
             ON CONFLICT (card_id) DO UPDATE SET \
                series_id = EXCLUDED.series_id, \
                name_default = EXCLUDED.name_default, \
                name_i18n = EXCLUDED.name_i18n, \
                type = EXCLUDED.type, \
                rarity = EXCLUDED.rarity, \
                base_cost = EXCLUDED.base_cost, \
                description_i18n = EXCLUDED.description_i18n, \
                effect_ref = EXCLUDED.effect_ref, \
                stats = EXCLUDED.stats, \
                updated_at = EXCLUDED.updated_at",
        )
        .bind(&card.card_id)
        .bind(&card.series_id)
        .bind(&card.name_default)
        .bind(name_i18n)
        .bind(type_num)
        .bind(rarity_num)
        .bind(card.base_cost as i32)
        .bind(desc_i18n)
        .bind(&card.effect_ref)
        .bind(stats)
        .bind(card.created_at)
        .bind(card.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(card.clone())
    }
}

pub struct PgCardSeriesRepository {
    pool: PgPool,
}

impl PgCardSeriesRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_card_series(row: sqlx::postgres::PgRow) -> Result<CardSeries> {
    let status_int: i16 = row.get("status");
    let price_type_int: i16 = row.get("price_type");
    let name_i18n: serde_json::Value = row.get("name_i18n");
    let drop_table: serde_json::Value = row.get("drop_table");
    Ok(CardSeries {
        series_id: row.get("series_id"),
        name_default: row.get("name_default"),
        name_i18n: jsonb_to_hashmap(name_i18n)?,
        pack_size: row.get::<i32, _>("pack_size") as u32,
        drop_table: jsonb_to_drop_table(drop_table)?,
        price: Currency {
            currency_type: CurrencyType::from_i32(i32::from(price_type_int)),
            amount: row.get("price_amount"),
        },
        released_at: row.get("released_at"),
        status: CardSeriesStatus::from_i32(i32::from(status_int)),
    })
}

#[async_trait]
impl CardSeriesRepository for PgCardSeriesRepository {
    async fn find_by_id(&self, series_id: &str) -> Result<Option<CardSeries>> {
        let row = sqlx::query(
            "SELECT series_id, name_default, name_i18n, pack_size, drop_table, \
             price_type, price_amount, released_at, status \
             FROM card_series WHERE series_id = $1",
        )
        .bind(series_id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(row_to_card_series(r)?)),
            None => Ok(None),
        }
    }

    async fn list(
        &self,
        filter: &CardSeriesFilter,
        page_req: PageRequest,
    ) -> Result<Page<CardSeries>> {
        let offset = ((page_req.page.saturating_sub(1)) * page_req.page_size) as i64;
        let limit = page_req.page_size as i64;

        let mut where_clauses: Vec<String> = Vec::new();
        if let Some(s) = filter.status_filter {
            where_clauses.push(format!("status = {}", s.as_i32()));
        }
        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM card_series {}", where_sql);
        let total: i64 = sqlx::query_scalar(&count_sql).fetch_one(&self.pool).await?;

        let list_sql = format!(
            "SELECT series_id, name_default, name_i18n, pack_size, drop_table, \
             price_type, price_amount, released_at, status \
             FROM card_series {} ORDER BY released_at DESC OFFSET {} LIMIT {}",
            where_sql, offset, limit
        );
        let rows = sqlx::query(&list_sql).fetch_all(&self.pool).await?;
        let items: Vec<CardSeries> = rows
            .into_iter()
            .map(row_to_card_series)
            .collect::<Result<Vec<_>>>()?;
        let has_next = (offset + items.len() as i64) < total;
        Ok(Page {
            items,
            total,
            page: page_req.page,
            page_size: page_req.page_size,
            has_next,
        })
    }

    async fn upsert(&self, series: &CardSeries) -> Result<CardSeries> {
        let name_i18n = hashmap_to_jsonb(&series.name_i18n)?;
        let drop_table = drop_table_to_jsonb(&series.drop_table)?;
        let status_num: i16 = series.status.as_i32() as i16;
        let price_type_num: i16 = series.price.currency_type.as_i32() as i16;

        sqlx::query(
            "INSERT INTO card_series \
             (series_id, name_default, name_i18n, pack_size, drop_table, \
              price_type, price_amount, released_at, status) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (series_id) DO UPDATE SET \
                name_default = EXCLUDED.name_default, \
                name_i18n = EXCLUDED.name_i18n, \
                pack_size = EXCLUDED.pack_size, \
                drop_table = EXCLUDED.drop_table, \
                price_type = EXCLUDED.price_type, \
                price_amount = EXCLUDED.price_amount, \
                released_at = EXCLUDED.released_at, \
                status = EXCLUDED.status",
        )
        .bind(&series.series_id)
        .bind(&series.name_default)
        .bind(name_i18n)
        .bind(series.pack_size as i32)
        .bind(drop_table)
        .bind(price_type_num)
        .bind(series.price.amount)
        .bind(series.released_at)
        .bind(status_num)
        .execute(&self.pool)
        .await?;
        Ok(series.clone())
    }
}

pub struct PgCardInstanceRepository {
    pool: PgPool,
}

impl PgCardInstanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_card_instance(row: sqlx::postgres::PgRow) -> Result<CardInstance> {
    let source_int: i16 = row.get("source");
    let attrs: serde_json::Value = row.get("attrs");
    Ok(CardInstance {
        instance_id: row.get("instance_id"),
        card_id: row.get("card_id"),
        owner_id: Uuid::parse_str(row.get::<String, _>("owner_id").as_str())
            .map_err(|e| crate::Error::Internal(anyhow::anyhow!("invalid owner_id uuid: {}", e)))?,
        acquired_at: row.get("acquired_at"),
        source: CardInstanceSource::from_i32(i32::from(source_int)),
        level: row.get::<i32, _>("level") as u32,
        attrs: jsonb_to_hashmap_i32(attrs)?,
        tradable: row.get("tradable"),
        locked: row.get("locked"),
    })
}

#[async_trait]
impl CardInstanceRepository for PgCardInstanceRepository {
    async fn find_by_id(&self, instance_id: Uuid) -> Result<Option<CardInstance>> {
        let row = sqlx::query(
            "SELECT instance_id, card_id, owner_id, acquired_at, source, level, attrs, tradable, locked \
             FROM card_instances WHERE instance_id = $1",
        )
        .bind(instance_id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(row_to_card_instance(r)?)),
            None => Ok(None),
        }
    }

    async fn list_by_owner(
        &self,
        owner_id: Uuid,
        filter: &CardInstanceFilter,
        page_req: PageRequest,
    ) -> Result<Page<CardInstance>> {
        let offset = ((page_req.page.saturating_sub(1)) * page_req.page_size) as i64;
        let limit = page_req.page_size as i64;

        // card_instances 与 cards 表 join (rarity / series 过滤)
        let mut where_clauses: Vec<String> = vec![format!("ci.owner_id = '{}'", owner_id)];
        if let Some(s) = filter.source_filter {
            where_clauses.push(format!("ci.source = {}", s.as_i32()));
        }
        if filter.rarity_filter.is_some() || filter.series_id_filter.is_some() {
            if let Some(r) = filter.rarity_filter {
                where_clauses.push(format!("c.rarity = {}", r.as_i32()));
            }
            if let Some(ref s) = filter.series_id_filter {
                where_clauses.push(format!("c.series_id = '{}'", s.replace('\'', "''")));
            }
            let where_sql = where_clauses.join(" AND ");
            let count_sql = format!(
                "SELECT COUNT(*) FROM card_instances ci JOIN cards c ON ci.card_id = c.card_id WHERE {}",
                where_sql
            );
            let total: i64 = sqlx::query_scalar(&count_sql).fetch_one(&self.pool).await?;
            let list_sql = format!(
                "SELECT ci.instance_id, ci.card_id, ci.owner_id, ci.acquired_at, ci.source, \
                 ci.level, ci.attrs, ci.tradable, ci.locked \
                 FROM card_instances ci JOIN cards c ON ci.card_id = c.card_id \
                 WHERE {} ORDER BY ci.acquired_at DESC OFFSET {} LIMIT {}",
                where_sql, offset, limit
            );
            let rows = sqlx::query(&list_sql).fetch_all(&self.pool).await?;
            let items: Vec<CardInstance> = rows
                .into_iter()
                .map(row_to_card_instance)
                .collect::<Result<Vec<_>>>()?;
            let has_next = (offset + items.len() as i64) < total;
            return Ok(Page {
                items,
                total,
                page: page_req.page,
                page_size: page_req.page_size,
                has_next,
            });
        }

        // 无 rarity / series 过滤, 走单表查询
        let where_sql = where_clauses.join(" AND ");
        let count_sql = format!("SELECT COUNT(*) FROM card_instances ci WHERE {}", where_sql);
        let total: i64 = sqlx::query_scalar(&count_sql).fetch_one(&self.pool).await?;
        let list_sql = format!(
            "SELECT ci.instance_id, ci.card_id, ci.owner_id, ci.acquired_at, ci.source, \
             ci.level, ci.attrs, ci.tradable, ci.locked \
             FROM card_instances ci WHERE {} \
             ORDER BY ci.acquired_at DESC OFFSET {} LIMIT {}",
            where_sql, offset, limit
        );
        let rows = sqlx::query(&list_sql).fetch_all(&self.pool).await?;
        let items: Vec<CardInstance> = rows
            .into_iter()
            .map(row_to_card_instance)
            .collect::<Result<Vec<_>>>()?;
        let has_next = (offset + items.len() as i64) < total;
        Ok(Page {
            items,
            total,
            page: page_req.page,
            page_size: page_req.page_size,
            has_next,
        })
    }

    async fn add_many(&self, instances: &[CardInstance]) -> Result<Vec<CardInstance>> {
        if instances.is_empty() {
            return Ok(Vec::new());
        }
        let mut tx = self.pool.begin().await?;
        for inst in instances {
            let attrs = hashmap_to_jsonb_i32(&inst.attrs)?;
            let source_num: i16 = inst.source.as_i32() as i16;
            sqlx::query(
                "INSERT INTO card_instances \
                 (instance_id, card_id, owner_id, acquired_at, source, level, attrs, tradable, locked) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(inst.instance_id)
            .bind(&inst.card_id)
            .bind(inst.owner_id.to_string())
            .bind(inst.acquired_at)
            .bind(source_num)
            .bind(inst.level as i32)
            .bind(attrs)
            .bind(inst.tradable)
            .bind(inst.locked)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(instances.to_vec())
    }

    async fn remove(&self, instance_id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM card_instances WHERE instance_id = $1")
            .bind(instance_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn count_by_owner(&self, owner_id: Uuid) -> Result<u64> {
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM card_instances WHERE owner_id = $1",
        )
        .bind(owner_id.to_string())
        .fetch_one(&self.pool)
        .await?;
        Ok(n.max(0) as u64)
    }
}

// ============================================================================
// InMemoryRepository (单测 / IT 用, 验证 trait 行为)
// ============================================================================

pub struct InMemoryCardRepository {
    inner: Mutex<HashMap<String, Card>>,
}

impl InMemoryCardRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryCardRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CardRepository for InMemoryCardRepository {
    async fn find_by_id(&self, card_id: &str) -> Result<Option<Card>> {
        Ok(self.inner.lock().unwrap().get(card_id).cloned())
    }

    async fn list(
        &self,
        filter: &CardFilter,
        page_req: PageRequest,
    ) -> Result<Page<Card>> {
        let guard = self.inner.lock().unwrap();
        let mut all: Vec<Card> = guard
            .values()
            .filter(|c| {
                if let Some(t) = filter.type_filter {
                    if c.card_type != t {
                        return false;
                    }
                }
                if let Some(r) = filter.rarity_filter {
                    if c.rarity != r {
                        return false;
                    }
                }
                if let Some(ref s) = filter.series_id_filter {
                    if &c.series_id != s {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        all.sort_by(|a, b| a.card_id.cmp(&b.card_id));
        let total = all.len() as i64;
        let offset = ((page_req.page.saturating_sub(1)) * page_req.page_size) as usize;
        let limit = page_req.page_size as usize;
        let items: Vec<Card> = all.into_iter().skip(offset).take(limit).collect();
        let has_next = (offset + items.len()) < total as usize;
        Ok(Page {
            items,
            total,
            page: page_req.page,
            page_size: page_req.page_size,
            has_next,
        })
    }

    async fn find_by_ids(&self, card_ids: &[String]) -> Result<Vec<Card>> {
        let guard = self.inner.lock().unwrap();
        Ok(card_ids
            .iter()
            .filter_map(|id| guard.get(id).cloned())
            .collect())
    }

    async fn create(&self, card: &Card) -> Result<Card> {
        self.inner
            .lock()
            .unwrap()
            .insert(card.card_id.clone(), card.clone());
        Ok(card.clone())
    }
}

pub struct InMemoryCardSeriesRepository {
    inner: Mutex<HashMap<String, CardSeries>>,
}

impl InMemoryCardSeriesRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryCardSeriesRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CardSeriesRepository for InMemoryCardSeriesRepository {
    async fn find_by_id(&self, series_id: &str) -> Result<Option<CardSeries>> {
        Ok(self.inner.lock().unwrap().get(series_id).cloned())
    }

    async fn list(
        &self,
        filter: &CardSeriesFilter,
        page_req: PageRequest,
    ) -> Result<Page<CardSeries>> {
        let guard = self.inner.lock().unwrap();
        let mut all: Vec<CardSeries> = guard
            .values()
            .filter(|s| {
                if let Some(st) = filter.status_filter {
                    if s.status != st {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        all.sort_by(|a, b| b.released_at.cmp(&a.released_at));
        let total = all.len() as i64;
        let offset = ((page_req.page.saturating_sub(1)) * page_req.page_size) as usize;
        let limit = page_req.page_size as usize;
        let items: Vec<CardSeries> = all.into_iter().skip(offset).take(limit).collect();
        let has_next = (offset + items.len()) < total as usize;
        Ok(Page {
            items,
            total,
            page: page_req.page,
            page_size: page_req.page_size,
            has_next,
        })
    }

    async fn upsert(&self, series: &CardSeries) -> Result<CardSeries> {
        self.inner
            .lock()
            .unwrap()
            .insert(series.series_id.clone(), series.clone());
        Ok(series.clone())
    }
}

pub struct InMemoryCardInstanceRepository {
    inner: Mutex<HashMap<Uuid, CardInstance>>,
    /// 引用 CardRepository 以支持 rarity / series 过滤
    cards: std::sync::Arc<dyn CardRepository>,
}

impl InMemoryCardInstanceRepository {
    pub fn new(cards: std::sync::Arc<dyn CardRepository>) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            cards,
        }
    }
}

#[async_trait]
impl CardInstanceRepository for InMemoryCardInstanceRepository {
    async fn find_by_id(&self, instance_id: Uuid) -> Result<Option<CardInstance>> {
        Ok(self.inner.lock().unwrap().get(&instance_id).cloned())
    }

    async fn list_by_owner(
        &self,
        owner_id: Uuid,
        filter: &CardInstanceFilter,
        page_req: PageRequest,
    ) -> Result<Page<CardInstance>> {
        // 先在 lock 内收集 candidate (克隆, 不持 guard 跨 await)
        let mut candidates: Vec<CardInstance> = {
            let guard = self.inner.lock().unwrap();
            guard
                .values()
                .filter(|i| i.owner_id == owner_id)
                .filter(|i| {
                    if let Some(s) = filter.source_filter {
                        if i.source != s {
                            return false;
                        }
                    }
                    true
                })
                .cloned()
                .collect()
        };
        // guard 已 drop (作用域结束), 后续可安全 await

        // rarity / series 过滤需要查 master card
        if filter.rarity_filter.is_some() || filter.series_id_filter.is_some() {
            // 收集需要查的 card_id
            let card_ids: Vec<String> = candidates.iter().map(|i| i.card_id.clone()).collect();
            // 一次拉 master
            let masters = self.cards.find_by_ids(&card_ids).await?;
            let master_map: HashMap<String, Card> =
                masters.into_iter().map(|c| (c.card_id.clone(), c)).collect();
            let target_rarity = filter.rarity_filter;
            let target_series = filter.series_id_filter.clone();
            candidates.retain(|i| {
                if let Some(c) = master_map.get(&i.card_id) {
                    if let Some(r) = target_rarity {
                        if c.rarity != r {
                            return false;
                        }
                    }
                    if let Some(ref s) = target_series {
                        if &c.series_id != s {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            });
        }

        candidates.sort_by(|a, b| b.acquired_at.cmp(&a.acquired_at));
        let total = candidates.len() as i64;
        let offset = ((page_req.page.saturating_sub(1)) * page_req.page_size) as usize;
        let limit = page_req.page_size as usize;
        let items: Vec<CardInstance> = candidates.into_iter().skip(offset).take(limit).collect();
        let has_next = (offset + items.len()) < total as usize;
        Ok(Page {
            items,
            total,
            page: page_req.page,
            page_size: page_req.page_size,
            has_next,
        })
    }

    async fn add_many(&self, instances: &[CardInstance]) -> Result<Vec<CardInstance>> {
        let mut guard = self.inner.lock().unwrap();
        for i in instances {
            guard.insert(i.instance_id, i.clone());
        }
        Ok(instances.to_vec())
    }

    async fn remove(&self, instance_id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&instance_id).is_some())
    }

    async fn count_by_owner(&self, owner_id: Uuid) -> Result<u64> {
        let guard = self.inner.lock().unwrap();
        Ok(guard.values().filter(|i| i.owner_id == owner_id).count() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn sample_card(id: &str) -> Card {
        let mut c = Card::new(
            id.to_string(),
            "series_001".to_string(),
            format!("Card {}", id),
            CardType::Creature,
            CardRarity::Common,
        );
        c.base_cost = 3;
        c
    }

    fn sample_series(id: &str) -> CardSeries {
        let mut s = CardSeries::new(id.to_string(), format!("Series {}", id), 5);
        s.drop_table = DropTable::new(vec![DropEntry {
            rarity: CardRarity::Common,
            count: 1,
            probability: 1.0,
            card_id: None,
        }]);
        s
    }

    #[tokio::test]
    async fn in_memory_card_repo_create_and_find() {
        let repo = InMemoryCardRepository::new();
        let c = sample_card("card_001");
        repo.create(&c).await.unwrap();
        let found = repo.find_by_id("card_001").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().card_id, "card_001");
    }

    #[tokio::test]
    async fn in_memory_card_repo_list_filter() {
        let repo = InMemoryCardRepository::new();
        repo.create(&sample_card("card_001")).await.unwrap();
        let mut c2 = sample_card("card_002");
        c2.rarity = CardRarity::Legendary;
        c2.card_type = CardType::Spell;
        repo.create(&c2).await.unwrap();

        let filter = CardFilter {
            rarity_filter: Some(CardRarity::Common),
            ..Default::default()
        };
        let page = repo.list(&filter, PageRequest::default()).await.unwrap();
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].card_id, "card_001");
        assert_eq!(page.total, 1);
    }

    #[tokio::test]
    async fn in_memory_card_series_repo_upsert_and_find() {
        let repo = InMemoryCardSeriesRepository::new();
        let s = sample_series("series_001");
        repo.upsert(&s).await.unwrap();
        let found = repo.find_by_id("series_001").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().pack_size, 5);
    }

    #[tokio::test]
    async fn in_memory_card_instance_repo_lifecycle() {
        let cards: std::sync::Arc<dyn CardRepository> = Arc::new(InMemoryCardRepository::new());
        cards.create(&sample_card("card_001")).await.unwrap();
        let repo = InMemoryCardInstanceRepository::new(cards);

        let owner = Uuid::new_v4();
        let inst1 = CardInstance::new("card_001".to_string(), owner, CardInstanceSource::Pack);
        let inst2 = CardInstance::new("card_001".to_string(), owner, CardInstanceSource::Reward);
        repo.add_many(&[inst1.clone(), inst2.clone()]).await.unwrap();

        let count = repo.count_by_owner(owner).await.unwrap();
        assert_eq!(count, 2);

        let page = repo
            .list_by_owner(owner, &CardInstanceFilter::default(), PageRequest::default())
            .await
            .unwrap();
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.total, 2);

        let removed = repo.remove(inst1.instance_id).await.unwrap();
        assert!(removed);
        let after = repo.count_by_owner(owner).await.unwrap();
        assert_eq!(after, 1);
    }

    #[tokio::test]
    async fn in_memory_card_instance_repo_rarity_filter() {
        let cards: std::sync::Arc<dyn CardRepository> = Arc::new(InMemoryCardRepository::new());
        let mut c_legendary = sample_card("card_legendary");
        c_legendary.rarity = CardRarity::Legendary;
        cards.create(&sample_card("card_common")).await.unwrap();
        cards.create(&c_legendary).await.unwrap();

        let repo = InMemoryCardInstanceRepository::new(cards);
        let owner = Uuid::new_v4();
        repo.add_many(&[
            CardInstance::new("card_common".to_string(), owner, CardInstanceSource::Pack),
            CardInstance::new("card_legendary".to_string(), owner, CardInstanceSource::Pack),
        ])
        .await
        .unwrap();

        let filter = CardInstanceFilter {
            rarity_filter: Some(CardRarity::Legendary),
            ..Default::default()
        };
        let page = repo
            .list_by_owner(owner, &filter, PageRequest::default())
            .await
            .unwrap();
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].card_id, "card_legendary");
    }
}
